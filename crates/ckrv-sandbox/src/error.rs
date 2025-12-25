//! Sandbox error types.

use thiserror::Error;

/// Errors from sandbox operations.
#[derive(Debug, Error)]
pub enum SandboxError {
    /// Docker/container runtime not available.
    #[error("Container runtime not available: {0}")]
    RuntimeNotAvailable(String),

    /// Image pull failed.
    #[error("Failed to pull image: {0}")]
    ImagePullFailed(String),

    /// Container creation failed.
    #[error("Failed to create container: {0}")]
    ContainerCreateFailed(String),

    /// Container start failed.
    #[error("Failed to start container: {0}")]
    ContainerStartFailed(String),

    /// Command execution failed.
    #[error("Command execution failed: {0}")]
    ExecutionFailed(String),

    /// Command not allowed by allowlist.
    #[error("Command not allowed: {0}")]
    CommandNotAllowed(String),

    /// Timeout exceeded.
    #[error("Execution timeout exceeded")]
    Timeout,

    /// Container error.
    #[error("Container error: {0}")]
    ContainerError(String),
}

impl SandboxError {
    /// Check if error is retryable.
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::RuntimeNotAvailable(_) | Self::ContainerStartFailed(_) | Self::Timeout
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = SandboxError::RuntimeNotAvailable("Docker not found".to_string());
        assert!(err.to_string().contains("Docker not found"));
    }

    #[test]
    fn test_is_retryable() {
        assert!(SandboxError::Timeout.is_retryable());
        assert!(!SandboxError::CommandNotAllowed("rm -rf".to_string()).is_retryable());
    }

    #[test]
    fn test_image_pull_error() {
        let err = SandboxError::ImagePullFailed("network error".to_string());
        assert!(err.to_string().contains("pull"));
    }
}
