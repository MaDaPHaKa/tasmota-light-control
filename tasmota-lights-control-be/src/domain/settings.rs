use serde::{Deserialize, Serialize};
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Settings {
    pub dimmer: u8,
    pub mode: String,
    pub rgb_color: Option<String>,
    pub color_temperature_kelvin: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mixed_color: Option<String>,
    pub fade: Option<u8>,
    pub speed: Option<u8>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SettingsInput {
    pub dimmer: i64,
    pub mode: String,
    pub rgb_color: Option<String>,
    pub color_temperature_kelvin: Option<i64>,
    pub mixed_color: Option<String>,
    pub fade: Option<i64>,
    pub speed: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct BulbSettings {
    pub fade: u8,
    pub speed: u8,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BulbSettingsInput {
    pub fade: i64,
    pub speed: i64,
}
