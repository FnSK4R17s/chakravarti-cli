//! Metrics error types.

use thiserror::Error;

/// Errors from metrics operations.
#[derive(Debug, Error)]
pub enum MetricsError {
    /// Failed to save metrics.
    #[error("Failed to save metrics: {0}")]
    StorageError(String),

    /// Failed to load metrics.
    #[error("Failed to load metrics: {0}")]
    LoadFailed(String),

    /// Metrics not found.
    #[error("Metrics not found for job: {0}")]
    NotFound(String),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    SerializationError(String),
}
