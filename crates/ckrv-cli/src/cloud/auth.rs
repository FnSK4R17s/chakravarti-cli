//! OAuth2 device flow authentication.

// ============================================================
// IMPORTS
// ============================================================

use crate::cloud::config::CloudConfig;
use crate::cloud::credentials::StoredTokens;
use crate::cloud::error::CloudError;
use serde::{Deserialize, Serialize};
use std::time::Duration;

// ============================================================
// TYPES
// ============================================================

/// Device code response from the authorization server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCodeResponse {
    /// Device code used to poll for authorization.
    pub device_code: String,
    /// Code the user enters on the verification page.
    pub user_code: String,
    /// URL the user should visit to authorize.
    pub verification_uri: String,
    /// Seconds until the device code expires.
    pub expires_in: u64,
    /// Minimum polling interval in seconds.
    pub interval: u64,
}

/// Token response from the authorization server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    /// OAuth2 access token.
    pub access_token: String,
    /// Optional refresh token for obtaining new access tokens.
    pub refresh_token: Option<String>,
    /// Token type (typically "Bearer").
    pub token_type: String,
    /// Seconds until the access token expires.
    pub expires_in: u64,
}

/// Token error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenErrorResponse {
    pub error: String,
    pub error_description: Option<String>,
}

// ============================================================
// IMPLEMENTATION
// ============================================================

/// OAuth2 Device Authorization Flow.
pub struct DeviceAuthFlow {
    config: CloudConfig,
    client: reqwest::Client,
}

impl DeviceAuthFlow {
    /// Create a new device auth flow
    pub fn new() -> Result<Self, CloudError> {
        let config = CloudConfig::load();
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(|e| CloudError::NetworkError(e.to_string()))?;

        Ok(Self { config, client })
    }

    /// Request a device code from the authorization server
    pub async fn request_device_code(&self) -> Result<DeviceCodeResponse, CloudError> {
        let response = self
            .client
            .post(&self.config.device_auth_url())
            .json(&serde_json::json!({
                "client_id": self.config.client_id
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(CloudError::AuthenticationFailed(error_text));
        }

        response.json().await.map_err(|e| {
            CloudError::InvalidResponse(format!("Failed to parse device code response: {}", e))
        })
    }

    /// Poll for token after user has authorized
    pub async fn poll_for_token(
        &self,
        device_code: &DeviceCodeResponse,
    ) -> Result<StoredTokens, CloudError> {
        let interval = Duration::from_secs(device_code.interval.max(5));
        let max_attempts = device_code.expires_in / device_code.interval.max(5);

        for _ in 0..max_attempts {
            tokio::time::sleep(interval).await;

            let response = self
                .client
                .post(&self.config.token_url())
                .json(&serde_json::json!({
                    "client_id": self.config.client_id,
                    "device_code": device_code.device_code,
                    "grant_type": "urn:ietf:params:oauth:grant-type:device_code"
                }))
                .send()
                .await?;

            if response.status().is_success() {
                let token_response: TokenResponse = response.json().await.map_err(|e| {
                    CloudError::InvalidResponse(format!("Failed to parse token response: {}", e))
                })?;

                let expires_at = chrono::Utc::now().timestamp() + token_response.expires_in as i64;

                return Ok(StoredTokens {
                    access_token: token_response.access_token,
                    refresh_token: token_response.refresh_token,
                    expires_at: Some(expires_at),
                });
            }

            // Check for pending/slow_down errors
            let error_response: TokenErrorResponse = response.json().await.map_err(|e| {
                CloudError::InvalidResponse(format!("Failed to parse error response: {}", e))
            })?;

            match error_response.error.as_str() {
                "authorization_pending" => continue,
                "slow_down" => {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
                "access_denied" => {
                    return Err(CloudError::AuthenticationFailed(
                        "Authorization was denied by the user".into(),
                    ));
                }
                "expired_token" => {
                    return Err(CloudError::AuthenticationFailed(
                        "Device code has expired. Please try again.".into(),
                    ));
                }
                _ => {
                    return Err(CloudError::AuthenticationFailed(
                        error_response
                            .error_description
                            .unwrap_or(error_response.error),
                    ));
                }
            }
        }

        Err(CloudError::AuthenticationFailed(
            "Authorization timed out".into(),
        ))
    }
}

/// Refresh an access token
pub async fn refresh_token(refresh_token: &str) -> Result<StoredTokens, CloudError> {
    let config = CloudConfig::load();
    let client = reqwest::Client::new();

    let response = client
        .post(&config.refresh_url())
        .json(&serde_json::json!({
            "refresh_token": refresh_token,
            "grant_type": "refresh_token"
        }))
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(CloudError::TokenExpired);
    }

    let token_response: TokenResponse = response.json().await.map_err(|e| {
        CloudError::InvalidResponse(format!("Failed to parse token response: {}", e))
    })?;

    let expires_at = chrono::Utc::now().timestamp() + token_response.expires_in as i64;

    Ok(StoredTokens {
        access_token: token_response.access_token,
        refresh_token: token_response.refresh_token,
        expires_at: Some(expires_at),
    })
}
