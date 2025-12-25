//! Model error types.

use thiserror::Error;

/// Errors from model operations.
#[derive(Debug, Error)]
pub enum ModelError {
    /// Configuration error.
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Network error.
    #[error("Network error: {0}")]
    NetworkError(String),

    /// API error with status code.
    #[error("API error ({status}): {message}")]
    ApiError {
        /// HTTP status code.
        status: u16,
        /// Error message.
        message: String,
    },

    /// Response parsing failed.
    #[error("Failed to parse response: {0}")]
    ParseError(String),

    /// Rate limit exceeded.
    #[error("Rate limit exceeded, retry after {retry_after:?}s")]
    RateLimited {
        /// Seconds to wait before retry.
        retry_after: Option<u32>,
    },

    /// Model not found.
    #[error("Model not found: {0}")]
    ModelNotFound(String),

    /// Timeout.
    #[error("Request timed out after {0}s")]
    Timeout(u64),
}

impl ModelError {
    /// Check if error is retryable.
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::NetworkError(_) | Self::RateLimited { .. } | Self::Timeout(_)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_error() {
        let err = ModelError::ConfigError("missing key".to_string());
        assert!(err.to_string().contains("Configuration"));
    }

    #[test]
    fn test_api_error() {
        let err = ModelError::ApiError {
            status: 401,
            message: "Unauthorized".to_string(),
        };
        assert!(err.to_string().contains("401"));
    }

    #[test]
    fn test_is_retryable() {
        assert!(ModelError::NetworkError("timeout".to_string()).is_retryable());
        assert!(ModelError::RateLimited {
            retry_after: Some(30)
        }
        .is_retryable());
        assert!(!ModelError::ConfigError("bad".to_string()).is_retryable());
    }
}
