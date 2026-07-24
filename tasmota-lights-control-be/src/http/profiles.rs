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
    Ok((
        StatusCode::CREATED,
        Json(state.profiles.save(None, input).await?),
    ))
}
pub async fn replace(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<uuid::Uuid>,
    ApiJson(input): ApiJson<LightInput>,
) -> AppResult<Json<Profile>> {
    Ok(Json(state.profiles.save(Some(id), input).await?))
}
pub async fn delete(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<uuid::Uuid>,
) -> AppResult<StatusCode> {
    state.profiles.delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}
