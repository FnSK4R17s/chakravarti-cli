use axum::{
    extract::{State, Json},
    response::IntoResponse,
};
use serde::Deserialize;
use crate::state::AppState;
use crate::services::command::CommandService;

#[derive(Deserialize)]
pub struct SpecPayload {
    pub description: String,
}

pub async fn run_init(
    State(state): State<AppState>,
) -> impl IntoResponse {
    match CommandService::run_init(&state).await {
        Ok(msg) => Json(serde_json::json!({ "success": true, "message": msg })),
        Err(e) => Json(serde_json::json!({ "success": false, "message": e })),
    }
}

pub async fn run_git_init(
    State(state): State<AppState>,
) -> impl IntoResponse {
    match CommandService::run_git_init(&state).await {
        Ok(msg) => Json(serde_json::json!({ "success": true, "message": msg })),
        Err(e) => Json(serde_json::json!({ "success": false, "message": e })),
    }
}

#[derive(Deserialize)]
pub struct SpecNewPayload {
    pub description: String,
    pub name: Option<String>,
}

pub async fn run_spec_new(
    State(state): State<AppState>,
    Json(payload): Json<SpecNewPayload>,
) -> impl IntoResponse {
    match CommandService::run_spec_new(&state, &payload.description, payload.name.as_deref()).await {
        Ok(msg) => Json(serde_json::json!({ "success": true, "message": msg })),
        Err(e) => Json(serde_json::json!({ "success": false, "message": e })),
    }
}

pub async fn run_spec_tasks(
    State(state): State<AppState>,
) -> impl IntoResponse {
    match CommandService::run_spec_tasks(&state).await {
        Ok(msg) => Json(serde_json::json!({ "success": true, "message": msg })),
        Err(e) => Json(serde_json::json!({ "success": false, "message": e })),
    }
}

#[derive(Deserialize)]
pub struct RunPayload {
    #[serde(default)]
    pub dry_run: bool,
}

pub async fn run_run(
    State(state): State<AppState>,
    Json(payload): Json<Option<RunPayload>>,
) -> impl IntoResponse {
    let dry_run = payload.map(|p| p.dry_run).unwrap_or(false);
    match CommandService::run_run(&state, dry_run).await {
        Ok(msg) => Json(serde_json::json!({ "success": true, "message": msg })),
        Err(e) => Json(serde_json::json!({ "success": false, "message": e })),
    }
}

#[derive(Deserialize)]
pub struct DiffPayload {
    pub base: Option<String>,
    pub stat: Option<bool>,
    pub files: Option<bool>,
    pub summary: Option<bool>,
}

pub async fn run_diff(
    State(state): State<AppState>,
    Json(payload): Json<DiffPayload>,
) -> impl IntoResponse {
    match CommandService::run_diff(&state, payload.base.as_deref(), payload.stat.unwrap_or(false), payload.files.unwrap_or(false), payload.summary.unwrap_or(false)).await {
        Ok(output) => Json(serde_json::json!({ "success": true, "data": output })),
        Err(e) => Json(serde_json::json!({ "success": false, "message": e })),
    }
}

#[derive(Deserialize)]
pub struct VerifyPayload {
    pub lint: Option<bool>,
    pub typecheck: Option<bool>,
    pub test: Option<bool>,
    pub fix: Option<bool>,
}

pub async fn run_verify(
    State(state): State<AppState>,
    Json(payload): Json<VerifyPayload>,
) -> impl IntoResponse {
    match CommandService::run_verify(&state, payload.lint.unwrap_or(false), payload.typecheck.unwrap_or(false), payload.test.unwrap_or(false), payload.fix.unwrap_or(false)).await {
        Ok(output) => Json(serde_json::json!({ "success": true, "data": output })),
        Err(e) => Json(serde_json::json!({ "success": false, "message": e })),
    }
}

#[derive(Deserialize)]
pub struct PromotePayload {
    pub base: Option<String>,
    pub draft: Option<bool>,
    pub push: Option<bool>,
}

pub async fn run_promote(
    State(state): State<AppState>,
    Json(payload): Json<PromotePayload>,
) -> impl IntoResponse {
    match CommandService::run_promote(&state, payload.base.as_deref(), payload.draft.unwrap_or(false), payload.push.unwrap_or(true)).await {
        Ok(output) => Json(serde_json::json!({ "success": true, "data": output })),
        Err(e) => Json(serde_json::json!({ "success": false, "message": e })),
    }
}

#[derive(Deserialize)]
pub struct FixPayload {
    pub lint: Option<bool>,
    pub typecheck: Option<bool>,
    pub test: Option<bool>,
    pub check: Option<bool>,
    pub error: Option<String>,
}

pub async fn run_fix(
    State(state): State<AppState>,
    Json(payload): Json<FixPayload>,
) -> impl IntoResponse {
    match CommandService::run_fix(
        &state,
        payload.lint.unwrap_or(false),
        payload.typecheck.unwrap_or(false),
        payload.test.unwrap_or(false),
        payload.check.unwrap_or(false),
        payload.error.as_deref(),
    ).await {
        Ok(output) => Json(serde_json::json!({ "success": true, "data": output })),
        Err(e) => Json(serde_json::json!({ "success": false, "message": e })),
    }
}
