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
        let (normalized, name, dimmer, mode, rgb_color, color_temperature_kelvin, mixed_color) =
            light(input)?;
        let timestamp = now();
        let profile = Profile {
            id: id.unwrap_or_else(Uuid::new_v4),
            name,
            dimmer,
            mode,
            rgb_color,
            color_temperature_kelvin,
            mixed_color,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AppError;
    use serde_json::json;

    #[tokio::test]
    async fn save_accepts_legacy_mixed_profile_and_round_trips_exact_hex() {
        let db = Arc::new(Db::open(":memory:".into(), 1000).unwrap());
        let service = ProfileService { db: db.clone() };
        let legacy = json!({
            "name": "test mixed", "dimmer": 100, "mode": "mixed",
            "rgbColor": "#88C0D0", "colorTemperatureKelvin": 3000
        });
        let profile = service
            .save(None, serde_json::from_value(legacy.clone()).unwrap())
            .await
            .unwrap();
        assert_eq!(profile.mixed_color.as_deref(), Some("#88C0D040BF"));
        assert_eq!(
            (profile.rgb_color, profile.color_temperature_kelvin),
            (None, None)
        );

        let modern =
            json!({"name":"test mixed", "dimmer":72, "mode":"mixed", "mixedColor":"88c0d03719"});
        service
            .save(Some(profile.id), serde_json::from_value(modern).unwrap())
            .await
            .unwrap();
        let saved = service.get(profile.id).await.unwrap();
        assert_eq!(saved.mixed_color.as_deref(), Some("#88C0D03719"));
        assert_eq!(saved.created_at, profile.created_at);
        assert_eq!(saved.dimmer, 72);
        assert!(matches!(
            service.save(None, serde_json::from_value(legacy).unwrap()).await,
            Err(AppError::Conflict(fields)) if fields.contains_key("name")
        ));
        db.shutdown().await.unwrap();
    }
}
