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
            mode: None,
            rgb_color: None,
            color_temperature_kelvin: None,
            applied_profile_id: None,
            checked_at: now(),
        }
    }

    fn profile(state: &DeviceState, profiles: &[Profile]) -> Option<Uuid> {
        let dimmer = state.dimmer?;
        if state.power.as_deref() != Some("on") || state.mode.is_none() {
            return None;
        }
        let matches = profiles
            .iter()
            .filter(|profile| match state.mode.as_deref() {
                Some("rgb") => {
                    profile.mode == "rgb"
                        && profile.dimmer == dimmer
                        && profile.rgb_color == state.rgb_color
                }
                Some("color_temperature") => {
                    profile.mode == "color_temperature"
                        && profile.dimmer == dimmer
                        && state.ct.is_some_and(|ct| {
                            (167..=333).contains(&ct)
                                && profile
                                    .color_temperature_kelvin
                                    .is_some_and(|kelvin| kelvin_to_ct(kelvin) == ct)
                        })
                }
                _ => false,
            })
            .collect::<Vec<_>>();
        (matches.len() == 1).then(|| matches[0].id)
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
            Ok(state) => LiveState {
                bulb_id: bulb.id,
                reachability: "reachable".into(),
                power: state.power.clone(),
                dimmer: state.dimmer,
                mode: state.mode.clone(),
                rgb_color: state.rgb_color.clone(),
                color_temperature_kelvin: state.ct.map(ct_to_kelvin),
                applied_profile_id: Self::profile(&state, &profiles),
                checked_at: now(),
            },
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
