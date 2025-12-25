//! Run command - execute spec-driven code changes.

use std::path::{Path, PathBuf};
use std::fs;
use std::time::Duration;

use clap::{Args, ValueEnum};
use serde::Serialize;

use ckrv_core::OptimizeMode;
use ckrv_git::{DefaultWorktreeManager, WorktreeManager, DefaultDiffGenerator, DiffGenerator};
use ckrv_metrics::{DefaultMetricsCollector, MetricsCollector, FileMetricsStorage, MetricsStorage};
use ckrv_sandbox::{DockerSandbox, Sandbox, ExecuteConfig};

/// Arguments for the run command
#[derive(Args)]
pub struct RunArgs {
    /// Path to the specification file
    pub spec: PathBuf,

    /// Optimization mode for model selection
    #[arg(long, value_enum, default_value = "balanced")]
    pub optimize: OptimizeModeArg,

    /// Maximum retry attempts
    #[arg(long, default_value = "3")]
    pub max_attempts: u32,

    /// Override planner model
    #[arg(long)]
    pub planner_model: Option<String>,

    /// Override executor model
    #[arg(long)]
    pub executor_model: Option<String>,

    /// Show plan without executing
    #[arg(long)]
    pub dry_run: bool,

    /// Skip file modifications (generate only)
    #[arg(long)]
    pub generate_only: bool,
}

/// CLI-compatible optimize mode
#[derive(Clone, Copy, ValueEnum)]
pub enum OptimizeModeArg {
    /// Prefer cheaper models
    Cost,
    /// Prefer faster models
    Time,
    /// Balance cost and time
    Balanced,
}

impl From<OptimizeModeArg> for OptimizeMode {
    fn from(arg: OptimizeModeArg) -> Self {
        match arg {
            OptimizeModeArg::Cost => OptimizeMode::Cost,
            OptimizeModeArg::Time => OptimizeMode::Time,
            OptimizeModeArg::Balanced => OptimizeMode::Balanced,
        }
    }
}

/// JSON output for run events
#[derive(Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
#[allow(dead_code)]
enum RunEvent {
    Started {
        job_id: String,
        spec_id: String,
    },
    Planning {
        spec_id: String,
    },
    PlanReady {
        plan_id: String,
        step_count: usize,
    },
    WorktreeCreated {
        path: String,
    },
    DryRunComplete {
        plan_id: String,
        steps: Vec<PlanStep>,
    },
    StepStarted {
        step_id: String,
        step_name: String,
    },
    StepCompleted {
        step_id: String,
        duration_ms: u64,
    },
    FileModified {
        path: String,
        lines_added: usize,
        lines_removed: usize,
    },
    Verifying {
        attempt: u32,
        image: String,
    },
    VerificationPassed {
        command: String,
        duration_ms: u64,
    },
    VerificationFailed {
        command: String,
        exit_code: i32,
        stderr: String,
    },
    Succeeded {
        job_id: String,
        attempt: u32,
        diff_lines: usize,
        files_changed: usize,
    },
    Failed {
        job_id: String,
        attempts: u32,
        error: String,
    },
    Error {
        code: String,
        message: String,
    },
}

#[derive(Serialize)]
struct PlanStep {
    id: String,
    name: String,
    step_type: String,
}

/// Parsed file change from AI response
#[derive(Debug, Clone)]
struct FileChange {
    path: String,
    content: String,
}

fn emit_event(event: &RunEvent, json: bool) {
    if json {
        if let Ok(s) = serde_json::to_string(event) {
            println!("{s}");
        }
    }
}

