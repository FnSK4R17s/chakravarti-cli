//! Promote command - create a pull/merge request for the current branch.

use std::path::PathBuf;

use clap::Args;
use serde::{Deserialize, Serialize};

use crate::ui::UiContext;
use crate::ui::Renderable;
use crate::ui::components::Banner;

/// Arguments for the promote command
#[derive(Args)]
pub struct PromoteArgs {
    /// Target branch for the PR (default: main or master)
    #[arg(short, long)]
    pub base: Option<String>,

    /// Create as draft PR
    #[arg(long)]
    pub draft: bool,

    /// Push branch to remote before creating PR
    #[arg(long)]
    pub push: bool,

    /// Remote name (default: origin)
    #[arg(long, default_value = "origin")]
    pub remote: String,

    /// Open PR URL in browser after creation
    #[arg(long)]
    pub open: bool,

    /// Skip verification checks
    #[arg(long)]
    pub skip_verify: bool,
}

#[derive(Serialize)]
pub struct PromoteOutput {
    pub success: bool,
    pub branch: String,
    pub base: String,
    pub pushed: bool,
    pub pr_url: Option<String>,
    pub pr_number: Option<u64>,
    pub message: String,
}

#[derive(Deserialize)]
struct ImplementationSummary {
    status: String,
    branch: String,
    tasks_completed: usize,
    message: String,
}

#[derive(Deserialize)]
struct SpecYaml {
    name: String,
    description: Option<String>,
}

