//! CLI commands for Tauri IPC

use crate::SharedState;
use ckrv_transport::handlers::commands::{
    run_diff_handler, run_execute_handler, run_fix_handler, run_git_init_handler, run_init_handler,
    run_plan_handler, run_promote_handler, run_spec_new_handler, run_spec_tasks_handler,
    run_verify_handler, CommandResponse, DiffRequest, FixRequest, PromoteRequest, SpecNewRequest,
    VerifyRequest,
};
use tauri::State;

/// Run ckrv init command.
#[tauri::command]
pub async fn run_init(state: State<'_, SharedState>) -> Result<CommandResponse, String> {
    let app_state = state.read().await;
    run_init_handler(&app_state)
        .await
        .map_err(|e| e.to_string())
}

/// Run git init command.
#[tauri::command]
pub async fn run_git_init(state: State<'_, SharedState>) -> Result<CommandResponse, String> {
    let app_state = state.read().await;
    run_git_init_handler(&app_state)
        .await
        .map_err(|e| e.to_string())
}

/// Run ckrv spec new command.
#[tauri::command]
pub async fn run_spec_new(
    state: State<'_, SharedState>,
    description: String,
    name: Option<String>,
) -> Result<CommandResponse, String> {
    let app_state = state.read().await;
    run_spec_new_handler(&app_state, SpecNewRequest { description, name })
        .await
        .map_err(|e| e.to_string())
}

/// Run ckrv spec tasks command.
#[tauri::command]
pub async fn run_spec_tasks(state: State<'_, SharedState>) -> Result<CommandResponse, String> {
    let app_state = state.read().await;
    run_spec_tasks_handler(&app_state)
        .await
        .map_err(|e| e.to_string())
}

/// Run ckrv plan command.
#[tauri::command]
pub async fn run_plan(state: State<'_, SharedState>) -> Result<CommandResponse, String> {
    let app_state = state.read().await;
    run_plan_handler(&app_state)
        .await
        .map_err(|e| e.to_string())
}

/// Run ckrv execute command.
#[tauri::command]
pub async fn run_execute(state: State<'_, SharedState>) -> Result<CommandResponse, String> {
    let app_state = state.read().await;
    run_execute_handler(&app_state)
        .await
        .map_err(|e| e.to_string())
}

/// Run ckrv diff command.
#[tauri::command]
pub async fn run_diff(
    state: State<'_, SharedState>,
    base: Option<String>,
    stat: Option<bool>,
    files: Option<bool>,
    summary: Option<bool>,
) -> Result<CommandResponse, String> {
    let app_state = state.read().await;
    run_diff_handler(
        &app_state,
        DiffRequest {
            base,
            stat,
            files,
            summary,
        },
    )
    .await
    .map_err(|e| e.to_string())
}

/// Run ckrv verify command.
#[tauri::command]
pub async fn run_verify(
    state: State<'_, SharedState>,
    lint: Option<bool>,
    typecheck: Option<bool>,
    test: Option<bool>,
    fix: Option<bool>,
) -> Result<CommandResponse, String> {
    let app_state = state.read().await;
    run_verify_handler(
        &app_state,
        VerifyRequest {
            lint,
            typecheck,
            test,
            fix,
        },
    )
    .await
    .map_err(|e| e.to_string())
}

/// Run ckrv promote command.
#[tauri::command]
pub async fn run_promote(
    state: State<'_, SharedState>,
    base: Option<String>,
    draft: Option<bool>,
    push: Option<bool>,
) -> Result<CommandResponse, String> {
    let app_state = state.read().await;
    run_promote_handler(&app_state, PromoteRequest { base, draft, push })
        .await
        .map_err(|e| e.to_string())
}

/// Run ckrv fix command.
#[tauri::command]
pub async fn run_fix(
    state: State<'_, SharedState>,
    lint: Option<bool>,
    typecheck: Option<bool>,
    test: Option<bool>,
    check: Option<bool>,
    error: Option<String>,
) -> Result<CommandResponse, String> {
    let app_state = state.read().await;
    run_fix_handler(
        &app_state,
        FixRequest {
            lint,
            typecheck,
            test,
            check,
            error,
        },
    )
    .await
    .map_err(|e| e.to_string())
}
