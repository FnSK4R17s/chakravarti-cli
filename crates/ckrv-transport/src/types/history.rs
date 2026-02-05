//! # History Types
//!
//! Types for execution history and run details.

use serde::{Deserialize, Serialize};

#[cfg(feature = "typescript")]
use ts_rs::TS;

/// Run status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[serde(rename_all = "lowercase")]
pub enum RunStatus {
    /// Currently running
    #[default]
    Running,
    /// Successfully completed
    Completed,
    /// Execution failed
    Failed,
    /// Execution aborted
    Aborted,
}

/// Summary of a past execution run.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct RunSummary {
    /// Unique run identifier
    pub id: String,

    /// Spec that was executed
    pub spec_name: String,

    /// Run status
    pub status: RunStatus,

    /// When the run started (ISO 8601)
    pub started_at: String,

    /// When the run completed
    pub ended_at: Option<String>,

    /// Number of batches
    pub batch_count: usize,

    /// Completed batches
    pub completed_batches: usize,
}

/// Detailed information about a past run.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct RunDetail {
    /// Unique run identifier
    pub id: String,

    /// Spec that was executed
    pub spec_name: String,

    /// Run status
    pub status: RunStatus,

    /// When the run started (ISO 8601)
    pub started_at: String,

    /// When the run completed
    pub ended_at: Option<String>,

    /// Was this a dry run
    pub dry_run: bool,

    /// Elapsed time in seconds
    pub elapsed_seconds: Option<u64>,

    /// Batches in this run
    pub batches: Vec<BatchRunDetail>,

    /// Error message if failed
    pub error: Option<String>,
}

/// Batch run detail.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct BatchRunDetail {
    /// Batch ID
    pub id: String,

    /// Batch name
    pub name: String,

    /// Batch status
    pub status: String,

    /// When started
    pub started_at: Option<String>,

    /// When completed
    pub ended_at: Option<String>,

    /// Git branch created
    pub branch: Option<String>,

    /// Error message if failed
    pub error: Option<String>,
}

// ============================================================================
// Request/Response Types
// ============================================================================

/// Request to create a run.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct CreateRunRequest {
    /// Run ID
    pub run_id: String,

    /// Dry run mode
    #[serde(default)]
    pub dry_run: bool,

    /// Batches to include
    pub batches: Vec<BatchInfo>,
}

/// Batch info for creating a run.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct BatchInfo {
    /// Batch ID
    pub id: String,

    /// Batch name
    pub name: String,
}

/// Request to update a run.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct UpdateRunRequest {
    /// New status
    pub status: Option<RunStatus>,

    /// Error message
    pub error: Option<String>,
}

/// Response for listing history.
pub type ListHistoryResponse = Vec<RunSummary>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_status_serialization() {
        let status = RunStatus::Completed;
        let json = serde_json::to_string(&status).expect("serialization failed");
        assert_eq!(json, "\"completed\"");
    }

    #[test]
    fn test_run_summary_serialization() {
        let summary = RunSummary {
            id: "run-001".to_string(),
            spec_name: "001-feature".to_string(),
            status: RunStatus::Completed,
            started_at: "2024-01-01T00:00:00Z".to_string(),
            ended_at: Some("2024-01-01T01:00:00Z".to_string()),
            batch_count: 3,
            completed_batches: 3,
        };

        let json = serde_json::to_string(&summary).expect("serialization failed");
        assert!(json.contains("\"status\":\"completed\""));
    }
}
