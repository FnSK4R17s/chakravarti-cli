//! Spec commands - create and manage specifications.

use std::path::PathBuf;

use clap::{Args, Subcommand};
use serde::Serialize;

/// Arguments for the spec command
#[derive(Args)]
pub struct SpecArgs {
    #[command(subcommand)]
    pub command: SpecCommand,
}

/// Spec subcommands
#[derive(Subcommand)]
pub enum SpecCommand {
    /// Create a new specification
    New {
        /// Unique identifier for the spec (alphanumeric with underscores)
        name: String,

        /// Goal statement for the spec
        #[arg(long)]
        goal: Option<String>,
    },
    /// Validate a specification file
    Validate {
        /// Path to the spec file
        path: PathBuf,
    },
    /// List all specifications
    List,
}

/// JSON output for spec new command
#[derive(Serialize)]
struct SpecNewOutput {
    success: bool,
    spec_path: PathBuf,
    id: String,
    message: String,
}

/// JSON output for spec validate command
#[derive(Serialize)]
struct SpecValidateOutput {
    valid: bool,
    errors: Vec<ValidationErrorOutput>,
    warnings: Vec<String>,
}

#[derive(Serialize)]
struct ValidationErrorOutput {
    field: String,
    message: String,
}

/// Execute the spec command
pub fn execute(args: SpecArgs, json: bool) -> anyhow::Result<()> {
    match args.command {
        SpecCommand::New { name, goal } => execute_new(&name, goal.as_deref(), json),
        SpecCommand::Validate { path } => execute_validate(&path, json),
        SpecCommand::List => execute_list(json),
    }
}

fn execute_new(name: &str, goal: Option<&str>, json: bool) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;

    // Check if initialized
    if !ckrv_git::is_initialized(&cwd) {
        if json {
            let output = serde_json::json!({
                "success": false,
                "error": "Not initialized. Run 'ckrv init' first.",
                "code": "NOT_INITIALIZED"
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            eprintln!("Error: Not initialized. Run 'ckrv init' first.");
        }
        std::process::exit(1);
    }

    // Validate name format
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        if json {
            let output = serde_json::json!({
                "success": false,
                "error": "Invalid name. Must be alphanumeric with underscores only.",
                "code": "INVALID_NAME"
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            eprintln!("Error: Invalid name. Must be alphanumeric with underscores only.");
        }
        std::process::exit(1);
    }

    // Create spec file
    let specs_dir = cwd.join(".specs");
    let spec_path = specs_dir.join(format!("{name}.yaml"));

    if spec_path.exists() {
        if json {
            let output = serde_json::json!({
                "success": false,
                "error": format!("Spec '{}' already exists", name),
                "code": "ALREADY_EXISTS"
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            eprintln!(
                "Error: Spec '{}' already exists at {}",
                name,
                spec_path.display()
            );
        }
        std::process::exit(1);
    }

    // Generate spec content
    let content = ckrv_spec::template::generate_spec_content(name, goal);
    std::fs::write(&spec_path, &content)?;

    if json {
        let output = SpecNewOutput {
            success: true,
            spec_path: spec_path.clone(),
            id: name.to_string(),
            message: "Spec created".to_string(),
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("Created spec: {}", spec_path.display());
        println!();
        println!("Next steps:");
        println!("  1. Edit the spec: {}", spec_path.display());
        println!(
            "  2. Validate:      ckrv spec validate {}",
            spec_path.display()
        );
        println!("  3. Run:           ckrv run {}", spec_path.display());
    }

    Ok(())
}

fn execute_validate(path: &PathBuf, json: bool) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let full_path = if path.is_absolute() {
        path.clone()
    } else {
        cwd.join(path)
    };

    // Load spec
    let loader = ckrv_spec::loader::YamlSpecLoader;
    let spec = match ckrv_spec::loader::SpecLoader::load(&loader, &full_path) {
        Ok(s) => s,
        Err(e) => {
            if json {
                let output = serde_json::json!({
                    "valid": false,
                    "errors": [{
                        "field": "file",
                        "message": e.to_string()
                    }],
                    "warnings": []
                });
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                eprintln!("Error: Failed to load spec: {e}");
            }
            std::process::exit(1);
        }
    };

    // Validate
    let result = ckrv_spec::validator::validate(&spec);

    if json {
        let output = SpecValidateOutput {
            valid: result.valid,
            errors: result
                .errors
                .iter()
                .map(|e| ValidationErrorOutput {
                    field: e.field.clone(),
                    message: e.message.clone(),
                })
                .collect(),
            warnings: result.warnings,
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else if result.valid {
        println!("✓ Spec is valid: {}", path.display());
        for warning in &result.warnings {
            println!("  ⚠ {warning}");
        }
    } else {
        eprintln!("✗ Spec validation failed: {}", path.display());
        for error in &result.errors {
            eprintln!("  • {}: {}", error.field, error.message);
        }
        for warning in &result.warnings {
            eprintln!("  ⚠ {warning}");
        }
    }

    if !result.valid {
        std::process::exit(1);
    }

    Ok(())
}

fn execute_list(json: bool) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let specs_dir = cwd.join(".specs");

    if !specs_dir.exists() {
        if json {
            let output = serde_json::json!({
                "specs": [],
                "count": 0
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            println!("No specs found. Run 'ckrv spec new <name>' to create one.");
        }
        return Ok(());
    }

    let loader = ckrv_spec::loader::YamlSpecLoader;
    let spec_files = ckrv_spec::loader::SpecLoader::list(&loader, &specs_dir)?;

    if json {
        let specs: Vec<_> = spec_files
            .iter()
            .map(|p| {
                serde_json::json!({
                    "path": p,
                    "name": p.file_stem().map(|s| s.to_string_lossy().to_string())
                })
            })
            .collect();
        let output = serde_json::json!({
            "specs": specs,
            "count": spec_files.len()
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else if spec_files.is_empty() {
        println!("No specs found. Run 'ckrv spec new <name>' to create one.");
    } else {
        println!("Specifications:");
        for path in &spec_files {
            if let Some(name) = path.file_stem() {
                println!("  • {} ({})", name.to_string_lossy(), path.display());
            }
        }
    }

    Ok(())
}
