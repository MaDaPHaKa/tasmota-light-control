use crate::domain::profile::RequiredOption;
use serde::{Deserialize, Serialize};
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Settings {
    pub dimmer: u8,
    pub mode: String,
    pub rgb_color: RequiredOption<String>,
    pub color_temperature_kelvin: RequiredOption<u16>,
    pub fade: Option<u8>,
    pub speed: Option<u8>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SettingsInput {
    pub dimmer: i64,
    pub mode: String,
    pub rgb_color: RequiredOption<String>,
    pub color_temperature_kelvin: RequiredOption<i64>,
    pub fade: Option<i64>,
    pub speed: Option<i64>,
}