/// Parse code blocks from AI response
fn parse_file_changes(content: &str) -> Vec<FileChange> {
    let mut changes = Vec::new();
    let mut current_path: Option<String> = None;
    let mut current_content = String::new();
    let mut in_code_block = false;

    for line in content.lines() {
        // Check for file path indicators
        if line.starts_with("#### File Path:") || line.starts_with("File Path:") || line.starts_with("**File:**") {
            // Extract path from the line
            if let Some(path) = extract_path(line) {
                if in_code_block && current_path.is_some() {
                    // Save previous block
                    changes.push(FileChange {
                        path: current_path.take().unwrap(),
                        content: current_content.trim().to_string(),
                    });
                    current_content.clear();
                }
                current_path = Some(path);
            }
        }
        // Check for code fence start
        else if line.starts_with("```") {
            if in_code_block {
                // End of code block
                in_code_block = false;
                if let Some(ref path) = current_path {
                    if !current_content.trim().is_empty() {
                        changes.push(FileChange {
                            path: path.clone(),
                            content: current_content.trim().to_string(),
                        });
                        current_content.clear();
                        current_path = None;
                    }
                }
            } else {
                // Start of code block
                in_code_block = true;
                // Check if there's a path in the fence
                let rest = line.trim_start_matches("```").trim();
                if rest.contains('/') || rest.ends_with(".rs") || rest.ends_with(".py") || rest.ends_with(".js") || rest.ends_with(".ts") {
                    current_path = Some(rest.to_string());
                }
            }
        }
        // Collect content if in code block
        else if in_code_block {
            current_content.push_str(line);
            current_content.push('\n');
        }
        // Check inline path patterns
        else if line.contains("src/") && (line.contains(".rs") || line.contains(".py") || line.contains(".js")) {
            if let Some(path) = extract_path(line) {
                current_path = Some(path);
            }
        }
    }

    // Handle any remaining content
    if let Some(path) = current_path {
        if !current_content.trim().is_empty() {
            changes.push(FileChange {
                path,
                content: current_content.trim().to_string(),
            });
        }
    }

    // If no structured changes found, try to extract from common patterns
    if changes.is_empty() {
        // Look for "// filename.rs" comments followed by code
        let mut lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        while i < lines.len() {
            let line = lines[i];
            if line.starts_with("// src/") || line.starts_with("// ./") {
                let path = line.trim_start_matches("// ").trim().to_string();
                let mut code = String::new();
                i += 1;
                while i < lines.len() && !lines[i].starts_with("// src/") && !lines[i].starts_with("// ./") {
                    if !lines[i].starts_with("```") {
                        code.push_str(lines[i]);
                        code.push('\n');
                    }
                    i += 1;
                }
                if !code.trim().is_empty() {
                    changes.push(FileChange { path, content: code.trim().to_string() });
                }
                continue;
            }
            i += 1;
        }
    }

    changes
}

fn extract_path(line: &str) -> Option<String> {
    // Try to find a file path in the line
    let cleaned = line
        .replace("#### File Path:", "")
        .replace("File Path:", "")
        .replace("**File:**", "")
        .replace("```", "")
        .replace('`', "")
        .trim()
        .to_string();

    // Extract just the path part
    for word in cleaned.split_whitespace() {
        let word = word.trim_matches(|c| c == ':' || c == ',' || c == '"' || c == '\'' || c == '`');
        if word.contains('/') || word.ends_with(".rs") || word.ends_with(".py") || word.ends_with(".js") || word.ends_with(".ts") {
            return Some(word.to_string());
        }
    }
    
    if cleaned.contains('/') || cleaned.ends_with(".rs") {
        return Some(cleaned);
    }
    
    None
}

