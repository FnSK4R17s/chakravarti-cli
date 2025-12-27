//! Job and Attempt types.

use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{RunState, Spec};

/// A job execution configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobConfig {
    /// Optimization mode for model selection.
    pub optimize: OptimizeMode,

    /// Override for the planner model.
    pub planner_model: Option<String>,

    /// Override for the executor model.
    pub executor_model: Option<String>,

    /// Maximum retry attempts.
    pub max_attempts: u32,
}

impl Default for JobConfig {
    fn default() -> Self {
        Self {
            optimize: OptimizeMode::Balanced,
            planner_model: None,
            executor_model: None,
            max_attempts: 3,
        }
    }
}

/// Optimization mode for model routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OptimizeMode {
    /// Prefer cheaper models, accept longer execution time.
    Cost,
    /// Prefer faster models, accept higher cost.
    Time,
    /// Balance cost and time.
    Balanced,
}

/// A job represents a single execution of a specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    /// Unique job identifier.
    pub id: String,

    /// ID of the specification being executed.
    pub spec_id: String,

    /// ID of the generated plan.
    pub plan_id: Option<String>,

    /// Current state in the lifecycle.
    pub state: RunState,

    /// Job configuration.
    pub config: JobConfig,

    /// All attempts made for this job.
    pub attempts: Vec<Attempt>,

    /// When the job was created.
    pub created_at: DateTime<Utc>,

    /// When the job was last updated.
    pub updated_at: DateTime<Utc>,
}

impl Job {
    /// Create a new job from a specification.
    #[must_use]
    pub fn new(spec_id: String, config: JobConfig) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            spec_id,
            plan_id: None,
            state: RunState::Pending,
            config,
            attempts: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Get the number of attempts made.
    #[must_use]
    pub fn attempt_count(&self) -> u32 {
        self.attempts.len() as u32
    }

    /// Add an attempt result.
    pub fn add_attempt(&mut self, result: AttemptResult) {
        let attempt = Attempt {
            id: uuid::Uuid::new_v4().to_string(),
            job_id: self.id.clone(),
            number: self.attempt_count() + 1,
            worktree_path: PathBuf::new(),
            result,
            started_at: Utc::now(),
            finished_at: Some(Utc::now()),
        };
        self.attempts.push(attempt);
        self.updated_at = Utc::now();
    }
}

/// An attempt represents one execution cycle within a job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attempt {
    /// Unique attempt identifier.
    pub id: String,

    /// Parent job ID.
    pub job_id: String,

    /// Attempt number (1-indexed).
    pub number: u32,

    /// Path to the worktree for this attempt.
    pub worktree_path: PathBuf,

    /// Result of the attempt.
    pub result: AttemptResult,

    /// When the attempt started.
    pub started_at: DateTime<Utc>,

    /// When the attempt finished (if complete).
    pub finished_at: Option<DateTime<Utc>>,
}

/// Result of an attempt execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum AttemptResult {
    /// Attempt is still in progress.
    InProgress,

    /// Attempt succeeded with a diff.
    Succeeded {
        /// The generated diff content.
        diff: String,
    },

    /// Verification failed.
    VerificationFailed {
        /// Reason for the failure.
        reason: String,
    },

    /// Execution failed.
    ExecutionFailed {
        /// The step that failed.
        step: String,
        /// Error message.
        error: String,
    },
}

impl AttemptResult {
    /// Create a success result.
    #[must_use]
    pub fn success(diff: impl Into<String>) -> Self {
        Self::Succeeded { diff: diff.into() }
    }

    /// Create a failure result.
    #[must_use]
    pub fn failure(error: impl Into<String>) -> Self {
        Self::ExecutionFailed {
            step: "unknown".to_string(),
            error: error.into(),
        }
    }

    /// Check if the result is a success.
    #[must_use]
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Succeeded { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_spec() -> Spec {
        Spec {
            id: "test_spec".to_string(),
            goal: "Test goal".to_string(),
            constraints: vec![],
            acceptance: vec!["Test passes".to_string()],
            verify: None,
            source_path: None,
        }
    }

    #[test]
    fn test_job_config_default() {
        let config = JobConfig::default();
        assert_eq!(config.optimize, OptimizeMode::Balanced);
        assert!(config.planner_model.is_none());
        assert!(config.executor_model.is_none());
        assert_eq!(config.max_attempts, 3);
    }

    #[test]
    fn test_job_new_generates_uuid() {
        let spec = test_spec();
        let job = Job::new(spec.id.clone(), JobConfig::default());

        assert!(!job.id.is_empty());
        assert_eq!(job.spec_id, "test_spec");
    }

    #[test]
    fn test_job_starts_pending() {
        let spec = test_spec();
        let job = Job::new(spec.id.clone(), JobConfig::default());

        assert_eq!(job.state, RunState::Pending);
    }

    #[test]
    fn test_job_starts_with_no_plan() {
        let spec = test_spec();
        let job = Job::new(spec.id.clone(), JobConfig::default());

        assert!(job.plan_id.is_none());
    }

    #[test]
    fn test_job_starts_with_no_attempts() {
        let spec = test_spec();
        let job = Job::new(spec.id.clone(), JobConfig::default());

        assert!(job.attempts.is_empty());
    }

    #[test]
    fn test_job_has_timestamps() {
        let spec = test_spec();
        let job = Job::new(spec.id.clone(), JobConfig::default());

        assert_eq!(job.created_at, job.updated_at);
    }

    #[test]
    fn test_job_uses_provided_config() {
        let spec = test_spec();
        let config = JobConfig {
            optimize: OptimizeMode::Cost,
            planner_model: Some("gpt-4".to_string()),
            executor_model: None,
            max_attempts: 5,
        };
        let job = Job::new(spec.id.clone(), config);

        assert_eq!(job.config.optimize, OptimizeMode::Cost);
        assert_eq!(job.config.planner_model, Some("gpt-4".to_string()));
        assert_eq!(job.config.max_attempts, 5);
    }

    #[test]
    fn test_optimize_mode_serialization() {
        let cost = serde_json::to_string(&OptimizeMode::Cost).expect("serialize");
        assert!(cost.contains("cost"));

        let time = serde_json::to_string(&OptimizeMode::Time).expect("serialize");
        assert!(time.contains("time"));
    }

    #[test]
    fn test_attempt_result_serialization() {
        let succeeded = AttemptResult::Succeeded {
            diff: "--- a/file\n+++ b/file".to_string(),
        };
        let json = serde_json::to_string(&succeeded).expect("serialize");
        assert!(json.contains("succeeded"));

        let parsed: AttemptResult = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(parsed, AttemptResult::Succeeded { .. }));
    }

    #[test]
    fn test_job_serialization_roundtrip() {
        let spec = test_spec();
        let job = Job::new(spec.id.clone(), JobConfig::default());

        let json = serde_json::to_string(&job).expect("serialize");
        let parsed: Job = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(job.id, parsed.id);
        assert_eq!(job.spec_id, parsed.spec_id);
        assert_eq!(job.state, parsed.state);
    }
}
