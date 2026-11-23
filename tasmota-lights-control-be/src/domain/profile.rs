use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LightInput {
    pub name: String,
    pub dimmer: i64,
    pub mode: String,
    pub rgb_color: Option<String>,
    pub color_temperature_kelvin: Option<i64>,
    pub mixed_color: Option<String>,
}
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: Uuid,
    pub name: String,
    pub dimmer: u8,
    pub mode: String,
    pub rgb_color: Option<String>,
    pub color_temperature_kelvin: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mixed_color: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
