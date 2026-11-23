use crate::{
    adapters::sqlite::{bulb_repository, profile_repository},
    application::bulb_service::BulbService,
    domain::{
        bulb::Bulb,
        control::{LiveState, ResultCode, TestResult},
        profile::Profile,
        validation::now,
    },
    error::{AppError, AppResult},
    ports::bulb_controller::{BulbController, DeviceState},
};
use futures_util::{StreamExt, stream};
use rusqlite::TransactionBehavior;
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;

#[derive(Clone)]
pub struct StatusService {
    pub bulbs: Arc<BulbService>,
    pub client: Arc<crate::adapters::tasmota::client::TasmotaClient>,
}

impl StatusService {
    fn unavailable(id: Uuid, status: ResultCode) -> LiveState {
        let reachability = match status {
            ResultCode::Success => "reachable",
            ResultCode::Timeout => "timeout",
            ResultCode::Unreachable => "unreachable",
            ResultCode::InvalidResponse => "invalid_response",
            ResultCode::DeviceError | ResultCode::HttpError => "device_error",
        };
        LiveState {
            bulb_id: id,
            reachability: reachability.into(),
            power: None,
            dimmer: None,
            rgb_color: None,
            white_channels: None,
            color_temperature_mired: None,
            color_temperature_kelvin: None,
            applied_profile_id: None,
            checked_at: now(),
        }
    }

    fn profile(state: &DeviceState, profiles: &[Profile]) -> Option<Uuid> {
        let dimmer = state.dimmer?;
        if state.power.as_deref() != Some("on") {
            return None;
        }
        let rgb_on = state
            .rgb_color
            .as_deref()
            .is_some_and(|rgb| rgb != "#000000");
        let white_on = state.white_channels.as_deref().map(|white| white != "0000");
        let mut matches = profiles.iter().filter(|profile| {
            profile.dimmer == dimmer
                && match profile.mode.as_str() {
                    "rgb" if rgb_on && white_on != Some(true) => rgb_matches_dimmer_scaled_color(
                        profile.rgb_color.as_deref(),
                        state.rgb_color.as_deref(),
                        dimmer,
                    ),
                    "color_temperature" if !rgb_on && white_on == Some(true) => profile
                        .color_temperature_kelvin
                        .map(kelvin_to_ct)
                        .is_some_and(|ct| state.ct == Some(ct)),
                    _ => false,
                }
        });
        let matched = matches.next()?;
        matches.next().is_none().then_some(matched.id)
    }

    async fn status_bulb(&self, bulb: Bulb, profiles: Arc<Vec<Profile>>) -> LiveState {
        let endpoint = match self
            .client
            .policy
            .endpoint(&bulb.ip_address, i64::from(bulb.port))
        {
            Ok(endpoint) => endpoint,
            Err(_) => {
                error!(bulb_id=%bulb.id, "saved bulb violates target policy");
                return Self::unavailable(bulb.id, ResultCode::InvalidResponse);
            }
        };
        match self.client.status(endpoint).await {
            Ok(state) => {
                let applied_profile_id = Self::profile(&state, &profiles);
                LiveState {
                    bulb_id: bulb.id,
                    reachability: "reachable".into(),
                    power: state.power,
                    dimmer: state.dimmer,
                    rgb_color: state.rgb_color,
                    white_channels: state.white_channels,
                    color_temperature_mired: state.ct,
                    color_temperature_kelvin: state.ct.map(ct_to_kelvin),
                    applied_profile_id,
                    checked_at: now(),
                }
            }
            Err(status) => {
                error!(
                    bulb_id = %bulb.id,
                    status = ?status,
                    "tasmota status check failed"
                );
                Self::unavailable(bulb.id, status)
            }
        }
    }

    pub async fn all(&self) -> AppResult<Vec<LiveState>> {
        let (bulbs, profiles) = self
            .bulbs
            .db
            .run(|connection| {
                let transaction = connection
                    .transaction_with_behavior(TransactionBehavior::Deferred)
                    .map_err(|_| AppError::Internal)?;
                let bulbs = bulb_repository::list(&transaction)?;
                let profiles = profile_repository::list(&transaction)?;
                transaction.commit().map_err(|_| AppError::Internal)?;
                Ok((bulbs, profiles))
            })
            .await?;
        let profiles = Arc::new(profiles);
        let mut indexed = stream::iter(bulbs.into_iter().enumerate().map(|(index, bulb)| {
            let profiles = profiles.clone();
            async move { (index, self.status_bulb(bulb, profiles).await) }
        }))
        .buffer_unordered(self.client.outbound.available_permits().max(1))
        .collect::<Vec<_>>()
        .await;
        indexed.sort_by_key(|(index, _)| *index);
        Ok(indexed.into_iter().map(|(_, state)| state).collect())
    }

