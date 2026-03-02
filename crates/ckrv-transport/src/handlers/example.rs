//! # Example Handler
//!
//! Reference implementation showing the handler pattern.
//!
//! This module demonstrates how to create a transport-agnostic handler
//! that can be used by both Axum and Tauri backends.
//!
//! ## Usage
//!
//! See [docs/adding-endpoints.md](../../../docs/adding-endpoints.md) for
//! detailed instructions on adding new endpoints.

use crate::error::TransportError;
use crate::state::AppState;
use serde::{Deserialize, Serialize};

#[cfg(feature = "typescript")]
use ts_rs::TS;

// ============================================================================
// Types
// ============================================================================

/// Example request type.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct ExampleRequest {
    /// A name to include in the response
    pub name: String,
    /// An optional message
    pub message: Option<String>,
}

/// Example response type.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct ExampleResponse {
    /// Greeting message
    pub greeting: String,
    /// Project root path
    pub project_root: String,
    /// Timestamp of response
    pub timestamp: String,
}

// ============================================================================
// Handler
// ============================================================================

/// Handle example request.
///
/// This handler demonstrates the standard pattern:
/// 1. Takes `&AppState` as first argument
/// 2. Takes request type as second argument (if needed)
/// 3. Returns `Result<ResponseType, TransportError>`
///
/// # Arguments
///
/// * `state` - Shared application state
/// * `request` - The example request
///
/// # Returns
///
/// Returns an example response with a greeting.
///
/// # Errors
///
/// Returns `TransportError::BadRequest` if name is empty.
pub fn example_handler(
    state: &AppState,
    request: ExampleRequest,
) -> Result<ExampleResponse, TransportError> {
    // Validate input
    if request.name.is_empty() {
        return Err(TransportError::BadRequest(
            "Name cannot be empty".to_string(),
        ));
    }

    // Access project root from state
    let project_root = state.project_root.display().to_string();

    // Build response
    let greeting = match request.message {
        Some(msg) => format!("Hello, {}! Message: {}", request.name, msg),
        None => format!("Hello, {}!", request.name),
    };

    Ok(ExampleResponse {
        greeting,
        project_root,
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

/// Get example info (no request body).
///
/// This shows a handler that takes no request body.
pub fn get_example_info_handler(state: &AppState) -> Result<ExampleResponse, TransportError> {
    Ok(ExampleResponse {
        greeting: "Welcome to the example endpoint!".to_string(),
        project_root: state.project_root.display().to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_state() -> AppState {
        AppState::new(PathBuf::from("/tmp/test-example"))
    }

    #[tokio::test]
    async fn test_example_handler_success() {
        let state = test_state();
        let request = ExampleRequest {
            name: "World".to_string(),
            message: None,
        };

        let result = example_handler(&state, request);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.greeting, "Hello, World!");
        assert!(response.project_root.contains("test-example"));
    }

    #[tokio::test]
    async fn test_example_handler_with_message() {
        let state = test_state();
        let request = ExampleRequest {
            name: "Developer".to_string(),
            message: Some("How are you?".to_string()),
        };

        let result = example_handler(&state, request);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.greeting.contains("Developer"));
        assert!(response.greeting.contains("How are you?"));
    }

    #[tokio::test]
    async fn test_example_handler_empty_name() {
        let state = test_state();
        let request = ExampleRequest {
            name: String::new(),
            message: None,
        };

        let result = example_handler(&state, request);
        assert!(result.is_err());

        match result {
            Err(TransportError::BadRequest(msg)) => {
                assert!(msg.contains("empty"));
            }
            _ => panic!("Expected BadRequest error"),
        }
    }

    #[tokio::test]
    async fn test_get_example_info() {
        let state = test_state();
        let result = get_example_info_handler(&state);
        assert!(result.is_ok());
    }
}
