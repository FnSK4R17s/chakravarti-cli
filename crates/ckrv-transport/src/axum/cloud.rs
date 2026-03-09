//! # Cloud Axum Routes
//!
//! Axum route wrappers for cloud handler.
//!
//! Uses `tokio::task::spawn_blocking` since the handler performs filesystem
//! reads to check API key configuration files.

use crate::error::TransportError;
use crate::handlers::cloud::get_cloud_status_handler;
use crate::state::AppState;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};

/// Get cloud status.
async fn get_cloud_status() -> impl IntoResponse {
    match tokio::task::spawn_blocking(get_cloud_status_handler).await {
        Ok(Ok(status)) => Json(status).into_response(),
        Ok(Err(e)) => e.into_response(),
        Err(e) => TransportError::Internal(format!("Task panicked: {e}")).into_response(),
    }
}

/// Create cloud routes.
pub fn routes() -> Router<AppState> {
    Router::new().route("/cloud", get(get_cloud_status))
}
