//! Fix command - use AI to fix verification errors.

use std::path::PathBuf;

use clap::Args;
use serde::{Deserialize, Serialize};

use crate::ui::UiContext;
use crate::ui::Renderable;
use crate::ui::components::Banner;

/// Arguments for the fix command
#[derive(Args)]
pub struct FixArgs {
    /// Fix only lint errors
    #[arg(long)]
    pub lint: bool,

    /// Fix only type errors
    #[arg(long, name = "type")]
    pub typecheck: bool,

    /// Fix only test failures
    #[arg(long)]
    pub test: bool,

    /// Re-run verification after fixing
    #[arg(long)]
    pub check: bool,

    /// Specific error message to fix (from UI)
    #[arg(long)]
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct FixOutput {
    pub success: bool,
    pub fixes_applied: usize,
    pub errors_remaining: usize,
    pub message: String,
}

#[derive(Deserialize)]
struct VerifyOutput {
    success: bool,
    checks: Vec<CheckResult>,
}

#[derive(Deserialize)]
struct CheckResult {
    name: String,
    passed: bool,
    error: Option<String>,
    output: Option<String>,
}

/// Execute the fix command
pub async fn execute(args: FixArgs, json: bool, ui: &UiContext) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;

    if !json {
        println!("{}", Banner::new("CKRV FIX").subtitle("AI-Powered Fixes").render(&ui.theme));
    }

    // Gather errors to fix
    let errors = if let Some(ref error_msg) = args.error {
        // Direct error message from UI
        vec![error_msg.clone()]
    } else {
        // Try to read from verification.yaml
        gather_verification_errors(&cwd, &args)?
    };

    if errors.is_empty() {
        if json {
            let output = FixOutput {
                success: true,
                fixes_applied: 0,
                errors_remaining: 0,
                message: "No errors to fix".to_string(),
            };
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            println!("✅ No errors to fix!");
            println!("   Run 'ckrv verify' to check for issues.");
        }
        return Ok(());
    }

    if !json {
        println!("🔍 Found {} error(s) to fix\n", errors.len());
        for (i, error) in errors.iter().enumerate() {
            println!("   {}. {}", i + 1, error.lines().next().unwrap_or("Unknown error"));
        }
        println!();
    }

    // Build prompt for Claude
    let prompt = build_fix_prompt(&cwd, &errors)?;

    if !json {
        println!("🤖 Running Claude Code to fix issues...\n");
    }

    // Run Claude Code
    let fix_result = run_claude_fix(&cwd, &prompt, json).await?;

    if json {
        let output = FixOutput {
            success: fix_result.success,
            fixes_applied: fix_result.fixes_applied,
            errors_remaining: if fix_result.success { 0 } else { errors.len() },
            message: fix_result.message,
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        if fix_result.success {
            println!("\n✅ Fixes applied successfully!");
            if fix_result.fixes_applied > 0 {
                println!("   {} fix(es) applied", fix_result.fixes_applied);
            }
        } else {
            println!("\n⚠️  Some issues could not be automatically fixed");
            println!("   {}", fix_result.message);
        }

        // Re-run verification if requested
        if args.check {
            println!("\n🔄 Re-running verification...\n");
            let verify_result = std::process::Command::new("ckrv")
                .args(["verify"])
                .current_dir(&cwd)
                .status();
            
            if let Ok(status) = verify_result {
                if !status.success() {
                    println!("\n💡 Some checks still failing. You may need to fix manually or run 'ckrv fix' again.");
                }
            }
        } else {
            println!("\n💡 Run 'ckrv verify' to check if issues are resolved");
            println!("   Or 'ckrv fix --check' to fix and verify in one step");
        }
    }

    Ok(())
}

fn gather_verification_errors(cwd: &PathBuf, args: &FixArgs) -> anyhow::Result<Vec<String>> {
    let mut errors = Vec::new();

    // Try to read verification.yaml from current spec directory
    let branch_output = std::process::Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(cwd)
        .output()?;
    
    let branch = String::from_utf8_lossy(&branch_output.stdout).trim().to_string();
    let verification_path = cwd.join(".specs").join(&branch).join("verification.yaml");

    if verification_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&verification_path) {
            if let Ok(verify_output) = serde_yaml::from_str::<VerifyOutput>(&content) {
                for check in verify_output.checks {
                    if !check.passed {
                        // Filter by type if specified
                        let include = if args.lint {
                            check.name.to_lowercase().contains("lint")
                        } else if args.typecheck {
                            check.name.to_lowercase().contains("type")
                        } else if args.test {
                            check.name.to_lowercase().contains("test")
                        } else {
                            true // Include all
                        };

                        if include {
                            let error_msg = format!(
                                "{}: {}",
                                check.name,
                                check.error.unwrap_or_else(|| "Unknown error".to_string())
                            );
                            errors.push(error_msg);
                        }
                    }
                }
            }
        }
    }

    // If no verification.yaml, run verify to get current errors
    if errors.is_empty() {
        let output = std::process::Command::new("ckrv")
            .args(["verify", "--json", "--continue-on-failure"])
            .current_dir(cwd)
            .output()?;

        if let Ok(verify_output) = serde_json::from_slice::<VerifyOutput>(&output.stdout) {
            for check in verify_output.checks {
                if !check.passed {
                    let include = if args.lint {
                        check.name.to_lowercase().contains("lint")
                    } else if args.typecheck {
                        check.name.to_lowercase().contains("type")
                    } else if args.test {
                        check.name.to_lowercase().contains("test")
                    } else {
                        true
                    };

                    if include {
                        let error_msg = format!(
                            "{}: {}",
                            check.name,
                            check.error.unwrap_or_else(|| "Unknown error".to_string())
                        );
                        errors.push(error_msg);
                    }
                }
            }
        }
    }

    Ok(errors)
}

