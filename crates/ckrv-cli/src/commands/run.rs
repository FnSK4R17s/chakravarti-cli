//! Run command - execute a job based on a specification.
//!
//! This command generates an execution plan and orchestrates
//! multiple agent tasks to implement a feature.

use std::path::{Path, PathBuf};

use clap::{Args, ValueEnum};
use serde::{Deserialize, Serialize};

use futures::future::join_all;
use tokio::process::Command as AsyncCommand;
use std::sync::Arc;

use ckrv_core::{
    AgentTask, Workflow, WorkflowStep, OptimizeMode,
    runner::{RunnerConfig, WorkflowRunner, WorkflowRunResult},
};
use ckrv_git::{DefaultDiffGenerator, DefaultWorktreeManager, DiffGenerator, WorktreeManager};
use ckrv_metrics::{DefaultMetricsCollector, FileMetricsStorage, MetricsCollector, MetricsStorage};

use crate::ui::UiContext;
use crate::ui::Renderable;
use crate::ui::components::{Banner, RichTable, Panel};
use tabled::{
    builder::Builder,
    settings::{object::{Columns, Rows}, Modify, Width, Alignment},
};

/// Arguments for the run command.
#[derive(Args)]
pub struct RunArgs {
    /// Path to the specification file. If not provided, will detect from branch name.
    #[arg()]
    pub spec: Option<PathBuf>,

    /// Optimization strategy.
    #[arg(short, long, value_enum, default_value = "balanced")]
    pub optimize: OptimizeModeArg,

    /// Override the AI model/agent to use for execution.
    #[arg(short, long)]
    pub executor_model: Option<String>,

    /// Show the execution plan without running tasks.
    #[arg(long)]
    pub dry_run: bool,
}

/// Optimization strategy for CLI argument.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptimizeModeArg {
    /// Minimize API costs.
    Cost,
    /// Minimize execution time.
    Time,
    /// Balanced approach.
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

/// Task file structure.
#[derive(Serialize, Deserialize)]
struct TaskFile {
    tasks: Vec<SpecTask>,
}

/// Task structure in tasks.yaml.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct SpecTask {
    pub id: String,
    pub phase: String,
    pub title: String,
    pub description: String,
    pub file: Option<String>,
    pub status: String,
    pub user_story: Option<String>,
    pub parallel: bool,
}

/// Execution plan structure.
#[derive(Serialize, Deserialize, Debug)]
struct ExecutionPlan {
    batches: Vec<ExecutionBatch>,
}

/// A batch of tasks to be executed together.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct ExecutionBatch {
    id: String,
    name: String,
    task_ids: Vec<String>,
    #[serde(default)]
    depends_on: Vec<String>,
    reasoning: String,
}

/// Helper to always serialize depends_on
fn serialize_plan_with_depends_on(plan: &ExecutionPlan) -> String {
    let mut output = String::from("batches:\n");
    for batch in &plan.batches {
        output.push_str(&format!("  - id: \"{}\"\n", batch.id));
        output.push_str(&format!("    name: \"{}\"\n", batch.name));
        output.push_str("    task_ids:\n");
        for tid in &batch.task_ids {
            output.push_str(&format!("      - \"{}\"\n", tid));
        }
        if batch.depends_on.is_empty() {
            output.push_str("    depends_on: []\n");
        } else {
            output.push_str("    depends_on:\n");
            for dep in &batch.depends_on {
                output.push_str(&format!("      - \"{}\"\n", dep));
            }
        }
        output.push_str(&format!("    reasoning: \"{}\"\n\n", batch.reasoning.replace('"', "\\\"")));
    }
    output
}

/// JSON output for validation errors.
#[derive(Serialize)]
struct ValidationErrorOutput {
    field: String,
    message: String,
}

