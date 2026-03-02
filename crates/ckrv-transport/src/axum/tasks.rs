//! # Tasks Axum Routes
//!
//! Axum route wrappers for task handlers.
//!
//! Routes match frontend expectations:
//! - GET /tasks?spec=X - List tasks for spec
//! - GET /tasks/detail?spec=X&task=Y - Get task details
//! - POST /tasks/save - Save task
//! - POST /tasks/status - Update task status

use crate::handlers::tasks::{get_task_handler, list_tasks_handler};
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;

/// Query params for GET /tasks.
#[derive(Deserialize)]
struct TasksQuery {
    spec: Option<String>,
}

/// Query params for GET /tasks/detail.
#[derive(Deserialize)]
struct TaskDetailQuery {
    spec: String,
    task: Option<String>,
}

/// List tasks for a spec.
async fn list_tasks(
    State(state): State<AppState>,
    Query(query): Query<TasksQuery>,
) -> impl IntoResponse {
    match list_tasks_handler(&state, query.spec) {
        Ok(tasks) => Json(tasks).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Get task details.
async fn get_task_detail(
    State(state): State<AppState>,
    Query(query): Query<TaskDetailQuery>,
) -> impl IntoResponse {
    match get_task_handler(&state, query.spec, query.task.unwrap_or_default()) {
        Ok(task) => Json(task).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Save task request.
#[derive(Deserialize)]
#[allow(dead_code)]
struct SaveTaskRequest {
    spec: String,
    task_id: String,
    status: Option<String>,
}

/// Save task - placeholder.
async fn save_task(
    State(_state): State<AppState>,
    Json(request): Json<SaveTaskRequest>,
) -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "spec": request.spec,
        "task_id": request.task_id
    }))
}

/// Update task status request.
#[derive(Deserialize)]
struct TaskStatusRequest {
    spec: String,
    task_id: String,
    status: String,
}

/// Update task status - placeholder.
async fn update_task_status(
    State(_state): State<AppState>,
    Json(request): Json<TaskStatusRequest>,
) -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "spec": request.spec,
        "task_id": request.task_id,
        "new_status": request.status
    }))
}

/// Create task routes.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/tasks", get(list_tasks))
        .route("/tasks/detail", get(get_task_detail))
        .route("/tasks/save", post(save_task))
        .route("/tasks/status", post(update_task_status))
}
