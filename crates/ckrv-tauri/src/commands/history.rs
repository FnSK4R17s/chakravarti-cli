//! History commands for Tauri IPC

use crate::SharedState;
use ckrv_transport::handlers::history::{
    create_run_handler, delete_run_handler, get_run_handler, list_history_handler,
    update_run_handler,
};
use ckrv_transport::types::{
    BatchInfo, CreateRunRequest, RunDetail, RunStatus, RunSummary, UpdateRunRequest,
};
use serde::Serialize;
use tauri::State;

/// Response wrapper for list_history.
#[derive(Serialize)]
pub struct ListHistoryWrapped {
    runs: Vec<RunSummary>,
}

/// List run history for a spec.
#[tauri::command]
pub async fn list_history(
    state: State<'_, SharedState>,
    spec: String,
) -> Result<ListHistoryWrapped, String> {
    let app_state = state.read().await;
    list_history_handler(&app_state, spec)
        .await
        .map(|runs| ListHistoryWrapped { runs })
        .map_err(|e| e.to_string())
}

/// Get a specific run.
#[tauri::command]
pub async fn get_run(
    state: State<'_, SharedState>,
    spec: String,
    run_id: String,
) -> Result<RunDetail, String> {
    let app_state = state.read().await;
    get_run_handler(&app_state, spec, run_id)
        .await
        .map_err(|e| e.to_string())
}

/// Create a new run.
#[tauri::command]
pub async fn create_run(
    state: State<'_, SharedState>,
    spec: String,
    run_id: String,
    dry_run: Option<bool>,
    batches: Option<Vec<BatchInfo>>,
) -> Result<RunDetail, String> {
    let app_state = state.read().await;
    create_run_handler(
        &app_state,
        spec,
        CreateRunRequest {
            run_id,
            dry_run: dry_run.unwrap_or(false),
            batches: batches.unwrap_or_default(),
        },
    )
    .await
    .map_err(|e| e.to_string())
}

/// Update a run status.
#[tauri::command]
pub async fn update_run(
    state: State<'_, SharedState>,
    spec: String,
    run_id: String,
    status: Option<RunStatus>,
    error: Option<String>,
) -> Result<RunDetail, String> {
    let app_state = state.read().await;
    update_run_handler(&app_state, spec, run_id, UpdateRunRequest { status, error })
        .await
        .map_err(|e| e.to_string())
}

/// Delete a run.
#[tauri::command]
pub async fn delete_run(
    state: State<'_, SharedState>,
    spec: String,
    run_id: String,
) -> Result<(), String> {
    let app_state = state.read().await;
    delete_run_handler(&app_state, spec, run_id)
        .await
        .map_err(|e| e.to_string())
}
