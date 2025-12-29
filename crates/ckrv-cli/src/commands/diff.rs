//! Diff command - view changes between main branch and current spec branch.

use std::path::PathBuf;

use clap::{Args, ValueEnum};
use serde::Serialize;

use crate::ui::UiContext;
use crate::ui::Renderable;
use crate::ui::components::Banner;

/// Arguments for the diff command
#[derive(Args)]
pub struct DiffArgs {
    /// Base branch to compare against (default: main or master)
    #[arg(short, long)]
    pub base: Option<String>,

    /// Show diff statistics only
    #[arg(long)]
    pub stat: bool,

    /// Show file list only
    #[arg(long)]
    pub files: bool,

    /// Generate AI summary of changes
    #[arg(long)]
    pub summary: bool,

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
pub struct DiffOutput {
    pub current_branch: String,
    pub base_branch: String,
    pub has_changes: bool,
    pub lines_added: usize,
    pub lines_removed: usize,
    pub files_changed: usize,
    pub files: Vec<FileChange>,
    pub summary: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct FileChange {
    pub file: String,
    pub status: String, // added, modified, deleted, renamed
    pub insertions: usize,
    pub deletions: usize,
}

/// Execute the diff command
pub async fn execute(args: DiffArgs, json: bool, ui: &UiContext) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;

    if !json {
        println!("{}", Banner::new("CKRV DIFF").subtitle("Review Changes").render(&ui.theme));
    }

    // Get current branch
    let current_branch = get_current_branch(&cwd)?;
    
    // Determine base branch
    let base_branch = args.base.unwrap_or_else(|| detect_default_branch(&cwd));

    if current_branch == base_branch {
        if json {
            let output = DiffOutput {
                current_branch: current_branch.clone(),
                base_branch: base_branch.clone(),
                has_changes: false,
                lines_added: 0,
                lines_removed: 0,
                files_changed: 0,
                files: vec![],
                summary: None,
            };
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            println!("⚠️  Currently on base branch '{}'. No diff to show.", base_branch);
            println!("   Switch to a spec branch first: git checkout <spec-branch>");
        }
        return Ok(());
    }

    if !json {
        println!("Comparing: {} → {}\n", base_branch, current_branch);
    }

    // Get diff statistics
    let diff_stat = std::process::Command::new("git")
        .args(["diff", "--stat", &format!("{}...{}", base_branch, current_branch)])
        .current_dir(&cwd)
        .output()?;

    // Get file list with status
    let diff_name_status = std::process::Command::new("git")
        .args(["diff", "--name-status", &format!("{}...{}", base_branch, current_branch)])
        .current_dir(&cwd)
        .output()?;

    // Parse files
    let files = parse_name_status(&String::from_utf8_lossy(&diff_name_status.stdout));
    
    // Get numstat for line counts
    let diff_numstat = std::process::Command::new("git")
        .args(["diff", "--numstat", &format!("{}...{}", base_branch, current_branch)])
        .current_dir(&cwd)
        .output()?;
    
    let (files, total_added, total_removed) = parse_numstat(
        &String::from_utf8_lossy(&diff_numstat.stdout),
        files,
    );

    let files_changed = files.len();
    let has_changes = files_changed > 0;

    // Generate AI summary if requested
    let summary = if args.summary && has_changes {
        if !json {
            println!("🤖 Generating AI summary...\n");
        }
        generate_ai_summary(&cwd, &base_branch, &current_branch).await.ok()
    } else {
        None
    };

