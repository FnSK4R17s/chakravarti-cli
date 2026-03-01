//! Diff commands for Tauri IPC

use crate::SharedState;
use ckrv_transport::handlers::diff::{
    get_branches_handler, get_default_branch_handler, get_diff_handler, BranchesResponse,
    DiffQuery, DiffResponse,
};
use serde::Serialize;
use tauri::State;

/// Response wrapper for default branch.
#[derive(Serialize)]
pub struct DefaultBranchResponse {
    /// Name of the default branch (e.g., "main" or "master").
    branch: String,
}

/// Get available git branches.
#[tauri::command]
pub async fn get_branches(state: State<'_, SharedState>) -> Result<BranchesResponse, String> {
    let app_state = state.read().await;
    get_branches_handler(&app_state)
        .await
        .map_err(|e| e.to_string())
}

/// Get default git branch (main or master).
#[tauri::command]
pub async fn get_default_branch(
    state: State<'_, SharedState>,
) -> Result<DefaultBranchResponse, String> {
    let app_state = state.read().await;
    get_default_branch_handler(&app_state)
        .await
        .map(|branch| DefaultBranchResponse { branch })
        .map_err(|e| e.to_string())
}

/// Get diff between branches.
#[tauri::command]
pub async fn get_diff(
    state: State<'_, SharedState>,
    base: Option<String>,
    target: Option<String>,
    path: Option<String>,
) -> Result<DiffResponse, String> {
    let app_state = state.read().await;
    get_diff_handler(&app_state, DiffQuery { base, target, path })
        .await
        .map_err(|e| e.to_string())
}