/// Execute the run command.
pub async fn execute(args: RunArgs, json: bool, ui: &UiContext) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    
    // Determine spec path: use provided arg, or detect from branch name
    let spec_path = if let Some(ref spec) = args.spec {
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
            .map_err(|e| anyhow::anyhow!("Failed to get current branch: {}", e))?;
        
        let branch_name = String::from_utf8_lossy(&branch_output.stdout).trim().to_string();
        
        if branch_name.is_empty() {
            return Err(anyhow::anyhow!("No spec provided and could not detect branch name. Run with a spec path or checkout a spec branch."));
        }
        
        // Look for spec in .specs/ directory matching branch name
        let specs_dir = cwd.join(".specs");
        let spec_dir = specs_dir.join(&branch_name);
        let spec_file = spec_dir.join("spec.yaml");
        
        if !spec_file.exists() {
            return Err(anyhow::anyhow!(
                "No spec provided and could not find .specs/{}/spec.yaml\nEither provide a spec path or checkout a branch matching a spec directory.",
                branch_name
            ));
        }
        
        if !json {
            println!("Auto-detected spec from branch '{}': {}", branch_name, spec_file.display());
        }
        
        spec_file
    };

    if !spec_path.exists() {
        return Err(anyhow::anyhow!("Spec file not found: {}", spec_path.display()));
    }

    if !json {
        println!("{}", Banner::new("CKRV RUN").subtitle(spec_path.display().to_string()).render(&ui.theme));
    }

    // Load and validate spec
    let loader = ckrv_spec::loader::YamlSpecLoader;
    let spec = ckrv_spec::loader::SpecLoader::load(&loader, &spec_path)
        .map_err(|e| anyhow::anyhow!("Failed to load spec: {}", e))?;

    let validation = ckrv_spec::validator::validate(&spec);
    if !validation.valid {
        if json {
            crate::commands::emit_json(
                serde_json::json!({
                    "success": false,
                    "error": "Validation failed",
                    "message": validation
                        .errors
                        .iter()
                        .map(|e| format!("{}: {}", e.field, e.message))
                        .collect::<Vec<_>>()
                        .join("; "),
                }),
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

    // Orchestration Logic: Run tasks from tasks.yaml
    let tasks_path = spec_path.parent().unwrap_or(&cwd).join("tasks.yaml");
    
    if !tasks_path.exists() {
        if !json {
            eprintln!("No tasks.yaml found at {}", tasks_path.display());
            eprintln!("Run `ckrv spec tasks` to generate tasks first.");
        }
        return Ok(());
    }

    if !json {
        println!("Loading tasks from {}", tasks_path.display());
    }
    
    let content = std::fs::read_to_string(&tasks_path)?;
    let file: TaskFile = serde_yaml::from_str(&content)
        .map_err(|e| anyhow::anyhow!("Failed to parse tasks.yaml: {}", e))?;
    
    let pending_tasks: Vec<_> = file.tasks.into_iter().filter(|t| t.status != "completed").collect();
    
    if pending_tasks.is_empty() {
         if !json { println!("All tasks are already completed."); }
         return Ok(());
    }
    
    if !json {
        println!("Found {} pending tasks.", pending_tasks.len());
        println!();
    }
    
    // Check for existing state/worktrees first
    let manager = DefaultWorktreeManager::new(&cwd).map_err(|e| anyhow::anyhow!("Init WT manager failed: {}", e))?;
    let existing_wts = manager.list()?;
    let batch_wts: Vec<_> = existing_wts.iter().filter(|wt| wt.job_id.starts_with("batch-")).collect();

    let plan_yaml_path = tasks_path.parent().unwrap_or(&cwd).join("plan.yaml");

    if !batch_wts.is_empty() {
        if !json {
            println!("Detected existing batch worktrees. Execution may already be in progress.");
            if plan_yaml_path.exists() {
                let content = std::fs::read_to_string(&plan_yaml_path)?;
                let plan: ExecutionPlan = serde_yaml::from_str(&content)?;
                
                println!("Current Execution Plan:");
                let mut builder = Builder::default();
                builder.push_record(["Batch", "Tasks", "Reasoning"]);
                for batch in &plan.batches {
                    builder.push_record([
                        batch.name.as_str(),
                        batch.task_ids.join(", ").as_str(),
                        batch.reasoning.as_str(),
                    ]);
                }
                let mut table = builder.build();
                let rich = RichTable::new(table);
                ui.print(rich);
            }
            println!("\nPlease clean up existing worktrees or finish the current run before starting a new one.");
        }
        return Ok(());
    }

    let plan: ExecutionPlan = if plan_yaml_path.exists() {
         if !json { println!("Found existing orchestration plan at {}", plan_yaml_path.display()); }
         let content = std::fs::read_to_string(&plan_yaml_path)?;
         serde_yaml::from_str(&content).map_err(|e| anyhow::anyhow!("Failed to parse plan at {}: {}", plan_yaml_path.display(), e))?
    } else {
        if !json {
            println!("Generating execution plan with Claude...");
        }

        let tasks_json = serde_json::to_string_pretty(&pending_tasks)?;
        let prompt_base = format!(r#"### ARCHITECTURAL PLANNER
Analyze these tasks and group them into logical execution batches.

DEPENDENCY MAPPING RULES:
1. Every batch MUST have 'id', 'name', 'task_ids', 'reasoning', and 'depends_on' fields.
2. 'depends_on' is a list of batch IDs this batch depends on.
3. If Batch B needs code created in Batch A, Batch B MUST have `depends_on: ["batch-a-id"]`.
4. For batches with no prerequisites, use `depends_on: []`.

Tasks:
{}

OUTPUT ONLY VALID YAML:
batches:
  - id: "foundation"
    name: "Core Infrastructure"
    task_ids: ["T001", "T002"]
    depends_on: []
    reasoning: "Standard setup."
  - id: "ui-components"
    name: "Component Development"
    task_ids: ["T003"]
    depends_on: ["foundation"]
    reasoning: "Depends on foundation."
"#, tasks_json);

        let plan_workflow = Workflow {
            version: "1.0".to_string(),
            name: "orchestrator-plan".to_string(),
            description: None,
            defaults: None,
            steps: vec![
                 WorkflowStep {
                     id: "plan".to_string(),
                     name: "Plan Execution".to_string(),
                     step_type: "agent".to_string(),
                     agent: None,
                     prompt: format!("{}\n\nIMPORTANT: Save the response as 'plan.yaml'. Include the depends_on field for EVERY batch.", prompt_base),
                     outputs: vec![
                         ckrv_core::StepOutput {
                             name: "plan_file".to_string(),
                             output_type: ckrv_core::OutputType::File,
                             description: Some("The generated plan yaml".to_string()),
                             filename: Some("plan.yaml".to_string()),
                         }
                     ],
                 }
            ],
        };

        let plan_id = format!("PLAN-{}", uuid::Uuid::new_v4().to_string().chars().take(8).collect::<String>());
        let plan_worktree = cwd.join(".ckrv").join("planning").join(&plan_id);
        std::fs::create_dir_all(&plan_worktree)?;

        let mut task = AgentTask::new(&plan_id, "Planning execution batches", "orchestrator-plan", plan_worktree.clone());
        task.save(&cwd)?;

        let config = RunnerConfig {
            agent_binary: "claude".to_string(),
            use_sandbox: !args.dry_run,
            keep_container: false,
            ..Default::default()
        };

        let mut task = AgentTask::new(
            "PLANNER",
            "Orchestration planning",
            "orchestrator-plan",
            plan_worktree.clone(),
        );

        let runner = WorkflowRunner::new(config);
        let result = runner.run(&plan_workflow, &mut task, &plan_worktree).await?;
        
        if !result.success {
             return Err(anyhow::anyhow!("Planning workflow failed."));
        }
        
        let plan_yaml_file = plan_worktree.join("plan.yaml");
        if !plan_yaml_file.exists() {
            return Err(anyhow::anyhow!("Agent failed to create plan.yaml"));
        }
        
        let content = std::fs::read_to_string(&plan_yaml_file)?;
        let mut plan: ExecutionPlan = serde_yaml::from_str(&content)
            .map_err(|e| anyhow::anyhow!("AI failed to generate valid YAML: {}\nContent: {}", e, content))?;

        // Auto-assign sequential dependencies if the AI didn't provide them
        // This guarantees that batches execute in order even if the AI forgets the field
        let batch_ids: Vec<String> = plan.batches.iter().map(|b| b.id.clone()).collect();
        for i in 1..plan.batches.len() {
            if plan.batches[i].depends_on.is_empty() {
                // Depend on the previous batch by default
                plan.batches[i].depends_on = vec![batch_ids[i - 1].clone()];
            }
        }

        // Debug: show computed dependencies
        if !json {
            println!("Computed Dependencies:");
            for batch in &plan.batches {
                let deps = if batch.depends_on.is_empty() { "[]".to_string() } else { batch.depends_on.join(", ") };
                println!("  {} -> {}", batch.id, deps);
            }
        }

        if let Some(parent) = tasks_path.parent() {
             let user_plan_path = parent.join("plan.yaml");
             let yaml = serialize_plan_with_depends_on(&plan);
             std::fs::write(&user_plan_path, &yaml).ok();
             if !json {
                 println!("Saved orchestration plan to {}", user_plan_path.display());
             }
        }
        plan
    };

    if !json {
        let parallel_count = plan.batches.iter().filter(|b| b.depends_on.is_empty()).count();
        println!("Execution Plan Ready: {} batches ({} parallelizable foundation batches)", plan.batches.len(), parallel_count);
        
        let mut builder = Builder::default();
        builder.push_record(["Batch", "Tasks", "Depends On", "Reasoning"]);
        for batch in &plan.batches {
            let deps = if batch.depends_on.is_empty() { "none".to_string() } else { batch.depends_on.join(", ") };
            builder.push_record([
                batch.name.clone(),
                batch.task_ids.join(", "),
                deps,
                batch.reasoning.clone(),
            ]);
        }
        let mut table = builder.build();
        table.with(Modify::new(Rows::first()).with(Alignment::center()));
        table.with(Modify::new(Columns::new(1..2)).with(Width::wrap(20))); // Tasks
        table.with(Modify::new(Columns::new(2..3)).with(Width::wrap(20))); // Depends On
        table.with(Modify::new(Columns::new(3..4)).with(Width::wrap(40))); // Reasoning
        let rich = RichTable::new(table);
        ui.print(rich);
        println!();
    }
    
    // Execute Plan with Dependency Awareness
    let exe = std::env::current_exe()?;
    
    // Map of batch_id -> task data
    let task_map: std::collections::HashMap<String, SpecTask> = pending_tasks.into_iter().map(|t| (t.id.clone(), t)).collect();

    // Map of batch_id -> task_ids (for updating status later)
    let batch_task_map: std::collections::HashMap<String, Vec<String>> = plan.batches.iter()
        .map(|b| (b.id.clone(), b.task_ids.clone()))
        .collect();

    let mut completed_batches = std::collections::HashSet::<String>::new();
    let mut pending_batches: std::collections::VecDeque<_> = plan.batches.into();
    let mut running_futures = futures::stream::FuturesUnordered::new();

    let exe_arc = std::sync::Arc::new(exe);
    let manager_arc = std::sync::Arc::new(manager);

    while !pending_batches.is_empty() || !running_futures.is_empty() {
        // 1. Identify and spawn unblocked batches
        let mut still_pending = std::collections::VecDeque::new();
        let mut spawned_any = false;

        while let Some(batch) = pending_batches.pop_front() {
            let unblocked = batch.depends_on.iter().all(|dep_id| completed_batches.contains(dep_id));
            
            if unblocked {
                let prefix = if args.dry_run { "[Simulated]" } else { "[Orchestrator]" };
                println!("{} Spawning batch: {}", prefix, batch.name);
                
                spawned_any = true;
                let exe = exe_arc.clone();
                let manager = manager_arc.clone();
                let args_executor_model = args.executor_model.clone();
                let args_dry_run = args.dry_run;
                let batch_name = batch.name.clone();
                let batch_id = batch.id.clone();
                let task_ids = batch.task_ids.clone();
                let reasoning = batch.reasoning.clone();

                // Build combined description
                let mut combined_desc = format!("MISSION: {}\nREASONING: {}\n\nTASKS:\n", batch_name, reasoning);
                for id in &task_ids {
                    if let Some(t) = task_map.get(id) {
                        combined_desc.push_str(&format!("- [{}]: {} ({})\n", t.id, t.title, t.description));
                    }
                }

                let handle = tokio::spawn(async move {
                    let mut worktree_path: Option<PathBuf> = None;
                    let mut worktree_branch = String::new();

                    if args_dry_run {
                        println!("[Batch: {}] WOULD execute mission in a new worktree.", batch_name);
                    } else {
                        let suffix: String = uuid::Uuid::new_v4().to_string().chars().take(6).collect();
                        let wt_job_id = format!("batch-{}-{}", batch_id, suffix); 
                        
                        let worktree = match manager.create(&wt_job_id, "1") {
                            Ok(wt) => wt,
                            Err(e) => {
                                eprintln!("[Batch: {}] Failed to create worktree: {}", batch_name, e);
                                return (batch_id, batch_name.clone(), None, Err(anyhow::anyhow!("Worktree for {} failed", batch_name)));
                            }
                        };
                        println!("[Batch: {}] EXECUTING MISSION in worktree: {}", batch_name, worktree.path.display());
                        worktree_path = Some(worktree.path.clone());
                        worktree_branch = worktree.branch;
                    }
                    
                    let mut cmd = AsyncCommand::new(exe.as_ref());
                    cmd.arg("task").arg(&combined_desc);
                    
                    if let Some(ref path) = worktree_path {
                        cmd.arg("--use-worktree").arg(path);
                    }
                    
                    let batch_run_id = format!("{}-run", batch_id);
                    cmd.arg("--continue-task").arg(&batch_run_id);
                    
                    if let Some(m) = &args_executor_model { cmd.arg("--agent").arg(m); }
                    if args_dry_run { cmd.arg("--dry-run"); }
                    
                    let status = match cmd.status().await {
                        Ok(s) => s,
                        Err(e) => return (batch_id, batch_name.clone(), worktree_path, Err(anyhow::anyhow!("Failed to run task: {}", e))),
                    };

                    if !status.success() {
                        return (batch_id, batch_name.clone(), worktree_path, Err(anyhow::anyhow!("Batch mission {} failed", batch_name)));
                    }
                    
                    let prefix = if args_dry_run { "[Simulated]" } else { "[Batch]" };
                    println!("{} Mission completed: {}", prefix, batch_name);
                    (batch_id, batch_name, worktree_path, Ok(worktree_branch))
                });
                
                running_futures.push(handle);
            } else {
                still_pending.push_back(batch);
            }
        }
        pending_batches = still_pending;

        // 2. Wait for at least one batch to complete if we are blocked
        if !running_futures.is_empty() {
            use futures::StreamExt;
            if let Some(result) = running_futures.next().await {
                match result {
                    Ok((id, name, wt_path_opt, Ok(branch))) => {
                        if !args.dry_run {
                            // 1. Commit changes in the worktree
                            if let Some(wt_path) = wt_path_opt {
                                println!("[Orchestrator] Committing changes for batch '{}'...", name);
                                let commit_msg = format!("feat(batch): {} - {}", name, id);
                                
                                let add_status = std::process::Command::new("git")
                                    .arg("add")
                                    .arg(".")
                                    .current_dir(&wt_path)
                                    .status()?;
                                
                                if !add_status.success() {
                                    return Err(anyhow::anyhow!("Failed to git add in worktree for batch {}", name));
                                }

                                // Check if there are changes to commit
                                let diff_status = std::process::Command::new("git")
                                    .args(["diff", "--staged", "--quiet"])
                                    .current_dir(&wt_path)
                                    .status()?;
                                
                                // Exit code 1 means differences exist (good to commit), 0 means empty
                                if !diff_status.success() {
                                    let commit_status = std::process::Command::new("git")
                                        .arg("commit")
                                        .arg("-m")
                                        .arg(&commit_msg)
                                        .current_dir(&wt_path)
                                        .status()?;

                                    if !commit_status.success() {
                                        return Err(anyhow::anyhow!("Failed to git commit in worktree for batch {}", name));
                                    }
                                } else {
                                    println!("[Orchestrator] No changes to commit for batch '{}'.", name);
                                }
                            }

                            // 2. Merge branch into current HEAD
                            if !branch.is_empty() {
                                println!("[Orchestrator] Merging batch '{}' ({}) into main branch...", name, branch);
                                // Using --no-ff to preserve batch history context
                                let merge_status = std::process::Command::new("git")
                                    .arg("merge")
                                    .arg("--no-ff")
                                    .arg("--no-edit")
                                    .arg(&branch)
                                    .current_dir(&cwd) // Merge into the repo root/current branch
                                    .status()?;

                                if !merge_status.success() {
                                    return Err(anyhow::anyhow!("Failed to merge batch branch {} into main branch. Please resolve conflicts manually.", branch));
                                }
                                println!("[Orchestrator] Successfully merged batch '{}'.", name);

                                // 3. Update tasks.yaml
                                if let Some(tids) = batch_task_map.get(&id) {
                                    if let Err(e) = mark_tasks_complete(&tasks_path, tids) {
                                        eprintln!("Failed to update tasks.yaml: {}", e);
                                    }
                                }
                            }
                        }

                        completed_batches.insert(id);
                    }
                    Ok((_id, name, _path, Err(e))) => {
                        return Err(anyhow::anyhow!("Batch '{}' failed: {}", name, e));
                    }
                    Err(e) => {
                        return Err(anyhow::anyhow!("Task panic: {}", e));
                    }
                }
            }
        } else if !pending_batches.is_empty() {
            // No futures running but still have pending batches = possible circular dependency or missing parent
            let pending_ids: Vec<_> = pending_batches.iter().map(|b| b.id.clone()).collect();
            return Err(anyhow::anyhow!("Scheduler deadlock: Pending batches {:?} are blocked by missing or failing dependencies.", pending_ids));
        }
    }

    Ok(())
}

fn mark_tasks_complete(tasks_path: &Path, task_ids: &[String]) -> anyhow::Result<()> {
    if task_ids.is_empty() { return Ok(()); }
    
    let content = std::fs::read_to_string(tasks_path)?;
    let mut file: TaskFile = serde_yaml::from_str(&content)?;

    let mut updated = false;
    for task in &mut file.tasks {
        if task_ids.contains(&task.id) {
            task.status = "completed".to_string();
            updated = true;
        }
    }

    if updated {
        let new_content = serde_yaml::to_string(&file)?;
        std::fs::write(tasks_path, new_content)?;
        println!("[Orchestrator] Marked {} tasks as completed in tasks.yaml", task_ids.len());
    }
    Ok(())
}
