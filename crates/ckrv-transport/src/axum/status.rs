//! # Status Axum Routes
//!
//! Axum route wrappers for status handler.
//!
//! Uses `tokio::task::spawn_blocking` since the handler performs synchronous
//! Git CLI operations and blocking lock reads.

use crate::error::TransportError;
use crate::handlers::status::get_status_handler;
use crate::state::AppState;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};

/// Get system status.
async fn get_status(State(state): State<AppState>) -> impl IntoResponse {
    match tokio::task::spawn_blocking(move || get_status_handler(&state)).await {
        Ok(Ok(status)) => Json(status).into_response(),
        Ok(Err(e)) => e.into_response(),
        Err(e) => TransportError::Internal(format!("Task panicked: {e}")).into_response(),
    }
}

/// Create status routes.
pub fn routes() -> Router<AppState> {
    Router::new().route("/status", get(get_status))
}
