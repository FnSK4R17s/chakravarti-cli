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

/// Execute the init command
pub fn execute(args: InitArgs, json: bool) -> anyhow::Result<()> {
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
            eprintln!("Error: Not a git repository. Chakravarti requires a git repository.");
            eprintln!("Run `git init` first, then try again.");
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
            println!("Already initialized. Use --force to reinitialize.");
        }
        return Ok(());
    }

    // Create directories
    let specs_dir = repo_root.join(".specs");
    let chakravarti_dir = repo_root.join(".chakravarti");
    let runs_dir = chakravarti_dir.join("runs");
    let secrets_dir = chakravarti_dir.join("secrets");
    let config_file = chakravarti_dir.join("config.json");

    std::fs::create_dir_all(&specs_dir)?;
    std::fs::create_dir_all(&runs_dir)?;
    std::fs::create_dir_all(&secrets_dir)?;

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

    // Update .gitignore to ignore secrets (but not .gitkeep and .env.example)
    update_gitignore(&repo_root)?;

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
        println!("Initialized Chakravarti in {}", repo_root.display());
        println!("  Created: {}", specs_dir.display());
        println!("  Created: {}", chakravarti_dir.display());
        println!("  Created: {}", secrets_dir.display());
        println!("  Created: {}", config_file.display());
        println!();
        println!("API Key Setup:");
        println!(
            "  1. Copy: cp {} {}",
            env_example_file.display(),
            secrets_dir.join(".env").display()
        );
        println!(
            "  2. Edit {} and add your API keys",
            secrets_dir.join(".env").display()
        );
        println!();
        println!("Next steps:");
        println!("  1. Create a spec: ckrv spec new <name>");
        println!("  2. Run the spec:  ckrv run .specs/<name>.yaml");
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
