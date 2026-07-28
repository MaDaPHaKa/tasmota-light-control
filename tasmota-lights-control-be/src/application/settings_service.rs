use crate::{
    adapters::sqlite::settings_repository,
    domain::{
        settings::{Settings, SettingsInput},
        validation::settings,
    },
    error::AppResult,
    ports::repositories::Db,
};
use std::sync::Arc;
#[derive(Clone)]
pub struct SettingsService {
    pub db: Arc<Db>,
}
impl SettingsService {
    pub async fn get(&self) -> AppResult<Settings> {
        self.db
            .run(|connection| settings_repository::get(connection))
            .await
    }
    pub async fn save(&self, input: SettingsInput) -> AppResult<Settings> {
        let (dimmer, mode, rgb_color, color_temperature_kelvin, fade, speed) = settings(input)?;
        self.db
            .run(move |connection| {
                settings_repository::save(
                    connection,
                    Settings {
                        dimmer,
                        mode,
                        rgb_color: crate::domain::profile::RequiredOption(rgb_color),
                        color_temperature_kelvin: crate::domain::profile::RequiredOption(
                            color_temperature_kelvin,
                        ),
                        fade,
                        speed,
                    },
                )
            })
            .await
    }
}
