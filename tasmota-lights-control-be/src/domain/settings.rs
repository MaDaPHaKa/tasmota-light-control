use crate::domain::profile::RequiredOption;
use serde::{Deserialize, Serialize};
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Settings {
    pub dimmer: u8,
    pub mode: String,
    pub rgb_color: RequiredOption<String>,
    pub color_temperature_kelvin: RequiredOption<u16>,
}
