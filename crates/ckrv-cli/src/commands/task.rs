//! Task command - execute workflow-based agent tasks.
//!
//! This command initiates a multi-step workflow (like Plan -> Implement)
//! using an AI agent in a sandboxed environment.

use std::path::PathBuf;

use clap::Args;
use serde::{Deserialize, Serialize};

use ckrv_core::{
    runner::{RunnerConfig, WorkflowRunner},
    AgentTask, Workflow,
};

use crate::ui::UiContext;

/// Arguments for the task command.
#[derive(Args)]
pub struct TaskArgs {
    /// Task description or ID (e.g., "T001").
    #[arg(required = true)]
    pub target: String,

    /// Workflow to use (name or path to YAML file).
    #[arg(short, long, default_value = "swe")]
    pub workflow: String,

    /// Show plan without executing (dry run).
    #[arg(long)]
    pub dry_run: bool,

    /// Continue a previous task by ID.
    #[arg(short, long)]
    pub continue_task: Option<String>,

    /// Agent tool to use.
    #[arg(long, default_value = "claude")]
    pub agent: String,

    /// Skip Docker sandbox and run agent locally (NOT RECOMMENDED).
    #[arg(long)]
    pub no_sandbox: bool,

    /// Keep Docker container after execution (for debugging).
    #[arg(long)]
    pub keep_container: bool,

    /// Use existing worktree path instead of creating one.
    #[arg(long)]
    pub use_worktree: Option<PathBuf>,
}

/// JSON output events for task execution.
#[derive(Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
enum TaskEvent {
    Started {
        task_id: String,
        workflow: String,
    },
    StepStarted {
        step_id: String,
        step_name: String,
    },
    StepCompleted {
        step_id: String,
        duration_ms: u64,
    },
    StepFailed {
        step_id: String,
        error: String,
    },
    Completed {
        task_id: String,
        duration_ms: u64,
        steps_completed: usize,
    },
    Failed {
        task_id: String,
        error: String,
    },
    Error {
        code: String,
        message: String,
    },
}

fn emit_event(event: &TaskEvent, json: bool) {
    if json {
        if let Ok(json_str) = serde_json::to_string(event) {
            println!("{json_str}");
        }
    }
}

