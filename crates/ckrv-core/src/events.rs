//! Job events for progress tracking.

// ============================================================
// IMPORTS
// ============================================================

use serde::{Deserialize, Serialize};

use crate::{AttemptResult, RunState};

// ============================================================
// TYPES
// ============================================================

/// Events emitted during job execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum JobEvent {
    /// Job state changed.
    StateChanged {
        /// The new run state.
        state: RunState,
    },

    /// A step started execution.
    StepStarted {
        /// ID of the step that started.
        step_id: String,
    },

    /// A step completed successfully.
    StepCompleted {
        /// ID of the step that completed.
        step_id: String,
        /// How long the step took in milliseconds.
        duration_ms: u64,
    },

    /// A step failed.
    StepFailed {
        /// ID of the step that failed.
        step_id: String,
        /// The error message.
        error: String,
    },

    /// An attempt started.
    AttemptStarted {
        /// The attempt number (1-indexed).
        number: u32,
    },

    /// An attempt completed.
    AttemptCompleted {
        /// The attempt number (1-indexed).
        number: u32,
        /// The result of the attempt.
        result: AttemptResult,
    },
}
