use crate::{
    adapters::sqlite::bulb_repository::database,
    error::{AppError, AppResult},
    http::dto::AppState,
};
use axum::{Json, extract::State};
use serde_json::{Value, json};
use std::sync::atomic::Ordering;
pub async fn live() -> Json<Value> {
    Json(json!({"status":"ok"}))
}
pub async fn ready(State(state): State<AppState>) -> AppResult<Json<Value>> {
    if state.shutting_down.load(Ordering::Acquire) || !state.bulbs.db.is_available() {
        return Err(AppError::Unavailable);
    }
    state
        .bulbs
        .db
        .run(|connection| {
            database(
                connection
                    .query_row("SELECT 1", [], |row| row.get::<_, i64>(0))
                    .map(|_| ()),
            )
        })
        .await
        .map_err(|_| AppError::Unavailable)?;
    Ok(Json(json!({"status":"ok"})))
}
