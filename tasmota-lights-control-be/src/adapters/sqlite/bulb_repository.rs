use crate::{
    domain::{
        bulb::Bulb,
        validation::{name, timestamp},
    },
    error::{AppError, AppResult},
};
use rusqlite::{Connection, ErrorCode, Row, TransactionBehavior, params};
use uuid::Uuid;
pub fn database<T>(result: rusqlite::Result<T>) -> AppResult<T> {
    result.map_err(|error| match error {
        error if is_constraint(&error) => AppError::Conflict(Default::default()),
        _ => AppError::Internal,
    })
}

pub fn is_constraint(error: &rusqlite::Error) -> bool {
    matches!(
        error,
        rusqlite::Error::SqliteFailure(error, _) if error.code == ErrorCode::ConstraintViolation
    )
}
pub fn bulb_row(row: &Row<'_>) -> rusqlite::Result<Bulb> {
    let id = row.get::<_, String>(0)?;
    let name_value = row.get::<_, String>(1)?;
    let ip_address = row.get::<_, String>(2)?;
    let port = row.get::<_, u16>(3)?;
    let created_at = row.get::<_, String>(4)?;
    let updated_at = row.get::<_, String>(5)?;
    let valid = id.parse::<Uuid>().is_ok()
        && name(name_value.clone()).is_ok()
        && ip_address.parse::<std::net::Ipv4Addr>().is_ok()
        && timestamp(&created_at)
        && timestamp(&updated_at);
    if !valid {
        return Err(rusqlite::Error::InvalidQuery);
    }
    Ok(Bulb {
        id: id.parse().map_err(|_| rusqlite::Error::InvalidQuery)?,
        name: name_value,
        ip_address,
        port,
        created_at,
        updated_at,
    })
}
pub fn list(connection: &Connection) -> AppResult<Vec<Bulb>> {
    database(connection.prepare("SELECT id,name,ip_address,port,created_at,updated_at FROM bulbs ORDER BY name_normalized,id")?.query_map([], bulb_row)?.collect())
}
pub fn get(connection: &Connection, id: Uuid) -> AppResult<Bulb> {
    match connection.query_row(
        "SELECT id,name,ip_address,port,created_at,updated_at FROM bulbs WHERE id=?1",
        [id.to_string()],
        bulb_row,
    ) {
        Ok(bulb) => Ok(bulb),
        Err(rusqlite::Error::QueryReturnedNoRows) => Err(AppError::NotFound("bulb")),
        Err(_) => Err(AppError::Internal),
    }
}
pub fn save(connection: &mut Connection, bulb: &Bulb, normalized: String) -> AppResult<Bulb> {
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|_| AppError::Internal)?;
    let existing = transaction
        .query_row(
            "SELECT created_at FROM bulbs WHERE id=?1",
            [bulb.id.to_string()],
            |row| row.get::<_, String>(0),
        )
        .ok();
    if let Err(error) = transaction.execute("INSERT INTO bulbs(id,name,name_normalized,ip_address,port,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7) ON CONFLICT(id) DO UPDATE SET name=excluded.name,name_normalized=excluded.name_normalized,ip_address=excluded.ip_address,port=excluded.port,updated_at=excluded.updated_at", params![bulb.id.to_string(), bulb.name, normalized, bulb.ip_address, bulb.port, existing.as_deref().unwrap_or(&bulb.created_at), bulb.updated_at]) {
        return if is_constraint(&error) {
            Err(conflict(&transaction, bulb, &normalized))
        } else {
            Err(AppError::Internal)
        };
    }
    transaction.commit().map_err(|_| AppError::Internal)?;
    get(connection, bulb.id)
}
fn conflict(connection: &Connection, bulb: &Bulb, normalized: &str) -> AppError {
    let name_taken = connection
        .query_row(
            "SELECT 1 FROM bulbs WHERE name_normalized=?1 AND id<>?2",
            params![normalized, bulb.id.to_string()],
            |_| Ok(()),
        )
        .is_ok();
    let ip_taken = connection
        .query_row(
            "SELECT 1 FROM bulbs WHERE ip_address=?1 AND id<>?2",
            params![bulb.ip_address, bulb.id.to_string()],
            |_| Ok(()),
        )
        .is_ok();
    let mut fields = std::collections::BTreeMap::new();
    if name_taken {
        fields.insert("name".into(), "Name already exists".into());
    }
    if ip_taken {
        fields.insert("ipAddress".into(), "IP address already exists".into());
    }
    AppError::Conflict(fields)
}
pub fn delete(connection: &Connection, id: Uuid) -> AppResult<()> {
    if database(connection.execute("DELETE FROM bulbs WHERE id=?1", [id.to_string()]))? == 0 {
        Err(AppError::NotFound("bulb"))
    } else {
        Ok(())
    }
}

#[expect(dead_code)]
pub fn get_many_snapshot(connection: &mut Connection, ids: &[Uuid]) -> AppResult<Vec<Bulb>> {
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Deferred)
        .map_err(|_| AppError::Internal)?;
    let bulbs = ids
        .iter()
        .map(|id| get(&transaction, *id))
        .collect::<AppResult<Vec<_>>>()?;
    transaction.commit().map_err(|_| AppError::Internal)?;
    Ok(bulbs)
}

#[expect(dead_code)]
pub fn all_snapshot(connection: &mut Connection) -> AppResult<Vec<Bulb>> {
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Deferred)
        .map_err(|_| AppError::Internal)?;
    let bulbs = list(&transaction)?;
    transaction.commit().map_err(|_| AppError::Internal)?;
    Ok(bulbs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_returns_internal_for_corrupt_persisted_row() {
        let connection = Connection::open_in_memory().unwrap();
        connection.execute_batch("CREATE TABLE bulbs(id TEXT PRIMARY KEY,name TEXT,name_normalized TEXT,ip_address TEXT,port INTEGER,created_at TEXT,updated_at TEXT); INSERT INTO bulbs VALUES('00000000-0000-0000-0000-000000000000','Lamp','lamp','192.168.1.2',80,'bad','bad');").unwrap();

        assert!(matches!(
            get(&connection, Uuid::nil()),
            Err(AppError::Internal)
        ));
    }
}
