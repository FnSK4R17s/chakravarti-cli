//! Usage command - view aggregate usage metrics across all jobs.

use std::collections::{BTreeMap, HashMap};

use clap::Args;
use serde::Serialize;

use ckrv_metrics::{format_ms, FileMetricsStorage, Metrics, MetricsStorage};

use super::emit_json;
use crate::services::agent_lookup::{self, AgentConfig, AgentType};

/// Arguments for the usage command.
#[derive(Args)]
pub struct UsageArgs {
    /// Show per-job breakdown
    #[arg(long)]
    pub detailed: bool,

    /// Show per-agent quota/usage view (Claude/Codex/OpenRouter/GLM/KiloCode)
    #[arg(long)]
    pub agents: bool,
}

/// Aggregate usage output for JSON serialization.
#[derive(Serialize)]
struct UsageOutput {
    total_jobs: usize,
    succeeded: usize,
    failed: usize,
    total_time_ms: u64,
    total_tokens: u64,
    total_cost_usd: f64,
    by_model: Vec<ModelUsage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    agents: Option<Vec<AgentQuotaUsage>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    jobs: Option<Vec<JobSummary>>,
}

/// Per-model usage entry.
#[derive(Serialize)]
struct ModelUsage {
    model: String,
    input_tokens: u64,
    output_tokens: u64,
    total_tokens: u64,
    cost_usd: f64,
}

/// Per-agent usage + optional quota details.
#[derive(Debug, Clone, Serialize)]
struct AgentQuotaUsage {
    agent_id: String,
    agent_name: String,
    agent_type: String,
    total_tokens: u64,
    total_cost_usd: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    token_limit: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    token_remaining: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    usd_limit: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    usd_remaining: Option<f64>,
}

/// Per-job summary entry.
#[derive(Serialize)]
struct JobSummary {
    job_id: String,
    spec_id: String,
    success: bool,
    duration_ms: u64,
    tokens: u64,
    cost_usd: f64,
}

