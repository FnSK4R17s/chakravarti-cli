//! # Docker Axum Routes
//!
//! Axum route wrappers for Docker handler.
//!
//! Uses `tokio::task::spawn_blocking` since the handler runs `docker info`
//! via `std::process::Command`.

use crate::error::TransportError;
use crate::handlers::docker::check_docker_handler;
use crate::state::AppState;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};

/// Check Docker status.
async fn check_docker() -> impl IntoResponse {
    match tokio::task::spawn_blocking(check_docker_handler).await {
        Ok(Ok(status)) => Json(status).into_response(),
        Ok(Err(e)) => e.into_response(),
        Err(e) => TransportError::Internal(format!("Task panicked: {e}")).into_response(),
    }
}

/// Create Docker routes.
pub fn routes() -> Router<AppState> {
    Router::new().route("/docker", get(check_docker))
}
