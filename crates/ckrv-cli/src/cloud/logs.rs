//! Log streaming client for SSE-based log streaming.

use crate::cloud::config::CloudConfig;
use crate::cloud::credentials::{load_tokens, StoredTokens};
use crate::cloud::error::CloudError;
use futures::StreamExt;

/// Log entry from the stream
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub source: String,
    pub message: String,
}

/// Get access token, returning error if not authenticated
fn get_access_token() -> Result<String, CloudError> {
    let tokens: StoredTokens = load_tokens()?;
    Ok(tokens.access_token)
}

/// Stream logs from a cloud job via SSE
pub async fn stream_logs<F>(
    job_id: &str,
    mut on_log: F,
) -> Result<(), CloudError>
where
    F: FnMut(LogEntry),
{
    let config = CloudConfig::load();
    let token = get_access_token()?;
    
    let url = format!("{}/v1/jobs/{}/logs?follow=true", config.api_url, job_id);
    
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Accept", "text/event-stream")
        .send()
        .await?;
    
    if !response.status().is_success() {
        let status = response.status().as_u16();
        let message = response.text().await.unwrap_or_default();
        return Err(CloudError::ApiError(format!("HTTP {}: {}", status, message)));
    }
    
    // Parse SSE stream
    let mut stream = response.bytes_stream();
    let mut buffer = String::new();
    
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        let text = String::from_utf8_lossy(&chunk);
        buffer.push_str(&text);
        
        // Parse SSE events
        while let Some(event_end) = buffer.find("\n\n") {
            let event_text = buffer[..event_end].to_string();
            buffer = buffer[event_end + 2..].to_string();
            
            // Parse data lines
            for line in event_text.lines() {
                if let Some(data) = line.strip_prefix("data: ") {
                    if let Ok(entry) = serde_json::from_str::<LogEntry>(data) {
                        on_log(entry);
                    } else if data == "[DONE]" {
                        return Ok(());
                    }
                }
            }
        }
    }
    
    Ok(())
}

/// Fetch historical logs (non-streaming)
pub async fn fetch_logs(job_id: &str) -> Result<Vec<LogEntry>, CloudError> {
    let config = CloudConfig::load();
    let token = get_access_token()?;
    
    let url = format!("{}/v1/jobs/{}/logs", config.api_url, job_id);
    
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;
    
    if !response.status().is_success() {
        let status = response.status().as_u16();
        let message = response.text().await.unwrap_or_default();
        return Err(CloudError::ApiError(format!("HTTP {}: {}", status, message)));
    }
    
    #[derive(serde::Deserialize)]
    struct LogsResponse {
        logs: Vec<LogEntry>,
    }
    
    let logs_response: LogsResponse = response.json().await
        .map_err(|e| CloudError::InvalidResponse(e.to_string()))?;
    
    Ok(logs_response.logs)
}