    pub async fn one(&self, id: Uuid) -> AppResult<LiveState> {
        let (bulb, profiles) = self
            .bulbs
            .db
            .run(move |connection| {
                let transaction = connection
                    .transaction_with_behavior(TransactionBehavior::Deferred)
                    .map_err(|_| AppError::Internal)?;
                let bulb = bulb_repository::get(&transaction, id)?;
                let profiles = profile_repository::list(&transaction)?;
                transaction.commit().map_err(|_| AppError::Internal)?;
                Ok((bulb, profiles))
            })
            .await?;
        Ok(self.status_bulb(bulb, Arc::new(profiles)).await)
    }

    pub async fn test(&self, bulb: Bulb) -> TestResult {
        let status = match self
            .client
            .policy
            .endpoint(&bulb.ip_address, i64::from(bulb.port))
        {
            Ok(endpoint) => {
                self.client
                    .invoke(endpoint, crate::ports::bulb_controller::Command::Test)
                    .await
            }
            Err(_) => {
                error!(bulb_id=%bulb.id, "saved bulb violates target policy");
                ResultCode::InvalidResponse
            }
        };
        let compatible = match status {
            ResultCode::Success => Some(true),
            ResultCode::Timeout | ResultCode::Unreachable => None,
            _ => Some(false),
        };
        TestResult {
            message: status.message().into(),
            status,
            compatible,
            checked_at: now(),
        }
    }
}

fn kelvin_to_ct(kelvin: u16) -> u16 {
    ((1_000_000 + u32::from(kelvin) / 2) / u32::from(kelvin)) as u16
}
fn ct_to_kelvin(ct: u16) -> u16 {
    ((1_000_000 + u32::from(ct) / 2) / u32::from(ct)) as u16
}

fn rgb_matches_dimmer_scaled_color(
    profile_color: Option<&str>,
    device_color: Option<&str>,
    dimmer: u8,
) -> bool {
    let (Some(profile), Some(device)) = (parse_rgb(profile_color), parse_rgb(device_color)) else {
        return false;
    };
    let dimmer = u16::from(dimmer);
    profile
        .into_iter()
        .zip(device)
        .all(|(profile_channel, device_channel)| {
            let expected = (u16::from(profile_channel) * dimmer + 50) / 100;
            expected.abs_diff(u16::from(device_channel)) <= 2
        })
}

fn parse_rgb(value: Option<&str>) -> Option<[u8; 3]> {
    let value = value?.strip_prefix('#')?;
    (value.len() == 6 && value.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .then(|| {
            Some([
                u8::from_str_radix(&value[0..2], 16).ok()?,
                u8::from_str_radix(&value[2..4], 16).ok()?,
                u8::from_str_radix(&value[4..6], 16).ok()?,
            ])
        })
        .flatten()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_uses_white_channels_instead_of_the_retained_ct_alone() {
        let profile = Profile {
            id: Uuid::nil(),
            name: "Warm white".into(),
            dimmer: 40,
            mode: "color_temperature".into(),
            rgb_color: None,
            color_temperature_kelvin: Some(2000),
            mixed_color: None,
            created_at: String::new(),
            updated_at: String::new(),
        };
        let mut state = DeviceState {
            power: Some("on".into()),
            dimmer: Some(40),
            rgb_color: Some("#000000".into()),
            white_channels: Some("0066".into()),
            ct: Some(500),
        };
        let profiles = [profile];
        assert_eq!(StatusService::profile(&state, &profiles), Some(Uuid::nil()));
        state.white_channels = Some("0000".into());
        assert_eq!(StatusService::profile(&state, &profiles), None);
    }

    #[test]
    fn rgb_profile_matches_device_color_scaled_by_dimmer() {
        assert!(rgb_matches_dimmer_scaled_color(
            Some("#FF8000"),
            Some("#663300"),
            40,
        ));
    }

    #[test]
    fn rgb_profile_rejects_different_color_after_scaling() {
        assert!(!rgb_matches_dimmer_scaled_color(
            Some("#FF8000"),
            Some("#660040"),
            40,
        ));
    }

    #[test]
    fn rgb_profile_accepts_rounding_difference() {
        assert!(rgb_matches_dimmer_scaled_color(
            Some("#123456"),
            Some("#071521"),
            40,
        ));
    }
}
