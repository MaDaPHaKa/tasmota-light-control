use crate::error::{AppError, REQUEST_ID, RequestId};
use axum::{
    extract::{MatchedPath, Request},
    http::{HeaderName, HeaderValue},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::time::Instant;
use tracing::info;
use uuid::Uuid;

pub async fn request_id(mut request: Request, next: Next) -> Response {
    let value = request
        .headers()
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .filter(|value| value.len() <= 128 && value.bytes().all(|byte| byte.is_ascii_graphic()))
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    request.extensions_mut().insert(RequestId(value.clone()));
    let mut response = REQUEST_ID.scope(value.clone(), next.run(request)).await;
    if let Ok(value) = HeaderValue::from_str(&value) {
        response
            .headers_mut()
            .insert(HeaderName::from_static("x-request-id"), value);
    }
    response
}

pub async fn trace(request: Request, next: Next) -> Response {
    let started = Instant::now();
    let method = request.method().clone();
    let route = request
        .extensions()
        .get::<MatchedPath>()
        .map(MatchedPath::as_str)
        .unwrap_or("unmatched")
        .to_owned();
    let request_id = request
        .extensions()
        .get::<RequestId>()
        .map(|request_id| request_id.0.clone())
        .unwrap_or_else(|| "unknown".to_owned());
    let response = next.run(request).await;
    info!(
        %request_id,
        %method,
        %route,
        status = %response.status(),
        duration_ms = started.elapsed().as_millis(),
        "http request completed"
    );
    response
}

pub async fn normalize_rejection(request: Request, next: Next) -> Response {
    let response = next.run(request).await;
    if response.status().is_client_error()
        && response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .is_none_or(|value| !value.starts_with("application/json"))
    {
        AppError::BadRequest.into_response()
    } else {
        response
    }
}
