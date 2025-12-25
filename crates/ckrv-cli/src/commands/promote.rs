//! Promote command - promote job changes to a branch.

use clap::Args;
use serde::Serialize;

use ckrv_git::{BranchManager, GitBranchManager, Worktree, WorktreeStatus};
use ckrv_metrics::{FileMetricsStorage, MetricsStorage};

/// Arguments for the promote command
#[derive(Args)]
pub struct PromoteArgs {
    /// Job ID to promote
    pub job_id: String,

    /// Target branch name
    #[arg(short, long)]
    pub branch: String,

    /// Force overwrite if branch exists
    #[arg(long)]
    pub force: bool,

    /// Push to remote after creating branch
    #[arg(long)]
    pub push: bool,

    /// Remote name for push (default: origin)
    #[arg(long, default_value = "origin")]
    pub remote: String,
}

#[derive(Serialize)]
struct PromoteOutput {
    job_id: String,
    branch: String,
    promoted: bool,
    pushed: bool,
    commit: Option<String>,
    error: Option<String>,
}

/// Execute the promote command
pub fn execute(args: PromoteArgs, json: bool) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;

    // Try to find repo root
    let repo_root = ckrv_git::repo_root(&cwd).unwrap_or(cwd);
    let chakravarti_dir = repo_root.join(".chakravarti");

    // Check if job exists
    let runs_dir = chakravarti_dir.join("runs").join(&args.job_id);

    if !runs_dir.exists() {
        if json {
            let output = PromoteOutput {
                job_id: args.job_id.clone(),
                branch: args.branch.clone(),
                promoted: false,
                pushed: false,
                commit: None,
                error: Some("Job not found".to_string()),
            };
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            eprintln!("Error: Job '{}' not found", args.job_id);
            eprintln!();
            eprintln!("Run `ckrv run <spec>` to create a job first.");
        }
        std::process::exit(1);
    }

    // Check if job succeeded (via metrics or diff)
    let storage = FileMetricsStorage::new(&chakravarti_dir);
    let job_succeeded = if storage.exists(&args.job_id) {
        storage
            .load(&args.job_id)
            .map(|m| m.success)
            .unwrap_or(false)
    } else {
        // Check for diff file as fallback
        runs_dir.join("diff.patch").exists()
    };

    if !job_succeeded && !args.force {
        if json {
            let output = PromoteOutput {
                job_id: args.job_id.clone(),
                branch: args.branch.clone(),
                promoted: false,
                pushed: false,
                commit: None,
                error: Some("Job did not succeed. Use --force to promote anyway.".to_string()),
            };
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            eprintln!("Error: Job '{}' did not succeed", args.job_id);
            eprintln!();
            eprintln!("Only successful jobs can be promoted.");
            eprintln!("Use --force to promote a failed job anyway.");
        }
        std::process::exit(1);
    }

    // Create branch manager
    let branch_manager = GitBranchManager::new(&repo_root);

    // Check if branch exists
    if branch_manager.exists(&args.branch) && !args.force {
        if json {
            let output = PromoteOutput {
                job_id: args.job_id.clone(),
                branch: args.branch.clone(),
                promoted: false,
                pushed: false,
                commit: None,
                error: Some(format!(
                    "Branch '{}' already exists. Use --force to overwrite.",
                    args.branch
                )),
            };
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            eprintln!("Error: Branch '{}' already exists", args.branch);
            eprintln!();
            eprintln!("Use --force to overwrite the existing branch.");
        }
        std::process::exit(1);
    }

    // Find worktree for this job
    let worktrees_dir = chakravarti_dir.join("worktrees");
    let mut worktree_path = None;

    if worktrees_dir.exists() {
        // Find worktrees matching this job_id (format: job_id_attempt-N)
        if let Ok(entries) = std::fs::read_dir(&worktrees_dir) {
            let prefix = format!("{}_attempt-", args.job_id);
            let mut matching: Vec<_> = entries
                .filter_map(|e| e.ok())
                .filter(|e| e.file_name().to_string_lossy().starts_with(&prefix))
                .collect();
            matching.sort_by_key(|e| e.file_name());
            if let Some(last) = matching.last() {
                worktree_path = Some(last.path());
            }
        }
    }

    let worktree_path = match worktree_path {
        Some(p) => p,
        None => {
            if json {
                let output = PromoteOutput {
                    job_id: args.job_id.clone(),
                    branch: args.branch.clone(),
                    promoted: false,
                    pushed: false,
                    commit: None,
                    error: Some("No worktree found for job".to_string()),
                };
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                eprintln!("Error: No worktree found for job '{}'", args.job_id);
            }
            std::process::exit(1);
        }
    };

    // Create worktree struct
    let worktree = Worktree {
        path: worktree_path.clone(),
        job_id: args.job_id.clone(),
        attempt_id: worktree_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default(),
        base_commit: String::new(), // Will be populated by branch manager
        status: WorktreeStatus::Ready,
    };

    // Create branch from worktree
    match branch_manager.create_from_worktree(&worktree, &args.branch, args.force) {
        Ok(()) => {
            // Get commit hash
            let commit = std::process::Command::new("git")
                .args(["rev-parse", &format!("refs/heads/{}", args.branch)])
                .current_dir(&repo_root)
                .output()
                .ok()
                .and_then(|o| {
                    if o.status.success() {
                        Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
                    } else {
                        None
                    }
                });

            // Push if requested
            let pushed = if args.push {
                match branch_manager.push(&args.branch, &args.remote, args.force) {
                    Ok(()) => true,
                    Err(e) => {
                        if !json {
                            eprintln!("Warning: Failed to push to remote: {}", e);
                        }
                        false
                    }
                }
            } else {
                false
            };

            if json {
                let output = PromoteOutput {
                    job_id: args.job_id.clone(),
                    branch: args.branch.clone(),
                    promoted: true,
                    pushed,
                    commit,
                    error: None,
                };
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                println!(
                    "✓ Promoted job '{}' to branch '{}'",
                    args.job_id, args.branch
                );
                if let Some(ref c) = commit.as_ref().map(|s| &s[..7.min(s.len())]) {
                    println!("  Commit: {}", c);
                }
                if pushed {
                    println!("  Pushed to: {}/{}", args.remote, args.branch);
                } else if args.push {
                    println!("  Push: failed (see warning above)");
                }
                println!();
                println!("To switch to this branch:");
                println!("  git checkout {}", args.branch);
            }
        }
        Err(e) => {
            if json {
                let output = PromoteOutput {
                    job_id: args.job_id.clone(),
                    branch: args.branch.clone(),
                    promoted: false,
                    pushed: false,
                    commit: None,
                    error: Some(e.to_string()),
                };
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                eprintln!("Error: Failed to create branch: {}", e);
            }
            std::process::exit(1);
        }
    }

    Ok(())
}
