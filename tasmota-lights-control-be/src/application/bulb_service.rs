use crate::{
    adapters::sqlite::bulb_repository,
    domain::{
        bulb::{Bulb, BulbInput},
        validation::{name, now},
    },
    error::AppResult,
    ports::{bulb_controller::PolicyState, repositories::Db},
};
use std::sync::Arc;
use uuid::Uuid;
#[derive(Clone)]
pub struct BulbService {
    pub db: Arc<Db>,
    pub policy: Arc<PolicyState>,
}
impl BulbService {
    pub async fn list(&self) -> AppResult<Vec<Bulb>> {
        self.db
            .run(|connection| bulb_repository::list(connection))
            .await
    }
    pub async fn get(&self, id: Uuid) -> AppResult<Bulb> {
        self.db
            .run(move |connection| bulb_repository::get(connection, id))
            .await
    }
    pub async fn save(&self, id: Option<Uuid>, input: BulbInput) -> AppResult<Bulb> {
        let endpoint = self.policy.endpoint(&input.ip_address, input.port)?;
        let (normalized, name) = name(input.name)?;
        let timestamp = now();
        let bulb = Bulb {
            id: id.unwrap_or_else(Uuid::new_v4),
            name,
            ip_address: endpoint.ip.to_string(),
            port: endpoint.port,
            created_at: timestamp.clone(),
            updated_at: timestamp,
        };
        self.db
            .run(move |connection| bulb_repository::save(connection, &bulb, normalized))
            .await
    }
    pub async fn delete(&self, id: Uuid) -> AppResult<()> {
        self.db
            .run(move |connection| bulb_repository::delete(connection, id))
            .await
    }
}
