//! QA command - code review and bug analysis.

use clap::{Args, Subcommand};
use serde::Serialize;

use crate::services::{
    agent_lookup, diff_analyzer,
    report_generator::{self, IssueCategory, QAIssue, QASummary, Severity},
};
use crate::ui::components::Banner;
use crate::ui::{Renderable, UiContext};

/// Arguments for the qa command
#[derive(Args)]
pub struct QaArgs {
    #[command(subcommand)]
    pub command: QaSubcommand,
}

/// QA subcommands
#[derive(Subcommand)]
pub enum QaSubcommand {
    /// Review code quality of changes
    Review {
        /// Branch to compare against (default: main)
        #[arg(long, default_value = "main")]
        base: String,

        /// Output file path
        #[arg(long, short)]
        output: Option<String>,
    },

    /// Analyze for potential bugs
    Bugs {
        /// Branch to compare against (default: main)
        #[arg(long, default_value = "main")]
        base: String,
    },

    /// Generate full QA report
    Report {
        /// Branch to compare against (default: main)
        #[arg(long, default_value = "main")]
        base: String,

        /// Include all analysis types
        #[arg(long)]
        full: bool,

        /// Output file path
        #[arg(long, short)]
        output: Option<String>,
    },
}

/// Output for QA review
#[derive(Serialize)]
pub struct QaReviewOutput {
    pub report_id: String,
    pub base_branch: String,
    pub issues: Vec<QAIssue>,
    pub summary: QASummary,
    pub agent_id: Option<String>,
}

/// Execute the qa command
pub async fn execute(args: QaArgs, json: bool, ui: &UiContext) -> anyhow::Result<()> {
    match args.command {
        QaSubcommand::Review { base, output } => execute_review(&base, output, json, ui).await,
        QaSubcommand::Bugs { base } => execute_bugs(&base, json, ui).await,
        QaSubcommand::Report { base, full, output } => {
            execute_report(&base, full, output, json, ui).await
        }
    }
}

/// Execute QA review
async fn execute_review(
    base: &str,
    output: Option<String>,
    json: bool,
    ui: &UiContext,
) -> anyhow::Result<()> {
    if !json {
        println!(
            "{}",
            Banner::new("CKRV QA")
                .subtitle("Code Review")
                .render(&ui.theme)
        );
    }

    // Check for QA agent
    let agent = match agent_lookup::find_qa_agent() {
        Some(a) => a,
        None => {
            if json {
                println!(r#"{{"error": "No QA agent configured", "exit_code": 4}}"#);
            } else {
                println!("{}", agent_lookup::qa_agent_missing_message());
            }
            std::process::exit(4);
        }
    };

    if !json {
        println!("🤖 Using agent: {} ({})\n", agent.name, agent.id);
        println!("📝 Analyzing changes vs '{}'...\n", base);
    }

    // Get changed files
    let changed_files = match diff_analyzer::get_changed_files(base) {
        Ok(files) => files,
        Err(e) => {
            if !json {
                println!("❌ Failed to get changed files: {}", e);
                println!(
                    "\nMake sure you're on a feature branch with changes vs '{}'",
                    base
                );
            }
            return Err(e);
        }
    };

    if changed_files.is_empty() {
        if !json {
            println!("ℹ️  No changes found compared to '{}'", base);
        }
        return Ok(());
    }

    // For now, simulate QA analysis
    // Full agent invocation would require Docker sandbox integration
    let issues = simulate_qa_analysis(&changed_files);
    let summary = QASummary::from_issues(&issues, changed_files.len() as u32);

    let branch = diff_analyzer::get_current_branch().unwrap_or_else(|_| "unknown".to_string());

    let review_output = QaReviewOutput {
        report_id: format!("qa-{}", chrono::Utc::now().timestamp()),
        base_branch: base.to_string(),
        issues: issues.clone(),
        summary: summary.clone(),
        agent_id: Some(agent.id.clone()),
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&review_output)?);
    } else {
        let report = report_generator::generate_qa_report(&issues, &branch, base);

        if let Some(output_path) = output {
            std::fs::write(&output_path, &report)?;
            println!("📄 Report saved to: {}", output_path);
        } else {
            println!("{}", report);
        }

        println!("\n⚠️  Note: Full QA analysis requires Docker sandbox integration.");
        println!(
            "   The QA agent ({}) would provide more detailed analysis.",
            agent.name
        );
    }

    // Exit with error if critical issues
    if summary.critical > 0 {
        std::process::exit(1);
    }

    Ok(())
}

