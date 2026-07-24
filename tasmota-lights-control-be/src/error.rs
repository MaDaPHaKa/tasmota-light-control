use axum::{
    Json,
    extract::{FromRequest, FromRequestParts, Path, Request},
    http::{HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use serde_json::json;
use std::collections::BTreeMap;
use uuid::Uuid;

tokio::task_local! { pub static REQUEST_ID: String; }

#[derive(Clone)]
pub struct RequestId(pub String);

#[derive(Debug)]
pub enum AppError {
    BadRequest,
    Validation(BTreeMap<String, String>),
    NotFound(&'static str),
    Conflict(BTreeMap<String, String>),
    TooMany,
    Unavailable,
    Internal,
}
pub type AppResult<T> = Result<T, AppError>;
impl From<rusqlite::Error> for AppError {
    fn from(_: rusqlite::Error) -> Self {
        Self::Internal
    }
}
pub fn validation(field: &str, message: &str) -> AppError {
    AppError::Validation(BTreeMap::from([(field.into(), message.into())]))
}
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message, fields) = match self {
            Self::BadRequest => (
                StatusCode::BAD_REQUEST,
                "bad_request",
                "Invalid request",
                None,
            ),
            Self::Validation(fields) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "validation_failed",
                "Request validation failed",
                Some(fields),
            ),
            Self::NotFound(resource) => (
                StatusCode::NOT_FOUND,
                "not_found",
                match resource {
                    "bulb" => "Bulb not found",
                    "profile" => "Profile not found",
                    _ => "Resource not found",
                },
                None,
            ),
            Self::Conflict(fields) => (
                StatusCode::CONFLICT,
                "conflict",
                "Resource conflicts with existing data",
                Some(fields),
            ),
            Self::TooMany => (
                StatusCode::TOO_MANY_REQUESTS,
                "too_many_requests",
                "Control operation already in progress",
                None,
            ),
            Self::Unavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "service_unavailable",
                "Service unavailable",
                None,
            ),
            Self::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "Unexpected internal failure",
                None,
            ),
        };
        let request_id = REQUEST_ID
            .try_with(Clone::clone)
            .unwrap_or_else(|_| Uuid::new_v4().to_string());
        let mut error = json!({"code":code,"message":message,"requestId":request_id});
        if let Some(fields) = fields {
            error["fields"] = json!(fields);
        }
        let mut response = (status, Json(json!({"error":error}))).into_response();
        if status == StatusCode::TOO_MANY_REQUESTS {
            response
                .headers_mut()
                .insert("retry-after", HeaderValue::from_static("1"));
        }
        if let Ok(header_value) = HeaderValue::from_str(&request_id) {
            response.headers_mut().insert("x-request-id", header_value);
        }
        response
    }
}
pub struct ApiJson<T>(pub T);
impl<S, T> FromRequest<S> for ApiJson<T>
where
    S: Send + Sync,
    T: serde::de::DeserializeOwned,
{
    type Rejection = AppError;
    async fn from_request(request: Request, state: &S) -> AppResult<Self> {
        Json::<T>::from_request(request, state)
            .await
            .map(|Json(value)| Self(value))
            .map_err(|_| AppError::BadRequest)
    }
}
pub struct ApiPath<T>(pub T);
impl<S, T> FromRequestParts<S> for ApiPath<T>
where
    S: Send + Sync,
    T: serde::de::DeserializeOwned + Send,
{
    type Rejection = AppError;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> AppResult<Self> {
        Path::<T>::from_request_parts(parts, state)
            .await
            .map(|Path(value)| Self(value))
            .map_err(|_| AppError::BadRequest)
    }
}
