//! # Cloud Axum Routes
//!
//! Axum route wrappers for cloud handler.

use crate::handlers::cloud::get_cloud_status_handler;
use crate::state::AppState;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};

/// Get cloud status.
async fn get_cloud_status() -> impl IntoResponse {
    match get_cloud_status_handler() {
        Ok(status) => Json(status).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Create cloud routes.
pub fn routes() -> Router<AppState> {
    Router::new().route("/cloud", get(get_cloud_status))
}
