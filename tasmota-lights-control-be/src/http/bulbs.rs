use crate::{
    domain::bulb::{Bulb, BulbInput},
    error::{ApiJson, ApiPath, AppResult},
    http::dto::AppState,
};
use axum::{Json, extract::State, http::StatusCode};
pub async fn list(State(state): State<AppState>) -> AppResult<Json<Vec<Bulb>>> {
    Ok(Json(state.bulbs.list().await?))
}
pub async fn get(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<uuid::Uuid>,
) -> AppResult<Json<Bulb>> {
    Ok(Json(state.bulbs.get(id).await?))
}
pub async fn create(
    State(state): State<AppState>,
    ApiJson(input): ApiJson<BulbInput>,
) -> AppResult<(StatusCode, Json<Bulb>)> {
    Ok((
        StatusCode::CREATED,
        Json(state.bulbs.save(None, input).await?),
    ))
}
pub async fn replace(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<uuid::Uuid>,
    ApiJson(input): ApiJson<BulbInput>,
) -> AppResult<Json<Bulb>> {
    Ok(Json(state.bulbs.save(Some(id), input).await?))
}
pub async fn delete(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<uuid::Uuid>,
) -> AppResult<StatusCode> {
    state.bulbs.delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}
