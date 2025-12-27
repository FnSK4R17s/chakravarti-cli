//! Diff command - view job diffs.

use std::path::PathBuf;

use clap::{Args, ValueEnum};
use serde::Serialize;

/// Arguments for the diff command
#[derive(Args)]
pub struct DiffArgs {
    /// Job ID to view diff for
    pub job_id: String,

    /// Show diff statistics only
    #[arg(long)]
    pub stat: bool,

    /// Color mode for diff output
    #[arg(long, value_enum, default_value = "auto")]
    pub color: ColorMode,
}

/// Color mode for diff output.
#[derive(Clone, Copy, ValueEnum)]
pub enum ColorMode {
    /// Auto-detect based on terminal.
    Auto,
    /// Always use colors.
    Always,
    /// Never use colors.
    Never,
}

#[derive(Serialize)]
struct DiffOutput {
    job_id: String,
    has_diff: bool,
    lines_added: usize,
    lines_removed: usize,
    files_changed: usize,
    diff: Option<String>,
}

#[derive(Serialize)]
struct DiffStat {
    file: String,
    insertions: usize,
    deletions: usize,
}

/// Execute the diff command
pub async fn execute(args: DiffArgs, json: bool) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;

    // Try to find repo root
    let repo_root = ckrv_git::repo_root(&cwd).unwrap_or(cwd);
    let chakravarti_dir = repo_root.join(".chakravarti");

    // Find diff file
    let diff_path = chakravarti_dir
        .join("runs")
        .join(&args.job_id)
        .join("diff.patch");

    if diff_path.exists() {
        let diff_content = std::fs::read_to_string(&diff_path)?;

        // Parse diff stats
        let (added, removed, files) = parse_diff_stats(&diff_content);

        if json {
            let output = DiffOutput {
                job_id: args.job_id.clone(),
                has_diff: !diff_content.is_empty(),
                lines_added: added,
                lines_removed: removed,
                files_changed: files,
                diff: if args.stat {
                    None
                } else {
                    Some(diff_content.clone())
                },
            };
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            if args.stat {
                println!("Diff statistics for job: {}", args.job_id);
                println!();
                println!("  {} files changed", files);
                println!("  +{} insertions", added);
                println!("  -{} deletions", removed);
            } else {
                // Print diff with optional coloring
                let use_color = match args.color {
                    ColorMode::Always => true,
                    ColorMode::Never => false,
                    ColorMode::Auto => atty::is(atty::Stream::Stdout),
                };

                println!("Diff for job: {}\n", args.job_id);

                for line in diff_content.lines() {
                    if use_color {
                        if line.starts_with('+') && !line.starts_with("+++") {
                            println!("\x1b[32m{}\x1b[0m", line);
                        } else if line.starts_with('-') && !line.starts_with("---") {
                            println!("\x1b[31m{}\x1b[0m", line);
                        } else if line.starts_with("@@") {
                            println!("\x1b[36m{}\x1b[0m", line);
                        } else if line.starts_with("diff ") || line.starts_with("index ") {
                            println!("\x1b[1m{}\x1b[0m", line);
                        } else {
                            println!("{}", line);
                        }
                    } else {
                        println!("{}", line);
                    }
                }
            }
        }
    } else {
        if json {
            let output = DiffOutput {
                job_id: args.job_id.clone(),
                has_diff: false,
                lines_added: 0,
                lines_removed: 0,
                files_changed: 0,
                diff: None,
            };
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            println!("Job: {}", args.job_id);
            println!("Status: No diff available");
            println!();
            println!("The job may not have been run yet, or it may have failed.");
            println!("Run `ckrv status {}` to check the job status.", args.job_id);
        }
    }

    Ok(())
}

fn parse_diff_stats(diff: &str) -> (usize, usize, usize) {
    let mut added = 0;
    let mut removed = 0;
    let mut files = 0;

    for line in diff.lines() {
        if line.starts_with("diff ") {
            files += 1;
        } else if line.starts_with('+') && !line.starts_with("+++") {
            added += 1;
        } else if line.starts_with('-') && !line.starts_with("---") {
            removed += 1;
        }
    }

    (added, removed, files)
}

/// Check if we're in a terminal.
mod atty {
    pub enum Stream {
        Stdout,
    }

    pub fn is(_: Stream) -> bool {
        // Simple check - in real impl would use atty crate
        std::env::var("TERM").is_ok() && std::env::var("NO_COLOR").is_err()
    }
}
