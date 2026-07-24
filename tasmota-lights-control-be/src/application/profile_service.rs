use crate::{
    adapters::sqlite::profile_repository,
    domain::{
        profile::{LightInput, Profile},
        validation::{light, now},
    },
    error::AppResult,
    ports::repositories::Db,
};
use std::sync::Arc;
use uuid::Uuid;
#[derive(Clone)]
pub struct ProfileService {
    pub db: Arc<Db>,
}
impl ProfileService {
    pub async fn list(&self) -> AppResult<Vec<Profile>> {
        self.db
            .run(|connection| profile_repository::list(connection))
            .await
    }
    pub async fn get(&self, id: Uuid) -> AppResult<Profile> {
        self.db
            .run(move |connection| profile_repository::get(connection, id))
            .await
    }
    pub async fn save(&self, id: Option<Uuid>, input: LightInput) -> AppResult<Profile> {
        let (normalized, name, dimmer, mode, rgb_color, color_temperature_kelvin) = light(input)?;
        let timestamp = now();
        let profile = Profile {
            id: id.unwrap_or_else(Uuid::new_v4),
            name,
            dimmer,
            mode,
            rgb_color,
            color_temperature_kelvin,
            created_at: timestamp.clone(),
            updated_at: timestamp,
        };
        self.db
            .run(move |connection| profile_repository::save(connection, &profile, normalized))
            .await
    }
    pub async fn delete(&self, id: Uuid) -> AppResult<()> {
        self.db
            .run(move |connection| profile_repository::delete(connection, id))
            .await
    }
}
