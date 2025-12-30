//! Cloud-specific error types.

use thiserror::Error;

/// Errors that can occur when interacting with Chakravarti Cloud.
#[derive(Debug, Error)]
pub enum CloudError {
    /// User is not authenticated
    #[error("Not authenticated. Run 'ckrv cloud login' to authenticate.")]
    NotAuthenticated,
    
    /// Authentication failed
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    /// Token has expired
    #[error("Token expired. Run 'ckrv cloud login' to re-authenticate.")]
    TokenExpired,
    
    /// API request failed
    #[error("API request failed: {0}")]
    ApiError(String),
    
    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),
    
    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    /// Credential storage error
    #[error("Credential storage error: {0}")]
    CredentialError(String),
    
    /// Quota exceeded
    #[error("Job quota exceeded. Resets at {reset_time}. Upgrade at {upgrade_url}")]
    QuotaExceeded {
        reset_time: String,
        upgrade_url: String,
    },
    
    /// Job not found
    #[error("Job not found: {0}")]
    JobNotFound(String),
    
    /// Invalid response from server
    #[error("Invalid response from server: {0}")]
    InvalidResponse(String),
}

impl From<reqwest::Error> for CloudError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_connect() {
            CloudError::NetworkError(format!("Connection failed: {}", err))
        } else if err.is_timeout() {
            CloudError::NetworkError(format!("Request timed out: {}", err))
        } else {
            CloudError::ApiError(err.to_string())
        }
    }
}

impl From<keyring::Error> for CloudError {
    fn from(err: keyring::Error) -> Self {
        CloudError::CredentialError(err.to_string())
    }
}