/// Execute the task command.
///
/// # Errors
///
/// Returns an error if task execution fails.
pub async fn execute(args: TaskArgs, json: bool, ui: &UiContext) -> anyhow::Result<()> {
    use ckrv_git::worktree::{DefaultWorktreeManager, WorktreeManager};

    // Get current directory as base
    let cwd = std::env::current_dir()?;

    // Load workflow
    let workflow = load_workflow(&args.workflow, &cwd)?;

    // Handle Task ID vs Description
    let target = &args.target;
    let target_is_id = target.starts_with('T') && target.len() == 4 && target[1..].chars().all(char::is_numeric);

    let (description, task_id): (String, String) = if target_is_id {
         // Auto-detect spec and look up task
         let branch_output = std::process::Command::new("git").args(["symbolic-ref", "--short", "HEAD"]).output().ok();
         let branch = branch_output.and_then(|o| if o.status.success() { String::from_utf8(o.stdout).ok().map(|s| s.trim().to_string()) } else { None });
         
         let detected_path = if let Some(b) = branch {
             let p = cwd.join(".specs").join(&b);
             let yaml = p.join("tasks.yaml");
             if yaml.exists() { Some(yaml) } else { None }
         } else { None };
         
         let spec_task = if let Some(path) = detected_path {
              let content = std::fs::read_to_string(&path).unwrap_or_default();
              if let Ok(file) = serde_yaml::from_str::<TaskFile>(&content) {
                  file.tasks.into_iter().find(|t| t.id == *target)
              } else { None }
         } else { None };

         if let Some(t) = spec_task {
              if !json {
                  eprintln!("Found task {} in spec: {}", t.id, t.title);
              }
              let desc = format!("Phase: {}\nTitle: {}\nDescription: {}\nFile: {}", t.phase, t.title, t.description, t.file);
              (desc, t.id)
         } else {
              (target.clone(), args.continue_task.clone().unwrap_or_else(AgentTask::generate_id))
         }
    } else {
         (target.clone(), args.continue_task.clone().unwrap_or_else(AgentTask::generate_id))
    };

    if !json {
        eprintln!(
            "Starting task with workflow '{}' ({} steps)",
            workflow.name,
            workflow.steps.len()
        );
        eprintln!("Task ID: {}", task_id);
    }

    emit_event(
        &TaskEvent::Started {
            task_id: task_id.clone(),
            workflow: workflow.name.clone(),
        },
        json,
    );

    // Prepare workspace
    let worktree_path = if let Some(path) = args.use_worktree {
         if !path.exists() {
             return Err(anyhow::anyhow!("Worktree path does not exist: {}", path.display()));
         }
         if !json { eprintln!("Using existing worktree at {}", path.display()); }
         path
    } else if args.dry_run {
        // Skip worktree creation on dry run
        cwd.clone()
    } else {
        match DefaultWorktreeManager::new(&cwd) {
            Ok(manager) => {
                // Create worktree for this task
                match manager.create(&task_id, "1") {
                    Ok(worktree) => {
                        if !json {
                            eprintln!("Created git worktree at {}", worktree.path.display());
                        }
                        worktree.path
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "Could not create worktree, using simple directory");
                        // Fall back to simple directory
                        let path = cwd.join(".ckrv").join("tasks").join(&task_id).join("workspace");
                        std::fs::create_dir_all(&path)?;
                        path
                    }
                }
            }
            Err(_) => {
                // Not a git repo, use simple directory
                let path = cwd.join(".ckrv").join("tasks").join(&task_id).join("workspace");
                std::fs::create_dir_all(&path)?;
                path
            }
        }
    };

    let mut task = AgentTask::new(&task_id, &description, &workflow.name, worktree_path);

    // Handle dry run
    if args.dry_run {
        if !json {
            eprintln!("Dry run - showing workflow steps:");
            for (i, step) in workflow.steps.iter().enumerate() {
                eprintln!("  {}. {} ({})", i + 1, step.name, step.id);
            }
        }
        return Ok(());
    }

    // Save initial task state
    task.save(&cwd)?;

    // Create runner and execute
    let config = RunnerConfig {
        agent_binary: args.agent.clone(),
        use_sandbox: !args.no_sandbox,
        keep_container: args.keep_container,
        ..Default::default()
    };

    let runner = WorkflowRunner::new(config);

    // Run the workflow
    let result = runner.run(&workflow, &mut task, &cwd).await;

    match result {
        Ok(run_result) => {
            emit_event(
                &TaskEvent::Completed {
                    task_id: task_id.clone(),
                    duration_ms: run_result.duration_ms,
                    steps_completed: run_result.step_results.len(),
                },
                json,
            );

            if !json {
                if run_result.success {
                    ui.success(
                        "Task Complete",
                        &format!("Completed successfully in {}ms", run_result.duration_ms),
                    );
                } else {
                    ui.error("Task Complete", "Completed with some failures");
                }
                eprintln!("Results at: .ckrv/tasks/{}/", task_id);
            }

            if run_result.success {
                Ok(())
            } else {
                std::process::exit(1);
            }
        }
        Err(e) => {
            emit_event(
                &TaskEvent::Failed {
                    task_id: task_id.clone(),
                    error: e.to_string(),
                },
                json,
            );

            if !json {
                ui.error("Task Failed", &e.to_string());
            }

            std::process::exit(1);
        }
    }
}

/// Load a workflow by name or path.
fn load_workflow(name_or_path: &str, base_dir: &std::path::Path) -> Result<Workflow, anyhow::Error> {
    // Check if it's a file path
    let path = PathBuf::from(name_or_path);
    if path.exists() {
        return Ok(Workflow::load(&path)?);
    }

    // Check in .ckrv/workflows/
    let ckrv_path = base_dir
        .join(".ckrv")
        .join("workflows")
        .join(format!("{}.yml", name_or_path));
    if ckrv_path.exists() {
        return Ok(Workflow::load(&ckrv_path)?);
    }

    // Check in .ckrv/workflows/ with .yaml extension
    let ckrv_yaml_path = base_dir
        .join(".ckrv")
        .join("workflows")
        .join(format!("{}.yaml", name_or_path));
    if ckrv_yaml_path.exists() {
        return Ok(Workflow::load(&ckrv_yaml_path)?);
    }

    // Return error if not found
    Err(anyhow::anyhow!("Workflow '{}' not found. Create it in .ckrv/workflows/ as .yml or .yaml", name_or_path))
}

#[derive(Deserialize)]
struct TaskFile {
    tasks: Vec<SpecTask>,
}

#[derive(Deserialize)]
struct SpecTask {
    id: String,
    phase: String,
    title: String,
    description: String,
    file: String,
}
