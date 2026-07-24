use crate::{
    domain::control::Link,
    error::{ApiJson, AppResult},
    http::dto::AppState,
};
use axum::{
    extract::State,
    response::{IntoResponse, Response},
};
pub async fn generate(
    State(state): State<AppState>,
    ApiJson(input): ApiJson<Link>,
) -> AppResult<Response> {
    Ok((
        [("content-type", "text/plain; charset=utf-8")],
        state
            .direct_links
            .generate(input.bulb_id, input.profile_id)
            .await?,
    )
        .into_response())
}
