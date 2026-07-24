use crate::{
    adapters::{
        sqlite::{bulb_repository, profile_repository, settings_repository},
        tasmota::command::command,
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
        let started_at = now();
        let mut indexed = stream::iter(bulbs.into_iter().enumerate().map(|(index, bulb)| {
            let client = self.client.clone();
            let command = command.clone();
            async move {
                let status = match client.policy.endpoint(&bulb.ip_address, bulb.port) {
                    Ok(endpoint) => client.invoke(endpoint, command).await,
                    Err(_) => ResultCode::DeviceError,
                };
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
        Operation {
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
        }
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
        let (profile, bulbs) = self
            .bulbs
            .db
            .run(move |connection| {
                let transaction = connection
                    .transaction_with_behavior(TransactionBehavior::Deferred)
                    .map_err(|_| AppError::Internal)?;
                let profile = profile_repository::get(&transaction, profile_id)?;
                let bulbs = snapshot_ids
                    .iter()
                    .map(|id| bulb_repository::get(&transaction, *id))
                    .collect::<AppResult<Vec<_>>>()?;
                transaction.commit().map_err(|_| AppError::Internal)?;
                Ok((profile, bulbs))
            })
            .await?;
        let command = command(
            profile.dimmer,
            &profile.mode,
            profile.rgb_color.as_deref(),
            profile.color_temperature_kelvin,
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
        let command = command(
            settings.dimmer,
            &settings.mode,
            settings.rgb_color.0.as_deref(),
            settings.color_temperature_kelvin.0,
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
        let command = command(
            settings.dimmer,
            &settings.mode,
            settings.rgb_color.0.as_deref(),
            settings.color_temperature_kelvin.0,
        );
        Ok(self.execute("reset_all", None, bulbs, command).await)
    }
}