/// Execute the usage command.
///
/// Loads all stored job metrics and displays aggregate usage
/// including total tokens, costs, and timing broken down by model.
pub async fn execute(args: UsageArgs, json: bool) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let repo_root = ckrv_git::repo_root(&cwd).unwrap_or(cwd);
    let chakravarti_dir = repo_root.join(".chakravarti");

    let storage = FileMetricsStorage::new(&chakravarti_dir);
    let all_metrics = storage.list_all().unwrap_or_default();

    if all_metrics.is_empty() {
        if json {
            emit_json(
                UsageOutput {
                    total_jobs: 0,
                    succeeded: 0,
                    failed: 0,
                    total_time_ms: 0,
                    total_tokens: 0,
                    total_cost_usd: 0.0,
                    by_model: Vec::new(),
                    agents: if args.agents { Some(Vec::new()) } else { None },
                    jobs: None,
                },
                json,
            );
        } else {
            println!("No usage data found.");
            println!("Run `ckrv run <spec>` to execute a job.");
        }
        return Ok(());
    }

    // Aggregate totals
    let total_jobs = all_metrics.len();
    let succeeded = all_metrics.iter().filter(|m| m.success).count();
    let failed = total_jobs - succeeded;
    let total_time_ms: u64 = all_metrics.iter().map(|m| m.total_time_ms).sum();
    let total_tokens: u64 = all_metrics.iter().map(|m| m.total_tokens()).sum();
    let total_cost_usd: f64 = all_metrics.iter().map(|m| m.cost.total_usd).sum();

    // Aggregate by model
    let mut model_input: BTreeMap<String, u64> = BTreeMap::new();
    let mut model_output: BTreeMap<String, u64> = BTreeMap::new();
    let mut model_cost: BTreeMap<String, f64> = BTreeMap::new();

    for m in &all_metrics {
        for t in &m.token_usage {
            *model_input.entry(t.model.clone()).or_default() += t.input_tokens;
            *model_output.entry(t.model.clone()).or_default() += t.output_tokens;
        }
        for (model, cost) in &m.cost.by_model {
            *model_cost.entry(model.clone()).or_default() += cost;
        }
    }

    let by_model: Vec<ModelUsage> = model_input
        .keys()
        .map(|model| {
            let input = model_input.get(model).copied().unwrap_or(0);
            let output = model_output.get(model).copied().unwrap_or(0);
            ModelUsage {
                model: model.clone(),
                input_tokens: input,
                output_tokens: output,
                total_tokens: input + output,
                cost_usd: model_cost.get(model).copied().unwrap_or(0.0),
            }
        })
        .collect();

    let agent_usage = if args.agents {
        Some(build_agent_usage(&all_metrics))
    } else {
        None
    };

    let job_summaries: Option<Vec<JobSummary>> = if args.detailed {
        Some(
            all_metrics
                .iter()
                .map(|m| JobSummary {
                    job_id: m.job_id.clone(),
                    spec_id: m.spec_id.clone(),
                    success: m.success,
                    duration_ms: m.total_time_ms,
                    tokens: m.total_tokens(),
                    cost_usd: m.cost.total_usd,
                })
                .collect(),
        )
    } else {
        None
    };

    if json {
        emit_json(
            UsageOutput {
                total_jobs,
                succeeded,
                failed,
                total_time_ms,
                total_tokens,
                total_cost_usd,
                by_model,
                agents: agent_usage,
                jobs: job_summaries,
            },
            json,
        );
    } else {
        println!("USAGE SUMMARY");
        println!("─────────────────────────────────────────────");
        println!(
            "  Jobs:      {} ({} succeeded, {} failed)",
            total_jobs, succeeded, failed
        );
        println!("  Duration:  {}", format_ms(total_time_ms));
        println!("  Tokens:    {}", total_tokens);
        println!("  Cost:      ${:.4}", total_cost_usd);
        println!();

        if !by_model.is_empty() {
            println!("BY MODEL");
            println!("─────────────────────────────────────────────");
            for mu in &by_model {
                println!(
                    "  {}: {} tokens ({} in / {} out), ${:.4}",
                    mu.model, mu.total_tokens, mu.input_tokens, mu.output_tokens, mu.cost_usd
                );
            }
            println!();
        }

        if let Some(agent_rows) = &agent_usage {
            println!("BY AGENT (QUOTA VIEW)");
            println!("─────────────────────────────────────────────");
            if agent_rows.is_empty() {
                println!("  No enabled agents configured or no attributable usage found.");
            } else {
                for row in agent_rows {
                    println!(
                        "  {} [{}]: {} tokens, ${:.4}",
                        row.agent_name, row.agent_type, row.total_tokens, row.total_cost_usd
                    );
                    if let Some(limit) = row.token_limit {
                        let remaining = row.token_remaining.unwrap_or(0);
                        println!(
                            "    Tokens quota: {} used / {} remaining (limit {})",
                            row.total_tokens, remaining, limit
                        );
                    }
                    if let Some(limit) = row.usd_limit {
                        let remaining = row.usd_remaining.unwrap_or(0.0);
                        println!(
                            "    Cost quota: ${:.4} used / ${:.4} remaining (limit ${:.4})",
                            row.total_cost_usd, remaining, limit
                        );
                    }
                }
            }
            println!();
        }

        if let Some(ref jobs) = job_summaries {
            println!("JOBS");
            println!("─────────────────────────────────────────────");
            for j in jobs {
                let status = if j.success { "ok" } else { "FAIL" };
                println!(
                    "  [{}] {} (spec: {}) {} tokens, ${:.4}, {}",
                    status,
                    j.job_id,
                    j.spec_id,
                    j.tokens,
                    j.cost_usd,
                    format_ms(j.duration_ms)
                );
            }
        }
    }

    Ok(())
}

fn build_agent_usage(all_metrics: &[Metrics]) -> Vec<AgentQuotaUsage> {
    let agents = match agent_lookup::load_agents_config() {
        Ok(cfg) => cfg
            .agents
            .into_iter()
            .filter(|a| a.enabled)
            .collect::<Vec<_>>(),
        Err(_) => Vec::new(),
    };

    let mut rows = Vec::new();
    for agent in agents {
        let (token_limit, usd_limit) = parse_quota_limits(agent.env_vars.as_ref());
        let (total_tokens, total_cost_usd) = aggregate_for_agent(&agent, all_metrics);

        let token_remaining = token_limit.map(|l| l.saturating_sub(total_tokens));
        let usd_remaining = usd_limit.map(|l| (l - total_cost_usd).max(0.0));

        rows.push(AgentQuotaUsage {
            agent_id: agent.id.clone(),
            agent_name: agent.name.clone(),
            agent_type: agent_type_name(&agent.agent_type).to_string(),
            total_tokens,
            total_cost_usd,
            token_limit,
            token_remaining,
            usd_limit,
            usd_remaining,
        });
    }

    rows.sort_by(|a, b| a.agent_name.cmp(&b.agent_name));
    rows
}

