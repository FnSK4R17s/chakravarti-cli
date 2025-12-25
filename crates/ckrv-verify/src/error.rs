//! Verify error types.

use thiserror::Error;

/// Errors from verification operations.
#[derive(Debug, Error)]
pub enum VerifyError {
    /// Test execution failed.
    #[error("Test execution failed: {0}")]
    ExecutionFailed(String),

    /// Test parsing failed.
    #[error("Failed to parse test results: {0}")]
    ParseFailed(String),

    /// Acceptance criteria not met.
    #[error("Acceptance criteria not met: {0}")]
    AcceptanceFailed(String),
}
