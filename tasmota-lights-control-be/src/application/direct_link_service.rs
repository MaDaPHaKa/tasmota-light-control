use crate::{
    adapters::{
        sqlite::{bulb_repository, profile_repository},
        tasmota::command::command,
    },
    application::{bulb_service::BulbService, profile_service::ProfileService},
    error::{AppError, AppResult},
    ports::bulb_controller::PolicyState,
};
use rusqlite::TransactionBehavior;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct DirectLinkService {
    pub bulbs: Arc<BulbService>,
    pub profiles: Arc<ProfileService>,
    pub policy: Arc<PolicyState>,
}

impl DirectLinkService {
    pub async fn generate(&self, bulb_id: Uuid, profile_id: Uuid) -> AppResult<String> {
        let _ = &self.profiles;
        let (bulb, profile) = self
            .bulbs
            .db
            .run(move |connection| {
                let transaction = connection
                    .transaction_with_behavior(TransactionBehavior::Deferred)
                    .map_err(|_| AppError::Internal)?;
                let bulb = bulb_repository::get(&transaction, bulb_id)?;
                let profile = profile_repository::get(&transaction, profile_id)?;
                transaction.commit().map_err(|_| AppError::Internal)?;
                Ok((bulb, profile))
            })
            .await?;
        let endpoint = self
            .policy
            .endpoint(&bulb.ip_address, i64::from(bulb.port))?;
        let command = command(
            profile.dimmer,
            &profile.mode,
            profile.rgb_color.as_deref(),
            profile.color_temperature_kelvin,
        );
        Ok(crate::adapters::tasmota::client::endpoint_url(&endpoint, &command)?.to_string())
    }
}
