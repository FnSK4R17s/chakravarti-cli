//! # Session Axum Routes
//!
//! Axum route wrappers for session handlers.
//!
//! Routes match frontend expectations:
//! - POST /session/start - Start session
//! - POST /session/exec - Execute in session
//! - POST /session/stop - Stop session

use crate::handlers::session::{
    exec_in_session_handler, start_session_handler, stop_session_handler, ExecRequest,
    StartSessionRequest, StopSessionRequest,
};
use crate::state::AppState;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Json, Router};

/// Start a new session.
async fn start_session(
    State(state): State<AppState>,
    Json(request): Json<StartSessionRequest>,
) -> impl IntoResponse {
    match start_session_handler(&state, request).await {
        Ok(session) => Json(session).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Execute in session.
async fn exec_in_session(
    State(state): State<AppState>,
    Json(request): Json<ExecRequest>,
) -> impl IntoResponse {
    match exec_in_session_handler(&state, request).await {
        Ok(result) => Json(result).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Stop a session.
async fn stop_session(
    State(state): State<AppState>,
    Json(request): Json<StopSessionRequest>,
) -> impl IntoResponse {
    match stop_session_handler(&state, request).await {
        Ok(()) => axum::http::StatusCode::NO_CONTENT.into_response(),
        Err(e) => e.into_response(),
    }
}

/// Create session routes.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/session/start", post(start_session))
        .route("/session/exec", post(exec_in_session))
        .route("/session/stop", post(stop_session))
}
