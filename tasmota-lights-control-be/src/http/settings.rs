use crate::{
    domain::settings::{BulbSettings, BulbSettingsInput, Settings, SettingsInput},
    error::{ApiJson, ApiPath, AppResult},
    http::dto::AppState,
};
use axum::{Json, extract::State, http::header::CACHE_CONTROL, response::IntoResponse};
use tracing::debug;
use uuid::Uuid;
pub async fn get(State(state): State<AppState>) -> AppResult<Json<Settings>> {
    Ok(Json(state.settings.get().await?))
}
pub async fn put(
    State(state): State<AppState>,
    ApiJson(input): ApiJson<SettingsInput>,
) -> AppResult<Json<Settings>> {
    let settings = state.settings.save(input).await?;
    debug!(
        action = "replace",
        resource = "reset_settings",
        result_code = "success",
        "CRUD completed"
    );
    Ok(Json(settings))
}

pub async fn get_bulb(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<Uuid>,
) -> AppResult<impl IntoResponse> {
    let settings = state.control.bulb_settings(id, None).await?;
    Ok(([(CACHE_CONTROL, "no-store")], Json(settings)))
}

pub async fn put_bulb(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<Uuid>,
    ApiJson(input): ApiJson<BulbSettingsInput>,
) -> AppResult<Json<BulbSettings>> {
    Ok(Json(state.control.bulb_settings(id, Some(input)).await?))
}
