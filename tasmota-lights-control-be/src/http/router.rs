use crate::http::{
    bulbs, control, direct_links,
    dto::AppState,
    health,
    middleware::{normalize_rejection, request_id, trace},
    profiles, settings, status,
};
use axum::{
    Router,
    extract::DefaultBodyLimit,
    middleware,
    routing::{get, post},
};

use crate::error::AppError;

pub fn router(state: AppState, body_limit: usize) -> Router {
    let api = Router::new()
        .route("/bulbs", get(bulbs::list).post(bulbs::create))
        .route("/bulbs/test", post(status::unsaved_test))
        .route("/bulbs/status", get(status::all))
        .route(
            "/bulbs/{bulbId}",
            get(bulbs::get).put(bulbs::replace).delete(bulbs::delete),
        )
        .route("/bulbs/{bulbId}/test", post(status::saved_test))
        .route("/bulbs/{bulbId}/status", get(status::one))
        .route("/profiles", get(profiles::list).post(profiles::create))
        .route(
            "/profiles/{profileId}",
            get(profiles::get)
                .put(profiles::replace)
                .delete(profiles::delete),
        )
        .route("/actions/apply-profile", post(control::apply))
        .route("/actions/reset", post(control::reset))
        .route("/actions/reset-all", post(control::reset_all))
        .route("/direct-links", post(direct_links::generate))
        .route("/settings/reset", get(settings::get).put(settings::put))
        .route("/health/live", get(health::live))
        .route("/health/ready", get(health::ready))
        .fallback(|| async { AppError::NotFound("resource") });
    Router::new()
        .nest("/api/v1", api)
        .layer(DefaultBodyLimit::max(body_limit))
        .layer(middleware::from_fn(normalize_rejection))
        .layer(middleware::from_fn(trace))
        .layer(middleware::from_fn(request_id))
        .with_state(state)
}