    if json {
        let output = DiffOutput {
            current_branch: current_branch.clone(),
            base_branch: base_branch.clone(),
            has_changes,
            lines_added: total_added,
            lines_removed: total_removed,
            files_changed,
            files: files.clone(),
            summary,
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        // Print summary header
        println!("📊 Diff Summary");
        println!("   {} files changed", files_changed);
        println!("   \x1b[32m+{} insertions\x1b[0m", total_added);
        println!("   \x1b[31m-{} deletions\x1b[0m", total_removed);
        println!();

        if args.files || args.stat {
            // Show file list
            println!("📁 Changed Files:");
            for file in &files {
                let status_icon = match file.status.as_str() {
                    "added" => "\x1b[32m+\x1b[0m",
                    "deleted" => "\x1b[31m-\x1b[0m",
                    "modified" => "\x1b[33m~\x1b[0m",
                    "renamed" => "\x1b[36m→\x1b[0m",
                    _ => " ",
                };
                println!("   {} {} \x1b[32m+{}\x1b[0m/\x1b[31m-{}\x1b[0m", 
                    status_icon, file.file, file.insertions, file.deletions);
            }
            println!();
        }

        if let Some(ref s) = summary {
            println!("📝 AI Summary:");
            for line in s.lines() {
                println!("   {}", line);
            }
            println!();
        }

        if !args.stat && !args.files && !args.summary {
            // Show full diff
            let use_color = match args.color {
                ColorMode::Always => true,
                ColorMode::Never => false,
                ColorMode::Auto => is_terminal(),
            };

            let diff_output = std::process::Command::new("git")
                .args(["diff", &format!("{}...{}", base_branch, current_branch)])
                .current_dir(&cwd)
                .output()?;

            let diff_content = String::from_utf8_lossy(&diff_output.stdout);
            
            println!("📄 Full Diff:\n");
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

        // Show next steps
        println!("\n💡 Next Steps:");
        println!("   ckrv verify     # Run tests and checks");
        println!("   ckrv promote    # Create pull request");
    }

    Ok(())
}

fn get_current_branch(cwd: &PathBuf) -> anyhow::Result<String> {
    let output = std::process::Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(cwd)
        .output()?;
    
    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if branch.is_empty() {
        return Err(anyhow::anyhow!("Not on a branch (detached HEAD state)"));
    }
    Ok(branch)
}

fn detect_default_branch(cwd: &PathBuf) -> String {
    // Try to detect main/master
    let output = std::process::Command::new("git")
        .args(["branch", "-l", "main", "master"])
        .current_dir(cwd)
        .output();
    
    if let Ok(out) = output {
        let branches = String::from_utf8_lossy(&out.stdout);
        if branches.contains("main") {
            return "main".to_string();
        }
    }
    "master".to_string()
}

fn parse_name_status(output: &str) -> Vec<FileChange> {
    output
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 2 {
                let status = match parts[0].chars().next()? {
                    'A' => "added",
                    'D' => "deleted",
                    'M' => "modified",
                    'R' => "renamed",
                    'C' => "copied",
                    _ => "modified",
                };
                Some(FileChange {
                    file: parts.last()?.to_string(),
                    status: status.to_string(),
                    insertions: 0,
                    deletions: 0,
                })
            } else {
                None
            }
        })
        .collect()
}

fn parse_numstat(output: &str, mut files: Vec<FileChange>) -> (Vec<FileChange>, usize, usize) {
    let mut total_added = 0;
    let mut total_removed = 0;

    for line in output.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 3 {
            let added: usize = parts[0].parse().unwrap_or(0);
            let removed: usize = parts[1].parse().unwrap_or(0);
            let file = parts[2];
            
            total_added += added;
            total_removed += removed;

            // Update file entry
            if let Some(f) = files.iter_mut().find(|f| f.file == file) {
                f.insertions = added;
                f.deletions = removed;
            }
        }
    }

    (files, total_added, total_removed)
}

async fn generate_ai_summary(cwd: &PathBuf, base: &str, current: &str) -> anyhow::Result<String> {
    // Get commit messages
    let log_output = std::process::Command::new("git")
        .args(["log", "--oneline", &format!("{}..{}", base, current)])
        .current_dir(cwd)
        .output()?;
    
    let commits = String::from_utf8_lossy(&log_output.stdout);

    // Get diff stat
    let stat_output = std::process::Command::new("git")
        .args(["diff", "--stat", &format!("{}...{}", base, current)])
        .current_dir(cwd)
        .output()?;
    
    let stat = String::from_utf8_lossy(&stat_output.stdout);

    // Simple summary without AI for now (can be enhanced with Claude)
    let commit_count = commits.lines().count();
    let summary = format!(
        "This branch contains {} commit(s).\n\nKey changes:\n{}",
        commit_count,
        commits.lines().take(10).collect::<Vec<_>>().join("\n")
    );

    Ok(summary)
}

fn is_terminal() -> bool {
    std::env::var("TERM").is_ok() && std::env::var("NO_COLOR").is_err()
}
