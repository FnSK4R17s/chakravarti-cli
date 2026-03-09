//! Specs commands for Tauri IPC.

// ============================================================
// Imports
// ============================================================

use crate::SharedState;
use ckrv_transport::handlers::specs::{
    create_spec_handler, delete_spec_handler, generate_design_handler, generate_tasks_handler,
    get_spec_handler, list_specs_handler, update_spec_handler, validate_spec_handler,
    DesignResponse, GenerateTasksResponse, ValidateSpecResponse,
};
use ckrv_transport::types::{
    CreateSpecRequest, ListSpecsResponse, SpecDetail, SpecSummary, UpdateSpecRequest,
};
use tauri::State;

// ============================================================
// Handlers
// ============================================================

/// List all specifications.
#[tauri::command]
pub async fn list_specs(state: State<'_, SharedState>) -> Result<ListSpecsResponse, String> {
    let app_state = state.read().await;
    list_specs_handler(&app_state).map_err(|e| e.to_string())
}

/// Get a single specification detail.
#[tauri::command]
pub async fn get_spec(state: State<'_, SharedState>, name: String) -> Result<SpecDetail, String> {
    let app_state = state.read().await;
    get_spec_handler(&app_state, name).map_err(|e| e.to_string())
}

/// Create a new specification.
#[tauri::command]
pub async fn create_spec(
    state: State<'_, SharedState>,
    description: String,
    name: Option<String>,
) -> Result<SpecSummary, String> {
    let app_state = state.read().await;
    create_spec_handler(&app_state, CreateSpecRequest { description, name })
        .map_err(|e| e.to_string())
}

/// Update a specification.
#[tauri::command]
pub async fn update_spec(
    state: State<'_, SharedState>,
    name: String,
    raw_yaml: Option<String>,
) -> Result<SpecDetail, String> {
    let app_state = state.read().await;
    update_spec_handler(&app_state, name, UpdateSpecRequest { raw_yaml }).map_err(|e| e.to_string())
}

/// Delete a specification.
#[tauri::command]
pub async fn delete_spec(state: State<'_, SharedState>, name: String) -> Result<(), String> {
    let app_state = state.read().await;
    delete_spec_handler(&app_state, name).map_err(|e| e.to_string())
}

/// Validate a specification.
#[tauri::command]
pub async fn validate_spec(
    state: State<'_, SharedState>,
    name: String,
) -> Result<ValidateSpecResponse, String> {
    let app_state = state.read().await;
    validate_spec_handler(&app_state, name).map_err(|e| e.to_string())
}

/// Generate design for a specification.
#[tauri::command]
pub async fn generate_design(
    state: State<'_, SharedState>,
    name: String,
) -> Result<DesignResponse, String> {
    let app_state = state.read().await;
    generate_design_handler(&app_state, name).map_err(|e| e.to_string())
}

/// Generate tasks for a specification.
#[tauri::command]
pub async fn generate_tasks(
    state: State<'_, SharedState>,
    name: String,
) -> Result<GenerateTasksResponse, String> {
    let app_state = state.read().await;
    generate_tasks_handler(&app_state, name).map_err(|e| e.to_string())
}
