//! Job run state machine.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// The current state of a job in its lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum RunState {
    /// Job created, not yet started.
    Pending,

    /// Generating plan from spec.
    Planning,

    /// Executing plan steps.
    Executing { attempt: u32, step: String },

    /// Running verification.
    Verifying { attempt: u32 },

    /// Job completed successfully.
    Succeeded { attempt: u32, diff_path: PathBuf },

    /// Job failed after all retries.
    Failed { attempts: u32, last_error: String },

    /// Job was cancelled by user.
    Cancelled,
}

impl RunState {
    /// Check if the job is in a terminal state.
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Succeeded { .. } | Self::Failed { .. } | Self::Cancelled
        )
    }

    /// Check if the job is running.
    #[must_use]
    pub fn is_running(&self) -> bool {
        matches!(
            self,
            Self::Planning | Self::Executing { .. } | Self::Verifying { .. }
        )
    }

    /// Get a display name for the state.
    #[must_use]
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Planning => "planning",
            Self::Executing { .. } => "executing",
            Self::Verifying { .. } => "verifying",
            Self::Succeeded { .. } => "succeeded",
            Self::Failed { .. } => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for RunState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "Pending"),
            Self::Planning => write!(f, "Planning"),
            Self::Executing { attempt, step } => {
                write!(f, "Executing (attempt {attempt}, step {step})")
            }
            Self::Verifying { attempt } => write!(f, "Verifying (attempt {attempt})"),
            Self::Succeeded { attempt, .. } => write!(f, "Succeeded (attempt {attempt})"),
            Self::Failed { attempts, .. } => write!(f, "Failed after {attempts} attempts"),
            Self::Cancelled => write!(f, "Cancelled"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pending_is_not_terminal() {
        assert!(!RunState::Pending.is_terminal());
    }

    #[test]
    fn test_planning_is_not_terminal() {
        assert!(!RunState::Planning.is_terminal());
    }

    #[test]
    fn test_executing_is_not_terminal() {
        let state = RunState::Executing {
            attempt: 1,
            step: "analyze".to_string(),
        };
        assert!(!state.is_terminal());
    }

    #[test]
    fn test_verifying_is_not_terminal() {
        let state = RunState::Verifying { attempt: 1 };
        assert!(!state.is_terminal());
    }

    #[test]
    fn test_succeeded_is_terminal() {
        let state = RunState::Succeeded {
            attempt: 1,
            diff_path: PathBuf::from("/tmp/diff"),
        };
        assert!(state.is_terminal());
    }

    #[test]
    fn test_failed_is_terminal() {
        let state = RunState::Failed {
            attempts: 3,
            last_error: "tests failed".to_string(),
        };
        assert!(state.is_terminal());
    }

    #[test]
    fn test_cancelled_is_terminal() {
        assert!(RunState::Cancelled.is_terminal());
    }

    #[test]
    fn test_pending_is_not_running() {
        assert!(!RunState::Pending.is_running());
    }

    #[test]
    fn test_planning_is_running() {
        assert!(RunState::Planning.is_running());
    }

    #[test]
    fn test_executing_is_running() {
        let state = RunState::Executing {
            attempt: 1,
            step: "generate".to_string(),
        };
        assert!(state.is_running());
    }

    #[test]
    fn test_verifying_is_running() {
        let state = RunState::Verifying { attempt: 2 };
        assert!(state.is_running());
    }

    #[test]
    fn test_succeeded_is_not_running() {
        let state = RunState::Succeeded {
            attempt: 1,
            diff_path: PathBuf::from("/tmp/diff"),
        };
        assert!(!state.is_running());
    }

    #[test]
    fn test_display_names() {
        assert_eq!(RunState::Pending.display_name(), "pending");
        assert_eq!(RunState::Planning.display_name(), "planning");
        assert_eq!(RunState::Cancelled.display_name(), "cancelled");
    }

    #[test]
    fn test_display_format() {
        let state = RunState::Executing {
            attempt: 2,
            step: "test".to_string(),
        };
        assert_eq!(format!("{state}"), "Executing (attempt 2, step test)");
    }

    #[test]
    fn test_serialization_roundtrip() {
        let state = RunState::Succeeded {
            attempt: 1,
            diff_path: PathBuf::from("/tmp/diff.patch"),
        };
        let json = serde_json::to_string(&state).expect("serialize");
        let parsed: RunState = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(state, parsed);
    }

    #[test]
    fn test_serialization_uses_snake_case() {
        let json = serde_json::to_string(&RunState::Pending).expect("serialize");
        assert!(json.contains("pending"));
    }
}