/// Execute the promote command
pub async fn execute(args: PromoteArgs, json: bool, ui: &UiContext) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;

    if !json {
        println!("{}", Banner::new("CKRV PROMOTE").subtitle("Create Pull Request").render(&ui.theme));
    }

    // Get current branch
    let current_branch = get_current_branch(&cwd)?;
    let base_branch = args.base.unwrap_or_else(|| detect_default_branch(&cwd));

    if current_branch == base_branch {
        let output = PromoteOutput {
            success: false,
            branch: current_branch.clone(),
            base: base_branch.clone(),
            pushed: false,
            pr_url: None,
            pr_number: None,
            message: format!("Cannot create PR: already on base branch '{}'", base_branch),
        };
        
        if json {
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            eprintln!("❌ {}", output.message);
        }
        std::process::exit(1);
    }

    if !json {
        println!("📋 Branch: {} → {}\n", current_branch, base_branch);
    }

    // Check for implementation.yaml to ensure run completed
    let spec_dir = cwd.join(".specs").join(&current_branch);
    let impl_path = spec_dir.join("implementation.yaml");
    
    if !impl_path.exists() && !args.skip_verify {
        let output = PromoteOutput {
            success: false,
            branch: current_branch.clone(),
            base: base_branch.clone(),
            pushed: false,
            pr_url: None,
            pr_number: None,
            message: "Implementation not complete. Run 'ckrv run' first or use --skip-verify".to_string(),
        };
        
        if json {
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            eprintln!("❌ {}", output.message);
        }
        std::process::exit(1);
    }

    // Load spec for PR description
    let spec_path = spec_dir.join("spec.yaml");
    let (spec_name, spec_description) = if spec_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&spec_path) {
            if let Ok(spec) = serde_yaml::from_str::<SpecYaml>(&content) {
                (spec.name, spec.description)
            } else {
                (current_branch.clone(), None)
            }
        } else {
            (current_branch.clone(), None)
        }
    } else {
        (current_branch.clone(), None)
    };

    // Load implementation summary
    let impl_summary = if impl_path.exists() {
        std::fs::read_to_string(&impl_path)
            .ok()
            .and_then(|c| serde_yaml::from_str::<ImplementationSummary>(&c).ok())
    } else {
        None
    };

    // Push if requested
    let pushed = if args.push {
        if !json {
            println!("📤 Pushing branch to {}...", args.remote);
        }
        
        let push_result = std::process::Command::new("git")
            .args(["push", "-u", &args.remote, &current_branch])
            .current_dir(&cwd)
            .status();
        
        match push_result {
            Ok(status) if status.success() => {
                if !json {
                    println!("   ✅ Pushed successfully\n");
                }
                true
            }
            _ => {
                if !json {
                    eprintln!("   ⚠️  Push failed (you may need to push manually)\n");
                }
                false
            }
        }
    } else {
        false
    };

    // Detect remote type and create PR
    let remote_url = get_remote_url(&cwd, &args.remote);
    let (pr_url, pr_number) = if let Some(url) = &remote_url {
        if url.contains("github.com") {
            create_github_pr(&cwd, &current_branch, &base_branch, &spec_name, &spec_description, &impl_summary, args.draft, json).await
        } else if url.contains("gitlab") {
            // GitLab MR creation would go here
            (generate_pr_url_github(url, &current_branch, &base_branch), None)
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    // Generate manual PR URL if we couldn't create one automatically
    let final_pr_url = pr_url.or_else(|| {
        remote_url.as_ref().and_then(|url| generate_pr_url_github(url, &current_branch, &base_branch))
    });

    let output = PromoteOutput {
        success: true,
        branch: current_branch.clone(),
        base: base_branch.clone(),
        pushed,
        pr_url: final_pr_url.clone(),
        pr_number,
        message: if pr_number.is_some() {
            format!("PR #{} created successfully", pr_number.unwrap())
        } else {
            "Ready to create PR".to_string()
        },
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("═══════════════════════════════════════════════════════════════");
        
        if let Some(num) = pr_number {
            println!("✅ Pull Request #{} created!", num);
        } else {
            println!("✅ Branch ready for Pull Request!");
        }
        
        println!("═══════════════════════════════════════════════════════════════\n");

        println!("📊 Summary:");
        println!("   Branch: {}", current_branch);
        println!("   Target: {}", base_branch);
        if let Some(ref summary) = impl_summary {
            println!("   Tasks:  {} completed", summary.tasks_completed);
        }
        println!();

        if let Some(ref url) = final_pr_url {
            println!("🔗 PR URL: {}", url);
            println!();
            
            if args.open {
                let _ = open_url(url);
            }
        }

        if pr_number.is_none() {
            println!("💡 Create PR manually:");
            if let Some(ref url) = final_pr_url {
                println!("   {}", url);
            } else {
                println!("   Go to your repository and create a PR from '{}' to '{}'", current_branch, base_branch);
            }
        }

        // Save PR info
        if let Some(ref url) = final_pr_url {
            save_pr_info(&spec_dir, url, pr_number)?;
        }
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

fn get_remote_url(cwd: &PathBuf, remote: &str) -> Option<String> {
    let output = std::process::Command::new("git")
        .args(["remote", "get-url", remote])
        .current_dir(cwd)
        .output()
        .ok()?;
    
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

fn generate_pr_url_github(remote_url: &str, branch: &str, base: &str) -> Option<String> {
    // Parse GitHub URL
    // Formats: git@github.com:owner/repo.git or https://github.com/owner/repo.git
    let repo_path = if remote_url.starts_with("git@github.com:") {
        remote_url.strip_prefix("git@github.com:")?.strip_suffix(".git")
    } else if remote_url.contains("github.com/") {
        let parts: Vec<&str> = remote_url.split("github.com/").collect();
        parts.get(1).and_then(|p| p.strip_suffix(".git").or(Some(*p)))
    } else {
        None
    }?;

    Some(format!(
        "https://github.com/{}/compare/{}...{}?expand=1",
        repo_path, base, branch
    ))
}

async fn create_github_pr(
    cwd: &PathBuf,
    branch: &str,
    base: &str,
    title: &str,
    description: &Option<String>,
    impl_summary: &Option<ImplementationSummary>,
    draft: bool,
    json: bool,
) -> (Option<String>, Option<u64>) {
    // Check if gh CLI is available
    let gh_check = std::process::Command::new("gh")
        .arg("--version")
        .output();
    
    if gh_check.is_err() || !gh_check.unwrap().status.success() {
        return (None, None);
    }

    // Build PR body
    let mut body = String::new();
    if let Some(ref desc) = description {
        body.push_str(desc);
        body.push_str("\n\n");
    }
    
    body.push_str("## Implementation Details\n\n");
    body.push_str("This PR was created using [CKRV](https://github.com/FnSK4R17s/chakravarti-cli) - Spec-driven Agent Orchestration.\n\n");
    
    if let Some(ref summary) = impl_summary {
        body.push_str(&format!("- **Tasks Completed**: {}\n", summary.tasks_completed));
        body.push_str(&format!("- **Status**: {}\n", summary.status));
    }

    if !json {
        println!("🚀 Creating GitHub PR using gh CLI...");
    }

    let mut args = vec![
        "pr", "create",
        "--title", title,
        "--body", &body,
        "--base", base,
        "--head", branch,
    ];

    if draft {
        args.push("--draft");
    }

    let output = std::process::Command::new("gh")
        .args(&args)
        .current_dir(cwd)
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let url = String::from_utf8_lossy(&out.stdout).trim().to_string();
            // Extract PR number from URL
            let pr_number = url.split('/').last().and_then(|s| s.parse().ok());
            (Some(url), pr_number)
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            if !json {
                eprintln!("   ⚠️  gh CLI failed: {}", stderr.lines().next().unwrap_or("unknown error"));
            }
            (None, None)
        }
        Err(_) => (None, None),
    }
}

fn save_pr_info(spec_dir: &PathBuf, url: &str, number: Option<u64>) -> anyhow::Result<()> {
    #[derive(Serialize)]
    struct PrInfo {
        url: String,
        number: Option<u64>,
        created_at: String,
    }

    let info = PrInfo {
        url: url.to_string(),
        number,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    if spec_dir.exists() {
        let yaml = serde_yaml::to_string(&info)?;
        std::fs::write(spec_dir.join("pr.yaml"), yaml)?;
    }

    Ok(())
}

fn open_url(url: &str) -> anyhow::Result<()> {
    #[cfg(target_os = "macos")]
    std::process::Command::new("open").arg(url).spawn()?;
    
    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open").arg(url).spawn()?;
    
    #[cfg(target_os = "windows")]
    std::process::Command::new("cmd").args(["/C", "start", url]).spawn()?;
    
    Ok(())
}
