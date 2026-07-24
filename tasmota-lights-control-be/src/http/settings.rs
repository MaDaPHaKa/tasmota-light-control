use crate::{
    domain::settings::Settings,
    error::{ApiJson, AppResult},
    http::dto::AppState,
};
use axum::{Json, extract::State};
pub async fn get(State(state): State<AppState>) -> AppResult<Json<Settings>> {
    Ok(Json(state.settings.get().await?))
}
pub async fn put(
    State(state): State<AppState>,
    ApiJson(input): ApiJson<Settings>,
) -> AppResult<Json<Settings>> {
    Ok(Json(state.settings.save(input).await?))
}
