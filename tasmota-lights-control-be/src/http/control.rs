use crate::{
    domain::control::{Apply, Ids, Operation},
    error::{ApiJson, AppResult},
    http::dto::AppState,
};
use axum::{Json, extract::State};
pub async fn apply(
    State(state): State<AppState>,
    ApiJson(input): ApiJson<Apply>,
) -> AppResult<Json<Operation>> {
    Ok(Json(
        state
            .control
            .apply(input.profile_id, input.bulb_ids)
            .await?,
    ))
}
pub async fn reset(
    State(state): State<AppState>,
    ApiJson(input): ApiJson<Ids>,
) -> AppResult<Json<Operation>> {
    Ok(Json(state.control.reset(input.bulb_ids).await?))
}
pub async fn reset_all(State(state): State<AppState>) -> AppResult<Json<Operation>> {
    Ok(Json(state.control.reset_all().await?))
}
