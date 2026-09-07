use crate::{
    adapters::{
        sqlite::{bulb_repository, profile_repository, settings_repository},
        tasmota::command,
    },
    application::{
        bulb_service::BulbService, profile_service::ProfileService,
        settings_service::SettingsService,
    },
    domain::{
        bulb::Bulb,
        control::{Operation, ResultCode, Summary, TargetResult},
        validation::now,
    },
    error::{AppError, AppResult, validation},
    ports::bulb_controller::{BulbController, Command},
};
use futures_util::{StreamExt, stream};
use rusqlite::TransactionBehavior;
use std::{collections::HashSet, sync::Arc};
use tokio::sync::Semaphore;
use tracing::{debug, error};
use uuid::Uuid;

#[derive(Clone)]
pub struct ControlService {
    pub bulbs: Arc<BulbService>,
    pub profiles: Arc<ProfileService>,
    pub settings: Arc<SettingsService>,
    pub client: Arc<crate::adapters::tasmota::client::TasmotaClient>,
    pub admission: Arc<Semaphore>,
}

impl ControlService {
    fn validate_ids(ids: &[Uuid]) -> AppResult<()> {
        if ids.is_empty() || ids.iter().collect::<HashSet<_>>().len() != ids.len() {
            return Err(validation(
                "bulbIds",
                "Bulb IDs must be non-empty and distinct",
            ));
        }
        Ok(())
    }

    async fn execute(
        &self,
        operation: &str,
        profile_id: Option<Uuid>,
        bulbs: Vec<Bulb>,
        command: Command,
    ) -> Operation {
        let operation_started = std::time::Instant::now();
        let started_at = now();
        let mut indexed = stream::iter(bulbs.into_iter().enumerate().map(|(index, bulb)| {
            let client = self.client.clone();
            let command = command.clone();
            async move {
                let target_started = std::time::Instant::now();
                let status = match client
                    .policy
                    .endpoint(&bulb.ip_address, i64::from(bulb.port))
                {
                    Ok(endpoint) => client.invoke(endpoint, command).await,
                    Err(_) => {
                        error!(bulb_id=%bulb.id, "saved bulb violates target policy");
                        ResultCode::InvalidResponse
                    }
                };
                debug!(
                    bulb_id=%bulb.id,
                    result_code=?status,
                    duration_ms=target_started.elapsed().as_millis(),
                    "target call completed"
                );
                if !matches!(status, ResultCode::Success) {
                    error!(
                        bulb_id = %bulb.id,
                        result_code = ?status,
                        "tasmota control call failed"
                    );
                }
                (
                    index,
                    TargetResult {
                        bulb_id: bulb.id,
                        bulb_name: bulb.name,
                        message: status.message().into(),
                        status,
                    },
                )
            }
        }))
        .buffer_unordered(self.client.outbound.available_permits().max(1))
        .collect::<Vec<_>>()
        .await;
        indexed.sort_by_key(|(index, _)| *index);
        let results: Vec<_> = indexed.into_iter().map(|(_, result)| result).collect();
        let succeeded = results
            .iter()
            .filter(|result| matches!(result.status, ResultCode::Success))
            .count();
        let result = Operation {
            operation: operation.into(),
            profile_id,
            started_at,
            completed_at: now(),
            summary: Summary {
                total: results.len(),
                succeeded,
                failed: results.len() - succeeded,
            },
            results,
        };
        if result.summary.failed > 0 {
            error!(
                operation,
                profile_id=?profile_id,
                target_count=result.summary.total,
                succeeded=result.summary.succeeded,
                failed=result.summary.failed,
                duration_ms=operation_started.elapsed().as_millis(),
                "control operation completed with failures"
            );
        } else {
            debug!(
                operation,
                profile_id=?profile_id,
                target_count=result.summary.total,
                succeeded=result.summary.succeeded,
                failed=result.summary.failed,
                duration_ms=operation_started.elapsed().as_millis(),
                "control operation completed"
            );
        }
        result
    }

