//! # Transport Errors
//!
//! Unified error types for the transport layer.
//!
//! ## Overview
//!
//! `TransportError` provides a consistent error type that can be converted
//! to transport-specific responses (HTTP status codes for Axum, error strings
//! for Tauri).
//!
//! ## Error Variants
//!
//! | Variant | HTTP Status | Use Case |
//! |---------|-------------|----------|
//! | `NotFound` | 404 | Resource doesn't exist |
//! | `BadRequest` | 400 | Invalid input |
//! | `Unauthorized` | 401 | Authentication required |
//! | `Forbidden` | 403 | Permission denied |
//! | `Conflict` | 409 | State conflict |
//! | `Internal` | 500 | Server error |
//! | `ServiceUnavailable` | 503 | Dependency unavailable |

use serde::Serialize;
use thiserror::Error;

/// Errors that can occur during API request handling.
///
/// Maps to HTTP status codes for Axum and error strings for Tauri.
#[derive(Debug, Error, Serialize, Clone)]
#[serde(tag = "error", content = "message")]
pub enum TransportError {
    /// Resource not found (404)
    #[error("Not found: {0}")]
    NotFound(String),

    /// Invalid request parameters (400)
    #[error("Bad request: {0}")]
    BadRequest(String),

    /// Authentication required (401)
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// Operation not permitted (403)
    #[error("Forbidden: {0}")]
    Forbidden(String),

    /// Conflict with current state (409)
    #[error("Conflict: {0}")]
    Conflict(String),

    /// Internal server error (500)
    #[error("Internal error: {0}")]
    Internal(String),

    /// Service unavailable (503)
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
}

// ============================================================================
// Axum Integration
// ============================================================================

#[cfg(feature = "axum")]
mod axum_impl {
    use super::TransportError;
    use axum::http::StatusCode;
    use axum::response::{IntoResponse, Response};
    use axum::Json;

    impl IntoResponse for TransportError {
        fn into_response(self) -> Response {
            let status = match &self {
                TransportError::NotFound(_) => StatusCode::NOT_FOUND,
                TransportError::BadRequest(_) => StatusCode::BAD_REQUEST,
                TransportError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
                TransportError::Forbidden(_) => StatusCode::FORBIDDEN,
                TransportError::Conflict(_) => StatusCode::CONFLICT,
                TransportError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
                TransportError::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            };

            let body = Json(serde_json::json!({
                "error": self.to_string()
            }));

            (status, body).into_response()
        }
    }
}

// ============================================================================
// Tauri Integration
// ============================================================================

#[cfg(feature = "tauri")]
impl From<TransportError> for String {
    fn from(err: TransportError) -> String {
        err.to_string()
    }
}

// ============================================================================
// From Implementations
// ============================================================================

impl From<std::io::Error> for TransportError {
    fn from(err: std::io::Error) -> Self {
        TransportError::Internal(format!("IO error: {err}"))
    }
}

impl From<serde_json::Error> for TransportError {
    fn from(err: serde_json::Error) -> Self {
        TransportError::BadRequest(format!("JSON error: {err}"))
    }
}

impl From<serde_yaml::Error> for TransportError {
    fn from(err: serde_yaml::Error) -> Self {
        TransportError::BadRequest(format!("YAML error: {err}"))
    }
}

impl From<anyhow::Error> for TransportError {
    fn from(err: anyhow::Error) -> Self {
        TransportError::Internal(format!("{err:#}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = TransportError::NotFound("spec-001".to_string());
        assert_eq!(err.to_string(), "Not found: spec-001");
    }

    #[test]
    fn test_error_serialization() {
        let err = TransportError::BadRequest("invalid input".to_string());
        let json = serde_json::to_string(&err).expect("serialization failed");
        // Tagged enum serializes as {"error":"BadRequest","message":"invalid input"}
        assert!(json.contains("BadRequest"));
        assert!(json.contains("invalid input"));
    }
}
