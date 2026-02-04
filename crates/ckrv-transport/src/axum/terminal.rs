//! # Terminal Axum Routes
//!
//! Axum route wrappers for terminal handlers.
//!
//! Routes match frontend expectations:
//! - POST /terminal/start - Start terminal session
//! - GET /terminal/ws - WebSocket connection
//! - POST /terminal/stop - Stop terminal session

use crate::handlers::terminal::{
    handle_terminal_ws, start_terminal_handler, stop_terminal_handler, StartTerminalRequest,
    StopTerminalRequest,
};
use crate::state::AppState;
use axum::extract::{Query, State, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;

/// Start terminal session.
async fn start_terminal(
    State(state): State<AppState>,
    Json(request): Json<StartTerminalRequest>,
) -> impl IntoResponse {
    match start_terminal_handler(&state, request).await {
        Ok(response) => Json(response).into_response(),
        Err(e) => e.into_response(),
    }
}

/// WebSocket query params.
#[derive(Deserialize)]
struct WsQuery {
    session_id: String,
}

/// Terminal WebSocket upgrade.
async fn terminal_ws(
    State(_state): State<AppState>,
    Query(query): Query<WsQuery>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_terminal_ws(socket, query.session_id))
}

/// Stop terminal session.
async fn stop_terminal(
    State(_state): State<AppState>,
    Json(request): Json<StopTerminalRequest>,
) -> impl IntoResponse {
    match stop_terminal_handler(request).await {
        Ok(response) => Json(response).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Create terminal routes.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/terminal/start", post(start_terminal))
        .route("/terminal/ws", get(terminal_ws))
        .route("/terminal/stop", post(stop_terminal))
}
