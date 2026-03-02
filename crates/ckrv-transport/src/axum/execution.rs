//! # Execution Axum Routes
//!
//! Axum route wrappers for execution handlers.
//!
//! Routes match frontend expectations:
//! - POST /execution/start - Start execution
//! - GET /execution/ws - WebSocket for execution logs
//! - POST /execution/stop - Stop execution
//! - GET /execution/status - Get execution status
//! - GET /execution/branches - Get worktree branches
//! - POST /execution/merge - Merge single branch
//! - POST /execution/merge-all - Merge all branches
//! - GET /execution/{id}/logs - Get execution logs
//! - GET /execution/{id}/logs/tail - Tail execution logs

use crate::handlers::execution::{
    get_execution_status_handler, get_logs_handler, list_branches_handler,
    merge_all_branches_handler, merge_branch_handler, start_execution_handler,
    stop_execution_handler, tail_logs_handler, ExecuteRequest, ListBranchesRequest,
    LogHistoryParams, LogTailParams, MergeAllRequest, MergeBranchRequest, StopRequest,
};
use crate::state::AppState;
use axum::extract::{Path, Query, State, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use tokio::sync::broadcast;

/// Start execution.
async fn start_execution(
    State(state): State<AppState>,
    Json(request): Json<ExecuteRequest>,
) -> impl IntoResponse {
    match start_execution_handler(&state, request) {
        Ok(result) => Json(result).into_response(),
        Err(e) => e.into_response(),
    }
}

/// WebSocket query params.
#[derive(Deserialize)]
struct WsQuery {
    run_id: String,
}

/// Execution WebSocket for streaming logs.
///
/// Subscribes to the Hub and forwards all orchestration events to the client.
/// The connection stays open until the client disconnects or a Success/Error event is received.
async fn execution_ws(
    State(state): State<AppState>,
    Query(query): Query<WsQuery>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    let run_id = query.run_id;
    let hub = state.hub;

    ws.on_upgrade(move |socket| async move {
        handle_execution_ws(socket, hub, run_id).await;
    })
}

/// Handle the WebSocket connection for execution log streaming.
async fn handle_execution_ws(
    socket: axum::extract::ws::WebSocket,
    hub: crate::hub::SharedHub,
    _run_id: String,
) {
    use axum::extract::ws::Message;
    use futures_util::{SinkExt, StreamExt};

    let (mut sender, mut receiver) = socket.split();
    let mut rx = hub.subscribe();

    // Spawn task to forward hub events to WebSocket
    let send_task = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(event) => {
                    // Serialize event to JSON
                    let json = match serde_json::to_string(&event) {
                        Ok(j) => j,
                        Err(e) => {
                            tracing::warn!("Failed to serialize event: {}", e);
                            continue;
                        }
                    };

                    // Send to WebSocket
                    if sender.send(Message::Text(json.into())).await.is_err() {
                        break; // Client disconnected
                    }

                    // Check if this is a terminal event
                    if matches!(
                        event,
                        crate::hub::OrchestrationEvent::Success { .. }
                            | crate::hub::OrchestrationEvent::Error { .. }
                    ) {
                        // Give client time to receive, then close
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                        let _ = sender.close().await;
                        break;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!("WebSocket client lagged, dropped {} events", n);
                }
                Err(broadcast::error::RecvError::Closed) => {
                    break; // Hub closed
                }
            }
        }
    });

    // Wait for client to close or send close message
    while let Some(msg) = receiver.next().await {
        if let Ok(Message::Close(_)) = msg {
            break;
        }
    }

    send_task.abort();
}

/// Stop execution.
async fn stop_execution(
    State(state): State<AppState>,
    Json(request): Json<StopRequest>,
) -> impl IntoResponse {
    match stop_execution_handler(&state, request) {
        Ok(()) => Json(serde_json::json!({"status": "stopped"})).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Branches query params.
#[derive(Deserialize)]
struct BranchesQuery {
    spec: Option<String>,
}

/// Get worktree branches.
async fn get_branches(
    State(state): State<AppState>,
    Query(query): Query<BranchesQuery>,
) -> impl IntoResponse {
    let request = ListBranchesRequest { spec: query.spec };
    match list_branches_handler(&state, request) {
        Ok(response) => Json(response).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Merge single branch.
async fn merge_branch(
    State(state): State<AppState>,
    Json(request): Json<MergeBranchRequest>,
) -> impl IntoResponse {
    match merge_branch_handler(&state, request) {
        Ok(response) => Json(response).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Merge all request body.
#[derive(Deserialize, Default)]
struct MergeAllBody {
    spec: Option<String>,
}

/// Merge all branches.
async fn merge_all(
    State(state): State<AppState>,
    body: Option<Json<MergeAllBody>>,
) -> impl IntoResponse {
    let request = MergeAllRequest {
        spec: body.and_then(|b| b.spec.clone()),
    };
    match merge_all_branches_handler(&state, request) {
        Ok(response) => Json(response).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Log query params.
#[derive(Deserialize, Default)]
struct LogQuery {
    offset: Option<usize>,
    limit: Option<usize>,
    since: Option<String>,
}

/// Get execution logs.
async fn get_logs(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<LogQuery>,
) -> impl IntoResponse {
    let params = LogHistoryParams {
        offset: query.offset,
        limit: query.limit,
        since: query.since,
    };
    match get_logs_handler(&state, id, params) {
        Ok(response) => Json(response).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Tail query params.
#[derive(Deserialize, Default)]
struct TailQuery {
    count: Option<usize>,
}

/// Tail execution logs.
async fn tail_logs(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<TailQuery>,
) -> impl IntoResponse {
    let params = LogTailParams { count: query.count };
    match tail_logs_handler(&state, id, params) {
        Ok(response) => Json(response).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Get execution status.
async fn get_status(State(state): State<AppState>) -> impl IntoResponse {
    match get_execution_status_handler(&state) {
        Ok(status) => Json(status).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Create execution routes.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/execution/start", post(start_execution))
        .route("/execution/ws", get(execution_ws))
        .route("/execution/stop", post(stop_execution))
        .route("/execution/status", get(get_status))
        .route("/execution/branches", get(get_branches))
        .route("/execution/merge", post(merge_branch))
        .route("/execution/merge-all", post(merge_all))
        .route("/execution/{id}/logs", get(get_logs))
        .route("/execution/{id}/logs/tail", get(tail_logs))
}
