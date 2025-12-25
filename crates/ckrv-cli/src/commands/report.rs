//! Report command - view job metrics.

use clap::Args;
use serde::Serialize;

use ckrv_metrics::{format_ms, FileMetricsStorage, MetricsStorage};

/// Arguments for the report command
#[derive(Args)]
pub struct ReportArgs {
    /// Job ID to view report for
    pub job_id: String,

    /// Show detailed per-step breakdown
    #[arg(long)]
    pub detailed: bool,
}

#[derive(Serialize)]
struct ReportOutput {
    job_id: String,
    spec_id: String,
    success: bool,
    duration_ms: u64,
    tokens: TokenReport,
    cost: CostReport,
    steps: Vec<StepReport>,
}

#[derive(Serialize)]
struct TokenReport {
    total: u64,
    by_model: Vec<ModelTokens>,
}

#[derive(Serialize)]
struct ModelTokens {
    model: String,
    input: u64,
    output: u64,
    total: u64,
}

#[derive(Serialize)]
struct CostReport {
    total_usd: f64,
    by_model: Vec<ModelCost>,
}

#[derive(Serialize)]
struct ModelCost {
    model: String,
    cost_usd: f64,
}

#[derive(Serialize)]
struct StepReport {
    step_id: String,
    duration_ms: u64,
}

/// Execute the report command
pub fn execute(args: ReportArgs, json: bool) -> anyhow::Result<()> {
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
                    let output = ReportOutput {
                        job_id: metrics.job_id.clone(),
                        spec_id: metrics.spec_id.clone(),
                        success: metrics.success,
                        duration_ms: metrics.total_time_ms,
                        tokens: TokenReport {
                            total: metrics.total_tokens(),
                            by_model: metrics
                                .token_usage
                                .iter()
                                .map(|t| ModelTokens {
                                    model: t.model.clone(),
                                    input: t.input_tokens,
                                    output: t.output_tokens,
                                    total: t.total(),
                                })
                                .collect(),
                        },
                        cost: CostReport {
                            total_usd: metrics.cost.total_usd,
                            by_model: metrics
                                .cost
                                .by_model
                                .iter()
                                .map(|(model, cost)| ModelCost {
                                    model: model.clone(),
                                    cost_usd: *cost,
                                })
                                .collect(),
                        },
                        steps: metrics
                            .step_metrics
                            .iter()
                            .map(|s| StepReport {
                                step_id: s.step_id.clone(),
                                duration_ms: s.duration_ms,
                            })
                            .collect(),
                    };
                    println!("{}", serde_json::to_string_pretty(&output)?);
                } else {
                    println!("═══════════════════════════════════════════════");
                    println!("           JOB METRICS REPORT");
                    println!("═══════════════════════════════════════════════");
                    println!();
                    println!("Job ID:   {}", metrics.job_id);
                    println!("Spec:     {}", metrics.spec_id);
                    println!(
                        "Status:   {}",
                        if metrics.success {
                            "✓ Succeeded"
                        } else {
                            "✗ Failed"
                        }
                    );
                    println!();

                    println!("─────────────────────────────────────────────");
                    println!("TIMING");
                    println!("─────────────────────────────────────────────");
                    println!("  Total Duration: {}", format_ms(metrics.total_time_ms));

                    if args.detailed && !metrics.step_metrics.is_empty() {
                        println!();
                        println!("  Steps:");
                        for step in &metrics.step_metrics {
                            println!("    • {} ({})", step.step_id, format_ms(step.duration_ms));
                        }
                    }
                    println!();

                    println!("─────────────────────────────────────────────");
                    println!("TOKEN USAGE");
                    println!("─────────────────────────────────────────────");
                    println!("  Total Tokens: {}", metrics.total_tokens());

                    if !metrics.token_usage.is_empty() {
                        println!();
                        println!("  By Model:");
                        for usage in &metrics.token_usage {
                            println!(
                                "    • {}: {} tokens ({} in / {} out)",
                                usage.model,
                                usage.total(),
                                usage.input_tokens,
                                usage.output_tokens
                            );
                        }
                    }
                    println!();

                    println!("─────────────────────────────────────────────");
                    println!("COST ESTIMATE");
                    println!("─────────────────────────────────────────────");
                    println!("  Total: ${:.4}", metrics.cost.total_usd);

                    if !metrics.cost.by_model.is_empty() {
                        println!();
                        println!("  By Model:");
                        for (model, cost) in &metrics.cost.by_model {
                            println!("    • {}: ${:.4}", model, cost);
                        }
                    }
                    println!();

                    println!("═══════════════════════════════════════════════");
                }
            }
            Err(e) => {
                if json {
                    let output = serde_json::json!({
                        "job_id": args.job_id,
                        "error": e.to_string()
                    });
                    println!("{}", serde_json::to_string_pretty(&output)?);
                } else {
                    eprintln!("Error loading job metrics: {e}");
                }
                std::process::exit(1);
            }
        }
    } else {
        if json {
            let output = serde_json::json!({
                "job_id": args.job_id,
                "error": "Metrics not found"
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            println!("Report for job: {}", args.job_id);
            println!();
            println!("No metrics found for this job.");
            println!("Run `ckrv status {}` to check the job status.", args.job_id);
        }
    }

    Ok(())
}
