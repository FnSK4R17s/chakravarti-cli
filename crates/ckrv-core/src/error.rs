//! Core error types.

use thiserror::Error;

/// Errors that can occur in the core domain.
#[derive(Debug, Error)]
pub enum CoreError {
    /// Spec not found at the given path.
    #[error("Spec not found: {0}")]
    SpecNotFound(String),

    /// Invalid spec format or content.
    #[error("Invalid spec: {0}")]
    InvalidSpec(String),

    /// Invalid state transition attempted.
    #[error("Invalid state transition: {from} -> {to}")]
    InvalidStateTransition { from: String, to: String },

    /// Maximum attempts exceeded.
    #[error("Max attempts exceeded: {attempts}")]
    MaxAttemptsExceeded { attempts: u32 },

    /// Job not found.
    #[error("Job not found: {0}")]
    JobNotFound(String),
}
