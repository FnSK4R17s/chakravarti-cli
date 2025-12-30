//! Status command - check job status.

use std::path::PathBuf;

use clap::Args;
use serde::Serialize;

use ckrv_metrics::{FileMetricsStorage, MetricsStorage};

/// Arguments for the status command
#[derive(Args)]
pub struct StatusArgs {
    /// Job ID to check status for
    pub job_id: String,
}

#[derive(Serialize)]
struct StatusOutput {
    job_id: String,
    status: String,
    spec_id: Option<String>,
    duration_ms: Option<u64>,
    total_tokens: Option<u64>,
    estimated_cost_usd: Option<f64>,
    success: Option<bool>,
}

use crate::ui::UiContext;

/// Execute the status command
pub async fn execute(args: StatusArgs, json: bool, ui: &UiContext) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;

    // Try to find repo root
    let repo_root = ckrv_git::repo_root(&cwd).unwrap_or(cwd);
    let chakravarti_dir = repo_root.join(".chakravarti");

    // Try to load metrics
    let storage = FileMetricsStorage::new(&chakravarti_dir);

    if storage.exists(&args.job_id) {
        match storage.load(&args.job_id) {
            Ok(metrics) => {
                if json {
                    let output = StatusOutput {
                        job_id: metrics.job_id.clone(),
                        status: if metrics.success {
                            "succeeded"
                        } else {
                            "failed"
                        }
                        .to_string(),
                        spec_id: Some(metrics.spec_id.clone()),
                        duration_ms: Some(metrics.total_time_ms),
                        total_tokens: Some(metrics.total_tokens()),
                        estimated_cost_usd: Some(metrics.cost.total_usd),
                        success: Some(metrics.success),
                    };
                    println!("{}", serde_json::to_string_pretty(&output)?);
                } else {
                    // Rich UI Output
                    let title = if metrics.success {
                        "Job Succeeded"
                    } else {
                        "Job Failed"
                    };
                    let msg = format!("Job ID: {}\nSpec: {}", metrics.job_id, metrics.spec_id);

                    if metrics.success {
                        ui.success(title, &msg);
                    } else {
                        ui.error(title, &msg);
                    }

                    let mut content = String::from("\n### Metrics\n");
                    content.push_str(&format!(
                        "* **Duration**: {:.2}s\n",
                        metrics.total_time_ms as f64 / 1000.0
                    ));
                    content.push_str(&format!("* **Tokens**: {}\n", metrics.total_tokens()));
                    content.push_str(&format!("* **Cost**: ${:.4}\n", metrics.cost.total_usd));
                    content.push_str(&format!("* **Steps**: {}\n", metrics.step_metrics.len()));
                    if metrics.retry_count > 0 {
                        content.push_str(&format!("* **Retries**: {}\n", metrics.retry_count));
                    }
                    ui.markdown(&content);
                }
            }
            Err(e) => {
                if json {
                    let output = StatusOutput {
                        job_id: args.job_id.clone(),
                        status: "error".to_string(),
                        spec_id: None,
                        duration_ms: None,
                        total_tokens: None,
                        estimated_cost_usd: None,
                        success: None,
                    };
                    println!("{}", serde_json::to_string_pretty(&output)?);
                } else {
                    ui.error("Error", &format!("Loading job metrics failed: {e}"));
                }
                std::process::exit(1);
            }
        }
    } else {
        // Check if runs directory exists for this job
        let runs_dir = chakravarti_dir.join("runs").join(&args.job_id);

        if runs_dir.exists() {
            if json {
                let output = StatusOutput {
                    job_id: args.job_id.clone(),
                    status: "completed".to_string(),
                    spec_id: None,
                    duration_ms: None,
                    total_tokens: None,
                    estimated_cost_usd: None,
                    success: None,
                };
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                ui.success(
                    "Job Completed",
                    &format!("Job ID: {}\n(Metrics not available)", args.job_id),
                );
                ui.markdown(&format!("**Run Directory**: `{}`", runs_dir.display()));
            }
        } else {
            // Not a local job - try cloud job lookup
            if let Ok(cloud_status) = check_cloud_job_status(&args.job_id, json).await {
                if cloud_status {
                    return Ok(());
                }
            }
            
            if json {
                let output = StatusOutput {
                    job_id: args.job_id.clone(),
                    status: "not_found".to_string(),
                    spec_id: None,
                    duration_ms: None,
                    total_tokens: None,
                    estimated_cost_usd: None,
                    success: None,
                };
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                ui.error(
                    "Job Not Found",
                    &format!("No job with ID '{}' was found.", args.job_id),
                );
                ui.markdown("Run `ckrv run <spec>` to create a new job, or `ckrv run --cloud <spec>` for cloud execution.");
            }
        }
    }

    Ok(())
}

/// Check cloud job status
async fn check_cloud_job_status(job_id: &str, json: bool) -> anyhow::Result<bool> {
    use crate::cloud::client::CloudClient;
    
    let client = match CloudClient::new() {
        Ok(c) => c,
        Err(_) => return Ok(false), // Not authenticated, skip cloud check
    };
    
    match client.get_job(job_id).await {
        Ok(job) => {
            if json {
                println!("{}", serde_json::to_string_pretty(&job)?);
            } else {
                let status_emoji = match job.status.as_str() {
                    "pending" => "⏳",
                    "running" => "🔄",
                    "succeeded" => "✅",
                    "failed" => "❌",
                    "timeout" => "⏰",
                    _ => "❓",
                };
                
                println!("☁️  Cloud Job Status: {} {}", status_emoji, job.status);
                println!();
                println!("   Job ID:     {}", job.id);
                println!("   Repository: {}", job.git_repo_url);
                println!("   Branch:     {}", job.git_base_branch);
                if let Some(ref feature) = job.feature_branch_name {
                    println!("   Feature:    {}", feature);
                }
                println!("   Created:    {}", job.created_at);
                if let Some(ref started) = job.started_at {
                    println!("   Started:    {}", started);
                }
                if let Some(ref completed) = job.completed_at {
                    println!("   Completed:  {}", completed);
                }
                if let Some(ref summary) = job.result_summary {
                    println!("   Summary:    {}", summary);
                }
                if let Some(ref error) = job.error_message {
                    println!("   Error:      {}", error);
                }
                println!();
                
                if job.status == "succeeded" {
                    println!("📦 Pull results: ckrv pull {}", job.id);
                } else if job.status == "running" || job.status == "pending" {
                    println!("📝 Stream logs: ckrv logs {} --follow", job.id);
                }
            }
            Ok(true)
        }
        Err(_) => Ok(false),
    }
}
