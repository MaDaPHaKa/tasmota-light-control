use crate::{
    adapters::sqlite::bulb_repository::database,
    domain::{
        profile::{LightInput, Profile},
        validation::{light, timestamp},
    },
    error::{AppError, AppResult},
};
use rusqlite::{Connection, Row, TransactionBehavior, params};
use std::collections::BTreeMap;
use uuid::Uuid;
pub fn profile_row(row: &Row<'_>) -> rusqlite::Result<Profile> {
    let id = row.get::<_, String>(0)?;
    let name = row.get::<_, String>(1)?;
    let dimmer = row.get::<_, u8>(2)?;
    let mode = row.get::<_, String>(3)?;
    let rgb_color = row.get::<_, Option<String>>(4)?;
    let color_temperature_kelvin = row.get::<_, Option<u16>>(5)?;
    let created_at = row.get::<_, String>(6)?;
    let updated_at = row.get::<_, String>(7)?;
    let mixed_color = row.get::<_, Option<String>>(8)?;
    if id.parse::<Uuid>().is_err()
        || light(LightInput {
            name: name.clone(),
            dimmer: i64::from(dimmer),
            mode: mode.clone(),
            rgb_color: rgb_color.clone(),
            color_temperature_kelvin: color_temperature_kelvin.map(i64::from),
            mixed_color: mixed_color.clone(),
        })
        .is_err()
        || !timestamp(&created_at)
        || !timestamp(&updated_at)
    {
        return Err(rusqlite::Error::InvalidQuery);
    }
    Ok(Profile {
        id: id.parse().map_err(|_| rusqlite::Error::InvalidQuery)?,
        name,
        dimmer,
        mode,
        rgb_color,
        color_temperature_kelvin,
        mixed_color,
        created_at,
        updated_at,
    })
}
pub fn list(connection: &Connection) -> AppResult<Vec<Profile>> {
    database(connection.prepare("SELECT id,name,dimmer,mode,rgb_color,color_temperature_kelvin,created_at,updated_at,mixed_color FROM profiles ORDER BY name_normalized,id")?.query_map([], profile_row)?.collect())
}
pub fn get(connection: &Connection, id: Uuid) -> AppResult<Profile> {
    match connection.query_row("SELECT id,name,dimmer,mode,rgb_color,color_temperature_kelvin,created_at,updated_at,mixed_color FROM profiles WHERE id=?1", [id.to_string()], profile_row) {
        Ok(profile) => Ok(profile),
        Err(rusqlite::Error::QueryReturnedNoRows) => Err(AppError::NotFound("profile")),
        Err(_) => Err(AppError::Internal),
    }
}
pub fn save(
    connection: &mut Connection,
    profile: &Profile,
    normalized: String,
) -> AppResult<Profile> {
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|_| AppError::Internal)?;
    let created_at = transaction
        .query_row(
            "SELECT created_at FROM profiles WHERE id=?1",
            [profile.id.to_string()],
            |row| row.get::<_, String>(0),
        )
        .ok();
    if let Err(error) = transaction.execute("INSERT INTO profiles(id,name,name_normalized,dimmer,mode,rgb_color,color_temperature_kelvin,created_at,updated_at,mixed_color) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10) ON CONFLICT(id) DO UPDATE SET name=excluded.name,name_normalized=excluded.name_normalized,dimmer=excluded.dimmer,mode=excluded.mode,rgb_color=excluded.rgb_color,color_temperature_kelvin=excluded.color_temperature_kelvin,updated_at=excluded.updated_at,mixed_color=excluded.mixed_color", params![profile.id.to_string(), profile.name, normalized, profile.dimmer, profile.mode, profile.rgb_color, profile.color_temperature_kelvin, created_at.as_deref().unwrap_or(&profile.created_at), profile.updated_at, profile.mixed_color]) {
        return if matches!(error, rusqlite::Error::SqliteFailure(error, _) if error.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_UNIQUE) {
            Err(AppError::Conflict(BTreeMap::from([("name".into(), "Name already exists".into())])))
        } else {
            Err(AppError::Internal)
        };
    }
    transaction.commit().map_err(|_| AppError::Internal)?;
    get(connection, profile.id)
}

pub fn delete(connection: &Connection, id: Uuid) -> AppResult<()> {
    if database(connection.execute("DELETE FROM profiles WHERE id=?1", [id.to_string()]))? == 0 {
        Err(AppError::NotFound("profile"))
    } else {
        Ok(())
    }
}
