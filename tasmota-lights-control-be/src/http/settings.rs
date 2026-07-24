use crate::{
    domain::settings::{Settings, SettingsInput},
    error::{ApiJson, AppResult},
    http::dto::AppState,
};
use axum::{Json, extract::State};
pub async fn get(State(state): State<AppState>) -> AppResult<Json<Settings>> {
    Ok(Json(state.settings.get().await?))
}
pub async fn put(
    State(state): State<AppState>,
    ApiJson(input): ApiJson<SettingsInput>,
) -> AppResult<Json<Settings>> {
    let settings = state.settings.save(input).await?;
    tracing::info!(
        action = "replace",
        resource = "reset_settings",
        result_code = "success",
        "CRUD completed"
    );
    Ok(Json(settings))
}
