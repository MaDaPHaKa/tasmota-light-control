use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BulbInput {
    pub name: String,
    pub ip_address: String,
    pub port: u16,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BulbEndpointInput {
    pub ip_address: String,
    pub port: u16,
}
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Bulb {
    pub id: Uuid,
    pub name: String,
    pub ip_address: String,
    pub port: u16,
    pub created_at: String,
    pub updated_at: String,
}
