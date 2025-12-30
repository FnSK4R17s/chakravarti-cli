//! Job-related cloud operations.

use crate::cloud::client::{CloudClient, CloudJob};
use crate::cloud::error::CloudError;

/// Dispatch a job to the cloud
pub async fn dispatch_job(
    spec_content: &str,
    git_repo_url: &str,
    git_base_branch: &str,
    credential_name: Option<&str>,
) -> Result<CloudJob, CloudError> {
    let client = CloudClient::new()?;
    client.create_job(spec_content, git_repo_url, git_base_branch, credential_name).await
}

/// Get job status
pub async fn get_job_status(job_id: &str) -> Result<CloudJob, CloudError> {
    let client = CloudClient::new()?;
    client.get_job(job_id).await
}

/// Get job diff artifact
pub async fn get_job_diff(job_id: &str) -> Result<String, CloudError> {
    let client = CloudClient::new()?;
    client.get_job_diff(job_id).await
}

/// Format job status for display
pub fn format_job_status(job: &CloudJob) -> String {
    let mut lines = vec![
        format!("Job: {}", job.id),
        format!("Status: {}", job.status),
    ];
    
    if let Some(ref branch) = job.feature_branch_name {
        lines.push(format!("Branch: {}", branch));
    }
    
    lines.push(format!("Repository: {}", job.git_repo_url));
    lines.push(format!("Base: {}", job.git_base_branch));
    lines.push(format!("Created: {}", job.created_at));
    
    if let Some(ref started) = job.started_at {
        lines.push(format!("Started: {}", started));
    }
    
    if let Some(ref completed) = job.completed_at {
        lines.push(format!("Completed: {}", completed));
    }
    
    if let Some(ref result) = job.result_status {
        lines.push(format!("Result: {}", result));
    }
    
    if let Some(ref summary) = job.result_summary {
        lines.push(format!("Summary: {}", summary));
    }
    
    if let Some(ref error) = job.error_message {
        lines.push(format!("Error: {}", error));
    }
    
    lines.join("\n")
}
