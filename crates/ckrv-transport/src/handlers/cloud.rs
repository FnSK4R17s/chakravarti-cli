//! # Cloud Handler
//!
//! Handler for cloud service status.

use crate::error::TransportError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Cloud service status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudStatus {
    /// Whether authenticated to cloud
    pub authenticated: bool,

    /// User email if authenticated
    pub email: Option<String>,

    /// Status message
    pub message: String,
}

impl Default for CloudStatus {
    fn default() -> Self {
        Self {
            authenticated: false,
            email: None,
            message: "Not logged in - run: ckrv cloud login".to_string(),
        }
    }
}

/// Stored cloud tokens from login.
#[derive(Deserialize)]
struct StoredTokens {
    access_token: String,
    #[allow(dead_code)]
    refresh_token: Option<String>,
    #[allow(dead_code)]
    expires_at: Option<i64>,
}

/// Get cloud service status.
///
/// Returns the current status of cloud services (if configured).
pub fn get_cloud_status_handler() -> Result<CloudStatus, TransportError> {
    Ok(check_cloud_auth())
}

/// Get the path to the token file.
fn get_token_file_path() -> Option<PathBuf> {
    let config_dir = dirs::config_dir()?;
    Some(config_dir.join("chakravarti").join("cloud-tokens.json"))
}

/// Check cloud authentication status.
fn check_cloud_auth() -> CloudStatus {
    // Check if token file exists
    let Some(token_path) = get_token_file_path() else {
        return CloudStatus {
            authenticated: false,
            email: None,
            message: "Could not find config directory".to_string(),
        };
    };

    if !token_path.exists() {
        return CloudStatus::default();
    }

    // Try to read and parse the tokens
    fs::read_to_string(&token_path).map_or_else(
        |_| CloudStatus {
            authenticated: false,
            email: None,
            message: "Could not read token file".to_string(),
        },
        |json| match serde_json::from_str::<StoredTokens>(&json) {
            Ok(tokens) => {
                // Try to extract email from JWT payload
                let email = extract_email_from_jwt(&tokens.access_token);

                CloudStatus {
                    authenticated: true,
                    email,
                    message: "Connected to Cloud".to_string(),
                }
            }
            Err(_) => CloudStatus {
                authenticated: false,
                email: None,
                message: "Invalid token format".to_string(),
            },
        },
    )
}

/// Extract email from JWT access token (without verification - just for display).
fn extract_email_from_jwt(token: &str) -> Option<String> {
    // JWT format: header.payload.signature
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return None;
    }

    // Decode the payload (base64url)
    let payload = parts[1];
    let decoded = base64_url_decode(payload)?;

    // Parse as JSON and extract email
    let json: serde_json::Value = serde_json::from_slice(&decoded).ok()?;
    json.get("email")
        .and_then(|v| v.as_str())
        .map(std::string::ToString::to_string)
}

/// Decode base64url encoded string.
fn base64_url_decode(input: &str) -> Option<Vec<u8>> {
    use base64::prelude::*;

    // Add padding if needed
    let padded = match input.len() % 4 {
        2 => format!("{}==", input),
        3 => format!("{}=", input),
        _ => input.to_string(),
    };

    // Replace URL-safe characters
    let standard = padded.replace('-', "+").replace('_', "/");

    BASE64_STANDARD.decode(&standard).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_cloud_status_handler() {
        let result = get_cloud_status_handler();
        assert!(result.is_ok());
    }

    #[test]
    fn test_cloud_status_default() {
        let status = CloudStatus::default();
        assert!(!status.authenticated);
        assert!(status.email.is_none());
    }

    #[test]
    fn test_base64_url_decode() {
        // Test basic decoding
        let result = base64_url_decode("dGVzdA");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), b"test");
    }

    #[test]
    fn test_extract_email_from_invalid_jwt() {
        let result = extract_email_from_jwt("invalid");
        assert!(result.is_none());
    }
}
