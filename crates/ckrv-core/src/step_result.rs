//! Step execution result for tracking workflow step outcomes.

// ============================================================
// IMPORTS
// ============================================================

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================
// TYPES
// ============================================================

/// Result of executing a single workflow step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepExecutionResult {
    /// Step ID that was executed.
    pub step_id: String,
    /// Execution status.
    pub status: StepExecutionStatus,
    /// Outputs collected from this step (name -> value).
    pub outputs: HashMap<String, String>,
    /// Standard output from the agent.
    pub stdout: String,
    /// Standard error from the agent.
    pub stderr: String,
    /// Execution duration in milliseconds.
    pub duration_ms: u64,
}

/// Status of a step execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StepExecutionStatus {
    /// Step completed successfully.
    Success,
    /// Step failed.
    Failed,
    /// Step was skipped.
    Skipped,
    /// Step timed out.
    Timeout,
}

// ============================================================
// IMPLEMENTATION
// ============================================================

impl StepExecutionResult {
    /// Create a successful result.
    #[must_use]
    pub fn success(step_id: &str, duration_ms: u64) -> Self {
        Self {
            step_id: step_id.to_owned(),
            status: StepExecutionStatus::Success,
            outputs: HashMap::new(),
            stdout: String::new(),
            stderr: String::new(),
            duration_ms,
        }
    }

    /// Create a failed result.
    #[must_use]
    pub fn failed(step_id: &str, error: &str) -> Self {
        Self {
            step_id: step_id.to_owned(),
            status: StepExecutionStatus::Failed,
            outputs: HashMap::new(),
            stdout: String::new(),
            stderr: error.to_owned(),
            duration_ms: 0,
        }
    }

    /// Add an output to the result.
    #[must_use]
    pub fn with_output(mut self, name: &str, value: &str) -> Self {
        self.outputs.insert(name.to_owned(), value.to_owned());
        self
    }

    /// Set stdout.
    #[must_use]
    pub fn with_stdout(mut self, stdout: &str) -> Self {
        stdout.clone_into(&mut self.stdout);
        self
    }

    /// Set stderr.
    #[must_use]
    pub fn with_stderr(mut self, stderr: &str) -> Self {
        stderr.clone_into(&mut self.stderr);
        self
    }

    /// Check if the step succeeded.
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.status == StepExecutionStatus::Success
    }
}

// ============================================================
// TESTS
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success_result() {
        let result = StepExecutionResult::success("plan", 1000)
            .with_output("plan_file", "plan.md")
            .with_stdout("Generated plan");

        assert!(result.is_success());
        assert_eq!(
            result.outputs.get("plan_file"),
            Some(&"plan.md".to_string())
        );
        assert_eq!(result.duration_ms, 1000);
    }

    #[test]
    fn test_failed_result() {
        let result = StepExecutionResult::failed("implement", "Agent crashed");

        assert!(!result.is_success());
        assert_eq!(result.status, StepExecutionStatus::Failed);
        assert_eq!(result.stderr, "Agent crashed");
    }
}
