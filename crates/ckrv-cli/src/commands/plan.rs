//! Plan command - generate execution plan using Claude Code in Docker.
//!
//! This command analyzes tasks.yaml and creates plan.yaml
//! using Claude Code running inside a Docker container.

use std::path::PathBuf;

use clap::Args;
use anyhow::Context;

use ckrv_sandbox::{DockerSandbox, ExecuteConfig, Sandbox};

use crate::ui::UiContext;
use crate::ui::Renderable;
use crate::ui::components::Banner;

/// Arguments for the plan command.
#[derive(Args)]
pub struct PlanArgs {
    /// Path to the specification directory. If not provided, will detect from branch name.
    #[arg()]
    pub spec: Option<PathBuf>,

    /// Force regeneration even if plan.yaml already exists.
    #[arg(long, short)]
    pub force: bool,
}

/// Execute the plan command.
pub async fn execute(args: PlanArgs, json: bool, ui: &UiContext) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    
    // Determine spec directory
    let spec_dir = if let Some(ref spec) = args.spec {
        if spec.is_absolute() {
            spec.clone()
        } else {
            cwd.join(spec)
        }
    } else {
        // Auto-detect from branch name
        let branch_output = std::process::Command::new("git")
            .args(["branch", "--show-current"])
            .current_dir(&cwd)
            .output()
            .context("Failed to get current branch")?;
        
        let branch_name = String::from_utf8_lossy(&branch_output.stdout).trim().to_string();
        
        if branch_name.is_empty() {
            anyhow::bail!("No spec provided and could not detect branch name.");
        }
        
        let specs_dir = cwd.join(".specs");
        let spec_dir = specs_dir.join(&branch_name);
        
        if !spec_dir.exists() {
            anyhow::bail!(
                "No spec found at .specs/{}/\nEither provide a spec path or checkout a branch matching a spec directory.",
                branch_name
            );
        }
        
        if !json {
            println!("Auto-detected spec from branch '{}': {}", branch_name, spec_dir.display());
        }
        
        spec_dir
    };

    let tasks_path = spec_dir.join("tasks.yaml");
    let plan_path = spec_dir.join("plan.yaml");
    let spec_path = spec_dir.join("spec.yaml");

    if !tasks_path.exists() {
        anyhow::bail!("No tasks.yaml found at {}\nRun `ckrv spec tasks` first.", tasks_path.display());
    }

    if !json {
        println!("{}", Banner::new("CKRV PLAN").subtitle(spec_dir.display().to_string()).render(&ui.theme));
        println!();
    }

    // Check if plan already exists
    if plan_path.exists() && !args.force {
        if !json {
            println!("⚠️  Plan already exists at {}", plan_path.display());
            println!("   Use --force to regenerate, or proceed to `ckrv run`.");
        }
        return Ok(());
    }

    // Read tasks
    let tasks_content = std::fs::read_to_string(&tasks_path)?;
    
    // Read spec for context
    let spec_content = if spec_path.exists() {
        std::fs::read_to_string(&spec_path).unwrap_or_default()
    } else {
        String::new()
    };

    // Build the planning prompt
    let prompt = build_planning_prompt(&tasks_content, &spec_content);

    if !json {
        println!("🐳 Starting planning in Docker container...");
    }

    // Execute planning in Docker (mounts ~/.claude for auth)
    execute_planning_docker(&spec_dir, &prompt, json).await?;

    if !json && plan_path.exists() {
        println!("\n✅ Plan generated successfully!");
        println!("   📄 {}", plan_path.display());
        println!("\nNext step: Run `ckrv run` to execute the plan.");
    }

    Ok(())
}

/// Build the planning prompt from tasks and spec
fn build_planning_prompt(tasks_yaml: &str, spec_yaml: &str) -> String {
    format!(r#"You are an expert software architect. Analyze these development tasks and create an execution plan.

## CONTEXT
{spec_yaml}

## TASKS TO PLAN
{tasks_yaml}

## INSTRUCTIONS
1. Group related tasks into logical batches
2. Determine dependencies between batches
3. Assign the best AI model for each batch based on complexity:
   - 'minimax/minimax-m2.1' for simple/standard tasks
   - 'z-ai/glm-4.7' for medium complexity
   - 'claude' for high complexity or security-critical tasks

## OUTPUT FORMAT
Create a file called `plan.yaml` with this structure:

```yaml
batches:
  - id: "setup"
    name: "Project Setup"
    task_ids:
      - "T001"
      - "T002"
    depends_on: []
    model_assignment:
      default: "minimax/minimax-m2.1"
      overrides: {{}}
    execution_strategy: "parallel"
    estimated_time: "2m"
    reasoning: "Initial project scaffolding tasks."
  - id: "core"
    name: "Core Implementation"
    task_ids:
      - "T003"
    depends_on:
      - "setup"
    model_assignment:
      default: "z-ai/glm-4.7"
      overrides: {{}}
    execution_strategy: "sequential"
    estimated_time: "5m"
    reasoning: "Depends on setup for project structure."
```

IMPORTANT: Save your output as `plan.yaml` in the current directory.
"#, spec_yaml = spec_yaml, tasks_yaml = tasks_yaml)
}

/// Execute planning using Docker sandbox with Claude Code
async fn execute_planning_docker(spec_dir: &PathBuf, prompt: &str, json: bool) -> anyhow::Result<()> {
    // Create Docker sandbox
    let sandbox = DockerSandbox::with_defaults()
        .context("Docker is required but not available. Please install and start Docker.")?;
    
    // Health check
    sandbox.health_check().await
        .context("Docker daemon not responding. Ensure Docker is running.")?;

    if !json {
        println!("   Docker connected ✓");
    }

    // Check for Claude credentials
    let home = std::env::var("HOME").unwrap_or_default();
    let claude_dir = format!("{}/.claude", home);
    let claude_json = format!("{}/.claude.json", home);
    
    if !std::path::Path::new(&claude_dir).exists() && !std::path::Path::new(&claude_json).exists() {
        anyhow::bail!(
            "Claude Code credentials not found.\n\
            Please run 'claude' once locally to authenticate, then try again.\n\
            The credentials are stored in ~/.claude/ and ~/.claude.json"
        );
    }

    if !json {
        println!("   Claude credentials found ✓");
    }

    // Escape the prompt for shell
    let escaped_prompt = prompt.replace("'", "'\\''");
    
    // Build Claude command - use --print for non-interactive mode
    // --dangerously-skip-permissions allows file writes without prompting
    let claude_cmd = format!(
        "claude --print --dangerously-skip-permissions '{}'",
        escaped_prompt
    );

    // Configure execution - Docker sandbox will mount ~/.claude automatically
    let config = ExecuteConfig::new("claude", spec_dir.clone())
        .shell(&claude_cmd)
        .with_timeout(std::time::Duration::from_secs(300))
        .env("NO_COLOR", "1");

    if !json {
        println!("   Executing Claude Code in sandbox...");
    }

    // Execute
    let result = sandbox.execute(config).await
        .context("Planning execution failed")?;

    // Log output
    if !json {
        for line in result.stdout.lines() {
            println!("   {}", line);
        }
        if !result.stderr.is_empty() {
            for line in result.stderr.lines() {
                eprintln!("   ⚠️  {}", line);
            }
        }
    }

    if !result.success() {
        anyhow::bail!("Planning failed with exit code {}", result.exit_code);
    }

    Ok(())
}
