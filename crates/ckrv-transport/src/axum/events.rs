//! # Events Axum Routes
//!
//! Axum SSE route for real-time events.
//!
//! `get_default_branch` uses `tokio::task::spawn_blocking` for the synchronous
//! Git CLI call. The SSE heartbeat stream is already async.
//!
//! Routes match frontend expectations:
//! - GET /events - SSE stream for real-time events
//! - GET /git/default-branch - Get default git branch

use crate::error::TransportError;
use crate::state::AppState;
use axum::extract::State;
use axum::response::sse::{Event, Sse};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use std::convert::Infallible;
use std::time::Duration;
use tokio_stream::wrappers::IntervalStream;
use tokio_stream::StreamExt as _;

/// SSE event stream.
async fn sse_handler(
    State(_state): State<AppState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let stream = IntervalStream::new(tokio::time::interval(Duration::from_secs(30)))
        .map(|_| Ok(Event::default().event("heartbeat").data("ping")));

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}

/// Get default git branch.
async fn get_default_branch(State(state): State<AppState>) -> impl IntoResponse {
    match tokio::task::spawn_blocking(move || {
        std::process::Command::new("git")
            .args(["symbolic-ref", "--short", "refs/remotes/origin/HEAD"])
            .current_dir(&state.project_root)
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    String::from_utf8(o.stdout).ok()
                } else {
                    None
                }
            })
            .map(|s| s.trim().replace("origin/", ""))
            .unwrap_or_else(|| "main".to_string())
    })
    .await
    {
        Ok(default_branch) => Json(serde_json::json!({
            "branch": default_branch
        }))
        .into_response(),
        Err(e) => TransportError::Internal(format!("Task panicked: {e}")).into_response(),
    }
}

/// Create events routes.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/events", get(sse_handler))
        .route("/git/default-branch", get(get_default_branch))
}
