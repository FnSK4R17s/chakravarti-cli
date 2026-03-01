//! HTTP client for Chakravarti Cloud API.

// ============================================================
// IMPORTS
// ============================================================

use crate::cloud::config::CloudConfig;
use crate::cloud::credentials::{self, StoredTokens};
use crate::cloud::error::CloudError;
use serde::{Deserialize, Serialize};
use std::time::Duration;

// ============================================================
// TYPES
// ============================================================

/// User information from the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique user identifier.
    pub id: String,
    /// User email address.
    pub email: String,
    /// Display name, if set.
    pub name: Option<String>,
    /// Subscription tier (e.g., "free", "pro").
    pub subscription_tier: String,
    /// Number of jobs remaining in the current billing cycle.
    pub job_quota_remaining: i32,
    /// When the billing cycle ends.
    pub billing_cycle_end: Option<String>,
}

/// Credential summary (no secret values).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialSummary {
    /// Display name for the credential.
    pub name: String,
    /// Git provider (github, gitlab, etc.).
    pub provider: String,
    /// Type of credential (pat, deploy_key).
    pub credential_type: String,
    /// When the credential was created.
    pub created_at: String,
    /// When the credential was last used.
    pub last_used_at: Option<String>,
}

/// Cloud job information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudJob {
    /// Unique job identifier.
    pub id: String,
    /// Current job status (pending, running, succeeded, failed, timeout).
    pub status: String,
    /// Git repository URL for the job.
    pub git_repo_url: String,
    /// Base branch the job is built from.
    pub git_base_branch: String,
    /// Feature branch created by the job.
    pub feature_branch_name: Option<String>,
    /// When the job was created.
    pub created_at: String,
    /// When execution started.
    pub started_at: Option<String>,
    /// When execution completed.
    pub completed_at: Option<String>,
    /// Result status after completion.
    pub result_status: Option<String>,
    /// Summary of the job results.
    pub result_summary: Option<String>,
    /// Error message if the job failed.
    pub error_message: Option<String>,
}

// ============================================================
// IMPLEMENTATION
// ============================================================

/// HTTP client for Cloud API.
pub struct CloudClient {
    config: CloudConfig,
    client: reqwest::Client,
    tokens: StoredTokens,
}

impl CloudClient {
    /// Create a new cloud client with stored credentials
    pub fn new() -> Result<Self, CloudError> {
        let config = CloudConfig::load();
        let tokens = credentials::load_tokens()?;

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(|e| CloudError::NetworkError(e.to_string()))?;

        Ok(Self {
            config,
            client,
            tokens,
        })
    }

    /// Get the current authenticated user
    pub async fn get_current_user(&self) -> Result<User, CloudError> {
        let response = self
            .client
            .get(&self.config.user_url())
            .bearer_auth(&self.tokens.access_token)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Create a new cloud job
    pub async fn create_job(
        &self,
        spec_content: &str,
        git_repo_url: &str,
        git_base_branch: &str,
        credential_name: Option<&str>,
    ) -> Result<CloudJob, CloudError> {
        let mut body = serde_json::json!({
            "spec_content": spec_content,
            "git_repo_url": git_repo_url,
            "git_base_branch": git_base_branch
        });

        if let Some(cred) = credential_name {
            body["git_credential_name"] = serde_json::Value::String(cred.to_string());
        }

        let response = self
            .client
            .post(&self.config.jobs_url())
            .bearer_auth(&self.tokens.access_token)
            .json(&body)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Get job status
    pub async fn get_job(&self, job_id: &str) -> Result<CloudJob, CloudError> {
        let response = self
            .client
            .get(&self.config.job_url(job_id))
            .bearer_auth(&self.tokens.access_token)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Get job diff artifact
    pub async fn get_job_diff(&self, job_id: &str) -> Result<String, CloudError> {
        let url = format!("{}/artifacts/diff", self.config.job_url(job_id));

        let response = self
            .client
            .get(&url)
            .bearer_auth(&self.tokens.access_token)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(CloudError::JobNotFound(job_id.to_string()));
        }

        response
            .text()
            .await
            .map_err(|e| CloudError::InvalidResponse(format!("Failed to read diff: {}", e)))
    }

    /// Add a git credential
    pub async fn add_credential(
        &self,
        name: &str,
        provider: &str,
        credential_type: &str,
        value: &str,
    ) -> Result<CredentialSummary, CloudError> {
        let response = self
            .client
            .post(&self.config.credentials_url())
            .bearer_auth(&self.tokens.access_token)
            .json(&serde_json::json!({
                "name": name,
                "provider": provider,
                "credential_type": credential_type,
                "value": value
            }))
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// List git credentials
    pub async fn list_credentials(&self) -> Result<Vec<CredentialSummary>, CloudError> {
        let response = self
            .client
            .get(&self.config.credentials_url())
            .bearer_auth(&self.tokens.access_token)
            .send()
            .await?;

        #[derive(Deserialize)]
        struct CredentialList {
            credentials: Vec<CredentialSummary>,
        }

        let list: CredentialList = self.handle_response(response).await?;
        Ok(list.credentials)
    }

    /// Remove a git credential
    pub async fn remove_credential(&self, name: &str) -> Result<(), CloudError> {
        let url = format!("{}/{}", self.config.credentials_url(), name);

        let response = self
            .client
            .delete(&url)
            .bearer_auth(&self.tokens.access_token)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(CloudError::ApiError(format!(
                "Failed to remove credential: {}",
                name
            )));
        }

        Ok(())
    }

    /// Handle API response with error checking
    async fn handle_response<T: for<'de> Deserialize<'de>>(
        &self,
        response: reqwest::Response,
    ) -> Result<T, CloudError> {
        let status = response.status();

        if status.is_success() {
            return response.json().await.map_err(|e| {
                CloudError::InvalidResponse(format!("Failed to parse response: {}", e))
            });
        }

        match status.as_u16() {
            401 => Err(CloudError::NotAuthenticated),
            402 => {
                // Quota exceeded - try to parse details
                #[derive(Deserialize)]
                struct QuotaError {
                    quota_resets_at: String,
                    upgrade_url: String,
                }
                if let Ok(details) = response.json::<QuotaError>().await {
                    Err(CloudError::QuotaExceeded {
                        reset_time: details.quota_resets_at,
                        upgrade_url: details.upgrade_url,
                    })
                } else {
                    Err(CloudError::QuotaExceeded {
                        reset_time: "unknown".into(),
                        upgrade_url: "https://ckrv.dev/billing".into(),
                    })
                }
            }
            404 => Err(CloudError::JobNotFound("Resource not found".into())),
            _ => {
                let error_text = response.text().await.unwrap_or_default();
                Err(CloudError::ApiError(format!("{}: {}", status, error_text)))
            }
        }
    }
}
