use crate::{
    domain::profile::{LightInput, Profile},
    error::{ApiJson, ApiPath, AppResult},
    http::dto::AppState,
};
use axum::{Json, extract::State, http::StatusCode};
pub async fn list(State(state): State<AppState>) -> AppResult<Json<Vec<Profile>>> {
    Ok(Json(state.profiles.list().await?))
}
pub async fn get(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<uuid::Uuid>,
) -> AppResult<Json<Profile>> {
    Ok(Json(state.profiles.get(id).await?))
}
pub async fn create(
    State(state): State<AppState>,
    ApiJson(input): ApiJson<LightInput>,
) -> AppResult<(StatusCode, Json<Profile>)> {
    let profile = state.profiles.save(None, input).await?;
    tracing::info!(action="create", resource="profile", resource_id=%profile.id, result_code="success", "CRUD completed");
    Ok((StatusCode::CREATED, Json(profile)))
}
pub async fn replace(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<uuid::Uuid>,
    ApiJson(input): ApiJson<LightInput>,
) -> AppResult<Json<Profile>> {
    let profile = state.profiles.save(Some(id), input).await?;
    tracing::info!(action="replace", resource="profile", resource_id=%id, result_code="success", "CRUD completed");
    Ok(Json(profile))
}
pub async fn delete(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<uuid::Uuid>,
) -> AppResult<StatusCode> {
    state.profiles.delete(id).await?;
    tracing::info!(action="delete", resource="profile", resource_id=%id, result_code="success", "CRUD completed");
    Ok(StatusCode::NO_CONTENT)
}
