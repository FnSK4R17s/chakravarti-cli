//! Init command - initialize Chakravarti in a repository.

use std::path::PathBuf;

use clap::Args;
use serde::Serialize;

/// Arguments for the init command
#[derive(Args)]
pub struct InitArgs {
    /// Force reinitialization even if already initialized
    #[arg(long)]
    pub force: bool,
}

/// JSON output for init command
#[derive(Serialize)]
struct InitOutput {
    success: bool,
    specs_dir: PathBuf,
    chakravarti_dir: PathBuf,
    config_file: PathBuf,
    secrets_dir: PathBuf,
    message: String,
    already_initialized: bool,
}

/// Template for .env.example
const ENV_EXAMPLE_TEMPLATE: &str = r#"# Chakravarti API Keys
# Copy this file to .env and fill in your keys.
# This file (.env) is git-ignored for security.

# OpenAI API Key (for GPT models)
# Get your key at: https://platform.openai.com/api-keys
# OPENAI_API_KEY=sk-...

# Anthropic API Key (for Claude models)
# Get your key at: https://console.anthropic.com/
# ANTHROPIC_API_KEY=sk-ant-...

# Custom model endpoint (optional)
# CKRV_MODEL_ENDPOINT=https://api.example.com/v1/chat/completions
# CKRV_MODEL_API_KEY=...
"#;

/// Default SWE workflow
const DEFAULT_SWE_WORKFLOW: &str = r#"# Software Engineering Workflow
# Defines a Plan -> Implement cycle for code modifications.

version: '1.0'
name: 'swe'
description: 'Software Engineering workflow: Plan, then Implement'

defaults:
  tool: claude

steps:
  - id: plan
    name: 'Planning'
    type: agent
    prompt: |
      You are a software engineer creating an implementation plan.

      Task: {{inputs.description}}

      Create a detailed plan that includes:
      1. Understanding of the task
      2. Files to modify or create
      3. Step-by-step implementation approach
      4. Testing strategy

      Then implement all changes as described in the plan.
      Follow best practices for the codebase.
      Write clean, documented code.
    outputs:
      - name: summary
        type: string
        description: "Implementation summary"

  - id: implement
    name: 'Implementation'
    type: agent
    prompt: |
      You are a software engineer implementing code changes.

      Original Task: {{inputs.description}}

      Implement all changes needed for this task.
      Follow best practices for the codebase.
      Write clean, documented code.
    outputs:
      - name: summary
        type: string
        description: "Implementation summary"
"#;

use crate::ui::UiContext;

