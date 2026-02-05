//! # History Handler
//!
//! Handlers for run history management.

use crate::error::TransportError;
use crate::state::AppState;
use crate::types::{
    CreateRunRequest, ListHistoryResponse, RunDetail, RunStatus, RunSummary, UpdateRunRequest,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

// ============================================================================
// Internal Types
// ============================================================================

/// Batch status in history.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HistoryBatchStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

impl Default for HistoryBatchStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// Batch in run history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryBatch {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub status: HistoryBatchStatus,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub ended_at: Option<chrono::DateTime<chrono::Utc>>,
    pub branch: Option<String>,
    pub error: Option<String>,
}

/// Run status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HistoryRunStatus {
    Running,
    Completed,
    Failed,
    Aborted,
}

impl Default for HistoryRunStatus {
    fn default() -> Self {
        Self::Running
    }
}

/// Run summary.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RunSummaryInternal {
    #[serde(default)]
    pub total_batches: usize,
    #[serde(default)]
    pub completed_batches: usize,
    #[serde(default)]
    pub failed_batches: usize,
}

/// Run in history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Run {
    pub id: String,
    pub spec_name: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub ended_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    pub status: HistoryRunStatus,
    #[serde(default)]
    pub dry_run: bool,
    pub elapsed_seconds: Option<u64>,
    #[serde(default)]
    pub batches: Vec<HistoryBatch>,
    #[serde(default)]
    pub summary: RunSummaryInternal,
    pub error: Option<String>,
}

/// History file structure.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HistoryFile {
    pub runs: Vec<Run>,
}

// ============================================================================
// Path Utilities
// ============================================================================

/// Get the specs directory path.
fn get_specs_dir() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".specs")
}

/// Get path to history file.
fn get_history_path(spec_name: &str) -> PathBuf {
    get_specs_dir().join(spec_name).join("history.yaml")
}

/// Load history for a spec.
fn load_history(spec_name: &str) -> Result<HistoryFile, TransportError> {
    let path = get_history_path(spec_name);

    if !path.exists() {
        return Ok(HistoryFile::default());
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| TransportError::Internal(format!("Failed to read history: {e}")))?;

    serde_yaml::from_str(&content)
        .map_err(|e| TransportError::Internal(format!("Failed to parse history: {e}")))
}

/// Save history for a spec.
fn save_history(spec_name: &str, history: &HistoryFile) -> Result<(), TransportError> {
    let path = get_history_path(spec_name);

    let yaml = serde_yaml::to_string(history)
        .map_err(|e| TransportError::Internal(format!("Failed to serialize history: {e}")))?;

    fs::write(&path, yaml)
        .map_err(|e| TransportError::Internal(format!("Failed to write history: {e}")))?;

    Ok(())
}

// ============================================================================
// Handlers
// ============================================================================

/// List run history for a spec.
pub async fn list_history_handler(
    _state: &AppState,
    spec_name: String,
) -> Result<ListHistoryResponse, TransportError> {
    let history = load_history(&spec_name)?;

    Ok(history
        .runs
        .iter()
        .map(|r| RunSummary {
            id: r.id.clone(),
            spec_name: r.spec_name.clone(),
            status: match r.status {
                HistoryRunStatus::Running => RunStatus::Running,
                HistoryRunStatus::Completed => RunStatus::Completed,
                HistoryRunStatus::Failed => RunStatus::Failed,
                HistoryRunStatus::Aborted => RunStatus::Aborted,
            },
            started_at: r.started_at.to_rfc3339(),
            ended_at: r.ended_at.map(|dt| dt.to_rfc3339()),
            batch_count: r.batches.len(),
            completed_batches: r
                .batches
                .iter()
                .filter(|b| b.status == HistoryBatchStatus::Completed)
                .count(),
        })
        .collect())
}

/// Get a specific run.
pub async fn get_run_handler(
    _state: &AppState,
    spec_name: String,
    run_id: String,
) -> Result<RunDetail, TransportError> {
    let history = load_history(&spec_name)?;

    let run = history
        .runs
        .iter()
        .find(|r| r.id == run_id)
        .ok_or_else(|| TransportError::NotFound(format!("Run not found: {run_id}")))?;

    Ok(RunDetail {
        id: run.id.clone(),
        spec_name: run.spec_name.clone(),
        status: match run.status {
            HistoryRunStatus::Running => RunStatus::Running,
            HistoryRunStatus::Completed => RunStatus::Completed,
            HistoryRunStatus::Failed => RunStatus::Failed,
            HistoryRunStatus::Aborted => RunStatus::Aborted,
        },
        started_at: run.started_at.to_rfc3339(),
        ended_at: run.ended_at.map(|dt| dt.to_rfc3339()),
        dry_run: run.dry_run,
        elapsed_seconds: run.elapsed_seconds,
        batches: run
            .batches
            .iter()
            .map(|b| crate::types::BatchRunDetail {
                id: b.id.clone(),
                name: b.name.clone(),
                status: match b.status {
                    HistoryBatchStatus::Pending => "pending".to_string(),
                    HistoryBatchStatus::Running => "running".to_string(),
                    HistoryBatchStatus::Completed => "completed".to_string(),
                    HistoryBatchStatus::Failed => "failed".to_string(),
                },
                started_at: b.started_at.map(|dt| dt.to_rfc3339()),
                ended_at: b.ended_at.map(|dt| dt.to_rfc3339()),
                branch: b.branch.clone(),
                error: b.error.clone(),
            })
            .collect(),
        error: run.error.clone(),
    })
}

