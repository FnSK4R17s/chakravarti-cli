//! # Spec Types
//!
//! Types for feature specification management.

use serde::{Deserialize, Serialize};

#[cfg(feature = "typescript")]
use ts_rs::TS;

/// Feature specification summary for listing.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct SpecSummary {
    /// Spec name (directory name)
    pub name: String,

    /// Relative path to spec directory
    pub path: String,

    /// Human-readable title
    pub title: Option<String>,

    /// Current status
    pub status: SpecStatus,

    /// Has execution plan
    pub has_plan: bool,

    /// Has tasks
    pub has_tasks: bool,

    /// Has design artifact
    pub has_design: bool,

    /// Has implementation artifact
    pub has_implementation: bool,

    /// Implementation branch name if available
    pub implementation_branch: Option<String>,

    /// Number of tasks
    pub task_count: usize,

    /// Created timestamp (ISO 8601)
    pub created_at: Option<String>,

    /// Last modified timestamp (ISO 8601)
    pub updated_at: Option<String>,
}

/// Detailed specification information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct SpecDetail {
    /// Spec ID
    pub id: String,

    /// Overview/goal text
    pub overview: Option<String>,

    /// Status string
    pub status: Option<String>,

    /// Branch name
    pub branch: Option<String>,

    /// Created timestamp
    pub created: Option<String>,

    /// User stories
    pub user_stories: Vec<serde_json::Value>,

    /// Requirements
    pub requirements: Vec<serde_json::Value>,

    /// Success criteria
    pub success_criteria: Vec<serde_json::Value>,

    /// Assumptions
    pub assumptions: Vec<String>,

    /// Edge cases
    pub edge_cases: Vec<String>,

    /// Raw YAML content
    pub raw_yaml: Option<String>,
}

/// Specification status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[serde(rename_all = "snake_case")]
pub enum SpecStatus {
    /// Initial draft state
    #[default]
    Draft,
    /// Has execution plan
    Planned,
    /// Currently in progress
    InProgress,
    /// Successfully implemented
    Implemented,
    /// Execution failed
    Failed,
}

// ============================================================
// Request/Response Types
// ============================================================

/// Request to create a new specification.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct CreateSpecRequest {
    /// Description of what to build
    pub description: String,

    /// Optional name for the spec
    pub name: Option<String>,
}

/// Request to update a specification.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct UpdateSpecRequest {
    /// Raw YAML content to save
    pub raw_yaml: Option<String>,
}

/// Response with a list of specs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct ListSpecsResponse {
    /// List of spec summaries
    pub specs: Vec<SpecSummary>,
    /// Total count of specs
    pub count: usize,
}

// ============================================================
// Plan Types
// ============================================================

/// Plan summary for listing.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct PlanSummary {
    /// Spec name this plan belongs to
    pub spec_name: String,

    /// Number of batches
    pub batch_count: usize,

    /// Total estimated cost
    pub total_estimated_cost: Option<f64>,

    /// Status
    pub status: Option<String>,
}

/// Plan detail.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct PlanDetail {
    /// Spec name
    pub spec_name: String,

    /// Batches in this plan
    pub batches: Vec<BatchSummary>,

    /// Raw YAML content
    pub raw_yaml: Option<String>,
}

/// Batch summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct BatchSummary {
    /// Batch ID
    pub id: String,

    /// Batch name
    pub name: String,

    /// Number of tasks
    pub task_count: usize,

    /// Batch status
    pub status: String,

    /// Estimated cost
    pub estimated_cost: Option<f64>,
}

/// Response with a list of plans.
pub type ListPlansResponse = Vec<PlanSummary>;

// ============================================================
// Task Types
// ============================================================

/// Task summary for listing.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct TaskSummary {
    /// Task ID
    pub id: String,

    /// Task title
    pub title: String,

    /// Current status
    pub status: TaskStatus,

    /// Complexity (1-5)
    pub complexity: u8,

    /// Phase this task belongs to
    pub phase: Option<String>,
}

/// Task detail.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct TaskDetail {
    /// Task ID
    pub id: String,

    /// Task title
    pub title: String,

    /// Task description
    pub description: Option<String>,

    /// Current status
    pub status: TaskStatus,

    /// Complexity (1-5)
    pub complexity: u8,

    /// Phase this task belongs to
    pub phase: Option<String>,

    /// Target file path
    pub file_path: Option<String>,

    /// Related user story
    pub user_story: Option<String>,

    /// Can run in parallel
    pub parallel: bool,

    /// Model tier
    pub model_tier: Option<String>,

    /// Estimated tokens
    pub estimated_tokens: Option<u32>,

    /// Context files required
    pub context_required: Option<Vec<String>>,
}

/// Task status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    /// Waiting to start
    #[default]
    Pending,
    /// Queued for execution
    Queued,
    /// Currently running
    Running,
    /// In progress (alias for running)
    InProgress,
    /// Successfully completed
    Completed,
    /// Task failed
    Failed,
    /// Task skipped
    Skipped,
}

/// Request to update a task.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct UpdateTaskRequest {
    /// New status
    pub status: Option<TaskStatus>,
}

/// Response with a list of tasks.
pub type ListTasksResponse = Vec<TaskSummary>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spec_status_serialization() {
        let status = SpecStatus::InProgress;
        let json = serde_json::to_string(&status).expect("serialization failed");
        assert_eq!(json, "\"in_progress\"");
    }

    #[test]
    fn test_task_status_serialization() {
        let status = TaskStatus::Completed;
        let json = serde_json::to_string(&status).expect("serialization failed");
        assert_eq!(json, "\"completed\"");
    }
}
