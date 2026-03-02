//! # Docker Axum Routes
//!
//! Axum route wrappers for Docker handler.

use crate::handlers::docker::check_docker_handler;
use crate::state::AppState;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};

/// Check Docker status.
async fn check_docker() -> impl IntoResponse {
    match check_docker_handler() {
        Ok(status) => Json(status).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Create Docker routes.
pub fn routes() -> Router<AppState> {
    Router::new().route("/docker", get(check_docker))
}
