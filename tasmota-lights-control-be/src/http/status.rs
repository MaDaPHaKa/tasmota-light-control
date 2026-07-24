use crate::{
    domain::{
        bulb::BulbEndpointInput,
        control::{LiveState, TestResult},
    },
    error::{ApiJson, ApiPath, AppResult},
    http::dto::AppState,
};
use axum::{Json, extract::State};
pub async fn all(State(state): State<AppState>) -> AppResult<Json<Vec<LiveState>>> {
    Ok(Json(state.status.all().await?))
}
pub async fn one(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<uuid::Uuid>,
) -> AppResult<Json<LiveState>> {
    Ok(Json(state.status.one(id).await?))
}
pub async fn saved_test(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<uuid::Uuid>,
) -> AppResult<Json<TestResult>> {
    Ok(Json(state.status.test(state.bulbs.get(id).await?).await))
}
pub async fn unsaved_test(
    State(state): State<AppState>,
    ApiJson(input): ApiJson<BulbEndpointInput>,
) -> AppResult<Json<TestResult>> {
    let endpoint = state
        .status
        .client
        .policy
        .endpoint(&input.ip_address, input.port)?;
    let bulb = crate::domain::bulb::Bulb {
        id: uuid::Uuid::nil(),
        name: String::new(),
        ip_address: endpoint.ip.to_string(),
        port: endpoint.port,
        created_at: String::new(),
        updated_at: String::new(),
    };
    Ok(Json(state.status.test(bulb).await))
}
