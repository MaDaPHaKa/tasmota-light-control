use crate::{
    adapters::sqlite::bulb_repository::database,
    domain::{
        profile::RequiredOption, settings::Settings, validation::settings as validate_settings,
    },
    error::{AppError, AppResult},
};
use rusqlite::{Connection, TransactionBehavior, params};
pub fn get(connection: &Connection) -> AppResult<Settings> {
    let settings = database(connection.query_row("SELECT dimmer,mode,rgb_color,color_temperature_kelvin FROM reset_settings WHERE singleton=1", [], |row| Ok(Settings { dimmer: row.get(0)?, mode: row.get(1)?, rgb_color: RequiredOption(row.get(2)?), color_temperature_kelvin: RequiredOption(row.get(3)?) })))?;
    validate_settings(Settings {
        dimmer: settings.dimmer,
        mode: settings.mode.clone(),
        rgb_color: RequiredOption(settings.rgb_color.0.clone()),
        color_temperature_kelvin: RequiredOption(settings.color_temperature_kelvin.0),
    })
    .map_err(|_| AppError::Internal)?;
    Ok(settings)
}
pub fn save(connection: &mut Connection, settings: Settings) -> AppResult<Settings> {
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|_| AppError::Internal)?;
    if database(transaction.execute("UPDATE reset_settings SET dimmer=?1,mode=?2,rgb_color=?3,color_temperature_kelvin=?4 WHERE singleton=1", params![settings.dimmer, settings.mode, settings.rgb_color.0, settings.color_temperature_kelvin.0]))? != 1 {
        return Err(AppError::Internal);
    }
    transaction.commit().map_err(|_| AppError::Internal)?;
    get(connection)
}

#[expect(dead_code)]
pub fn with_all_bulbs_snapshot<T>(
    connection: &mut Connection,
    action: impl FnOnce(Settings, Vec<crate::domain::bulb::Bulb>) -> AppResult<T>,
) -> AppResult<T> {
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Deferred)
        .map_err(|_| AppError::Internal)?;
    let settings = get(&transaction)?;
    let bulbs = crate::adapters::sqlite::bulb_repository::list(&transaction)?;
    let result = action(settings, bulbs)?;
    transaction.commit().map_err(|_| AppError::Internal)?;
    Ok(result)
}
