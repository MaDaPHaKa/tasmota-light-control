use crate::domain::validation::now;
use rusqlite::{Connection, TransactionBehavior};
const MIGRATION: &str = include_str!("../../../migrations/0001_initial.sql");
const MIGRATION_2: &str = include_str!("../../../migrations/0002_fade_speed.sql");
const MIGRATION_3: &str =
    include_str!("../../../migrations/0003_color_temperature_lower_limit.sql");
const MIGRATION_4: &str = include_str!("../../../migrations/0004_mixed_color.sql");
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
    if version.unwrap_or(0) > 4 {
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
    if version.unwrap_or(0) < 3 {
        transaction
            .execute_batch(MIGRATION_3)
            .map_err(|error| error.to_string())?;
        transaction
            .execute(
                "INSERT INTO schema_migrations(version,applied_at) VALUES(3,?1)",
                [now()],
            )
            .map_err(|error| error.to_string())?;
    }
    if version.unwrap_or(0) < 4 {
        transaction
            .execute_batch(MIGRATION_4)
            .map_err(|error| error.to_string())?;
        transaction
            .execute(
                "INSERT INTO schema_migrations(version,applied_at) VALUES(4,?1)",
                [now()],
            )
            .map_err(|error| error.to_string())?;
    }
    transaction.commit().map_err(|error| error.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::sqlite::{profile_repository, settings_repository};

    #[test]
    fn migration_preserves_existing_profiles_and_reset_settings() {
        let mut connection = Connection::open_in_memory().unwrap();
        for sql in [MIGRATION, MIGRATION_2, MIGRATION_3] {
            connection.execute_batch(sql).unwrap();
        }
        connection.execute_batch("\
            CREATE TABLE schema_migrations(version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL);\
            INSERT INTO schema_migrations VALUES(1,'old'),(2,'old'),(3,'old');\
            INSERT INTO profiles VALUES\
                ('00000000-0000-0000-0000-000000000001','RGB','rgb',80,'rgb','#88C0D0',NULL,'2026-01-01T00:00:00Z','2026-01-02T00:00:00Z'),\
                ('00000000-0000-0000-0000-000000000002','White','white',90,'color_temperature',NULL,2100,'2026-01-01T00:00:00Z','2026-01-02T00:00:00Z');\
            UPDATE reset_settings SET dimmer=50, color_temperature_kelvin=2300, fade=0, speed=19;\
        ").unwrap();

        migrate(&mut connection).unwrap();
        migrate(&mut connection).unwrap();

        let profiles = profile_repository::list(&connection).unwrap();
        assert_eq!(profiles.len(), 2);
        assert_eq!(profiles[0].rgb_color.as_deref(), Some("#88C0D0"));
        assert_eq!(profiles[0].updated_at, "2026-01-02T00:00:00Z");
        assert_eq!(profiles[1].color_temperature_kelvin, Some(2100));
        assert!(profiles.iter().all(|profile| profile.mixed_color.is_none()));
        let settings = settings_repository::get(&connection).unwrap();
        assert_eq!(settings.color_temperature_kelvin, Some(2300));
        assert_eq!(
            (settings.dimmer, settings.fade, settings.speed),
            (50, Some(0), Some(19))
        );
        assert_eq!(settings.mixed_color, None);
    }
}