fn build_fix_prompt(cwd: &PathBuf, errors: &[String]) -> anyhow::Result<String> {
    let mut prompt = String::new();
    
    prompt.push_str("I need you to fix the following verification errors in this project:\n\n");
    
    for (i, error) in errors.iter().enumerate() {
        prompt.push_str(&format!("Error {}:\n```\n{}\n```\n\n", i + 1, error));
    }

    prompt.push_str("Please analyze these errors and fix them. Common fixes include:\n");
    prompt.push_str("- For lint errors: Fix code style issues, add missing imports, etc.\n");
    prompt.push_str("- For type errors: Add type annotations, fix type mismatches\n");
    prompt.push_str("- For test failures: Fix the failing tests or the code they're testing\n");
    prompt.push_str("- For missing tools: Add them to requirements.txt/package.json or suggest alternatives\n\n");
    
    // Add project context
    prompt.push_str("Project context:\n");
    
    // Check for common project files
    if cwd.join("pyproject.toml").exists() || cwd.join("requirements.txt").exists() {
        prompt.push_str("- This is a Python project\n");
    }
    if cwd.join("package.json").exists() {
        prompt.push_str("- This is a JavaScript/TypeScript project\n");
    }
    if cwd.join("Cargo.toml").exists() {
        prompt.push_str("- This is a Rust project\n");
    }

    prompt.push_str("\nFix the issues by editing the necessary files.");

    Ok(prompt)
}

struct FixResult {
    success: bool,
    fixes_applied: usize,
    message: String,
}

async fn run_claude_fix(cwd: &PathBuf, prompt: &str, json: bool) -> anyhow::Result<FixResult> {
    // Check if claude CLI is available
    let claude_check = std::process::Command::new("claude")
        .arg("--version")
        .output();

    if claude_check.is_err() || !claude_check.unwrap().status.success() {
        return Ok(FixResult {
            success: false,
            fixes_applied: 0,
            message: "Claude CLI not found. Install from: https://docs.anthropic.com/claude-code".to_string(),
        });
    }

    // Run Claude to analyze and fix issues
    // Use -p for prompt and --dangerously-skip-permissions to allow file edits
    let output = std::process::Command::new("claude")
        .args(["-p", prompt, "--dangerously-skip-permissions"])
        .current_dir(cwd)
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !json {
        // Print Claude's response
        for line in stdout.lines() {
            println!("   {}", line);
        }
    }

    if output.status.success() {
        // Count how many files were modified (heuristic based on output)
        let fixes_applied = stdout.matches("Updated").count()
            + stdout.matches("Modified").count()
            + stdout.matches("Created").count()
            + stdout.matches("Fixed").count();

        Ok(FixResult {
            success: true,
            fixes_applied: fixes_applied.max(1), // At least 1 if successful
            message: "Fixes applied".to_string(),
        })
    } else {
        Ok(FixResult {
            success: false,
            fixes_applied: 0,
            message: stderr.lines().next().unwrap_or("Unknown error").to_string(),
        })
    }
}

