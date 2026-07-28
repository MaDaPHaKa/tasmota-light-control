use crate::{
    domain::bulb::{Bulb, BulbInput},
    error::{ApiJson, ApiPath, AppResult},
    http::dto::AppState,
};
use axum::{Json, extract::State, http::StatusCode};
use tracing::debug;
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
    let bulb = state.bulbs.save(None, input).await?;
    debug!(action="create", resource="bulb", resource_id=%bulb.id, result_code="success", "CRUD completed");
    Ok((StatusCode::CREATED, Json(bulb)))
}
pub async fn replace(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<uuid::Uuid>,
    ApiJson(input): ApiJson<BulbInput>,
) -> AppResult<Json<Bulb>> {
    let bulb = state.bulbs.save(Some(id), input).await?;
    debug!(action="replace", resource="bulb", resource_id=%id, result_code="success", "CRUD completed");
    Ok(Json(bulb))
}
pub async fn delete(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<uuid::Uuid>,
) -> AppResult<StatusCode> {
    state.bulbs.delete(id).await?;
    debug!(action="delete", resource="bulb", resource_id=%id, result_code="success", "CRUD completed");
    Ok(StatusCode::NO_CONTENT)
}
