//! Plans commands for Tauri IPC

use crate::SharedState;
use ckrv_transport::handlers::plans::{
    delete_plan_handler, get_plan_handler, list_plans_handler, update_plan_handler,
};
use ckrv_transport::types::{ListPlansResponse, PlanDetail};
use serde::Serialize;
use tauri::State;

/// Response wrapper for list_plans.
#[derive(Serialize)]
pub struct ListPlansWrapped {
    plans: ListPlansResponse,
}

/// List all plans.
#[tauri::command]
pub async fn list_plans(state: State<'_, SharedState>) -> Result<ListPlansWrapped, String> {
    let app_state = state.read().await;
    list_plans_handler(&app_state)
        .await
        .map(|plans| ListPlansWrapped { plans })
        .map_err(|e| e.to_string())
}

/// Get a plan for a spec.
#[tauri::command]
pub async fn get_plan(state: State<'_, SharedState>, spec: String) -> Result<PlanDetail, String> {
    let app_state = state.read().await;
    get_plan_handler(&app_state, spec)
        .await
        .map_err(|e| e.to_string())
}

/// Save/update a plan.
#[tauri::command]
pub async fn save_plan(
    state: State<'_, SharedState>,
    spec: String,
    raw_yaml: String,
) -> Result<PlanDetail, String> {
    let app_state = state.read().await;
    update_plan_handler(&app_state, spec, raw_yaml)
        .await
        .map_err(|e| e.to_string())
}

/// Delete a plan.
#[tauri::command]
pub async fn delete_plan(state: State<'_, SharedState>, spec: String) -> Result<(), String> {
    let app_state = state.read().await;
    delete_plan_handler(&app_state, spec)
        .await
        .map_err(|e| e.to_string())
}
