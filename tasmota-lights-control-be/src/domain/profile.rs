use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LightInput {
    pub name: String,
    pub dimmer: u8,
    pub mode: String,
    pub rgb_color: RequiredOption<String>,
    pub color_temperature_kelvin: RequiredOption<u16>,
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
    pub created_at: String,
    pub updated_at: String,
}
#[derive(Serialize)]
#[serde(transparent)]
pub struct RequiredOption<T>(pub Option<T>);
impl<'de, T: Deserialize<'de>> Deserialize<'de> for RequiredOption<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Option::deserialize(deserializer).map(Self)
    }
}
