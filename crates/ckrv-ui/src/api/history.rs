//! Run history API handlers.
//!
//! REST endpoints for managing persistent run history.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::services::history::HistoryService;
use crate::models::history::{Run, RunSummary, HistoryBatchStatus};
use crate::state::AppState;

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct HistoryListQuery {
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(default)]
    pub offset: usize,
    pub status: Option<String>,
}

fn default_limit() -> usize {
    50
}

#[derive(Debug, Serialize)]
pub struct HistoryListResponse {
    pub success: bool,
    pub spec_name: String,
    pub total_count: usize,
    pub runs: Vec<RunListItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RunListItem {
    pub id: String,
    pub spec_name: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub status: String,
    pub dry_run: bool,
    pub elapsed_seconds: Option<u64>,
    pub summary: RunSummary,
}

impl From<&Run> for RunListItem {
    fn from(run: &Run) -> Self {
        Self {
            id: run.id.clone(),
            spec_name: run.spec_name.clone(),
            started_at: run.started_at.to_rfc3339(),
            ended_at: run.ended_at.map(|dt| dt.to_rfc3339()),
            status: format!("{:?}", run.status).to_lowercase(),
            dry_run: run.dry_run,
            elapsed_seconds: run.elapsed_seconds,
            summary: run.summary.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct HistoryDetailResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run: Option<Run>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRunRequest {
    pub run_id: String,
    #[serde(default)]
    pub dry_run: bool,
    pub batches: Vec<BatchInfo>,
}

#[derive(Debug, Deserialize)]
pub struct BatchInfo {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct CreateRunResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub existing_run_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub existing_started_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRunRequest {
    pub status: Option<String>,
    pub ended_at: Option<String>,
    pub summary: Option<RunSummary>,
    pub batch_update: Option<BatchUpdate>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BatchUpdate {
    pub batch_id: String,
    pub status: String,
    pub ended_at: Option<String>,
    pub branch: Option<String>,
    pub merged: Option<bool>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UpdateRunResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DeleteRunResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_run_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// ============================================================================
// Handlers
// ============================================================================

/// GET /api/history/{spec}
/// List all runs for a specification
pub async fn list_runs(
    State(state): State<AppState>,
    Path(spec): Path<String>,
    Query(query): Query<HistoryListQuery>,
) -> impl IntoResponse {
    let service = HistoryService::new(&state.project_root);
    
    // Check if spec exists
    if !service.spec_exists(&spec) {
        return (
            StatusCode::NOT_FOUND,
            Json(HistoryListResponse {
                success: false,
                spec_name: spec.clone(),
                total_count: 0,
                runs: vec![],
                error: Some(format!("Specification not found: {}", spec)),
            }),
        );
    }
    
    match service.load_history(&spec) {
        Ok(history) => {
            let mut runs: Vec<&Run> = history.runs.iter().collect();
            
            // Filter by status if provided
            if let Some(ref status_filter) = query.status {
                runs.retain(|r| format!("{:?}", r.status).to_lowercase() == *status_filter);
            }
            
            let total_count = runs.len();
            
            // Paginate
            let start = query.offset.min(runs.len());
            let end = (start + query.limit).min(runs.len());
            let paginated: Vec<RunListItem> = runs[start..end]
                .iter()
                .map(|r| RunListItem::from(*r))
                .collect();
            
            (
                StatusCode::OK,
                Json(HistoryListResponse {
                    success: true,
                    spec_name: spec,
                    total_count,
                    runs: paginated,
                    error: None,
                }),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(HistoryListResponse {
                success: false,
                spec_name: spec,
                total_count: 0,
                runs: vec![],
                error: Some(format!("Failed to load history: {}", e)),
            }),
        ),
    }
}

/// GET /api/history/{spec}/{run_id}
/// Get details for a single run
pub async fn get_run(
    State(state): State<AppState>,
    Path((spec, run_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let service = HistoryService::new(&state.project_root);
    
    match service.get_run(&spec, &run_id) {
        Ok(Some(run)) => (
            StatusCode::OK,
            Json(HistoryDetailResponse {
                success: true,
                run: Some(run),
                error: None,
            }),
        ),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(HistoryDetailResponse {
                success: false,
                run: None,
                error: Some(format!("Run not found: {}", run_id)),
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(HistoryDetailResponse {
                success: false,
                run: None,
                error: Some(format!("Failed to get run: {}", e)),
            }),
        ),
    }
}

/// POST /api/history/{spec}
/// Create a new run entry
pub async fn create_run(
    State(state): State<AppState>,
    Path(spec): Path<String>,
    Json(request): Json<CreateRunRequest>,
) -> impl IntoResponse {
    let service = HistoryService::new(&state.project_root);
    
    // Check if spec exists
    if !service.spec_exists(&spec) {
        return (
            StatusCode::NOT_FOUND,
            Json(CreateRunResponse {
                success: false,
                run_id: None,
                started_at: None,
                error: Some(format!("Specification not found: {}", spec)),
                existing_run_id: None,
                existing_started_at: None,
            }),
        );
    }
    
    let batches: Vec<(String, String)> = request.batches
        .into_iter()
        .map(|b| (b.id, b.name))
        .collect();
    
    match service.create_run(&spec, &request.run_id, batches, request.dry_run) {
        Ok(run) => (
            StatusCode::CREATED,
            Json(CreateRunResponse {
                success: true,
                run_id: Some(run.id),
                started_at: Some(run.started_at.to_rfc3339()),
                error: None,
                existing_run_id: None,
                existing_started_at: None,
            }),
        ),
        Err(e) => {
            let error_msg = e.to_string();
            // Check for concurrent run error
            if error_msg.contains("already in progress") {
                // Extract existing run info from error message
                (
                    StatusCode::CONFLICT,
                    Json(CreateRunResponse {
                        success: false,
                        run_id: None,
                        started_at: None,
                        error: Some("Another run is already in progress".to_string()),
                        existing_run_id: None, // Could parse from error
                        existing_started_at: None,
                    }),
                )
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(CreateRunResponse {
                        success: false,
                        run_id: None,
                        started_at: None,
                        error: Some(format!("Failed to create run: {}", e)),
                        existing_run_id: None,
                        existing_started_at: None,
                    }),
                )
            }
        }
    }
}

/// PATCH /api/history/{spec}/{run_id}
/// Update run or batch status
pub async fn update_run(
    State(state): State<AppState>,
    Path((spec, run_id)): Path<(String, String)>,
    Json(request): Json<UpdateRunRequest>,
) -> impl IntoResponse {
    let service = HistoryService::new(&state.project_root);
    
    // Handle batch update
    if let Some(batch_update) = request.batch_update {
        let status = match batch_update.status.as_str() {
            "running" => HistoryBatchStatus::Running,
            "completed" => HistoryBatchStatus::Completed,
            "failed" => HistoryBatchStatus::Failed,
            _ => HistoryBatchStatus::Pending,
        };
        
        match service.update_batch_status(
            &spec,
            &run_id,
            &batch_update.batch_id,
            status,
            batch_update.branch.as_deref(),
            batch_update.error.as_deref(),
        ) {
            Ok(_) => {
                return (
                    StatusCode::OK,
                    Json(UpdateRunResponse {
                        success: true,
                        updated_at: Some(chrono::Utc::now().to_rfc3339()),
                        error: None,
                    }),
                );
            }
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(UpdateRunResponse {
                        success: false,
                        updated_at: None,
                        error: Some(format!("Failed to update batch: {}", e)),
                    }),
                );
            }
        }
    }
    
    // Handle run status update
    if let Some(ref status) = request.status {
        let result = match status.as_str() {
            "completed" => service.complete_run(&spec, &run_id),
            "failed" => service.fail_run(&spec, &run_id, request.error.as_deref().unwrap_or("Unknown error")),
            "aborted" => service.abort_run(&spec, &run_id),
            _ => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(UpdateRunResponse {
                        success: false,
                        updated_at: None,
                        error: Some(format!("Invalid status: {}", status)),
                    }),
                );
            }
        };
        
        match result {
            Ok(_) => (
                StatusCode::OK,
                Json(UpdateRunResponse {
                    success: true,
                    updated_at: Some(chrono::Utc::now().to_rfc3339()),
                    error: None,
                }),
            ),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(UpdateRunResponse {
                    success: false,
                    updated_at: None,
                    error: Some(format!("Failed to update run: {}", e)),
                }),
            ),
        }
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(UpdateRunResponse {
                success: false,
                updated_at: None,
                error: Some("No update provided".to_string()),
            }),
        )
    }
}

/// DELETE /api/history/{spec}/{run_id}
/// Delete a run from history
pub async fn delete_run(
    State(state): State<AppState>,
    Path((spec, run_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let service = HistoryService::new(&state.project_root);
    
    match service.delete_run(&spec, &run_id) {
        Ok(_) => (
            StatusCode::OK,
            Json(DeleteRunResponse {
                success: true,
                deleted_run_id: Some(run_id),
                error: None,
            }),
        ),
        Err(e) => {
            let error_msg = e.to_string();
            let status = if error_msg.contains("Cannot delete a running") {
                StatusCode::BAD_REQUEST
            } else if error_msg.contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            
            (
                status,
                Json(DeleteRunResponse {
                    success: false,
                    deleted_run_id: None,
                    error: Some(error_msg),
                }),
            )
        }
    }
}
