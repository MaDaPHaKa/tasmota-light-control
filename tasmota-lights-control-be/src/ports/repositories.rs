pub use crate::adapters::sqlite::executor::Db;

use crate::{
    domain::{bulb::Bulb, profile::Profile, settings::Settings},
    error::AppResult,
};
use uuid::Uuid;

#[expect(dead_code)]
pub struct ApplySnapshot {
    pub profile: Profile,
    pub bulbs: Vec<Bulb>,
}

#[expect(dead_code)]
pub struct ResetSnapshot {
    pub settings: Settings,
    pub bulbs: Vec<Bulb>,
}

#[expect(dead_code)]
pub fn apply_snapshot(
    connection: &mut rusqlite::Connection,
    profile_id: Uuid,
    bulb_ids: &[Uuid],
) -> AppResult<ApplySnapshot> {
    let transaction = connection
        .transaction_with_behavior(rusqlite::TransactionBehavior::Deferred)
        .map_err(|_| crate::error::AppError::Internal)?;
    let profile = crate::adapters::sqlite::profile_repository::get(&transaction, profile_id)?;
    let bulbs = bulb_ids
        .iter()
        .map(|id| crate::adapters::sqlite::bulb_repository::get(&transaction, *id))
        .collect::<AppResult<Vec<_>>>()?;
    transaction
        .commit()
        .map_err(|_| crate::error::AppError::Internal)?;
    Ok(ApplySnapshot { profile, bulbs })
}

#[expect(dead_code)]
pub fn reset_all_snapshot(connection: &mut rusqlite::Connection) -> AppResult<ResetSnapshot> {
    crate::adapters::sqlite::settings_repository::with_all_bulbs_snapshot(
        connection,
        |settings, bulbs| Ok(ResetSnapshot { settings, bulbs }),
    )
}
