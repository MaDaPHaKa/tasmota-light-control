use crate::domain::validation::now;
use rusqlite::{Connection, TransactionBehavior};
const MIGRATION: &str = include_str!("../../../migrations/0001_initial.sql");
const MIGRATION_2: &str = include_str!("../../../migrations/0002_fade_speed.sql");
pub fn migrate(connection: &mut Connection) -> Result<(), String> {
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|error| error.to_string())?;
    transaction.execute_batch("CREATE TABLE IF NOT EXISTS schema_migrations(version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL);").map_err(|error| error.to_string())?;
    let version: Option<i64> = transaction
        .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
            row.get(0)
        })
        .map_err(|error| error.to_string())?;
    if version.unwrap_or(0) > 2 {
        return Err("Database version is newer than binary".into());
    }
    if version.unwrap_or(0) < 1 {
        transaction
            .execute_batch(MIGRATION)
            .map_err(|error| error.to_string())?;
        transaction
            .execute(
                "INSERT INTO schema_migrations(version,applied_at) VALUES(1,?1)",
                [now()],
            )
            .map_err(|error| error.to_string())?;
    }
    if version.unwrap_or(0) < 2 {
        transaction
            .execute_batch(MIGRATION_2)
            .map_err(|error| error.to_string())?;
        transaction
            .execute(
                "INSERT INTO schema_migrations(version,applied_at) VALUES(2,?1)",
                [now()],
            )
            .map_err(|error| error.to_string())?;
    }
    transaction.commit().map_err(|error| error.to_string())?;
    Ok(())
}
