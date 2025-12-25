//! Job events for progress tracking.

use serde::{Deserialize, Serialize};

use crate::{AttemptResult, RunState};

/// Events emitted during job execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum JobEvent {
    /// Job state changed.
    StateChanged { state: RunState },

    /// A step started execution.
    StepStarted { step_id: String },

    /// A step completed successfully.
    StepCompleted { step_id: String, duration_ms: u64 },

    /// A step failed.
    StepFailed { step_id: String, error: String },

    /// An attempt started.
    AttemptStarted { number: u32 },

    /// An attempt completed.
    AttemptCompleted { number: u32, result: AttemptResult },
}
