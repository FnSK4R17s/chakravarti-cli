//! # Specs Axum Routes
//!
//! Axum route wrappers for spec handlers.
//!
//! Routes match the frontend expectations:
//! - GET /specs - List all specs
//! - GET /specs/detail?name=X - Get spec details
//! - POST /specs/create - Create new spec
//! - POST /specs/save - Save spec
//! - GET /specs/{name}/validate - Validate spec
//! - POST /specs/{name}/design - Generate design
//! - POST /specs/{name}/tasks - Generate tasks
//! - GET /specs/{name}/clarifications - Get clarifications
//! - POST /specs/{name}/clarify - Answer clarifications
//!
//! Handlers that call synchronous filesystem/CLI operations use
//! `tokio::task::spawn_blocking` to avoid blocking the async runtime.

// ============================================================
// IMPORTS
// ============================================================

use crate::error::TransportError;
use crate::handlers::specs::{
    create_spec_handler, generate_design_handler, generate_tasks_handler, get_spec_handler,
    list_specs_handler, update_spec_handler, validate_spec_handler,
};
use crate::state::AppState;
use crate::types::{CreateSpecRequest, UpdateSpecRequest};
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;

// ============================================================
// TYPES
// ============================================================

/// Query params for GET /specs/detail.
#[derive(Deserialize)]
struct SpecDetailQuery {
    name: String,
}

// ============================================================
// HANDLERS
// ============================================================

/// List all specs.
async fn list_specs(State(state): State<AppState>) -> impl IntoResponse {
    match tokio::task::spawn_blocking(move || list_specs_handler(&state)).await {
        Ok(Ok(specs)) => Json(specs).into_response(),
        Ok(Err(e)) => e.into_response(),
        Err(e) => TransportError::Internal(format!("Task panicked: {e}")).into_response(),
    }
}

/// Get spec by name (query param style).
///
/// Returns `{ success, spec, raw_yaml }` to match frontend expectations.
async fn get_spec_detail(
    State(state): State<AppState>,
    Query(query): Query<SpecDetailQuery>,
) -> impl IntoResponse {
    let name = query.name;
    match tokio::task::spawn_blocking(move || get_spec_handler(&state, name)).await {
        Ok(Ok(spec)) => {
            let raw_yaml = spec.raw_yaml.clone();
            Json(serde_json::json!({
                "success": true,
                "spec": spec,
                "raw_yaml": raw_yaml,
            }))
            .into_response()
        }
        Ok(Err(e)) => e.into_response(),
        Err(e) => TransportError::Internal(format!("Task panicked: {e}")).into_response(),
    }
}

/// Create a new spec.
async fn create_spec(
    State(state): State<AppState>,
    Json(request): Json<CreateSpecRequest>,
) -> impl IntoResponse {
    match tokio::task::spawn_blocking(move || create_spec_handler(&state, request)).await {
        Ok(Ok(spec)) => (axum::http::StatusCode::CREATED, Json(spec)).into_response(),
        Ok(Err(e)) => e.into_response(),
        Err(e) => TransportError::Internal(format!("Task panicked: {e}")).into_response(),
    }
}

/// Save spec request (includes name).
#[derive(Deserialize)]
struct SaveSpecRequest {
    name: String,
    raw_yaml: Option<String>,
}

/// Save spec (update with name in body).
async fn save_spec(
    State(state): State<AppState>,
    Json(request): Json<SaveSpecRequest>,
) -> impl IntoResponse {
    let name = request.name;
    let update_request = UpdateSpecRequest {
        raw_yaml: request.raw_yaml,
    };
    match tokio::task::spawn_blocking(move || update_spec_handler(&state, name, update_request))
        .await
    {
        Ok(Ok(spec)) => Json(spec).into_response(),
        Ok(Err(e)) => e.into_response(),
        Err(e) => TransportError::Internal(format!("Task panicked: {e}")).into_response(),
    }
}