/// Execute the run command
pub fn execute(args: RunArgs, json: bool) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let start_time = std::time::Instant::now();

    // Try to load secrets from .chakravarti/secrets/.env
    load_secrets(&cwd);

    // Resolve spec path
    let spec_path = if args.spec.is_absolute() {
        args.spec.clone()
    } else {
        cwd.join(&args.spec)
    };

    // Get repo root
    let repo_root = ckrv_git::repo_root(&cwd).unwrap_or_else(|_| cwd.clone());
    let chakravarti_dir = repo_root.join(".chakravarti");

    // Initialize metrics collector
    let metrics = DefaultMetricsCollector::new();

    // Load and validate spec
    let loader = ckrv_spec::loader::YamlSpecLoader;
    let spec = match ckrv_spec::loader::SpecLoader::load(&loader, &spec_path) {
        Ok(s) => s,
        Err(e) => {
            if json {
                emit_event(
                    &RunEvent::Error {
                        code: "SPEC_LOAD_FAILED".to_string(),
                        message: e.to_string(),
                    },
                    json,
                );
            } else {
                eprintln!("Error: Failed to load spec: {e}");
            }
            std::process::exit(1);
        }
    };

    // Validate spec
    let validation = ckrv_spec::validator::validate(&spec);
    if !validation.valid {
        if json {
            emit_event(
                &RunEvent::Error {
                    code: "SPEC_INVALID".to_string(),
                    message: validation
                        .errors
                        .iter()
                        .map(|e| format!("{}: {}", e.field, e.message))
                        .collect::<Vec<_>>()
                        .join("; "),
                },
                json,
            );
        } else {
            eprintln!("Error: Spec validation failed:");
            for error in &validation.errors {
                eprintln!("  • {}: {}", error.field, error.message);
            }
        }
        std::process::exit(1);
    }

    // Generate job ID
    let job_id = uuid::Uuid::new_v4().to_string();

    if json {
        emit_event(
            &RunEvent::Started {
                job_id: job_id.clone(),
                spec_id: spec.id.clone(),
            },
            json,
        );
    } else {
        println!("Starting job: {job_id}");
        println!("Spec: {} ({})", spec.id, spec_path.display());
        println!();
    }

    // Planning phase
    if json {
        emit_event(
            &RunEvent::Planning {
                spec_id: spec.id.clone(),
            },
            json,
        );
    } else {
        println!("Phase: Planning");
        println!("  Goal: {}", spec.goal);
    }

    // Generate a simple plan
    let plan_id = uuid::Uuid::new_v4().to_string();
    let steps = vec![
        PlanStep {
            id: "analyze".to_string(),
            name: "Analyze codebase".to_string(),
            step_type: "analyze".to_string(),
        },
        PlanStep {
            id: "generate".to_string(),
            name: "Generate changes".to_string(),
            step_type: "generate".to_string(),
        },
        PlanStep {
            id: "apply".to_string(),
            name: "Apply changes".to_string(),
            step_type: "execute".to_string(),
        },
        PlanStep {
            id: "verify".to_string(),
            name: "Verify changes".to_string(),
            step_type: "verify".to_string(),
        },
    ];

    if json {
        emit_event(
            &RunEvent::PlanReady {
                plan_id: plan_id.clone(),
                step_count: steps.len(),
            },
            json,
        );
    } else {
        println!("  Plan generated: {plan_id}");
        println!("  Steps: {}", steps.len());
        for step in &steps {
            println!("    • {} ({})", step.name, step.step_type);
        }
        println!();
    }

    // Handle dry-run mode
    if args.dry_run {
        if json {
            emit_event(&RunEvent::DryRunComplete { plan_id, steps }, json);
        } else {
            println!("Dry run complete. No changes made.");
            println!();
            println!("To execute, run without --dry-run:");
            println!("  ckrv run {}", spec_path.display());
        }
        return Ok(());
    }

    // Check for API key (required for actual execution)
    let has_openai_key = std::env::var("OPENAI_API_KEY").is_ok();
    let has_anthropic_key = std::env::var("ANTHROPIC_API_KEY").is_ok();
    let has_custom_key = std::env::var("CKRV_MODEL_API_KEY").is_ok();

    if !has_openai_key && !has_anthropic_key && !has_custom_key {
        if json {
            emit_event(
                &RunEvent::Error {
                    code: "NO_API_KEY".to_string(),
                    message: "No model API key configured. Set OPENAI_API_KEY, ANTHROPIC_API_KEY, or CKRV_MODEL_API_KEY.".to_string(),
                },
                json,
            );
        } else {
            eprintln!("Error: No model API key configured.");
            eprintln!();
            eprintln!("Set one of the following environment variables:");
            eprintln!("  OPENAI_API_KEY      - For OpenAI models");
            eprintln!("  ANTHROPIC_API_KEY   - For Anthropic models");
            eprintln!("  CKRV_MODEL_API_KEY  - For custom endpoint");
            eprintln!();
            eprintln!("Or use --dry-run to see the plan without executing.");
        }
        std::process::exit(1);
    }

    // Create worktree for isolated execution
    let worktree = match DefaultWorktreeManager::new(&repo_root) {
        Ok(worktree_manager) => match worktree_manager.create(&job_id, "attempt-1") {
            Ok(wt) => {
                if json {
                    emit_event(
                        &RunEvent::WorktreeCreated {
                            path: wt.path.display().to_string(),
                        },
                        json,
                    );
                } else {
                    println!("Phase: Setup");
                    println!("  Created worktree: {}", wt.path.display());
                    println!();
                }
                wt
            }
            Err(e) => {
                if json {
                    emit_event(
                        &RunEvent::Error {
                            code: "WORKTREE_FAILED".to_string(),
                            message: format!("Could not create worktree: {}", e),
                        },
                        json,
                    );
                } else {
                    eprintln!("Error: Could not create worktree: {e}");
                    eprintln!();
                    eprintln!("Worktree is required for isolated execution.");
                    eprintln!("Make sure you have at least one commit in the repository:");
                    eprintln!("  git add -A && git commit -m 'Initial commit'");
                }
                std::process::exit(1);
            }
        },
        Err(e) => {
            if json {
                emit_event(
                    &RunEvent::Error {
                        code: "WORKTREE_INIT_FAILED".to_string(),
                        message: format!("Could not initialize worktree manager: {}", e),
                    },
                    json,
                );
            } else {
                eprintln!("Error: Could not initialize worktree manager: {e}");
                eprintln!();
                eprintln!("Make sure you are in a git repository:");
                eprintln!("  git init && git add -A && git commit -m 'Initial commit'");
            }
            std::process::exit(1);
        }
    };

    // Working directory for changes (always the worktree)
    let work_dir = worktree.path.clone();

    // Execute with model API
    if !json {
        println!("Phase: Execution");
        println!("  Optimize: {:?}", OptimizeMode::from(args.optimize));
        println!("  Max attempts: {}", args.max_attempts);
        if let Some(ref model) = args.planner_model {
            println!("  Planner model: {model}");
        }
        if let Some(ref model) = args.executor_model {
            println!("  Executor model: {model}");
        }
        println!();
    }

    // Create model router
    let router = match ckrv_model::ModelRouter::new() {
        Ok(r) => r,
        Err(e) => {
            if json {
                emit_event(
                    &RunEvent::Error {
                        code: "ROUTER_INIT_FAILED".to_string(),
                        message: e.to_string(),
                    },
                    json,
                );
            } else {
                eprintln!("Error: Failed to initialize model router: {e}");
            }
            std::process::exit(1);
        }
    };

    if !json {
        println!("  Model providers: {}", router.provider_names().join(", "));
        println!();
    }

    // Build the prompt for code generation
    let system_prompt = format!(
        r#"You are an expert software engineer. Your task is to implement code changes based on a specification.

SPECIFICATION:
Goal: {}

Constraints:
{}

Acceptance Criteria:
{}

IMPORTANT: Output your changes in the following format:

For EACH file you want to modify or create, output:

File Path: path/to/file.ext
```language
<complete file contents>
```

Include the COMPLETE file contents, not just the changes.
Output each file change with a clear "File Path:" header followed by a code block."#,
        spec.goal,
        spec.constraints
            .iter()
            .map(|c| format!("- {c}"))
            .collect::<Vec<_>>()
            .join("\n"),
        spec.acceptance
            .iter()
            .map(|a| format!("- {a}"))
            .collect::<Vec<_>>()
            .join("\n"),
    );

    let user_prompt = "Please implement the changes. Output each file with 'File Path:' followed by a code block with the complete file contents.";

    // Select model based on optimization mode
    let context = ckrv_model::RoutingContext {
        optimize: OptimizeMode::from(args.optimize),
        task_type: ckrv_model::TaskType::Execution,
        estimated_tokens: None,
        model_override: args.executor_model.clone(),
    };
    let model = router.select_model(&context);

    if !json {
        println!("  Using model: {model}");
        println!("  Generating implementation...");
        println!();
    }

    // Make the API call
    let request = ckrv_model::CompletionRequest {
        model: model.clone(),
        messages: vec![
            ckrv_model::Message {
                role: "system".to_string(),
                content: system_prompt,
            },
            ckrv_model::Message {
                role: "user".to_string(),
                content: user_prompt.to_string(),
            },
        ],
        max_tokens: Some(4096),
        temperature: Some(0.7),
    };

    // Use tokio runtime for async call
    let rt = tokio::runtime::Runtime::new()?;
    let response = rt.block_on(async { router.complete(request).await });

    match response {
        Ok(completion) => {
            if !json {
                println!("✓ Generation complete!");
                println!();
                println!("Model: {}", completion.model);
                println!(
                    "Tokens: {} prompt + {} completion = {} total",
                    completion.usage.prompt_tokens,
                    completion.usage.completion_tokens,
                    completion.usage.total_tokens
                );
                println!();
            }

            // Parse file changes from response
            let changes = parse_file_changes(&completion.content);
            
            if changes.is_empty() {
                if !json {
                    println!("─────────────────────────────────────────────");
                    println!("GENERATED IMPLEMENTATION:");
                    println!("─────────────────────────────────────────────");
                    println!();
                    println!("{}", completion.content);
                    println!();
                    println!("─────────────────────────────────────────────");
                    println!();
                    println!("Note: Could not parse structured file changes.");
                    println!("      Please review and apply changes manually.");
                }
            } else if args.generate_only {
                if !json {
                    println!("Parsed {} file change(s):", changes.len());
                    for change in &changes {
                        println!("  • {}", change.path);
                    }
                    println!();
                    println!("--generate-only mode: Not applying changes.");
                }
            } else {
                // Apply changes to worktree
                if !json {
                    println!("Phase: Apply Changes");
                    println!("  Target: {}", work_dir.display());
                    println!();
                }

                let mut files_modified = 0;
                for change in &changes {
                    let file_path = work_dir.join(&change.path);
                    
                    // Create parent directories if needed
                    if let Some(parent) = file_path.parent() {
                        if !std::path::Path::new(parent).exists() {
                            fs::create_dir_all(parent)?;
                        }
                    }

                    // Write the file
                    match fs::write(&file_path, &change.content) {
                        Ok(_) => {
                            files_modified += 1;
                            if json {
                                emit_event(
                                    &RunEvent::FileModified {
                                        path: change.path.clone(),
                                        lines_added: change.content.lines().count(),
                                        lines_removed: 0,
                                    },
                                    json,
                                );
                            } else {
                                println!("  ✓ Modified: {}", change.path);
                            }
                        }
                        Err(e) => {
                            if !json {
                                eprintln!("  ✗ Failed to write {}: {}", change.path, e);
                            }
                        }
                    }
                }

                if !json && files_modified > 0 {
                    println!();
                    println!("Applied {} file change(s)", files_modified);
                    println!();
                }

                // Run verification if configured
                if let Some(ref verify_config) = spec.verify {
                    if !verify_config.commands.is_empty() {
                        // Determine Docker image
                        let image = verify_config.image.clone()
                            .unwrap_or_else(|| detect_docker_image(&repo_root));

                        if !json {
                            println!("Phase: Verify");
                            println!("  Image: {}", image);
                            println!("  Commands: {}", verify_config.commands.len());
                            println!();
                        } else {
                            emit_event(
                                &RunEvent::Verifying {
                                    attempt: 1,
                                    image: image.clone(),
                                },
                                json,
                            );
                        }

                        // Run verification in Docker
                        let verification_passed = rt.block_on(async {
                            run_verification(&work_dir, &verify_config.commands, &image, json).await
                        });

                        match verification_passed {
                            Ok(true) => {
                                if !json {
                                    println!("  All verification checks passed!");
                                    println!();
                                }
                            }
                            Ok(false) => {
                                if json {
                                    emit_event(
                                        &RunEvent::Failed {
                                            job_id: job_id.clone(),
                                            attempts: 1,
                                            error: "Verification failed".to_string(),
                                        },
                                        json,
                                    );
                                } else {
                                    eprintln!();
                                    eprintln!("Error: Verification failed");
                                    eprintln!("Changes have been applied but verification did not pass.");
                                }
                                std::process::exit(1);
                            }
                            Err(e) => {
                                if !json {
                                    eprintln!("Warning: Verification error: {}", e);
                                }
                            }
                        }
                    }
                }

                // Stage changes in worktree so they appear in diff
                {
                    use std::process::Command;
                    let _ = Command::new("git")
                        .args(["add", "-A"])
                        .current_dir(&work_dir)
                        .output();
                }

                // Generate diff
                {
                    let diff_gen = DefaultDiffGenerator::new();
                    if let Ok(diff) = diff_gen.diff_path(&work_dir) {
                        // Save diff to runs directory
                        let runs_dir = chakravarti_dir.join("runs").join(&job_id);
                        fs::create_dir_all(&runs_dir)?;
                        
                        if !diff.content.is_empty() {
                            fs::write(runs_dir.join("diff.patch"), &diff.content)?;

                            if !json {
                                println!("Phase: Diff");
                                println!("  Files changed: {}", diff.files.len());
                                let insertions: usize = diff.files.iter().map(|f| f.additions).sum();
                                let deletions: usize = diff.files.iter().map(|f| f.deletions).sum();
                                println!("  Insertions: {}", insertions);
                                println!("  Deletions: {}", deletions);
                                println!();
                            }
                        }
                    }
                }

                // Save metrics
                let duration = start_time.elapsed();
                metrics.start_job(&job_id, &spec.id);
                metrics.record_tokens(
                    &completion.model,
                    completion.usage.prompt_tokens.into(),
                    completion.usage.completion_tokens.into(),
                );
                let final_metrics = metrics.finish_job(true);

                let storage = FileMetricsStorage::new(&chakravarti_dir);
                if storage.save(&final_metrics).is_ok() && !json {
                    println!("Metrics saved to .chakravarti/runs/{}/metrics.json", job_id);
                    println!();
                }

                // Success output
                if json {
                    emit_event(
                        &RunEvent::Succeeded {
                            job_id: job_id.clone(),
                            attempt: 1,
                            diff_lines: completion.content.lines().count(),
                            files_changed: files_modified,
                        },
                        json,
                    );
                } else {
                    println!("═══════════════════════════════════════════════");
                    println!("✓ Job completed successfully!");
                    println!("═══════════════════════════════════════════════");
                    println!();
                    println!("Job ID: {}", job_id);
                    println!("Files modified: {}", files_modified);
                    println!("Duration: {:.2}s", duration.as_secs_f64());
                    println!("Est. cost: ${:.4}", final_metrics.cost.total_usd);
                    println!();
                    
                    println!("Changes are in worktree: {}", work_dir.display());
                    println!();
                    println!("Next steps:");
                    println!("  ckrv diff {}              # View changes", job_id);
                    println!("  ckrv promote {} --branch feature/x  # Promote to branch", job_id);
                }
            }
        }
        Err(e) => {
            if json {
                emit_event(
                    &RunEvent::Error {
                        code: "MODEL_ERROR".to_string(),
                        message: e.to_string(),
                    },
                    json,
                );
            } else {
                eprintln!("Error: Model API call failed: {e}");
            }
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Load secrets from .chakravarti/secrets/.env if it exists
fn load_secrets(cwd: &std::path::Path) {
    // Try to find repo root and load secrets
    if let Ok(repo_root) = ckrv_git::repo_root(cwd) {
        let secrets_env_path = repo_root.join(".chakravarti").join("secrets").join(".env");

        if secrets_env_path.exists() {
            // Load the .env file (sets environment variables)
            if dotenvy::from_path(&secrets_env_path).is_ok() {
                tracing::debug!("Loaded secrets from {}", secrets_env_path.display());
            }
        }
    }
}

/// Detect Docker image based on project files
fn detect_docker_image(project_dir: &Path) -> String {
    // Check for various project manifest files
    if project_dir.join("Cargo.toml").exists() {
        return "rust:1.75-slim".to_string();
    }
    if project_dir.join("package.json").exists() {
        return "node:20-slim".to_string();
    }
    if project_dir.join("go.mod").exists() {
        return "golang:1.21-alpine".to_string();
    }
    if project_dir.join("pyproject.toml").exists() || project_dir.join("requirements.txt").exists() {
        return "python:3.11-slim".to_string();
    }
    if project_dir.join("Gemfile").exists() {
        return "ruby:3.2-slim".to_string();
    }
    
    // Default fallback
    "ubuntu:22.04".to_string()
}

/// Run verification commands in Docker sandbox
async fn run_verification(
    work_dir: &Path,
    commands: &[String],
    _image: &str,
    json: bool,
) -> Result<bool, String> {
    // Try to create Docker sandbox
    let sandbox = match DockerSandbox::with_defaults() {
        Ok(s) => s,
        Err(e) => {
            if !json {
                println!("  Note: Docker not available ({}), using local execution", e);
            }
            // Fall back to local execution
            return run_local_verification(work_dir, commands, json).await;
        }
    };
    
    // Check if Docker is available
    if sandbox.health_check().await.is_err() {
        if !json {
            println!("  Note: Docker health check failed, using local execution");
        }
        return run_local_verification(work_dir, commands, json).await;
    }

    for cmd in commands {
        let config = ExecuteConfig::new(cmd.clone(), work_dir.to_path_buf())
            .shell(cmd)
            .with_timeout(Duration::from_secs(300));

        match sandbox.execute(config).await {
            Ok(result) => {
                if result.success() {
                    if !json {
                        println!("  ✓ {}", cmd);
                    }
                } else {
                    if !json {
                        println!("  ✗ {} (exit code: {})", cmd, result.exit_code);
                        if !result.stderr.is_empty() {
                            for line in result.stderr.lines().take(10) {
                                println!("    {}", line);
                            }
                        }
                    }
                    return Ok(false);
                }
            }
            Err(e) => {
                if !json {
                    println!("  ✗ {} (error: {})", cmd, e);
                }
                return Ok(false);
            }
        }
    }

    Ok(true)
}

/// Run verification commands locally (fallback when Docker not available)
async fn run_local_verification(
    work_dir: &Path,
    commands: &[String],
    json: bool,
) -> Result<bool, String> {
    use std::process::Command;

    for cmd in commands {
        let output = Command::new("sh")
            .args(["-c", cmd])
            .current_dir(work_dir)
            .output();

        match output {
            Ok(result) => {
                if result.status.success() {
                    if !json {
                        println!("  ✓ {}", cmd);
                    }
                } else {
                    if !json {
                        let exit_code = result.status.code().unwrap_or(-1);
                        println!("  ✗ {} (exit code: {})", cmd, exit_code);
                        let stderr = String::from_utf8_lossy(&result.stderr);
                        if !stderr.is_empty() {
                            for line in stderr.lines().take(10) {
                                println!("    {}", line);
                            }
                        }
                    }
                    return Ok(false);
                }
            }
            Err(e) => {
                if !json {
                    println!("  ✗ {} (error: {})", cmd, e);
                }
                return Ok(false);
            }
        }
    }

    Ok(true)
}

