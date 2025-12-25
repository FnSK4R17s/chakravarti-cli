//! Integration error types.

use thiserror::Error;

/// Errors from integration operations.
#[derive(Debug, Error)]
pub enum IntegrationError {
    /// API request failed.
    #[error("API request failed: {0}")]
    RequestFailed(String),

    /// Authentication failed.
    #[error("Authentication failed: {0}")]
    AuthFailed(String),
}
