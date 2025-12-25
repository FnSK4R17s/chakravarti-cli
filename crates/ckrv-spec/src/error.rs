//! Spec loading and parsing errors.

use thiserror::Error;

/// Errors from spec operations.
#[derive(Debug, Error)]
pub enum SpecError {
    /// Failed to read spec file.
    #[error("Failed to read spec file: {0}")]
    ReadError(String),

    /// Failed to parse spec YAML.
    #[error("Failed to parse spec: {0}")]
    ParseError(String),

    /// Spec validation failed.
    #[error("Spec validation failed: {0}")]
    ValidationError(String),
}