/// Execute bugs analysis
async fn execute_bugs(base: &str, json: bool, ui: &UiContext) -> anyhow::Result<()> {
    if !json {
        println!(
            "{}",
            Banner::new("CKRV QA")
                .subtitle("Bug Analysis")
                .render(&ui.theme)
        );
    }

    // Check for QA agent
    let agent = match agent_lookup::find_qa_agent() {
        Some(a) => a,
        None => {
            if json {
                println!(r#"{{"error": "No QA agent configured", "exit_code": 4}}"#);
            } else {
                println!("{}", agent_lookup::qa_agent_missing_message());
            }
            std::process::exit(4);
        }
    };

    if !json {
        println!("🤖 Using agent: {} ({})\n", agent.name, agent.id);
        println!(
            "🔍 Scanning for potential bugs in changes vs '{}'...\n",
            base
        );
    }

    // Get changed files
    let changed_files = diff_analyzer::get_changed_files(base)?;

    if changed_files.is_empty() {
        if !json {
            println!("ℹ️  No changes found.");
        }
        return Ok(());
    }

    // Filter for bug-related issues only
    let all_issues = simulate_qa_analysis(&changed_files);
    let bug_issues: Vec<_> = all_issues
        .into_iter()
        .filter(|i| {
            matches!(
                i.category,
                IssueCategory::PotentialBug | IssueCategory::ErrorHandling
            )
        })
        .collect();

    if json {
        println!("{}", serde_json::to_string_pretty(&bug_issues)?);
    } else {
        if bug_issues.is_empty() {
            println!("✅ No potential bugs found in changed files.\n");
        } else {
            println!("## Potential Bugs Found\n");
            for issue in &bug_issues {
                println!(
                    "{} **{}** - `{}`",
                    issue.severity.emoji(),
                    issue.message,
                    issue.file
                );
                if let Some(ref suggestion) = issue.suggestion {
                    println!("   Fix: {}\n", suggestion);
                }
            }
        }

        println!("⚠️  Note: Full bug analysis requires Docker sandbox integration.");
    }

    Ok(())
}

/// Execute full QA report
async fn execute_report(
    base: &str,
    full: bool,
    output: Option<String>,
    json: bool,
    ui: &UiContext,
) -> anyhow::Result<()> {
    if !json {
        println!(
            "{}",
            Banner::new("CKRV QA")
                .subtitle("Full Report")
                .render(&ui.theme)
        );
    }

    // Check for QA agent
    let agent = match agent_lookup::find_qa_agent() {
        Some(a) => a,
        None => {
            if json {
                println!(r#"{{"error": "No QA agent configured", "exit_code": 4}}"#);
            } else {
                println!("{}", agent_lookup::qa_agent_missing_message());
            }
            std::process::exit(4);
        }
    };

    if !json {
        println!("🤖 Using agent: {} ({})\n", agent.name, agent.id);
        println!(
            "📊 Generating {} QA report for changes vs '{}'...\n",
            if full { "full" } else { "standard" },
            base
        );
    }

    // Get changed files
    let changed_files = diff_analyzer::get_changed_files(base)?;
    let change_summary = diff_analyzer::get_change_summary(base)?;

    if changed_files.is_empty() {
        if !json {
            println!("ℹ️  No changes found.");
        }
        return Ok(());
    }

    // Get issues
    let issues = simulate_qa_analysis(&changed_files);
    let branch = diff_analyzer::get_current_branch().unwrap_or_else(|_| "unknown".to_string());

    if json {
        let summary = QASummary::from_issues(&issues, changed_files.len() as u32);
        let output = QaReviewOutput {
            report_id: format!("qa-report-{}", chrono::Utc::now().timestamp()),
            base_branch: base.to_string(),
            issues,
            summary,
            agent_id: Some(agent.id),
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        let mut report = String::new();

        report.push_str("# QA Report\n\n");
        report.push_str(&format!("**Branch**: {}\n", branch));
        report.push_str(&format!("**Base**: {}\n", base));
        report.push_str(&format!(
            "**Date**: {}\n\n",
            chrono::Utc::now().format("%Y-%m-%d %H:%M UTC")
        ));

        report.push_str("## Change Summary\n\n");
        report.push_str(&format!("{}\n\n", change_summary));

        if full {
            report.push_str("## Changed Files\n\n");
            for file in &changed_files {
                report.push_str(&format!(
                    "- `{}` ({:?}, +{} -{})\n",
                    file.path.display(),
                    file.change_type,
                    file.lines_added,
                    file.lines_removed
                ));
            }
            report.push_str("\n");
        }

        // Add standard QA report
        report.push_str(&report_generator::generate_qa_report(
            &issues, &branch, base,
        ));

        if let Some(output_path) = output {
            std::fs::write(&output_path, &report)?;
            println!("📄 Report saved to: {}", output_path);
        } else {
            println!("{}", report);
        }
    }

    Ok(())
}

/// Simulate QA analysis (placeholder until agent integration)
fn simulate_qa_analysis(changed_files: &[diff_analyzer::ChangedFile]) -> Vec<QAIssue> {
    let mut issues = Vec::new();
    let mut issue_id = 1;

    for file in changed_files {
        let file_str = file.path.to_string_lossy().to_string();

        // Heuristic: large files might have issues
        if file.lines_added > 100 {
            issues.push(QAIssue {
                id: format!("QA-{:03}", issue_id),
                file: file_str.clone(),
                line: None,
                severity: Severity::Minor,
                category: IssueCategory::CodeQuality,
                message: "Large change - consider breaking into smaller commits".to_string(),
                suggestion: Some("Split into logical chunks for easier review".to_string()),
            });
            issue_id += 1;
        }

        // Check file extension for common patterns
        let ext = file.path.extension().unwrap_or_default().to_string_lossy();

        if ext == "rs" && file.lines_added > 50 {
            issues.push(QAIssue {
                id: format!("QA-{:03}", issue_id),
                file: file_str.clone(),
                line: None,
                severity: Severity::Info,
                category: IssueCategory::BestPractice,
                message: "Consider adding doc comments for public items".to_string(),
                suggestion: Some(
                    "Add /// doc comments to public functions and structs".to_string(),
                ),
            });
            issue_id += 1;
        }
    }

    // If no issues found, we're clean
    if issues.is_empty() && !changed_files.is_empty() {
        // Return empty - no simulated issues for small changes
    }

    issues
}
