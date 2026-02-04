//! # Status Axum Routes
//!
//! Axum route wrappers for status handler.

use crate::handlers::status::get_status_handler;
use crate::state::AppState;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};

/// Get system status.
async fn get_status(State(state): State<AppState>) -> impl IntoResponse {
    match get_status_handler(&state).await {
        Ok(status) => Json(status).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Create status routes.
pub fn routes() -> Router<AppState> {
    Router::new().route("/status", get(get_status))
}