/// Execute the init command
pub async fn execute(args: InitArgs, json: bool, ui: &UiContext) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;

    // Check if in a git repository
    if !ckrv_git::is_git_repo(&cwd)? {
        if json {
            let output = serde_json::json!({
                "success": false,
                "error": "Not a git repository",
                "code": "NOT_GIT_REPO"
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            ui.error(
                "Initialization Failed",
                "Current directory is not a git repository.\nRun `git init` first, then try again.",
            );
        }
        std::process::exit(1);
    }

    // Get repo root
    let repo_root = ckrv_git::repo_root(&cwd)?;

    // Check if already initialized
    let already_initialized = ckrv_git::is_initialized(&repo_root);

    if already_initialized && !args.force {
        let specs_dir = repo_root.join(".specs");
        let chakravarti_dir = repo_root.join(".chakravarti");
        let config_file = chakravarti_dir.join("config.json");
        let secrets_dir = chakravarti_dir.join("secrets");

        if json {
            let output = InitOutput {
                success: true,
                specs_dir,
                chakravarti_dir,
                config_file,
                secrets_dir,
                message: "Already initialized".to_string(),
                already_initialized: true,
            };
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            ui.success(
                "Already Initialized",
                "Chakravarti is already set up in this repository.\nUse `--force` to reinitialize.",
            );
        }
        return Ok(());
    }

    // Create directories
    let specs_dir = repo_root.join(".specs");
    let chakravarti_dir = repo_root.join(".chakravarti");
    let ckrv_dir = repo_root.join(".ckrv");
    let runs_dir = chakravarti_dir.join("runs");
    let secrets_dir = chakravarti_dir.join("secrets");
    let workflows_dir = ckrv_dir.join("workflows");
    let config_file = chakravarti_dir.join("config.json");

    std::fs::create_dir_all(&specs_dir)?;
    std::fs::create_dir_all(&runs_dir)?;
    std::fs::create_dir_all(&secrets_dir)?;
    std::fs::create_dir_all(&workflows_dir)?;

    // Create default config
    let config = ckrv_core::Config::default();
    config.save(&config_file)?;

    // Create secrets files
    let gitkeep_file = secrets_dir.join(".gitkeep");
    let env_example_file = secrets_dir.join(".env.example");

    // Create .gitkeep (empty file to track directory)
    std::fs::write(&gitkeep_file, "")?;

    // Create .env.example template
    std::fs::write(&env_example_file, ENV_EXAMPLE_TEMPLATE)?;

    // Create default swe workflow
    let swe_workflow_file = workflows_dir.join("swe.yml");
    if !swe_workflow_file.exists() || args.force {
        std::fs::write(&swe_workflow_file, DEFAULT_SWE_WORKFLOW)?;
    }

    // Update .gitignore to ignore secrets (but not .gitkeep and .env.example)
    update_gitignore(&repo_root)?;

    // Create initial commit if repo has no commits yet (fixes HEAD resolution issues)
    create_initial_commit_if_needed(&repo_root)?;

    if json {
        let output = InitOutput {
            success: true,
            specs_dir: specs_dir.clone(),
            chakravarti_dir: chakravarti_dir.clone(),
            config_file: config_file.clone(),
            secrets_dir: secrets_dir.clone(),
            message: if already_initialized {
                "Reinitialized".to_string()
            } else {
                "Initialized".to_string()
            },
            already_initialized,
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        ui.success(
            "Chakravarti Initialized",
            &format!("Ready in {}", repo_root.display()),
        );

        // Relative paths make output cleaner if inside
        let make_relative = |p: &PathBuf| {
            p.strip_prefix(&repo_root)
                .unwrap_or(p)
                .display()
                .to_string()
        };

        let mut content = String::new();
        content.push_str(&format!("* **Specs**: `{}`\n", make_relative(&specs_dir)));
        content.push_str(&format!(
            "* **Data**: `{}`\n",
            make_relative(&chakravarti_dir)
        ));
        content.push_str(&format!(
            "* **Config**: `{}`\n",
            make_relative(&config_file)
        ));
        ui.markdown(&content);

        let mut help = String::from("### API Key Setup\n");
        help.push_str(&format!(
            "1. Copy template: `cp {} {}`\n",
            make_relative(&env_example_file),
            make_relative(&secrets_dir.join(".env"))
        ));
        help.push_str(&format!(
            "2. Add keys to `{}`\n",
            make_relative(&secrets_dir.join(".env"))
        ));
        help.push_str("\n### Next Steps\n");
        help.push_str("1. Create a spec: `ckrv spec new <name>`\n");
        help.push_str("2. Run the spec:  `ckrv run .specs/<name>.yaml`\n");

        ui.markdown(&help);
    }

    Ok(())
}

/// Update .gitignore to ignore secrets but not structure files
fn update_gitignore(repo_root: &std::path::Path) -> anyhow::Result<()> {
    let gitignore_path = repo_root.join(".gitignore");
    let secrets_pattern = ".chakravarti/secrets/.env";

    // Read existing or start fresh
    let mut content = if gitignore_path.exists() {
        std::fs::read_to_string(&gitignore_path)?
    } else {
        String::new()
    };

    // Check if pattern already exists
    if !content.contains(secrets_pattern) {
        if !content.is_empty() && !content.ends_with('\n') {
            content.push('\n');
        }
        content.push_str("\n# Chakravarti secrets (API keys)\n");
        content.push_str(".chakravarti/secrets/.env\n");
        std::fs::write(&gitignore_path, content)?;
    }

    Ok(())
}

/// Create initial commit if the repository has no commits yet.
/// This prevents "HEAD unknown" errors when running spec tasks.
fn create_initial_commit_if_needed(repo_root: &std::path::Path) -> anyhow::Result<()> {
    use std::process::Command;

    // Check if HEAD exists (i.e., there's at least one commit)
    let head_check = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo_root)
        .output();

    let has_commits = head_check
        .map(|output| output.status.success())
        .unwrap_or(false);

    if has_commits {
        // Already has commits, nothing to do
        return Ok(());
    }

    // Create README.md with a friendly message
    let readme_path = repo_root.join("README.md");
    if !readme_path.exists() {
        let readme_content = r#"# Project

Made with ❤️ using [CKRV](https://github.com/FnSK4R17s/chakravarti-cli) - AI-powered spec-driven development.

## Getting Started

```bash
# Create a new specification
ckrv spec new "your feature description"

# Generate implementation tasks
ckrv spec tasks

# Run tasks with AI
ckrv run
```
"#;
        std::fs::write(&readme_path, readme_content)?;
    }

    // Configure git user if not set (for fresh repos)
    let _ = Command::new("git")
        .args(["config", "user.email"])
        .current_dir(repo_root)
        .output()
        .and_then(|output| {
            if output.stdout.is_empty() {
                Command::new("git")
                    .args(["config", "user.email", "ckrv@local"])
                    .current_dir(repo_root)
                    .output()
            } else {
                Ok(output)
            }
        });

    let _ = Command::new("git")
        .args(["config", "user.name"])
        .current_dir(repo_root)
        .output()
        .and_then(|output| {
            if output.stdout.is_empty() {
                Command::new("git")
                    .args(["config", "user.name", "CKRV"])
                    .current_dir(repo_root)
                    .output()
            } else {
                Ok(output)
            }
        });

    // Stage all files
    let _ = Command::new("git")
        .args(["add", "-A"])
        .current_dir(repo_root)
        .output();

    // Create initial commit
    let _ = Command::new("git")
        .args(["commit", "-m", "🚀 Initialize project with CKRV"])
        .current_dir(repo_root)
        .output();

    Ok(())
}