/// Validate spec via CLI.
async fn validate_spec(
    State(state): State<AppState>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> impl IntoResponse {
    match tokio::task::spawn_blocking(move || validate_spec_handler(&state, name)).await {
        Ok(Ok(result)) => Json(serde_json::json!(result)).into_response(),
        Ok(Err(e)) => e.into_response(),
        Err(e) => TransportError::Internal(format!("Task panicked: {e}")).into_response(),
    }
}

/// Generate design via CLI.
async fn generate_design(
    State(state): State<AppState>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> impl IntoResponse {
    match tokio::task::spawn_blocking(move || generate_design_handler(&state, name)).await {
        Ok(Ok(result)) => Json(serde_json::json!(result)).into_response(),
        Ok(Err(e)) => e.into_response(),
        Err(e) => TransportError::Internal(format!("Task panicked: {e}")).into_response(),
    }
}

/// Generate tasks via CLI.
async fn generate_tasks(
    State(state): State<AppState>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> impl IntoResponse {
    match tokio::task::spawn_blocking(move || generate_tasks_handler(&state, name)).await {
        Ok(Ok(result)) => Json(serde_json::json!(result)).into_response(),
        Ok(Err(e)) => e.into_response(),
        Err(e) => TransportError::Internal(format!("Task panicked: {e}")).into_response(),
    }
}

/// Get clarifications from spec YAML.
async fn get_clarifications(
    State(state): State<AppState>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> impl IntoResponse {
    let state_clone = state.clone();
    let name_clone = name.clone();
    match tokio::task::spawn_blocking(move || {
        let spec_path = state_clone
            .project_root
            .join(".specs")
            .join(&name_clone)
            .join("spec.yaml");
        if !spec_path.exists() {
            return Ok::<serde_json::Value, String>(serde_json::json!({
                "spec": name_clone,
                "clarifications": [],
                "unresolved_count": 0
            }));
        }
        let content = std::fs::read_to_string(&spec_path).map_err(|e| e.to_string())?;
        let yaml: serde_json::Value = serde_yaml::from_str(&content).map_err(|e| e.to_string())?;
        let clarifications = yaml
            .get("clarifications")
            .cloned()
            .unwrap_or(serde_json::json!([]));
        let unresolved_count = clarifications
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter(|c| c.get("resolved").map_or(true, serde_json::Value::is_null))
                    .count()
            })
            .unwrap_or(0);
        Ok(serde_json::json!({
            "spec": name_clone,
            "clarifications": clarifications,
            "unresolved_count": unresolved_count
        }))
    })
    .await
    {
        Ok(Ok(result)) => Json(result).into_response(),
        Ok(Err(e)) => TransportError::Internal(e).into_response(),
        Err(e) => TransportError::Internal(format!("Task panicked: {e}")).into_response(),
    }
}

/// Request body for POST /specs/{name}/clarify.
#[derive(Deserialize)]
struct ClarifyRequest {
    answers: Vec<ClarifyAnswer>,
}

/// A single clarification answer.
#[derive(Deserialize)]
struct ClarifyAnswer {
    topic: String,
    answer: String,
}

/// Answer clarifications by writing resolved values into spec.yaml.
async fn clarify(
    State(state): State<AppState>,
    axum::extract::Path(name): axum::extract::Path<String>,
    Json(request): Json<ClarifyRequest>,
) -> impl IntoResponse {
    let answers = request.answers;
    match tokio::task::spawn_blocking(move || {
        let spec_path = state
            .project_root
            .join(".specs")
            .join(&name)
            .join("spec.yaml");
        if !spec_path.exists() {
            return Err(format!("Spec '{}' not found", name));
        }

        let content = std::fs::read_to_string(&spec_path).map_err(|e| e.to_string())?;
        let mut yaml: serde_yaml::Value =
            serde_yaml::from_str(&content).map_err(|e| e.to_string())?;

        // Build a lookup map from topic -> answer
        let answer_map: std::collections::HashMap<String, String> =
            answers.into_iter().map(|a| (a.topic, a.answer)).collect();

        // Update clarifications in-place
        if let Some(clarifications) = yaml.get_mut("clarifications") {
            if let Some(arr) = clarifications.as_sequence_mut() {
                for item in arr.iter_mut() {
                    if let Some(topic) = item.get("topic").and_then(|t| t.as_str()) {
                        if let Some(answer) = answer_map.get(topic) {
                            item.as_mapping_mut().map(|m| {
                                m.insert(
                                    serde_yaml::Value::String("resolved".to_string()),
                                    serde_yaml::Value::String(answer.clone()),
                                )
                            });
                        }
                    }
                }
            }
        }

        let updated_yaml = serde_yaml::to_string(&yaml).map_err(|e| e.to_string())?;
        std::fs::write(&spec_path, &updated_yaml).map_err(|e| e.to_string())?;

        Ok(serde_json::json!({
            "success": true,
            "spec": name,
            "answers_saved": answer_map.len()
        }))
    })
    .await
    {
        Ok(Ok(result)) => Json(result).into_response(),
        Ok(Err(e)) => TransportError::Internal(e).into_response(),
        Err(e) => TransportError::Internal(format!("Task panicked: {e}")).into_response(),
    }
}

/// Get design artifacts (design.md and research.md) for a spec.
async fn get_design_artifacts(
    State(state): State<AppState>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> impl IntoResponse {
    match tokio::task::spawn_blocking(move || {
        let spec_dir = state.project_root.join(".specs").join(&name);
        if !spec_dir.exists() {
            return Err(format!("Spec '{}' not found", name));
        }

        let design_content = std::fs::read_to_string(spec_dir.join("design.md")).ok();
        let research_content = std::fs::read_to_string(spec_dir.join("research.md")).ok();

        Ok(serde_json::json!({
            "spec": name,
            "has_design": design_content.is_some(),
            "has_research": research_content.is_some(),
            "design": design_content,
            "research": research_content,
        }))
    })
    .await
    {
        Ok(Ok(result)) => Json(result).into_response(),
        Ok(Err(e)) => TransportError::Internal(e).into_response(),
        Err(e) => TransportError::Internal(format!("Task panicked: {e}")).into_response(),
    }
}

/// Get tasks artifacts (tasks.yaml content) for a spec.
async fn get_tasks_artifacts(
    State(state): State<AppState>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> impl IntoResponse {
    match tokio::task::spawn_blocking(move || {
        let spec_dir = state.project_root.join(".specs").join(&name);
        if !spec_dir.exists() {
            return Err(format!("Spec '{}' not found", name));
        }

        let tasks_path = spec_dir.join("tasks.yaml");
        let tasks_content = std::fs::read_to_string(&tasks_path).ok();
        let tasks: Vec<serde_json::Value> = tasks_content
            .as_ref()
            .and_then(|c| {
                let yaml: serde_json::Value = serde_yaml::from_str(c).ok()?;
                yaml.get("tasks")?.as_array().cloned()
            })
            .unwrap_or_default();

        Ok(serde_json::json!({
            "spec": name,
            "has_tasks": tasks_content.is_some() && !tasks.is_empty(),
            "tasks": tasks,
            "task_count": tasks.len(),
            "raw": tasks_content,
        }))
    })
    .await
    {
        Ok(Ok(result)) => Json(result).into_response(),
        Ok(Err(e)) => TransportError::Internal(e).into_response(),
        Err(e) => TransportError::Internal(format!("Task panicked: {e}")).into_response(),
    }
}

// ============================================================
// ROUTES
// ============================================================

/// Create spec routes.
pub fn routes() -> Router<AppState> {
    Router::new()
        // List and create
        .route("/specs", get(list_specs))
        .route("/specs/detail", get(get_spec_detail))
        .route("/specs/create", post(create_spec))
        .route("/specs/save", post(save_spec))
        // Per-spec operations
        .route("/specs/{name}/validate", get(validate_spec))
        .route("/specs/{name}/design", post(generate_design))
        .route("/specs/{name}/design/artifacts", get(get_design_artifacts))
        .route("/specs/{name}/tasks", post(generate_tasks))
        .route("/specs/{name}/tasks/artifacts", get(get_tasks_artifacts))
        .route("/specs/{name}/clarifications", get(get_clarifications))
        .route("/specs/{name}/clarify", post(clarify))
}
