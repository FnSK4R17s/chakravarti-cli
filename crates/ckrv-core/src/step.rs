//! Step types for plan execution.

use serde::{Deserialize, Serialize};

/// A single step in an execution plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    /// Unique identifier within the plan.
    pub id: String,

    /// Human-readable name.
    pub name: String,

    /// Type of step (determines execution strategy).
    pub step_type: StepType,

    /// IDs of steps that must complete before this one.
    #[serde(default)]
    pub dependencies: Vec<String>,

    /// Current execution status.
    pub status: StepStatus,

    /// Captured output/result.
    pub output: Option<String>,

    /// Execution duration in milliseconds.
    #[serde(default)]
    pub duration_ms: Option<u64>,
}

impl Step {
    /// Create a new step with default status.
    #[must_use]
    pub fn new(id: impl Into<String>, name: impl Into<String>, step_type: StepType) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            step_type,
            dependencies: Vec::new(),
            status: StepStatus::Pending,
            output: None,
            duration_ms: None,
        }
    }

    /// Add a dependency to this step.
    #[must_use]
    pub fn with_dependency(mut self, dep_id: impl Into<String>) -> Self {
        self.dependencies.push(dep_id.into());
        self
    }
}

/// Type of step, determining how it should be executed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepType {
    /// Analyze code/context (model call).
    Analyze,
    /// Generate code (model call).
    Generate,
    /// Execute a command (sandbox).
    Execute,
    /// Run tests (verification).
    Test,
    /// Commit changes (git).
    Commit,
}

/// Status of a step's execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum StepStatus {
    /// Not yet started.
    Pending,
    /// Currently running.
    Running,
    /// Completed successfully.
    Completed,
    /// Failed with an error.
    Failed { error: String },
    /// Skipped (dependency failed or not needed).
    Skipped { reason: String },
}

impl StepStatus {
    /// Check if the step is complete (succeeded, failed, or skipped).
    #[must_use]
    pub fn is_complete(&self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Failed { .. } | Self::Skipped { .. }
        )
    }

    /// Check if the step succeeded.
    #[must_use]
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Completed)
    }
}
