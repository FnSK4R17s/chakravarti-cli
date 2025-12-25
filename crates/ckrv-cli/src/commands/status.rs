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

/// Execute the status command
pub fn execute(args: StatusArgs, json: bool) -> anyhow::Result<()> {
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
                    println!("Job: {}", metrics.job_id);
                    println!("Spec: {}", metrics.spec_id);
                    println!(
                        "Status: {}",
                        if metrics.success {
                            "Succeeded ✓"
                        } else {
                            "Failed ✗"
                        }
                    );
                    println!();
                    println!("Metrics:");
                    println!("  Duration: {:.2}s", metrics.total_time_ms as f64 / 1000.0);
                    println!("  Tokens: {}", metrics.total_tokens());
                    println!("  Est. Cost: ${:.4}", metrics.cost.total_usd);
                    println!("  Steps: {}", metrics.step_metrics.len());
                    if metrics.retry_count > 0 {
                        println!("  Retries: {}", metrics.retry_count);
                    }
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
                    eprintln!("Error loading job metrics: {e}");
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
                println!("Job: {}", args.job_id);
                println!("Status: Completed (metrics not available)");
                println!();
                println!("Run directory: {}", runs_dir.display());
            }
        } else {
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
                println!("Job: {}", args.job_id);
                println!("Status: Not found");
                println!();
                println!("No job with this ID was found.");
                println!("Run `ckrv run <spec>` to create a new job.");
            }
        }
    }

    Ok(())
}
