//! # Execution Types
//!
//! Types for task execution and status tracking.

use serde::{Deserialize, Serialize};

#[cfg(feature = "typescript")]
use ts_rs::TS;

/// Execution run summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct ExecutionRun {
    /// Unique run identifier
    pub id: String,

    /// Spec this run belongs to
    pub spec: String,

    /// When the run started (ISO 8601)
    pub started_at: String,

    /// When the run completed (if finished)
    pub completed_at: Option<String>,

    /// Current status
    pub status: ExecutionStatus,

    /// Tasks in this run
    pub tasks: Vec<TaskRun>,
}

/// Execution status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus {
    /// Waiting to start
    #[default]
    Pending,
    /// Currently running
    Running,
    /// Successfully completed
    Completed,
    /// Execution failed
    Failed,
    /// Execution cancelled
    Cancelled,
}

/// Individual task within an execution run.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct TaskRun {
    /// Task identifier
    pub id: String,

    /// Task title
    pub title: String,

    /// Current status
    pub status: TaskStatus,

    /// Agent assigned to this task
    pub agent: Option<String>,

    /// Git worktree path
    pub worktree: Option<String>,

    /// Attempt count
    pub attempts: u32,

    /// Error message (if failed)
    pub error: Option<String>,

    /// Task output/logs if available
    pub output: Option<String>,
}

// Re-export TaskStatus from specs module
pub use super::specs::TaskStatus;

// ============================================================================
// Request/Response Types
// ============================================================================

/// Request to start execution.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct StartExecutionRequest {
    /// Spec to execute
    pub spec: String,

    /// Optional agent to use (defaults to system default)
    pub agent: Option<String>,

    /// Whether to perform a dry run
    pub dry_run: Option<bool>,

    /// Optional list of specific task IDs to run
    pub task_ids: Option<Vec<String>>,
}

/// Request to stop execution.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct StopExecutionRequest {
    /// Spec to stop execution for
    pub spec: String,
}

/// Response from start execution.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct StartExecutionResponse {
    /// Run identifier
    pub run_id: String,

    /// Initial status
    pub status: ExecutionStatus,
}

/// Query parameters for execution status.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ExecutionStatusQuery {
    /// Include task details
    pub include_tasks: Option<bool>,

    /// Include task output
    pub include_output: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_status_serialization() {
        let status = ExecutionStatus::Running;
        let json = serde_json::to_string(&status).expect("serialization failed");
        assert_eq!(json, "\"running\"");
    }

    #[test]
    fn test_task_status_serialization() {
        let status = TaskStatus::Queued;
        let json = serde_json::to_string(&status).expect("serialization failed");
        assert_eq!(json, "\"queued\"");
    }

    #[test]
    fn test_execution_run_serialization() {
        let run = ExecutionRun {
            id: "run-001".to_string(),
            spec: "001-feature".to_string(),
            started_at: "2024-01-01T00:00:00Z".to_string(),
            completed_at: None,
            status: ExecutionStatus::Running,
            tasks: vec![],
        };

        let json = serde_json::to_string(&run).expect("serialization failed");
        assert!(json.contains("\"status\":\"running\""));
    }
}
