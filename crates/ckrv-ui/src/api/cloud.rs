//! Cloud authentication status API endpoint

use axum::{response::IntoResponse, Json};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize)]
pub struct CloudStatus {
    pub authenticated: bool,
    pub email: Option<String>,
    pub message: String,
}

#[derive(serde::Deserialize)]
struct StoredTokens {
    access_token: String,
    #[allow(dead_code)]
    refresh_token: Option<String>,
    #[allow(dead_code)]
    expires_at: Option<i64>,
}

/// GET /api/cloud - Check cloud authentication status
pub async fn get_cloud_status() -> impl IntoResponse {
    let status = check_cloud_auth();
    Json(status)
}

fn get_token_file_path() -> Option<PathBuf> {
    let config_dir = dirs::config_dir()?;
    Some(config_dir.join("chakravarti").join("cloud-tokens.json"))
}

fn check_cloud_auth() -> CloudStatus {
    // Check if token file exists
    let token_path = match get_token_file_path() {
        Some(path) => path,
        None => {
            return CloudStatus {
                authenticated: false,
                email: None,
                message: "Could not find config directory".to_string(),
            };
        }
    };

    if !token_path.exists() {
        return CloudStatus {
            authenticated: false,
            email: None,
            message: "Not logged in - run: ckrv cloud login".to_string(),
        };
    }

    // Try to read and parse the tokens
    match fs::read_to_string(&token_path) {
        Ok(json) => {
            match serde_json::from_str::<StoredTokens>(&json) {
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
            }
        }
        Err(_) => CloudStatus {
            authenticated: false,
            email: None,
            message: "Could not read token file".to_string(),
        },
    }
}

/// Extract email from JWT access token (without verification - just for display)
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
        .map(|s| s.to_string())
}

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