fn aggregate_for_agent(agent: &AgentConfig, all_metrics: &[Metrics]) -> (u64, f64) {
    let mut tokens = 0_u64;
    let mut cost = 0.0_f64;

    for m in all_metrics {
        for t in &m.token_usage {
            if model_belongs_to_agent_type(&t.model, &agent.agent_type) {
                tokens = tokens.saturating_add(t.total());
            }
        }

        for (model, model_cost) in &m.cost.by_model {
            if model_belongs_to_agent_type(model, &agent.agent_type) {
                cost += model_cost;
            }
        }
    }

    (tokens, cost)
}

fn parse_quota_limits(env_vars: Option<&HashMap<String, String>>) -> (Option<u64>, Option<f64>) {
    let Some(env_vars) = env_vars else {
        return (None, None);
    };

    let token_limit = env_vars
        .get("CKRV_QUOTA_LIMIT_TOKENS")
        .and_then(|v| v.trim().parse::<u64>().ok());
    let usd_limit = env_vars
        .get("CKRV_QUOTA_LIMIT_USD")
        .and_then(|v| v.trim().parse::<f64>().ok());

    (token_limit, usd_limit)
}

fn model_belongs_to_agent_type(model: &str, agent_type: &AgentType) -> bool {
    let m = model.to_ascii_lowercase();
    match agent_type {
        AgentType::Claude => m.contains("claude"),
        AgentType::ClaudeOpenRouter => {
            m.contains("openrouter") || (m.contains("anthropic/") && m.contains("claude"))
        }
        AgentType::ClaudeGlm => m.contains("glm") || m.contains("zhipu"),
        AgentType::Codex => m.contains("codex") || m.contains("gpt-"),
        AgentType::KiloCode => m.contains("kilocode") || m.contains("kilo"),
    }
}

fn agent_type_name(agent_type: &AgentType) -> &'static str {
    match agent_type {
        AgentType::Claude => "claude",
        AgentType::ClaudeOpenRouter => "claude_openrouter",
        AgentType::ClaudeGlm => "claude_glm",
        AgentType::Codex => "codex",
        AgentType::KiloCode => "kilocode",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_belongs_to_agent_type() {
        assert!(model_belongs_to_agent_type(
            "claude-3-5-sonnet",
            &AgentType::Claude
        ));
        assert!(model_belongs_to_agent_type(
            "anthropic/claude-3.7-sonnet",
            &AgentType::ClaudeOpenRouter
        ));
        assert!(model_belongs_to_agent_type(
            "glm-4.5",
            &AgentType::ClaudeGlm
        ));
        assert!(model_belongs_to_agent_type(
            "gpt-5-codex",
            &AgentType::Codex
        ));
        assert!(!model_belongs_to_agent_type(
            "claude-3-opus",
            &AgentType::Codex
        ));
    }

    #[test]
    fn test_parse_quota_limits() {
        let mut env = HashMap::new();
        env.insert("CKRV_QUOTA_LIMIT_TOKENS".to_string(), "500000".to_string());
        env.insert("CKRV_QUOTA_LIMIT_USD".to_string(), "42.5".to_string());

        let (tokens, usd) = parse_quota_limits(Some(&env));
        assert_eq!(tokens, Some(500_000));
        assert_eq!(usd, Some(42.5));
    }

    #[test]
    fn test_parse_quota_limits_invalid_values() {
        let mut env = HashMap::new();
        env.insert("CKRV_QUOTA_LIMIT_TOKENS".to_string(), "nope".to_string());
        env.insert("CKRV_QUOTA_LIMIT_USD".to_string(), "bad".to_string());

        let (tokens, usd) = parse_quota_limits(Some(&env));
        assert_eq!(tokens, None);
        assert_eq!(usd, None);
    }
}
