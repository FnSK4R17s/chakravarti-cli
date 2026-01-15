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
    /// Create a new specification using AI from a natural language description
    New {
        /// Natural language description of the feature (e.g., "Add user authentication")
        description: String,

        /// Optional short name for the spec (auto-generated from description if not provided)
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Resolve clarifications in an existing spec
    Clarify {
        /// Path to the spec file (optional - auto-detects from current branch if not provided)
        spec: Option<PathBuf>,
    },
    /// Generate technical design document from a specification
    Design {
        /// Path to the spec file (optional - auto-detects from current branch if not provided)
        spec: Option<PathBuf>,

        /// Force regeneration of design even if it exists
        #[arg(short, long)]
        force: bool,
    },
    /// Initialize an empty spec directory with templates
    Init {
        /// Name for the new spec directory
        name: String,
    },
    /// Generate implementation tasks from a specification
    Tasks {
        /// Path to the spec file (optional - auto-detects from current branch if not provided)
        spec: Option<PathBuf>,

        /// Force regeneration of tasks even if they exist
        #[arg(short, long)]
        force: bool,
    },
    /// Validate a specification file
    Validate {
        /// Path to the spec file (optional - auto-detects from current branch if not provided)
        path: Option<PathBuf>,
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

use crate::ui::{UiContext, Renderable};
use crate::ui::components::{RichTable, Banner};

/// Execute the spec command
pub async fn execute(args: SpecArgs, json: bool, ui: &UiContext) -> anyhow::Result<()> {
    match args.command {
        SpecCommand::New { description, name } => {
            execute_generate(&description, name.as_deref(), json).await
        }
        SpecCommand::Clarify { spec } => execute_clarify(spec.as_ref(), json).await,
        SpecCommand::Design { spec, force } => execute_design(spec.as_ref(), force, json).await,
        SpecCommand::Init { name } => execute_init(&name, json),
        SpecCommand::Tasks { spec, force } => execute_tasks(spec.as_ref(), force, json, ui).await,
        SpecCommand::Validate { path } => execute_validate(path.as_ref(), json),
        SpecCommand::List => execute_list(json),
    }
}

/// Create a new spec using Claude AI from a natural language description.
async fn execute_generate(description: &str, name: Option<&str>, json: bool) -> anyhow::Result<()> {
    use ckrv_sandbox::{DockerSandbox, ExecuteConfig, Sandbox};
    use std::time::Duration;

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

    // Generate short name from description if not provided
    let short_name = name.map(String::from).unwrap_or_else(|| {
        generate_short_name(description)
    });

    // Validate name format
    if !short_name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        if json {
            let output = serde_json::json!({
                "success": false,
                "error": "Invalid name. Must be alphanumeric with underscores or hyphens only.",
                "code": "INVALID_NAME"
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            eprintln!("Error: Invalid name. Must be alphanumeric with underscores or hyphens only.");
        }
        std::process::exit(1);
    }

    let specs_dir = cwd.join(".specs");
    
    // Check if spec with this name already exists (before creating folder)
    if let Some(existing) = find_spec_by_name(&specs_dir, &short_name)? {
        if json {
            let output = serde_json::json!({
                "success": false,
                "error": format!("Spec '{}' already exists at {}", short_name, existing.display()),
                "code": "ALREADY_EXISTS"
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            eprintln!("Error: Spec '{}' already exists at {}", short_name, existing.display());
        }
        std::process::exit(1);
    }

    // Find next available number
    let next_number = get_next_spec_number(&specs_dir)?;
    let numbered_name = format!("{:03}-{}", next_number, short_name);
    
    // Create spec folder and spec.yaml inside it
    let spec_folder = specs_dir.join(&numbered_name);
    std::fs::create_dir_all(&spec_folder)?;
    let spec_path = spec_folder.join("spec.yaml");

    if !json {
        eprintln!("Generating specification from: \"{}\"", description);
        eprintln!("Spec ID: {}", numbered_name);
        eprintln!();
    }

    // Build the rich prompt for Claude using the prompts module
    let prompt = crate::prompts::build_spec_prompt(description, &numbered_name);

    // Run Claude in Docker sandbox
    let result = {
        let sandbox = DockerSandbox::new(ckrv_sandbox::DefaultAllowList::default())
            .map_err(|e| anyhow::anyhow!("Failed to create sandbox: {}", e))?;

        let command = format!(
            "claude -p {} --dangerously-skip-permissions --output-format text --tools \"\"",
            shell_escape::escape(prompt.clone().into())
        );

        let config = ExecuteConfig::new("", specs_dir.clone())
            .shell(&command)
            .with_timeout(Duration::from_secs(300));

        sandbox.execute(config).await
            .map_err(|e| anyhow::anyhow!("Sandbox execution failed: {}", e))
    }?;

    if !result.success() {
        if json {
            let output = serde_json::json!({
                "success": false,
                "error": format!("AI generation failed: {}", result.stderr),
                "code": "GENERATION_FAILED"
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            eprintln!("Error: AI generation failed");
            eprintln!("{}", result.stderr);
        }
        std::process::exit(1);
    }

    // Write the generated spec (strip any markdown code fences)
    let spec_content = result.stdout.trim();
    let spec_content = crate::prompts::strip_yaml_fences(spec_content);
    
    // Validate the generated YAML before writing
    let parsed: Result<super::spec_structs::SpecOutput, _> = serde_yaml::from_str(&spec_content);
    let (final_content, has_clarifications) = match parsed {
        Ok(spec) => {
            let has_clarifications = spec.has_unresolved_clarifications();
            let user_story_count = spec.user_story_count();
            let requirement_count = spec.requirement_count();
            
            if !json {
                eprintln!("✓ Generated spec with {} user stories, {} requirements", 
                    user_story_count, requirement_count);
                if has_clarifications {
                    eprintln!("⚠ Spec has unresolved clarifications - run 'ckrv spec clarify' to resolve");
                }
            }
            (spec_content, has_clarifications)
        }
        Err(e) => {
            if !json {
                eprintln!("Warning: Generated YAML may have formatting issues: {}", e);
                eprintln!("Writing raw output - manual review recommended");
            }
            (spec_content, false)
        }
    };
    
    std::fs::write(&spec_path, &final_content)?;

    // Create a new git branch with the spec name
    let repo_root = ckrv_git::repo_root(&cwd).unwrap_or_else(|_| cwd.clone());
    let branch_created = match std::process::Command::new("git")
        .args(["checkout", "-b", &numbered_name])
        .current_dir(&repo_root)
        .output()
    {
        Ok(output) if output.status.success() => {
            if !json {
                eprintln!("Created and switched to branch: {}", numbered_name);
            }
            true
        }
        Ok(_) => {
            // Branch might already exist, try to switch to it
            match std::process::Command::new("git")
                .args(["checkout", &numbered_name])
                .current_dir(&repo_root)
                .output()
            {
                Ok(output) if output.status.success() => {
                    if !json {
                        eprintln!("Switched to existing branch: {}", numbered_name);
                    }
                    true
                }
                _ => {
                    if !json {
                        eprintln!("Warning: Could not create/switch to branch '{}', continuing on current branch", numbered_name);
                    }
                    false
                }
            }
        }
        Err(_) => {
            if !json {
                eprintln!("Warning: Git not available, skipping branch creation");
            }
            false
        }
    };

    if json {
        // Try to parse spec for detailed output
        let spec_details = serde_yaml::from_str::<super::spec_structs::SpecOutput>(&final_content).ok();
        let output = serde_json::json!({
            "success": true,
            "spec_folder": spec_folder,
            "spec_path": spec_path,
            "id": numbered_name,
            "branch": if branch_created { Some(&numbered_name) } else { None },
            "message": "Spec generated with AI",
            "spec": {
                "user_story_count": spec_details.as_ref().map(|s| s.user_story_count()).unwrap_or(0),
                "requirement_count": spec_details.as_ref().map(|s| s.requirement_count()).unwrap_or(0),
                "has_clarifications": has_clarifications,
                "status": spec_details.as_ref().map(|s| s.status.to_string()).unwrap_or_else(|| "unknown".to_string()),
            }
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("✓ Generated spec: {}", spec_folder.display());
        if branch_created {
            println!("✓ On branch: {}", numbered_name);
        }
        println!();
        if has_clarifications {
            println!("Next steps:");
            println!("  1. Clarify: ckrv spec clarify");
            println!("  2. Then:    ckrv spec tasks");
        } else {
            println!("Next steps:");
            println!("  1. Review:   cat {}/spec.yaml", spec_folder.display());
            println!("  2. Tasks:    ckrv spec tasks");
        }
    }

    Ok(())
}

/// Resolve clarifications in an existing spec
async fn execute_clarify(spec_path: Option<&PathBuf>, json: bool) -> anyhow::Result<()> {
    use std::io::{self, Write};

    let cwd = std::env::current_dir()?;
    
    // Resolve spec path - auto-detect from current branch if not provided
    let spec_path = resolve_spec_path(spec_path, &cwd)?;
    
    // Read the spec file
    let spec_content = std::fs::read_to_string(&spec_path)?;
    let mut spec: super::spec_structs::SpecOutput = serde_yaml::from_str(&spec_content)
        .map_err(|e| anyhow::anyhow!("Failed to parse spec: {}", e))?;
    
    // Check for unresolved clarifications - collect indices first
    let unresolved_indices: Vec<usize> = spec.clarifications
        .iter()
        .enumerate()
        .filter(|(_, c)| c.resolved.is_none())
        .map(|(idx, _)| idx)
        .collect();
    
    if unresolved_indices.is_empty() {
        if json {
            let output = serde_json::json!({
                "success": true,
                "message": "No clarifications needed",
                "clarifications_resolved": 0
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            println!("✓ No clarifications needed - spec is ready");
            println!();
            println!("Next steps:");
            println!("  1. Design:   ckrv spec design");
            println!("  2. Or Tasks: ckrv spec tasks");
        }
        return Ok(());
    }
    
    if !json {
        println!("Found {} clarification(s) to resolve:\n", unresolved_indices.len());
    }
    
    let mut resolved_count = 0;
    
    for idx in unresolved_indices {
        let clarification = &spec.clarifications[idx];
        if !json {
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("Topic: {}", clarification.topic);
            println!("Question: {}", clarification.question);
            println!();
            
            for (i, option) in clarification.options.iter().enumerate() {
                let label = (b'A' + i as u8) as char;
                println!("  {}) {}", label, option.answer);
                if let Some(impl_) = &option.implications {
                    println!("     → {}", impl_);
                }
            }
            println!();
            print!("Choose an option (A, B, ...): ");
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim().to_uppercase();
            
            if let Some(choice_idx) = input.chars().next().map(|c| (c as u8 - b'A') as usize) {
                if choice_idx < spec.clarifications[idx].options.len() {
                    let answer = spec.clarifications[idx].options[choice_idx].answer.clone();
                    spec.clarifications[idx].resolved = Some(answer.clone());
                    resolved_count += 1;
                    println!("✓ Selected: {}\n", answer);
                } else {
                    println!("⚠ Invalid option, skipping\n");
                }
            } else {
                println!("⚠ Invalid input, skipping\n");
            }
        }
    }
    
    // Update the spec file if any clarifications were resolved
    if resolved_count > 0 {
        // Update status if all clarifications are now resolved
        if !spec.has_unresolved_clarifications() {
            spec.status = super::spec_structs::SpecStatus::Ready;
        }
        
        let updated_yaml = serde_yaml::to_string(&spec)?;
        std::fs::write(&spec_path, &updated_yaml)?;
    }
    
    if json {
        let output = serde_json::json!({
            "success": true,
            "clarifications_resolved": resolved_count,
            "remaining": spec.clarifications.iter().filter(|c| c.resolved.is_none()).count(),
            "status": spec.status.to_string()
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("✓ Resolved {} clarification(s)", resolved_count);
        println!("✓ Updated: {}", spec_path.display());
        
        if spec.has_unresolved_clarifications() {
            println!("\n⚠ Some clarifications still unresolved - run 'ckrv spec clarify' again");
        } else {
            println!("\nNext steps:");
            println!("  1. Design:   ckrv spec design");
            println!("  2. Or Tasks: ckrv spec tasks");
        }
    }
    
    Ok(())
}

/// Generate technical design document from a specification
async fn execute_design(spec_path: Option<&PathBuf>, force: bool, json: bool) -> anyhow::Result<()> {
    use ckrv_sandbox::{DockerSandbox, ExecuteConfig, Sandbox};
    use std::time::Duration;

    let cwd = std::env::current_dir()?;
    
    // Resolve spec path - auto-detect from current branch if not provided
    let spec_path = resolve_spec_path(spec_path, &cwd)?;
    let spec_folder = spec_path.parent().unwrap().to_path_buf();
    
    // Check if design already exists
    let design_path = spec_folder.join("design.md");
    let research_path = spec_folder.join("research.md");
    
    if design_path.exists() && !force {
        if json {
            let output = serde_json::json!({
                "success": false,
                "error": "Design already exists. Use --force to regenerate.",
                "code": "DESIGN_EXISTS"
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            eprintln!("Error: design.md already exists at {}", design_path.display());
            eprintln!("Use --force to regenerate");
        }
        return Ok(());
    }
    
    // Read the spec file
    let spec_content = std::fs::read_to_string(&spec_path)?;
    let spec: super::spec_structs::SpecOutput = serde_yaml::from_str(&spec_content)
        .map_err(|e| anyhow::anyhow!("Failed to parse spec: {}", e))?;
    
    // Check for unresolved clarifications
    if spec.has_unresolved_clarifications() {
        if json {
            let output = serde_json::json!({
                "success": false,
                "error": "Spec has unresolved clarifications. Run 'ckrv spec clarify' first.",
                "code": "NEEDS_CLARIFICATION"
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            eprintln!("Error: Spec has unresolved clarifications");
            eprintln!("Run 'ckrv spec clarify' first");
        }
        return Ok(());
    }
    
    if !json {
        eprintln!("Generating design document for: {}", spec.id);
        eprintln!();
    }
    
    // Build the design prompt
    let prompt = crate::prompts::build_design_prompt(&spec_content, &spec.id);
    
    // Run Claude in Docker sandbox
    let result = {
        let sandbox = DockerSandbox::new(ckrv_sandbox::DefaultAllowList::default())
            .map_err(|e| anyhow::anyhow!("Failed to create sandbox: {}", e))?;

        let command = format!(
            "claude -p {} --dangerously-skip-permissions --output-format text --tools \"\"",
            shell_escape::escape(prompt.clone().into())
        );

        let config = ExecuteConfig::new("", spec_folder.clone())
            .shell(&command)
            .with_timeout(Duration::from_secs(300));

        sandbox.execute(config).await
            .map_err(|e| anyhow::anyhow!("Sandbox execution failed: {}", e))
    }?;

    if !result.success() {
        if json {
            let output = serde_json::json!({
                "success": false,
                "error": format!("AI generation failed: {}", result.stderr),
                "code": "GENERATION_FAILED"
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            eprintln!("Error: AI generation failed");
            eprintln!("{}", result.stderr);
        }
        std::process::exit(1);
    }

    // Write the design document
    let design_content = result.stdout.trim();
    std::fs::write(&design_path, design_content)?;
    
    // Create a basic research.md if it doesn't exist
    if !research_path.exists() {
        let research_content = format!(r#"# Research: {}

**Generated**: {}
**Status**: Auto-generated during design phase

## Technical Decisions

(Extract from design.md or add manually)

## Dependencies

(List external dependencies identified during design)

## Risks & Mitigations

(Document any risks identified during design)
"#, spec.id, chrono::Local::now().format("%Y-%m-%d"));
        std::fs::write(&research_path, research_content)?;
    }
    
    if json {
        let output = serde_json::json!({
            "success": true,
            "design_path": design_path,
            "research_path": research_path,
            "message": "Design generated successfully"
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("✓ Generated: {}", design_path.display());
        println!("✓ Generated: {}", research_path.display());
        println!();
        println!("Next steps:");
        println!("  1. Review:   cat {}", design_path.display());
        println!("  2. Tasks:    ckrv spec tasks");
    }
    
    Ok(())
}

/// Initialize an empty spec directory with templates
fn execute_init(name: &str, json: bool) -> anyhow::Result<()> {
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
        return Ok(());
    }
    
    let specs_dir = cwd.join(".specs");
    
    // Generate a numbered name
    let existing_count = if specs_dir.exists() {
        std::fs::read_dir(&specs_dir)
            .map(|entries| entries.filter_map(|e| e.ok()).count())
            .unwrap_or(0)
    } else {
        0
    };
    
    let numbered_name = format!("{:03}-{}", existing_count + 1, name);
    let spec_folder = specs_dir.join(&numbered_name);
    
    if spec_folder.exists() {
        if json {
            let output = serde_json::json!({
                "success": false,
                "error": format!("Spec folder already exists: {}", spec_folder.display()),
                "code": "FOLDER_EXISTS"
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            eprintln!("Error: Spec folder already exists: {}", spec_folder.display());
        }
        return Ok(());
    }
    
    // Create the directory structure
    std::fs::create_dir_all(&spec_folder)?;
    std::fs::create_dir_all(spec_folder.join("checklists"))?;
    
    // Create empty template files
    let spec_yaml = format!(r#"id: "{}"
branch: "{}"
created: "{}"
status: draft

overview: |
  [Brief description - replace this with your feature description]

user_stories: []

requirements:
  functional: []
  non_functional: []
  security: []

success_criteria: []

edge_cases: []

assumptions: []

clarifications: []
"#, numbered_name, numbered_name, chrono::Local::now().format("%Y-%m-%d"));
    
    std::fs::write(spec_folder.join("spec.yaml"), spec_yaml)?;
    
    // Create empty checklist
    let checklist = r#"# Requirements Checklist

## Specification Quality

- [ ] Overview clearly describes feature purpose
- [ ] At least one user story defined
- [ ] User stories have acceptance criteria
- [ ] Functional requirements are testable
- [ ] Success criteria are measurable
"#;
    std::fs::write(spec_folder.join("checklists").join("requirements.md"), checklist)?;
    
    if json {
        let output = serde_json::json!({
            "success": true,
            "spec_folder": spec_folder,
            "id": numbered_name,
            "message": "Spec folder initialized with templates"
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("✓ Created: {}", spec_folder.display());
        println!("✓ Created: {}/spec.yaml", spec_folder.display());
        println!("✓ Created: {}/checklists/requirements.md", spec_folder.display());
        println!();
        println!("Next steps:");
        println!("  1. Edit:   {} {}/spec.yaml", std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string()), spec_folder.display());
        println!("  2. Or use: ckrv spec new \"description\" --name {}", name);
    }
    
    Ok(())
}

/// Helper function to resolve spec path from current branch if not provided
fn resolve_spec_path(spec_path: Option<&PathBuf>, cwd: &std::path::Path) -> anyhow::Result<PathBuf> {
    match spec_path {
        Some(path) => {
            if path.is_absolute() {
                Ok(path.clone())
            } else {
                Ok(cwd.join(path))
            }
        }
        None => {
            // Get current branch name
            let branch_output = std::process::Command::new("git")
                .args(["rev-parse", "--abbrev-ref", "HEAD"])
                .current_dir(cwd)
                .output();

            match branch_output {
                Ok(output) if output.status.success() => {
                    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    
                    let specs_dir = cwd.join(".specs");
                    let spec_folder = specs_dir.join(&branch);
                    let spec_file = spec_folder.join("spec.yaml");
                    let spec_file_yml = spec_folder.join("spec.yml");
                    
                    if spec_file.exists() {
                        Ok(spec_file)
                    } else if spec_file_yml.exists() {
                        Ok(spec_file_yml)
                    } else {
                        Err(anyhow::anyhow!(
                            "No spec found for branch '{}'. Expected at: {}",
                            branch,
                            spec_file.display()
                        ))
                    }
                }
                _ => Err(anyhow::anyhow!("Failed to detect current branch. Provide spec path explicitly."))
            }
        }
    }
}

/// Generate implementation tasks from a spec file.
/// Auto-detects spec from current branch if not provided.
async fn execute_tasks(spec_path: Option<&PathBuf>, force: bool, json: bool, ui: &UiContext) -> anyhow::Result<()> {
    use ckrv_sandbox::{DockerSandbox, ExecuteConfig, Sandbox};
    use std::time::Duration;

    let cwd = std::env::current_dir()?;
    let is_auto_detected = spec_path.is_none();

    // Resolve spec path - auto-detect from current branch if not provided
    let spec_path = match spec_path {
        Some(path) => {
            if path.is_absolute() {
                path.clone()
            } else {
                cwd.join(path)
            }
        }
        None => {
            // Get current branch name
            let branch_output = std::process::Command::new("git")
                .args(["rev-parse", "--abbrev-ref", "HEAD"])
                .current_dir(&cwd)
                .output();

            match branch_output {
                Ok(output) if output.status.success() => {
                    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    
                    // Try to find spec file in folder matching branch name
                    let specs_dir = cwd.join(".specs");
                    let spec_folder = specs_dir.join(&branch);
                    let spec_file = spec_folder.join("spec.yaml");
                    let spec_file_yml = spec_folder.join("spec.yml");
                    
                    if spec_file.exists() {
                        spec_file
                    } else if spec_file_yml.exists() {
                        spec_file_yml
                    } else {
                        if json {
                            let output = serde_json::json!({
                                "success": false,
                                "error": format!("No spec found for branch '{}'. Expected: {}", branch, spec_file.display()),
                                "code": "SPEC_NOT_FOUND"
                            });
                            println!("{}", serde_json::to_string_pretty(&output)?);
                        } else {
                            eprintln!("Error: No spec found for branch '{}'", branch);
                            eprintln!("Expected: {}", spec_file.display());
                            eprintln!();
                            eprintln!("Either:");
                            eprintln!("  1. Switch to a spec branch: git checkout <spec-branch>");
                            eprintln!("  2. Create the spec first: ckrv spec new \"description\"");
                        }
                        std::process::exit(1);
                    }
                }
                _ => {
                    if json {
                        let output = serde_json::json!({
                            "success": false,
                            "error": "Could not determine current branch. Provide spec path explicitly.",
                            "code": "BRANCH_UNKNOWN"
                        });
                        println!("{}", serde_json::to_string_pretty(&output)?);
                    } else {
                        eprintln!("Error: Could not determine current branch.");
                        eprintln!("Provide spec path explicitly: ckrv spec tasks .specs/<spec>.yaml");
                    }
                    std::process::exit(1);
                }
            }
        }
    };

    // Check if spec exists
    if !spec_path.exists() {
        if json {
            let output = serde_json::json!({
                "success": false,
                "error": format!("Spec file not found: {}", spec_path.display()),
                "code": "SPEC_NOT_FOUND"
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            eprintln!("Error: Spec file not found: {}", spec_path.display());
        }
        std::process::exit(1);
    }

    // Read the spec content
    let spec_content = std::fs::read_to_string(&spec_path)?;

    // Extract spec ID from parent folder name (e.g., "001-add-user-authentication" from "001-add-user-authentication/spec.yaml")
    let spec_id = spec_path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    if !json {
        println!("{}", Banner::new(spec_id).subtitle("Tasks").render(&ui.theme));
        println!();
        if is_auto_detected {
             println!("Auto-detected spec: {}", spec_path.display());
             println!();
        }
    }

    // Check for existing tasks.yaml
    let spec_folder = spec_path.parent().unwrap_or(&cwd);
    let tasks_path = spec_folder.join("tasks.yaml");

    if tasks_path.exists() && !force {
        
        // Parse YAML
        let content = std::fs::read_to_string(&tasks_path)?;
        // Handle potentially loose format or strict TaskFile
        // We asked Claude for "tasks: [...]" object
        let task_file: Result<TaskFile, _> = serde_yaml::from_str(&content);
        
        match task_file {
            Ok(file) => {
                 if json {
                     let output = serde_json::json!({
                        "success": true,
                        "spec_id": spec_id,
                        "tasks_path": tasks_path,
                        "tasks": file.tasks
                    });
                    println!("{}", serde_json::to_string_pretty(&output)?);
                } else {
                     let table = tabled::Table::new(file.tasks);
                     let rich_table = RichTable::new(table);
                     println!("{}", rich_table.render(&ui.theme));
                     println!();
                     println!("Use --force to regenerate tasks with AI.");
                }
                return Ok(());
            }
            Err(e) => {
                if !json {
                     eprintln!("Warning: Could not parse existing tasks.yaml: {}", e);
                     eprintln!("Proceeding to regenerate...");
                }
            }
        }
    }

    if !json {
        eprintln!("Generating tasks for spec: {}", spec_id);
        eprintln!();
    }

    // Ensure we're on the spec's branch (branch was created during spec new)
    let repo_root = ckrv_git::repo_root(&cwd).unwrap_or_else(|_| cwd.clone());
    
    let on_branch = match std::process::Command::new("git")
        .args(["checkout", spec_id])
        .current_dir(&repo_root)
        .output()
    {
        Ok(output) if output.status.success() => {
            if !json {
                eprintln!("On branch: {}", spec_id);
            }
            true
        }
        _ => {
            if !json {
                eprintln!("Warning: Could not switch to branch '{}'. Run 'ckrv spec new' first to create the branch.", spec_id);
            }
            false
        }
    };

    // Build the prompt for task generation
    let prompt = format!(
        r#"You are a technical project planner. Generate implementation tasks from this specification.

SPECIFICATION:
{}

OUTPUT FORMAT:
Generate a YAML file with the following structure. Do not use markdown formatting or code blocks. Just raw YAML.

tasks:
  - id: T001
    phase: "Setup"  # e.g., Setup, Foundation, User Story 1, Polish
    title: "Create project structure"
    description: "Create directory structure and initialize config files..."
    file: "src/main.rs"  # Primary file target (optional, use empty string "" if none)
    user_story: null     # e.g., "US1" (optional)
    parallel: false      # true if can run in parallel with previous task
    complexity: 1        # 1-5 scale (1=simple file/config, 3=standard logic, 5=complex algorithm/architecture)
    model_tier: "light"  # light | standard | heavy | reasoning
    estimated_tokens: 500
    risk: "low"          # low | medium | high | critical
    context_required: [] # List of files/concepts needed, e.g. ["README.md", "auth_flow"]
    status: "pending"

INSTRUCTIONS:
1. Divide the project into Phases (Setup, Foundation, User Stories, Polish).
2. Break down each phase into concrete, actionable tasks.
3. Assign sequential IDs (T001, T002...).
4. "parallel: true" means it doesn't strictly depend on the immediately preceding task in the same phase.
5. "file" should be the main file being created or modified, if applicable.
6. "complexity" (1-5):
   - 1: Boilerplate, config files, simple HTML/CSS.
   - 2: Basic CRUD, simple functions.
   - 3: Standard business logic, API endpoints.
   - 4: Complex logic, security critical.
   - 5: Core architecture, complex algorithms.
7. "model_tier":
   - light: Complexity 1-2 (e.g., config, simple UI)
   - standard: Complexity 2-3 (e.g., CRUD, endpoints)
   - heavy: Complexity 3-4 (e.g., tough logic, refactoring)
   - reasoning: Complexity 5 (e.g., architecture, security)
8. "estimated_tokens": Guess input/output tokens (e.g. 500 for small, 5000 for huge).
9. "risk":
   - low: Unlikely to break things.
   - medium: standard feature.
   - high: Changes core shared logic.
   - critical: Security/Auth/Payment.
10. Ensure every User Story in the spec is covered.

Output ONLY the raw YAML content. No ```yaml fences."#,
        spec_content
    );

    if !json {
        eprintln!("Generating tasks with AI...");
        eprintln!();
    }

    // Run Claude in Docker sandbox
    let result = {
        let sandbox = DockerSandbox::new(ckrv_sandbox::DefaultAllowList::default())
            .map_err(|e| anyhow::anyhow!("Failed to create sandbox: {}", e))?;

        let command = format!(
            "claude -p {} --dangerously-skip-permissions --output-format text --tools \"\"",
            shell_escape::escape(prompt.clone().into())
        );

        let specs_dir = spec_path.parent().unwrap_or(&cwd);
        let config = ExecuteConfig::new("", specs_dir.to_path_buf())
            .shell(&command)
            .with_timeout(Duration::from_secs(300));

        sandbox.execute(config).await
            .map_err(|e| anyhow::anyhow!("Sandbox execution failed: {}", e))
    }?;

    if !result.success() {
        if json {
            let output = serde_json::json!({
                "success": false,
                "error": format!("AI task generation failed: {}", result.stderr),
                "code": "GENERATION_FAILED"
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            eprintln!("Error: AI task generation failed");
            eprintln!("{}", result.stderr);
        }
        std::process::exit(1);
    }

    // Write tasks.yaml in the same folder as spec.yaml
    let tasks_content = strip_code_fences(result.stdout.trim());
    let spec_folder = spec_path.parent().unwrap_or(&cwd);
    let tasks_path = spec_folder.join("tasks.yaml");
    std::fs::write(&tasks_path, &tasks_content)?;

    if json {
        let output = serde_json::json!({
            "success": true,
            "spec_id": spec_id,
            "branch": if on_branch { Some(spec_id) } else { None },
            "tasks_path": tasks_path,
            "message": "Tasks generated"
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("✓ Generated tasks: {}", tasks_path.display());
        if on_branch {
            println!("✓ On branch: {}", spec_id);
        }
        println!();
        println!("Next steps:");
        println!("  1. Review:  cat {}/tasks.yaml", spec_folder.display());
        println!("  2. Start:   ckrv task \"T001 - [description]\"");
    }

    Ok(())
}

/// Generate a short name from a description.
fn generate_short_name(description: &str) -> String {
    // Extract meaningful words and create a short identifier
    let stop_words = ["the", "and", "for", "with", "that", "this", "from", "into"];
    let lowercase = description.to_lowercase();
    let words: Vec<String> = lowercase
        .split_whitespace()
        .filter(|w| w.len() > 2)
        .filter(|w| !stop_words.contains(w))
        .take(3)
        .map(|s| s.to_string())
        .collect();

    if words.is_empty() {
        "feature".to_string()
    } else {
        words.join("-").replace(|c: char| !c.is_alphanumeric() && c != '-', "")
    }
}

/// Strip markdown code fences from output.
fn strip_code_fences(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();
    let mut in_fence = false;
    let mut first_fence_skipped = false;

    for line in lines {
        if line.starts_with("```") {
            if !first_fence_skipped {
                // Skip the opening fence
                first_fence_skipped = true;
                in_fence = true;
                continue;
            } else if in_fence {
                // Skip the closing fence
                in_fence = false;
                continue;
            }
        }
        result.push(line);
    }

    result.join("\n")
}

fn execute_validate(path: Option<&PathBuf>, json: bool) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    
    // Resolve spec path - auto-detect from current branch if not provided
    let full_path = match path {
        Some(p) => {
            if p.is_absolute() {
                p.clone()
            } else {
                cwd.join(p)
            }
        }
        None => resolve_spec_path(None, &cwd)?
    };

    // Try to load as rich SpecOutput first for additional validation
    let spec_content = std::fs::read_to_string(&full_path)?;
    let rich_spec: Option<super::spec_structs::SpecOutput> = serde_yaml::from_str(&spec_content).ok();
    
    // Additional validation for rich spec
    let mut additional_errors: Vec<ValidationErrorOutput> = Vec::new();
    let mut additional_warnings: Vec<String> = Vec::new();
    
    if let Some(ref spec) = rich_spec {
        // Check for unresolved clarifications
        if spec.has_unresolved_clarifications() {
            additional_errors.push(ValidationErrorOutput {
                field: "clarifications".to_string(),
                message: "Spec has unresolved clarifications. Run 'ckrv spec clarify' first.".to_string(),
            });
        }
        
        // Check for at least one user story
        if spec.user_stories.is_empty() {
            additional_errors.push(ValidationErrorOutput {
                field: "user_stories".to_string(),
                message: "Spec must have at least one user story.".to_string(),
            });
        }
        
        // Check user stories have acceptance scenarios
        for story in &spec.user_stories {
            if story.acceptance_scenarios.is_empty() {
                additional_warnings.push(format!(
                    "User story '{}' has no acceptance scenarios",
                    story.id
                ));
            }
        }
        
        // Check for at least one requirement
        if spec.requirement_count() == 0 {
            additional_errors.push(ValidationErrorOutput {
                field: "requirements".to_string(),
                message: "Spec must have at least one requirement.".to_string(),
            });
        }
        
        // Check success criteria are measurable (contain numbers)
        for criteria in &spec.success_criteria {
            let has_number = criteria.metric.chars().any(|c| c.is_ascii_digit());
            if !has_number {
                additional_warnings.push(format!(
                    "Success criterion '{}' may not be measurable (no numeric target)",
                    criteria.id
                ));
            }
        }
        
        // Check overview is not placeholder
        if let Some(ref overview) = spec.overview {
            if overview.contains("[") && overview.contains("]") {
                additional_warnings.push("Overview appears to contain placeholder text".to_string());
            }
        }
    }

    // Load spec using original loader
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

    // Validate using original validator
    let result = ckrv_spec::validator::validate(&spec);

    // Combine errors and warnings
    let all_errors: Vec<ValidationErrorOutput> = result
        .errors
        .iter()
        .map(|e| ValidationErrorOutput {
            field: e.field.clone(),
            message: e.message.clone(),
        })
        .chain(additional_errors)
        .collect();
    
    let all_warnings: Vec<String> = result.warnings.iter().cloned()
        .chain(additional_warnings)
        .collect();
    
    let is_valid = result.valid && all_errors.len() == result.errors.len();

    if json {
        let output = SpecValidateOutput {
            valid: is_valid,
            errors: all_errors,
            warnings: all_warnings,
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else if is_valid {
        println!("✓ Spec is valid: {}", full_path.display());
        for warning in &all_warnings {
            println!("  ⚠ {warning}");
        }
        println!();
        println!("Next steps:");
        println!("  1. Tasks:    ckrv spec tasks");
    } else {
        eprintln!("✗ Spec validation failed: {}", full_path.display());
        for error in &all_errors {
            eprintln!("  • {}: {}", error.field, error.message);
        }
        for warning in &all_warnings {
            eprintln!("  ⚠ {warning}");
        }
    }

    if !is_valid {
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
            println!("No specs found. Run 'ckrv spec new \"description\"' to create one.");
        }
        return Ok(());
    }

    // Collect spec directories
    let mut spec_dirs: Vec<PathBuf> = Vec::new();
    for entry in std::fs::read_dir(&specs_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() && path.join("spec.yaml").exists() {
            spec_dirs.push(path);
        }
    }

    // Sort by directory name (which includes the number prefix)
    spec_dirs.sort();

    if json {
        let specs: Vec<_> = spec_dirs
            .iter()
            .map(|p| {
                let has_tasks = p.join("tasks.yaml").exists();
                serde_json::json!({
                    "path": p,
                    "name": p.file_name().map(|s| s.to_string_lossy().to_string()),
                    "has_tasks": has_tasks
                })
            })
            .collect();
        let output = serde_json::json!({
            "specs": specs,
            "count": spec_dirs.len()
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else if spec_dirs.is_empty() {
        println!("No specs found. Run 'ckrv spec new \"description\"' to create one.");
    } else {
        println!("Specifications:");
        for path in &spec_dirs {
            if let Some(name) = path.file_name() {
                let has_tasks = path.join("tasks.yaml").exists();
                let status = if has_tasks { "✓" } else { " " };
                println!("  {} {} ({})", status, name.to_string_lossy(), path.display());
            }
        }
        println!();
        println!("✓ = has tasks.yaml");
    }

    Ok(())
}

/// Get the next available spec number by scanning existing spec directories.
/// Returns 1 if no specs exist, otherwise max + 1.
fn get_next_spec_number(specs_dir: &std::path::Path) -> anyhow::Result<u32> {
    if !specs_dir.exists() {
        return Ok(1);
    }

    let mut max_number: u32 = 0;

    for entry in std::fs::read_dir(specs_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        // Look for directories (e.g., "001-my-feature/")
        if path.is_dir() {
            if let Some(dir_name) = path.file_name() {
                let dir_str = dir_name.to_string_lossy();
                // Extract number prefix (e.g., "001" from "001-my_feature")
                if let Some(num_str) = dir_str.split('-').next() {
                    if let Ok(num) = num_str.parse::<u32>() {
                        max_number = max_number.max(num);
                    }
                }
            }
        }
    }

    Ok(max_number + 1)
}

/// Find an existing spec by name (ignoring the number prefix).
/// Returns the path to the spec directory if found, None otherwise.
fn find_spec_by_name(specs_dir: &std::path::Path, name: &str) -> anyhow::Result<Option<PathBuf>> {
    if !specs_dir.exists() {
        return Ok(None);
    }

    for entry in std::fs::read_dir(specs_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        // Look for directories (e.g., "001-my-feature/")
        if path.is_dir() {
            if let Some(dir_name) = path.file_name() {
                let dir_str = dir_name.to_string_lossy();
                // Check if name matches after the number prefix (e.g., "001-my_feature" -> "my_feature")
                let parts: Vec<&str> = dir_str.splitn(2, '-').collect();
                if parts.len() == 2 && parts[1] == name {
                    return Ok(Some(path));
                }
                // Also check exact match (for directories without number prefix)
                if dir_str == name {
                    return Ok(Some(path));
                }
            }
        }
    }

    Ok(None)
}

#[derive(Debug, serde::Serialize, serde::Deserialize, tabled::Tabled)]
struct Task {
    #[tabled(rename = "ID")]
    pub id: String,
    
    #[tabled(rename = "Phase")]
    pub phase: String,

    #[tabled(rename = "Title")]
    pub title: String,
    
    #[tabled(skip)]
    pub description: String,
    
    #[tabled(rename = "File")]
    #[serde(default)]
    pub file: String,
    
    #[tabled(skip)]
    pub user_story: Option<String>,
    
    #[tabled(skip)]
    pub parallel: bool,
    
    #[tabled(rename = "Level")]
    #[serde(default = "default_complexity")]
    pub complexity: u8, // 1-5 scale

    #[tabled(rename = "Tier")]
    #[serde(default = "default_tier")]
    pub model_tier: String, // light | standard | heavy | reasoning

    #[serde(default)]
    #[tabled(skip)]
    pub estimated_tokens: u32,

    #[tabled(rename = "Risk")]
    #[serde(default = "default_risk")]
    pub risk: String, // low | medium | high | critical

    #[serde(default)]
    #[tabled(skip)]
    pub context_required: Vec<String>,

    #[tabled(rename = "Status")]
    pub status: String,
}

fn default_complexity() -> u8 { 3 }
fn default_tier() -> String { "standard".to_string() }
fn default_risk() -> String { "low".to_string() }

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct TaskFile {
    pub tasks: Vec<Task>,
}