    pub async fn apply(&self, profile_id: Uuid, ids: Vec<Uuid>) -> AppResult<Operation> {
        let _ = &self.profiles;
        Self::validate_ids(&ids)?;
        let _permit = self
            .admission
            .clone()
            .try_acquire_owned()
            .map_err(|_| AppError::TooMany)?;
        let snapshot_ids = ids.clone();
        let (profile, settings, bulbs) = self
            .bulbs
            .db
            .run(move |connection| {
                let transaction = connection
                    .transaction_with_behavior(TransactionBehavior::Deferred)
                    .map_err(|_| AppError::Internal)?;
                let profile = profile_repository::get(&transaction, profile_id)?;
                let settings = settings_repository::get(&transaction)?;
                let bulbs = snapshot_ids
                    .iter()
                    .map(|id| bulb_repository::get(&transaction, *id))
                    .collect::<AppResult<Vec<_>>>()?;
                transaction.commit().map_err(|_| AppError::Internal)?;
                Ok((profile, settings, bulbs))
            })
            .await?;
        let command = command::properties(
            Some(profile.dimmer),
            Some(&profile.mode),
            profile.rgb_color.as_deref(),
            profile.color_temperature_kelvin,
            settings.fade,
            settings.speed,
        );
        Ok(self
            .execute("apply_profile", Some(profile.id), bulbs, command)
            .await)
    }

    pub async fn reset(&self, ids: Vec<Uuid>) -> AppResult<Operation> {
        let _ = &self.settings;
        Self::validate_ids(&ids)?;
        let _permit = self
            .admission
            .clone()
            .try_acquire_owned()
            .map_err(|_| AppError::TooMany)?;
        let snapshot_ids = ids.clone();
        let (settings, bulbs) = self
            .bulbs
            .db
            .run(move |connection| {
                let transaction = connection
                    .transaction_with_behavior(TransactionBehavior::Deferred)
                    .map_err(|_| AppError::Internal)?;
                let settings = settings_repository::get(&transaction)?;
                let bulbs = snapshot_ids
                    .iter()
                    .map(|id| bulb_repository::get(&transaction, *id))
                    .collect::<AppResult<Vec<_>>>()?;
                transaction.commit().map_err(|_| AppError::Internal)?;
                Ok((settings, bulbs))
            })
            .await?;
        let command = command::properties(
            Some(settings.dimmer),
            Some(&settings.mode),
            settings.rgb_color.0.as_deref(),
            settings.color_temperature_kelvin.0,
            settings.fade,
            settings.speed,
        );
        Ok(self.execute("reset", None, bulbs, command).await)
    }

    pub async fn reset_all(&self) -> AppResult<Operation> {
        let _permit = self
            .admission
            .clone()
            .try_acquire_owned()
            .map_err(|_| AppError::TooMany)?;
        let (settings, bulbs) = self
            .settings
            .db
            .run(|connection| {
                let transaction = connection
                    .transaction_with_behavior(TransactionBehavior::Deferred)
                    .map_err(|_| AppError::Internal)?;
                let settings = settings_repository::get(&transaction)?;
                let bulbs = bulb_repository::list(&transaction)?;
                transaction.commit().map_err(|_| AppError::Internal)?;
                Ok((settings, bulbs))
            })
            .await?;
        let command = command::properties(
            Some(settings.dimmer),
            Some(&settings.mode),
            settings.rgb_color.0.as_deref(),
            settings.color_temperature_kelvin.0,
            settings.fade,
            settings.speed,
        );
        Ok(self.execute("reset_all", None, bulbs, command).await)
    }

    pub async fn set_properties(
        &self,
        ids: Vec<Uuid>,
        dimmer: Option<i64>,
        rgb_color: Option<String>,
        color_temperature_kelvin: Option<i64>,
    ) -> AppResult<Operation> {
        Self::validate_ids(&ids)?;
        let _permit = self
            .admission
            .clone()
            .try_acquire_owned()
            .map_err(|_| AppError::TooMany)?;
        let (dimmer, rgb_color, color_temperature_kelvin) =
            crate::domain::validation::properties(dimmer, rgb_color, color_temperature_kelvin)?;
        let snapshot_ids = ids;
        let bulbs = self
            .bulbs
            .db
            .run(move |connection| {
                snapshot_ids
                    .iter()
                    .map(|id| bulb_repository::get(connection, *id))
                    .collect::<AppResult<Vec<_>>>()
            })
            .await?;
        let command = crate::adapters::tasmota::command::properties(
            dimmer,
            None,
            rgb_color.as_deref(),
            color_temperature_kelvin,
            None,
            None,
        );
        Ok(self.execute("set_properties", None, bulbs, command).await)
    }
}
