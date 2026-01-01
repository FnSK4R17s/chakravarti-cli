//! Logs command - Stream or view cloud job logs.

use clap::Args;
use chrono::{DateTime, Utc};

/// Arguments for the logs command
#[derive(Debug, Args)]
pub struct LogsArgs {
    /// Job ID to get logs for
    pub job_id: String,
    
    /// Follow log output (stream in real-time)
    #[arg(long, short, default_value = "false")]
    pub follow: bool,
    
    /// Output as JSON
    #[arg(long, default_value = "false")]
    pub json: bool,
    
    /// Number of recent log lines to show (default: 100)
    #[arg(long, short = 'n', default_value = "100")]
    pub tail: usize,
}

/// Execute the logs command
pub async fn execute(args: LogsArgs, ui: &crate::ui::UiContext) -> anyhow::Result<()> {
    use crate::cloud::client::CloudClient;
    use crate::cloud::logs::{stream_logs, fetch_logs, LogEntry};
    
    // Verify job exists first
    let client = CloudClient::new().map_err(|e| anyhow::anyhow!("{}", e))?;
    let job = client.get_job(&args.job_id).await.map_err(|e| anyhow::anyhow!("{}", e))?;
    
    if args.follow {
        // Check if job is still running
        if job.status == "succeeded" || job.status == "failed" || job.status == "timeout" {
            println!("Job already completed with status: {}", job.status);
            println!("Fetching historical logs...\n");
            return show_historical_logs(&args.job_id, args.tail, args.json).await;
        }
        
        println!("☁️  Streaming logs for job {}...", args.job_id);
        println!("   Status: {}", job.status);
        println!("   (Press Ctrl+C to stop)\n");
        println!("{}", "─".repeat(60));
        
        // Stream logs via SSE
        let json_mode = args.json;
        let result = stream_logs(&args.job_id, move |entry: LogEntry| {
            print_log_entry(&entry, json_mode);
        }).await;
        
        match result {
            Ok(()) => {
                println!("{}", "─".repeat(60));
                println!("Log stream ended.");
            }
            Err(e) => {
                // Fallback to polling if SSE not available
                eprintln!("SSE not available, falling back to polling: {}", e);
                poll_for_completion(&args.job_id, args.json).await?;
            }
        }
    } else {
        // Get historical logs
        show_historical_logs(&args.job_id, args.tail, args.json).await?;
    }
    
    let _ = ui; // Suppress unused warning
    Ok(())
}

/// Show historical logs for a completed job
async fn show_historical_logs(job_id: &str, tail: usize, json: bool) -> anyhow::Result<()> {
    use crate::cloud::logs::fetch_logs;
    
    match fetch_logs(job_id).await {
        Ok(logs) => {
            let len = logs.len();
            let logs_to_show: Vec<_> = if len > tail {
                logs.into_iter().skip(len - tail).collect()
            } else {
                logs
            };
            
            if logs_to_show.is_empty() {
                println!("No logs available for this job.");
                println!("Logs may not be available until the job completes.");
            } else {
                for entry in logs_to_show {
                    print_log_entry(&entry, json);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to fetch logs: {}", e);
            println!("Logs may not be available for this job yet.");
        }
    }
    
    Ok(())
}

/// Poll for job completion (fallback when SSE not available)
async fn poll_for_completion(job_id: &str, json: bool) -> anyhow::Result<()> {
    use crate::cloud::client::CloudClient;
    
    let client = CloudClient::new().map_err(|e| anyhow::anyhow!("{}", e))?;
    
    loop {
        let job = client.get_job(job_id).await.map_err(|e| anyhow::anyhow!("{}", e))?;
        
        if !json {
            print!("\rStatus: {} ", job.status);
            std::io::Write::flush(&mut std::io::stdout())?;
        }
        
        if job.status == "succeeded" || job.status == "failed" || job.status == "timeout" {
            println!("\nJob completed with status: {}", job.status);
            
            // Show final logs
            show_historical_logs(job_id, 50, json).await?;
            break;
        }
        
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }
    
    Ok(())
}

/// Print a single log entry
fn print_log_entry(entry: &crate::cloud::logs::LogEntry, json: bool) {
    if json {
        if let Ok(json_str) = serde_json::to_string(entry) {
            println!("{}", json_str);
        }
    } else {
        let level_indicator = match entry.level.as_str() {
            "error" => "❌",
            "warn" => "⚠️ ",
            "debug" => "🔍",
            _ => "📝",
        };
        
        // Format timestamp nicely
        let timestamp = if let Ok(dt) = entry.timestamp.parse::<DateTime<Utc>>() {
            dt.format("%H:%M:%S").to_string()
        } else {
            entry.timestamp.chars().take(8).collect()
        };
        
        println!("{} [{}] {}: {}", 
            level_indicator,
            timestamp,
            entry.source,
            entry.message
        );
    }
}
