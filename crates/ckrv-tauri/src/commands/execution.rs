//! Execution commands for Tauri IPC.
//!
//! Provides commands for starting/stopping executions and listening to events.
//! Events are emitted to the frontend via Tauri's event system.

// ============================================================
// Imports
// ============================================================

use crate::SharedState;
use ckrv_transport::handlers::execution::{
    get_execution_status_handler, get_logs_handler, list_branches_handler, start_execution_handler,
    stop_execution_handler, ExecuteRequest, ExecuteResponse, ListBranchesRequest, LogHistoryParams,
    StopRequest,
};
use ckrv_transport::hub::OrchestrationEvent;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

// ============================================================
// Handlers
// ============================================================

/// Start an execution and emit events to the frontend.
///
/// Subscribes to the Hub and emits all orchestration events to the frontend.
#[tauri::command]
pub async fn start_execution(
    state: State<'_, SharedState>,
    app: AppHandle,
    spec: String,
    _run_id: String,
) -> Result<ExecuteResponse, String> {
    let app_state = state.read().await;
    let hub = app_state.hub.clone();

    // Start the execution
    let request = ExecuteRequest {
        spec: spec.clone(),
        batch_id: None,
        dry_run: false,
    };

    let result = start_execution_handler(&app_state, request)
        .await
        .map_err(|e| e.to_string())?;

    drop(app_state);

    // Subscribe to hub events after starting execution
    let mut rx = hub.subscribe();

    // Spawn task to forward hub events to frontend
    let app_clone = app.clone();
    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(event) => {
                    // Emit event to frontend
                    let event_name = match &event {
                        OrchestrationEvent::Log { .. } => "execution:log",
                        OrchestrationEvent::StepStart { .. } => "execution:step_start",
                        OrchestrationEvent::StepEnd { .. } => "execution:step_end",
                        OrchestrationEvent::Error { .. } => "execution:error",
                        OrchestrationEvent::Success { .. } => "execution:success",
                    };

                    if let Err(e) = app_clone.emit(event_name, &event) {
                        tracing::warn!("Failed to emit event: {}", e);
                    }

                    // Stop listening on terminal events
                    if matches!(
                        event,
                        OrchestrationEvent::Success { .. } | OrchestrationEvent::Error { .. }
                    ) {
                        break;
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!("Event receiver lagged, dropped {} events", n);
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    break;
                }
            }
        }
    });

    Ok(result)
}

/// Stop a running execution.
#[tauri::command]
pub async fn stop_execution(
    state: State<'_, SharedState>,
    spec: String,
    run_id: Option<String>,
) -> Result<(), String> {
    let app_state = state.read().await;
    let request = StopRequest { spec, run_id };
    stop_execution_handler(&app_state, request)
        .await
        .map_err(|e| e.to_string())
}

// ============================================================
// Types
// ============================================================

/// Execution status response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStatusResponse {
    /// Whether an execution is currently running.
    pub is_running: bool,
    /// Name of the spec being executed.
    pub current_spec: Option<String>,
    /// ID of the currently executing batch.
    pub current_batch: Option<String>,
    /// Overall progress as a fraction (0.0 to 1.0).
    pub progress: f32,
}

/// Get current execution status.
#[tauri::command]
pub async fn get_execution_status(
    state: State<'_, SharedState>,
) -> Result<ExecutionStatusResponse, String> {
    let app_state = state.read().await;
    let status = get_execution_status_handler(&app_state)
        .await
        .map_err(|e| e.to_string())?;

    Ok(ExecutionStatusResponse {
        is_running: status.running,
        current_spec: status.spec_name,
        current_batch: status.batch_id,
        progress: status.progress,
    })
}

/// Get execution logs.
#[tauri::command]
pub async fn get_execution_logs(
    state: State<'_, SharedState>,
    run_id: String,
    since: Option<String>,
) -> Result<Vec<serde_json::Value>, String> {
    let app_state = state.read().await;
    let params = LogHistoryParams {
        offset: None,
        limit: None,
        since,
    };
    let response = get_logs_handler(&app_state, run_id, params)
        .await
        .map_err(|e| e.to_string())?;

    // Convert LogEntry vec to serde_json::Value vec
    let entries: Vec<serde_json::Value> = response
        .logs
        .into_iter()
        .filter_map(|e| serde_json::to_value(e).ok())
        .collect();

    Ok(entries)
}

/// List worktree branches (execution branches).
#[tauri::command]
pub async fn list_execution_branches(
    state: State<'_, SharedState>,
    spec: Option<String>,
) -> Result<serde_json::Value, String> {
    let app_state = state.read().await;
    let request = ListBranchesRequest { spec };
    let response = list_branches_handler(&app_state, request)
        .await
        .map_err(|e| e.to_string())?;

    serde_json::to_value(response).map_err(|e| e.to_string())
}
