//! Step execution result for tracking workflow step outcomes.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

impl StepExecutionResult {
    /// Create a successful result.
    #[must_use]
    pub fn success(step_id: impl Into<String>, duration_ms: u64) -> Self {
        Self {
            step_id: step_id.into(),
            status: StepExecutionStatus::Success,
            outputs: HashMap::new(),
            stdout: String::new(),
            stderr: String::new(),
            duration_ms,
        }
    }

    /// Create a failed result.
    #[must_use]
    pub fn failed(step_id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            step_id: step_id.into(),
            status: StepExecutionStatus::Failed,
            outputs: HashMap::new(),
            stdout: String::new(),
            stderr: error.into(),
            duration_ms: 0,
        }
    }

    /// Add an output to the result.
    #[must_use]
    pub fn with_output(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.outputs.insert(name.into(), value.into());
        self
    }

    /// Set stdout.
    #[must_use]
    pub fn with_stdout(mut self, stdout: impl Into<String>) -> Self {
        self.stdout = stdout.into();
        self
    }

    /// Set stderr.
    #[must_use]
    pub fn with_stderr(mut self, stderr: impl Into<String>) -> Self {
        self.stderr = stderr.into();
        self
    }

    /// Check if the step succeeded.
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.status == StepExecutionStatus::Success
    }
}

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
