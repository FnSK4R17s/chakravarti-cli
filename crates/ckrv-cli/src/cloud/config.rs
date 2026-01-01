//! Cloud configuration management.

use std::env;

/// Default Chakravarti Cloud API URL
const DEFAULT_API_URL: &str = "https://api.ckrv.dev/v1";

/// Cloud configuration
#[derive(Debug, Clone)]
pub struct CloudConfig {
    /// Base URL for the Cloud API
    pub api_url: String,
    /// Client ID for OAuth2
    pub client_id: String,
    /// Request timeout in seconds
    pub timeout_secs: u64,
}

impl CloudConfig {
    /// Load configuration from environment or use defaults
    pub fn load() -> Self {
        Self {
            api_url: env::var("CHAKRAVARTI_API_URL")
                .unwrap_or_else(|_| DEFAULT_API_URL.to_string()),
            client_id: env::var("CHAKRAVARTI_CLIENT_ID")
                .unwrap_or_else(|_| "ckrv-cli".to_string()),
            timeout_secs: env::var("CHAKRAVARTI_TIMEOUT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),
        }
    }
    
    /// Get the device authorization endpoint
    pub fn device_auth_url(&self) -> String {
        format!("{}/auth/device", self.api_url)
    }
    
    /// Get the token endpoint
    pub fn token_url(&self) -> String {
        format!("{}/auth/token", self.api_url)
    }
    
    /// Get the refresh token endpoint
    pub fn refresh_url(&self) -> String {
        format!("{}/auth/refresh", self.api_url)
    }
    
    /// Get the user info endpoint
    pub fn user_url(&self) -> String {
        format!("{}/users/me", self.api_url)
    }
    
    /// Get the jobs endpoint
    pub fn jobs_url(&self) -> String {
        format!("{}/jobs", self.api_url)
    }
    
    /// Get a specific job endpoint
    pub fn job_url(&self, job_id: &str) -> String {
        format!("{}/jobs/{}", self.api_url, job_id)
    }
    
    /// Get the credentials endpoint
    pub fn credentials_url(&self) -> String {
        format!("{}/credentials", self.api_url)
    }
}

impl Default for CloudConfig {
    fn default() -> Self {
        Self::load()
    }
}