/// Create a new run.
pub async fn create_run_handler(
    _state: &AppState,
    spec_name: String,
    request: CreateRunRequest,
) -> Result<RunDetail, TransportError> {
    let mut history = load_history(&spec_name)?;

    // Check for concurrent run
    if history.runs.iter().any(|r| r.status == HistoryRunStatus::Running) {
        return Err(TransportError::Conflict(
            "Another run is already in progress".to_string(),
        ));
    }

    let now = chrono::Utc::now();
    let run = Run {
        id: request.run_id.clone(),
        spec_name: spec_name.clone(),
        started_at: now,
        ended_at: None,
        status: HistoryRunStatus::Running,
        dry_run: request.dry_run,
        elapsed_seconds: None,
        batches: request
            .batches
            .iter()
            .map(|b| HistoryBatch {
                id: b.id.clone(),
                name: b.name.clone(),
                status: HistoryBatchStatus::Pending,
                started_at: None,
                ended_at: None,
                branch: None,
                error: None,
            })
            .collect(),
        summary: RunSummaryInternal {
            total_batches: request.batches.len(),
            completed_batches: 0,
            failed_batches: 0,
        },
        error: None,
    };

    history.runs.insert(0, run.clone());
    save_history(&spec_name, &history)?;

    Ok(RunDetail {
        id: run.id,
        spec_name: run.spec_name,
        status: RunStatus::Running,
        started_at: run.started_at.to_rfc3339(),
        ended_at: None,
        dry_run: run.dry_run,
        elapsed_seconds: None,
        batches: vec![],
        error: None,
    })
}

/// Update a run.
pub async fn update_run_handler(
    _state: &AppState,
    spec_name: String,
    run_id: String,
    request: UpdateRunRequest,
) -> Result<RunDetail, TransportError> {
    let mut history = load_history(&spec_name)?;

    let run = history
        .runs
        .iter_mut()
        .find(|r| r.id == run_id)
        .ok_or_else(|| TransportError::NotFound(format!("Run not found: {run_id}")))?;

    // Update status
    if let Some(status) = request.status {
        run.status = match status {
            RunStatus::Running => HistoryRunStatus::Running,
            RunStatus::Completed => HistoryRunStatus::Completed,
            RunStatus::Failed => HistoryRunStatus::Failed,
            RunStatus::Aborted => HistoryRunStatus::Aborted,
        };

        if matches!(
            run.status,
            HistoryRunStatus::Completed | HistoryRunStatus::Failed | HistoryRunStatus::Aborted
        ) {
            run.ended_at = Some(chrono::Utc::now());
            if let Some(started) = Some(run.started_at) {
                run.elapsed_seconds = Some(
                    (chrono::Utc::now() - started).num_seconds().max(0) as u64,
                );
            }
        }
    }

    // Update error
    if let Some(error) = request.error {
        run.error = Some(error);
    }

    save_history(&spec_name, &history)?;

    get_run_handler(_state, spec_name, run_id).await
}

/// Delete a run.
pub async fn delete_run_handler(
    _state: &AppState,
    spec_name: String,
    run_id: String,
) -> Result<(), TransportError> {
    let mut history = load_history(&spec_name)?;

    // Check if run exists and is not running
    if let Some(run) = history.runs.iter().find(|r| r.id == run_id) {
        if run.status == HistoryRunStatus::Running {
            return Err(TransportError::BadRequest(
                "Cannot delete a running run".to_string(),
            ));
        }
    } else {
        return Err(TransportError::NotFound(format!("Run not found: {run_id}")));
    }

    history.runs.retain(|r| r.id != run_id);
    save_history(&spec_name, &history)?;

    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_batch_status_default() {
        assert_eq!(HistoryBatchStatus::default(), HistoryBatchStatus::Pending);
    }

    #[test]
    fn test_history_run_status_default() {
        assert_eq!(HistoryRunStatus::default(), HistoryRunStatus::Running);
    }
}
