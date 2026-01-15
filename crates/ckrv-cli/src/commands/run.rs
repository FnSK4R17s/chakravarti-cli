//! Run command - execute a job based on a specification.
//!
//! This command generates an execution plan and orchestrates
//! multiple agent tasks to implement a feature.

use std::path::{Path, PathBuf};
use std::time::Duration;

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
use ckrv_sandbox::{DockerSandbox, ExecuteConfig, Sandbox};

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

    /// Execute job in Chakravarti Cloud instead of locally.
    #[arg(long)]
    pub cloud: bool,

    /// Git credential name to use for cloud execution (for private repos).
    #[arg(long)]
    pub credential: Option<String>,
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
    #[serde(default = "default_complexity")]
    pub complexity: u8,
    #[serde(default)]
    pub model_tier: Option<String>,
    #[serde(default)]
    pub risk: Option<String>,
}

fn default_complexity() -> u8 { 3 }

/// Execution plan structure.
#[derive(Serialize, Deserialize, Debug)]
struct ExecutionPlan {
    batches: Vec<ExecutionBatch>,
}

/// Batch execution status
#[derive(Serialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
enum BatchStatus {
    #[default]
    Pending,
    Running,
    Completed,
    Failed,
}

// Custom deserializer that handles empty strings as Pending
impl<'de> serde::Deserialize<'de> for BatchStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "" | "pending" => Ok(BatchStatus::Pending),
            "running" => Ok(BatchStatus::Running),
            "completed" => Ok(BatchStatus::Completed),
            "failed" => Ok(BatchStatus::Failed),
            _ => Ok(BatchStatus::Pending), // Default to pending for unknown values
        }
    }
}

/// Model assignment for a batch
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct ModelAssignment {
    default: Option<String>,
    #[serde(default)]
    overrides: std::collections::HashMap<String, String>,
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
    #[serde(default)]
    status: BatchStatus,
    /// Branch name created for this batch (for resume)
    #[serde(default)]
    branch: Option<String>,
    
    // Enhanced Fields
    #[serde(default)]
    model_assignment: ModelAssignment,
    #[serde(default)]
    execution_strategy: Option<String>, // "parallel" | "sequential"
    #[serde(default)]
    estimated_cost: f64,
    #[serde(default)]
    estimated_time: String,
}

/// Helper to always serialize depends_on
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
        
        // Serialize model_assignment
        output.push_str("    model_assignment:\n");
        if let Some(default_model) = &batch.model_assignment.default {
            output.push_str(&format!("      default: \"{}\"\n", default_model));
        } else {
             output.push_str("      default: null\n");
        }
        
        if batch.model_assignment.overrides.is_empty() {
            output.push_str("      overrides: {}\n");
        } else {
            output.push_str("      overrides:\n");
            for (key, val) in &batch.model_assignment.overrides {
                output.push_str(&format!("        {}: \"{}\"\n", key, val));
            }
        }

        if let Some(strategy) = &batch.execution_strategy {
             output.push_str(&format!("    execution_strategy: \"{}\"\n", strategy));
        }

        output.push_str(&format!("    estimated_cost: {}\n", batch.estimated_cost));
        output.push_str(&format!("    estimated_time: \"{}\"\n", batch.estimated_time));

        output.push_str(&format!("    reasoning: \"{}\"\n", batch.reasoning.replace('"', "\\\"")));
        
        // Include status and branch for resume capability
        let status_str = match batch.status {
            BatchStatus::Pending => "pending",
            BatchStatus::Running => "running",
            BatchStatus::Completed => "completed",
            BatchStatus::Failed => "failed",
        };
        output.push_str(&format!("    status: {}\n", status_str));
        
        if let Some(ref branch) = batch.branch {
            output.push_str(&format!("    branch: \"{}\"\n", branch));
        }
        
        output.push('\n');
    }
    output
}

/// JSON output for validation errors.
#[derive(Serialize)]
struct ValidationErrorOutput {
    field: String,
    message: String,
}

/// Agent configuration from agent.yaml
#[derive(Serialize, Deserialize, Debug, Clone)]
struct AgentConfig {
    models: Vec<AgentModelDef>,
}

/// Model definition in agent.yaml
#[derive(Serialize, Deserialize, Debug, Clone)]
struct AgentModelDef {
    id: String,
    description: String,
}

fn load_agent_model_instructions(cwd: &Path) -> String {
    let locations = [cwd.join("agent.yaml"), cwd.join(".ckrv/agent.yaml")];
    for loc in &locations {
        if loc.exists() {
             if let Ok(content) = std::fs::read_to_string(loc) {
                 if let Ok(config) = serde_yaml::from_str::<AgentConfig>(&content) {
                     let mut instructions = String::new();
                     for model in config.models {
                         instructions.push_str(&format!("   - Use '{}' {}.\n", model.id, model.description));
                     }
                     if !instructions.is_empty() {
                         return instructions;
                     }
                 }
             }
        }
    }
    // Default fallback
    r#"   - Use 'minimax/minimax-m2.1' for light/standard tasks (Level 1-3).
   - Use 'z-ai/glm-4.7' for complex logic (Level 4).
   - Use 'claude' (default) if high reasoning/risk required (Level 5)."#.to_string()
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

    // ═══════════════════════════════════════════════════════════════════════════
    // CLOUD EXECUTION PATH - Dispatch to Chakravarti Cloud
    // ═══════════════════════════════════════════════════════════════════════════
    if args.cloud {
        return execute_cloud_job(&spec_path, args.credential.as_deref(), json, ui).await;
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
    
    let all_tasks = file.tasks.clone();
    let pending_tasks: Vec<_> = file.tasks.into_iter().filter(|t| t.status != "completed").collect();
    let spec_dir = spec_path.parent().unwrap_or(&cwd);
    let impl_path = spec_dir.join("implementation.yaml");
    let plan_yaml_path = spec_dir.join("plan.yaml");
    
    // Check for existing state/worktrees first
    let manager = DefaultWorktreeManager::new(&cwd).map_err(|e| anyhow::anyhow!("Init WT manager failed: {}", e))?;
    let existing_wts = manager.list()?;
    let batch_wts: Vec<_> = existing_wts.iter().filter(|wt| wt.job_id.starts_with("batch-")).collect();

    // ═══════════════════════════════════════════════════════════════════════════
    // COMPLETION CHECKLIST - Handle case where all tasks are already completed
    // ═══════════════════════════════════════════════════════════════════════════
    if pending_tasks.is_empty() {
        if !json {
            println!("\n📋 Completion Checklist");
            println!("   ✅ All {} tasks marked as completed", all_tasks.len());
        }
        
        // Check 1: Is implementation.yaml already created?
        if impl_path.exists() {
            if !json {
                println!("   ✅ Implementation summary exists");
                println!("\n✨ Run already completed! Nothing to do.");
                println!("   See: {}", impl_path.display());
            }
            return Ok(());
        }
        
        // Check 2: Are there any unmerged worktrees?
        if !batch_wts.is_empty() {
            if !json {
                println!("   ⚠️  Found {} unmerged worktrees", batch_wts.len());
                println!("\n🔄 Attempting to merge remaining worktrees...\n");
            }
            
            // Try to merge each worktree
            let mut merged_count = 0;
            for wt in &batch_wts {
                // Get the branch name from worktree
                let branch_output = std::process::Command::new("git")
                    .args(["rev-parse", "--abbrev-ref", "HEAD"])
                    .current_dir(&wt.path)
                    .output();
                
                let branch = match branch_output {
                    Ok(out) => String::from_utf8_lossy(&out.stdout).trim().to_string(),
                    Err(_) => continue,
                };
                
                if branch.is_empty() || branch == "HEAD" {
                    continue;
                }
                
                if !json {
                    println!("   Merging worktree: {} ({})", wt.job_id, branch);
                }
                
                // Commit any uncommitted changes in worktree
                let _ = std::process::Command::new("git")
                    .args(["add", "."])
                    .current_dir(&wt.path)
                    .status();
                
                let diff_status = std::process::Command::new("git")
                    .args(["diff", "--staged", "--quiet"])
                    .current_dir(&wt.path)
                    .status();
                
                if diff_status.map(|s| !s.success()).unwrap_or(false) {
                    let _ = std::process::Command::new("git")
                        .args(["commit", "-m", &format!("feat(batch): {} - finalize", wt.job_id)])
                        .current_dir(&wt.path)
                        .status();
                }
                
                // Try to merge into current branch
                let merge_status = std::process::Command::new("git")
                    .args(["merge", "--no-ff", "--no-edit", &branch])
                    .current_dir(&cwd)
                    .status();
                
                if merge_status.map(|s| s.success()).unwrap_or(false) {
                    merged_count += 1;
                    if !json {
                        println!("      ✅ Merged successfully");
                    }
                    
                    // Clean up worktree
                    let _ = std::process::Command::new("git")
                        .args(["worktree", "remove", "--force", &wt.path.to_string_lossy()])
                        .current_dir(&cwd)
                        .status();
                } else if has_merge_conflicts(&cwd) {
                    // Try AI resolution
                    if !json {
                        println!("      🤖 Conflict detected, attempting AI resolution...");
                    }
                    
                    match tokio::runtime::Handle::current().block_on(
                        resolve_conflicts_with_ai(&cwd, &branch, Some(spec_path.as_path()))
                    ) {
                        Ok(()) => {
                            merged_count += 1;
                            if !json {
                                println!("      ✅ Conflicts resolved and merged");
                            }
                            let _ = std::process::Command::new("git")
                                .args(["worktree", "remove", "--force", &wt.path.to_string_lossy()])
                                .current_dir(&cwd)
                                .status();
                        }
                        Err(e) => {
                            if !json {
                                println!("      ❌ Failed to merge: {}", e);
                            }
                            let _ = std::process::Command::new("git")
                                .args(["merge", "--abort"])
                                .current_dir(&cwd)
                                .status();
                        }
                    }
                } else {
                    if !json {
                        println!("      ❌ Merge failed (non-conflict error)");
                    }
                }
            }
            
            if !json {
                println!("\n   Merged {}/{} worktrees", merged_count, batch_wts.len());
            }
        } else {
            if !json {
                println!("   ✅ No unmerged worktrees");
            }
        }
        
        // Check 3: Create implementation.yaml since all tasks are done
        let branch_output = std::process::Command::new("git")
            .args(["branch", "--show-current"])
            .current_dir(&cwd)
            .output()?;
        let current_branch = String::from_utf8_lossy(&branch_output.stdout).trim().to_string();
        
        // Count batches from plan if it exists
        let batches_count = if plan_yaml_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&plan_yaml_path) {
                if let Ok(plan) = serde_yaml::from_str::<ExecutionPlan>(&content) {
                    plan.batches.len()
                } else { 0 }
            } else { 0 }
        } else { 0 };
        
        create_implementation_summary(
            spec_dir,
            &current_branch,
            all_tasks.len(),
            batches_count,
        )?;
        
        return Ok(());
    }
    
    if !json {
        println!("Found {} pending tasks.", pending_tasks.len());
        println!();
    }

    // Track if we're resuming from a previous run
    let resuming = !batch_wts.is_empty() && plan_yaml_path.exists();
    
    if resuming && !json {
        println!("🔄 Resuming previous execution run...");
        println!("   Found {} existing worktrees", batch_wts.len());
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
        let model_instructions = load_agent_model_instructions(&cwd);
        let prompt_base = format!(r#"### ARCHITECTURAL PLANNER
Analyze these tasks and group them into logical execution batches.

DEPENDENCY MAPPING RULES:
1. Every batch MUST have 'id', 'name', 'task_ids', 'reasoning', 'depends_on', and assignment fields.
2. 'depends_on' is a list of batch IDs this batch depends on.
3. If Batch B needs code created in Batch A, Batch B MUST have `depends_on: ["batch-a-id"]`.
4. For batches with no prerequisites, use `depends_on: []`.
5. 'model_assignment': Assign the best model based on task complexity/risk.
{}
6. 'execution_strategy': "parallel" if tasks within batch don't depend on each other, else "sequential".

Tasks:
{}

OUTPUT ONLY VALID YAML:
batches:
  - id: "foundation"
    name: "Core Infrastructure"
    task_ids: ["T001", "T002"]
    depends_on: []
    reasoning: "Standard setup."
    model_assignment:
      default: "minimax/minimax-m2.1"
      overrides: {{}}
    execution_strategy: "parallel"
    estimated_cost: 0.01
    estimated_time: "30s"
  - id: "ui-components"
    name: "Component Development"
    task_ids: ["T003"]
    depends_on: ["foundation"]
    reasoning: "Depends on foundation."
    model_assignment:
      default: "z-ai/glm-4.7"
      overrides: {{}}
    execution_strategy: "sequential"
    estimated_cost: 0.05
    estimated_time: "2m"
"#, model_instructions, tasks_json);

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
            use_sandbox: true,
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
    
    // Resume handling: process previously completed batches
    let mut mutable_plan = plan;
    if resuming {
        if !json {
            println!("\n📋 Checking batch status for resume...");
        }
        
        for batch in &mut mutable_plan.batches {
            // Check if this batch was already marked as completed
            if batch.status == BatchStatus::Completed {
                if !json {
                    println!("   ✅ Batch '{}' already completed, skipping", batch.name);
                }
                completed_batches.insert(batch.id.clone());
                continue;
            }
            
            // Check if there's a worktree for this batch that has uncommitted work
            let batch_prefix = format!("batch-{}-", batch.id);
            let batch_worktree = batch_wts.iter().find(|wt| wt.job_id.starts_with(&batch_prefix));
            
            if let Some(wt) = batch_worktree {
                // Check if this worktree has commits that haven't been merged
                let has_commits = std::process::Command::new("git")
                    .args(["log", "--oneline", "-1"])
                    .current_dir(&wt.path)
                    .output()
                    .map(|o| o.status.success() && !o.stdout.is_empty())
                    .unwrap_or(false);
                
                if has_commits && batch.branch.is_some() {
                    // Try to merge this completed worktree
                    let branch = batch.branch.as_ref().unwrap();
                    if !json {
                        println!("   🔀 Found incomplete batch '{}' with commits, attempting to merge...", batch.name);
                    }
                    
                    // Commit any uncommitted changes
                    let _ = std::process::Command::new("git")
                        .args(["add", "."])
                        .current_dir(&wt.path)
                        .status();
                    
                    let diff_status = std::process::Command::new("git")
                        .args(["diff", "--staged", "--quiet"])
                        .current_dir(&wt.path)
                        .status();
                    
                    if diff_status.map(|s| !s.success()).unwrap_or(false) {
                        let _ = std::process::Command::new("git")
                            .args(["commit", "-m", &format!("feat(batch): {} - recovered", batch.name)])
                            .current_dir(&wt.path)
                            .status();
                    }
                    
                    // Try to merge
                    let merge_status = std::process::Command::new("git")
                        .args(["merge", "--no-ff", "--no-edit", branch])
                        .current_dir(&cwd)
                        .status();
                    
                    if merge_status.map(|s| s.success()).unwrap_or(false) {
                        if !json {
                            println!("   ✅ Successfully merged batch '{}'", batch.name);
                        }
                        batch.status = BatchStatus::Completed;
                        completed_batches.insert(batch.id.clone());
                        
                        // Clean up worktree
                        let _ = std::process::Command::new("git")
                            .args(["worktree", "remove", "--force", &wt.path.to_string_lossy()])
                            .current_dir(&cwd)
                            .status();
                    } else if has_merge_conflicts(&cwd) {
                        // Try AI-assisted resolution
                        if !json {
                            println!("   🤖 Merge conflict detected, attempting AI resolution...");
                        }
                        let spec_path_ref: Option<&Path> = Some(spec_path.as_path());
                        match tokio::runtime::Handle::current().block_on(
                            resolve_conflicts_with_ai(&cwd, branch, spec_path_ref)
                        ) {
                            Ok(()) => {
                                if !json {
                                    println!("   ✅ Conflicts resolved, batch '{}' completed", batch.name);
                                }
                                batch.status = BatchStatus::Completed;
                                completed_batches.insert(batch.id.clone());
                                
                                let _ = std::process::Command::new("git")
                                    .args(["worktree", "remove", "--force", &wt.path.to_string_lossy()])
                                    .current_dir(&cwd)
                                    .status();
                            }
                            Err(e) => {
                                if !json {
                                    println!("   ⚠️  Could not auto-merge batch '{}': {}", batch.name, e);
                                }
                                let _ = std::process::Command::new("git")
                                    .args(["merge", "--abort"])
                                    .current_dir(&cwd)
                                    .status();
                                batch.status = BatchStatus::Failed;
                            }
                        }
                    } else {
                        if !json {
                            println!("   ⚠️  Batch '{}' merge failed, will retry", batch.name);
                        }
                        batch.status = BatchStatus::Failed;
                    }
                } else if has_commits {
                    // Worktree has commits but no branch recorded - mark as running
                    if !json {
                        println!("   ⏳ Batch '{}' has in-progress worktree", batch.name);
                    }
                    batch.status = BatchStatus::Running;
                }
            }
        }
        
        // Save updated plan with status
        if let Some(parent) = tasks_path.parent() {
            let user_plan_path = parent.join("plan.yaml");
            let yaml = serialize_plan_with_depends_on(&mutable_plan);
            std::fs::write(&user_plan_path, &yaml).ok();
        }
        
        if !json {
            let completed_count = completed_batches.len();
            let remaining = mutable_plan.batches.iter().filter(|b| b.status != BatchStatus::Completed).count();
            println!("\n   Resume summary: {} completed, {} remaining\n", completed_count, remaining);
        }
    }
    
    let mut pending_batches: std::collections::VecDeque<_> = mutable_plan.batches
        .into_iter()
        .filter(|b| b.status != BatchStatus::Completed)
        .collect();
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
                let prefix = "[Orchestrator]";
                println!("{} Spawning batch: {}", prefix, batch.name);
                
                spawned_any = true;
                let exe = exe_arc.clone();
                let manager = manager_arc.clone();
                let args_executor_model = args.executor_model.clone();
                let batch_name = batch.name.clone();
                let batch_id = batch.id.clone();
                let task_ids = batch.task_ids.clone();
                let reasoning = batch.reasoning.clone();

                // Build combined description & calculate max complexity
                let mut combined_desc = format!("MISSION: {}\nREASONING: {}\n\nTASKS:\n", batch_name, reasoning);
                let mut max_complexity: u8 = 1;

                for id in &task_ids {
                    if let Some(t) = task_map.get(id) {
                        combined_desc.push_str(&format!("- [{}]: {} ({})\n", t.id, t.title, t.description));
                        max_complexity = max_complexity.max(t.complexity);
                    }
                }
                
                // Intelligent Agent Selection
                // Priority: 1. CLI Override, 2. AI Plan (model_assignment), 3. Complexity Auto-Select
                let resolved_agent = if args.executor_model.is_none() {
                    let mut agent_id = None;
                    
                    // 1. Try Plan Assignment
                    if let Some(model_str) = &batch.model_assignment.default {
                        agent_id = find_agent_for_model_string(&cwd, model_str);
                        if let Some(ref id) = agent_id {
                             println!("   🧠 Plan-selected agent '{}' for model '{}', Batch Level {}", id, model_str, max_complexity);
                        }
                    } 
                    
                    // 2. Fallback to Complexity
                    if agent_id.is_none() {
                         agent_id = find_best_agent_for_level(&cwd, max_complexity);
                         if let Some(ref id) = agent_id {
                             println!("   🧠 Auto-selecting agent '{}' for Batch Level {}", id, max_complexity);
                         }
                    }
                    agent_id
                } else {
                    None
                };

                // Store plan path for status updates
                let plan_path = tasks_path.parent().unwrap_or(&cwd).join("plan.yaml");
                let plan_path_clone = plan_path.clone();
                let batch_id_for_status = batch_id.clone();
                
                let handle = tokio::spawn(async move {
                    let mut worktree_path: Option<PathBuf> = None;
                    let mut worktree_branch = String::new();

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
                    worktree_branch = worktree.branch.clone();
                    
                    // Update plan.yaml with running status and branch
                    let _ = update_batch_status(&plan_path_clone, &batch_id_for_status, BatchStatus::Running, Some(&worktree.branch));
                    
                    let mut cmd = AsyncCommand::new(exe.as_ref());
                    cmd.arg("task").arg(&combined_desc);
                    
                    if let Some(ref path) = worktree_path {
                        cmd.arg("--use-worktree").arg(path);
                    }
                    
                    let batch_run_id = format!("{}-run", batch_id);
                    cmd.arg("--continue-task").arg(&batch_run_id);
                    
                    // Prioritize CLI arg, then auto-resolved agent
                    let final_agent = args_executor_model.or(resolved_agent);
                    if let Some(m) = &final_agent { cmd.arg("--agent").arg(m); }
                    
                    let status = match cmd.status().await {
                        Ok(s) => s,
                        Err(e) => return (batch_id, batch_name.clone(), worktree_path, Err(anyhow::anyhow!("Failed to run task: {}", e))),
                    };

                    if !status.success() {
                        return (batch_id, batch_name.clone(), worktree_path, Err(anyhow::anyhow!("Batch mission {} failed", batch_name)));
                    }
                    
                    println!("[Batch] Mission completed: {}", batch_name);
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
                    Ok((id, name, ref wt_path_opt, Ok(branch))) => {
                        // 1. Commit changes in the worktree
                        if let Some(ref wt_path) = wt_path_opt {
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
                                // Check if it's a conflict that we can resolve with AI
                                if has_merge_conflicts(&cwd) {
                                    println!("[Orchestrator] Merge conflict detected, attempting AI-assisted resolution...");
                                    
                                    // Try to resolve conflicts with Claude Code
                                    let spec_path_ref: Option<&Path> = Some(spec_path.as_path());
                                    match tokio::runtime::Handle::current().block_on(
                                        resolve_conflicts_with_ai(&cwd, &branch, spec_path_ref)
                                    ) {
                                        Ok(()) => {
                                            println!("[Orchestrator] AI successfully resolved merge conflicts!");
                                        }
                                        Err(e) => {
                                            // Abort the merge and return error
                                            let _ = std::process::Command::new("git")
                                                .args(["merge", "--abort"])
                                                .current_dir(&cwd)
                                                .status();
                                            return Err(anyhow::anyhow!(
                                                "Failed to merge batch branch {} - AI conflict resolution failed: {}",
                                                branch, e
                                            ));
                                        }
                                    }
                                } else {
                                    return Err(anyhow::anyhow!(
                                        "Failed to merge batch branch {} into main branch. Please resolve manually.",
                                        branch
                                    ));
                                }
                            }
                            println!("[Orchestrator] Successfully merged batch '{}'.", name);

                            // 3. Update tasks.yaml
                            if let Some(tids) = batch_task_map.get(&id) {
                                if let Err(e) = mark_tasks_complete(&tasks_path, tids) {
                                    eprintln!("Failed to update tasks.yaml: {}", e);
                                }
                            }
                            
                            // 4. Update plan.yaml batch status
                            let plan_path = tasks_path.parent().unwrap_or(&cwd).join("plan.yaml");
                            let _ = update_batch_status(&plan_path, &id, BatchStatus::Completed, Some(&branch));
                            
                            // 5. Verify merge and cleanup worktree
                            // Check if the branch is now an ancestor of HEAD (merge was successful)
                            let verify_merge = std::process::Command::new("git")
                                .args(["merge-base", "--is-ancestor", &branch, "HEAD"])
                                .current_dir(&cwd)
                                .status();
                            
                            if verify_merge.map(|s| s.success()).unwrap_or(false) {
                                // Branch is merged, safe to remove worktree
                                if let Some(ref wt_path) = wt_path_opt {
                                    println!("[Orchestrator] Cleaning up worktree for batch '{}'...", name);
                                    let remove_result = std::process::Command::new("git")
                                        .args(["worktree", "remove", "--force", &wt_path.to_string_lossy()])
                                        .current_dir(&cwd)
                                        .status();
                                    
                                    if remove_result.map(|s| s.success()).unwrap_or(false) {
                                        println!("[Orchestrator] Worktree cleaned up ✓");
                                    } else {
                                        eprintln!("[Orchestrator] Warning: Could not remove worktree at {}", wt_path.display());
                                    }
                                }
                            }
                        }

                        completed_batches.insert(id);
                    }
                    Ok((failed_id, name, _path, Err(e))) => {
                        // Mark batch as failed in plan.yaml for potential retry
                        let plan_path = tasks_path.parent().unwrap_or(&cwd).join("plan.yaml");
                        let _ = update_batch_status(&plan_path, &failed_id, BatchStatus::Failed, None);
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

    // All batches completed successfully - create implementation summary
    if !completed_batches.is_empty() {
        // Get current branch name for the summary
        let branch_output = std::process::Command::new("git")
            .args(["branch", "--show-current"])
            .current_dir(&cwd)
            .output()?;
        let current_branch = String::from_utf8_lossy(&branch_output.stdout).trim().to_string();
        
        // Get the spec directory (parent of spec.yaml)
        if let Some(spec_dir) = spec_path.parent() {
            // Count total tasks that were completed
            let total_tasks = batch_task_map.values().map(|v| v.len()).sum();
            
            create_implementation_summary(
                spec_dir,
                &current_branch,
                total_tasks,
                completed_batches.len(),
            )?;
        }
    }

    Ok(())
}

/// Implementation summary structure written after successful run completion
#[derive(Serialize, Deserialize, Debug)]
struct ImplementationSummary {
    /// Status of the implementation
    status: String,
    /// Branch where all code has been merged
    branch: String,
    /// Timestamp of completion
    completed_at: String,
    /// Total number of tasks completed
    tasks_completed: usize,
    /// Total number of batches merged
    batches_merged: usize,
    /// Summary message
    message: String,
}

/// Create implementation.yaml after successful completion of all batches
fn create_implementation_summary(
    spec_dir: &Path,
    branch: &str,
    tasks_completed: usize,
    batches_merged: usize,
) -> anyhow::Result<()> {
    let summary = ImplementationSummary {
        status: "completed".to_string(),
        branch: branch.to_string(),
        completed_at: chrono::Utc::now().to_rfc3339(),
        tasks_completed,
        batches_merged,
        message: format!(
            "Implementation complete. {} tasks executed across {} batches. Code merged to branch '{}'.",
            tasks_completed, batches_merged, branch
        ),
    };

    let yaml = serde_yaml::to_string(&summary)?;
    let impl_path = spec_dir.join("implementation.yaml");
    std::fs::write(&impl_path, yaml)?;
    
    println!("\n✅ Implementation complete!");
    println!("   Summary saved to: {}", impl_path.display());
    println!("   Branch: {}", branch);
    println!("   Tasks: {} completed", tasks_completed);
    println!("   Ready for code review.\n");
    
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

/// Update batch status in plan.yaml for resume capability
fn update_batch_status(plan_path: &Path, batch_id: &str, status: BatchStatus, branch: Option<&str>) -> anyhow::Result<()> {
    if !plan_path.exists() {
        return Ok(());
    }
    
    let content = std::fs::read_to_string(plan_path)?;
    let mut plan: ExecutionPlan = serde_yaml::from_str(&content)?;
    
    for batch in &mut plan.batches {
        if batch.id == batch_id {
            batch.status = status;
            if let Some(b) = branch {
                batch.branch = Some(b.to_string());
            }
            break;
        }
    }
    
    let yaml = serialize_plan_with_depends_on(&plan);
    std::fs::write(plan_path, yaml)?;
    Ok(())
}

/// Check if there are merge conflicts in the current repository
fn has_merge_conflicts(cwd: &Path) -> bool {
    let output = std::process::Command::new("git")
        .args(["diff", "--name-only", "--diff-filter=U"])
        .current_dir(cwd)
        .output();
    
    match output {
        Ok(out) => !out.stdout.is_empty(),
        Err(_) => false,
    }
}

/// Get list of files with merge conflicts
fn get_conflicted_files(cwd: &Path) -> Vec<String> {
    let output = std::process::Command::new("git")
        .args(["diff", "--name-only", "--diff-filter=U"])
        .current_dir(cwd)
        .output();
    
    match output {
        Ok(out) => {
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .map(|s| s.to_string())
                .collect()
        }
        Err(_) => vec![],
    }
}

/// Resolve merge conflicts using Claude Code AI
async fn resolve_conflicts_with_ai(cwd: &Path, branch_name: &str, spec_path: Option<&Path>) -> anyhow::Result<()> {
    let conflicted_files = get_conflicted_files(cwd);
    
    if conflicted_files.is_empty() {
        return Ok(());
    }

    println!("\n🔀 Merge Conflict Detected!");
    println!("   Conflicting files:");
    for file in &conflicted_files {
        println!("     • {}", file);
    }
    println!("\n🤖 Invoking Claude Code to resolve conflicts...\n");

    // Read spec context if available
    let spec_context = if let Some(path) = spec_path {
        std::fs::read_to_string(path).unwrap_or_default()
    } else {
        String::new()
    };

    // Read the actual content of conflicted files so Claude can see them
    let mut file_contents = String::new();
    for file in &conflicted_files {
        let file_path = cwd.join(file);
        if let Ok(content) = std::fs::read_to_string(&file_path) {
            file_contents.push_str(&format!("\n=== {} ===\n{}\n", file, content));
        }
    }

    // Build prompt for Claude - use interactive mode so Claude can edit files
    let prompt = format!(
        r#"You are resolving Git merge conflicts. Your ONLY task is to edit the conflicting files to resolve the conflicts.

BRANCH BEING MERGED: {branch}

SPEC CONTEXT (what we're building):
{spec}

CONFLICTING FILES AND THEIR CURRENT CONTENT:
{files}

YOUR TASK:
1. For each file above, find the conflict markers: <<<<<<<, =======, >>>>>>>
2. EDIT each file to create a merged version that:
   - Removes ALL conflict markers
   - Preserves functionality from BOTH sides (HEAD and incoming)
   - Combines imports, adds all features from both branches
   - Does NOT duplicate code that exists in both sides
3. After editing all files, run: git add -A

CRITICAL RULES:
- You MUST edit the actual files using your file editing tools
- You MUST remove ALL conflict markers (<<<<<<<, =======, >>>>>>>)
- You MUST preserve features from both HEAD and incoming changes
- After editing, stage all files with: git add -A

Start by editing the first conflicted file now."#,
        branch = branch_name,
        spec = if spec_context.is_empty() { "(No spec provided)".to_string() } else { spec_context },
        files = file_contents
    );

    // Run Claude in Docker sandbox without -p flag so it can use tools to edit files
    let sandbox = DockerSandbox::new(ckrv_sandbox::DefaultAllowList::default())
        .map_err(|e| anyhow::anyhow!("Failed to create sandbox for conflict resolution: {}", e))?;

    // Use Claude in interactive mode with the prompt passed via stdin-like mechanism
    // Using --print flag to print the prompt and let Claude process it with tool access
    let escaped_prompt = shell_escape::escape(prompt.into());
    let command = format!(
        "echo {} | claude --dangerously-skip-permissions",
        escaped_prompt
    );

    let config = ExecuteConfig::new("", cwd.to_path_buf())
        .shell(&command)
        .with_timeout(Duration::from_secs(300)); // 5 minutes for conflict resolution

    let result = sandbox.execute(config).await
        .map_err(|e| anyhow::anyhow!("AI conflict resolution failed: {}", e))?;

    // Log output for debugging
    if !result.stdout.is_empty() {
        println!("[AI Resolution] {}", result.stdout);
    }

    if !result.success() {
        // Don't fail immediately - check if conflicts are resolved anyway
        eprintln!("[AI Resolution] Command returned non-zero: {}", result.stderr);
    }

    // Check if conflicts are resolved
    if has_merge_conflicts(cwd) {
        // Try one more time with a simpler approach - just accept incoming changes
        println!("⚠️  Some conflicts remain. Attempting fallback resolution...");
        
        for file in &conflicted_files {
            let _ = std::process::Command::new("git")
                .args(["checkout", "--theirs", file])
                .current_dir(cwd)
                .status();
            let _ = std::process::Command::new("git")
                .args(["add", file])
                .current_dir(cwd)
                .status();
        }
    }

    // Final check
    if has_merge_conflicts(cwd) {
        return Err(anyhow::anyhow!(
            "Could not automatically resolve all conflicts. Please resolve manually:\n  {}",
            conflicted_files.join("\n  ")
        ));
    }

    println!("✅ Conflicts resolved successfully!");
    
    // Stage all changes
    let _ = std::process::Command::new("git")
        .args(["add", "-A"])
        .current_dir(cwd)
        .status();
    
    // Complete the merge
    let commit_status = std::process::Command::new("git")
        .args(["commit", "--no-edit"])
        .current_dir(cwd)
        .status()?;

    if !commit_status.success() {
        // Might already be committed or no changes
        let _ = std::process::Command::new("git")
            .args(["commit", "-m", &format!("Merge {} with AI-assisted conflict resolution", branch_name)])
            .current_dir(cwd)
            .status();
    }

    Ok(())
}

/// Execute a job in Chakravarti Cloud
async fn execute_cloud_job(
    spec_path: &Path,
    credential_name: Option<&str>,
    json: bool,
    ui: &UiContext,
) -> anyhow::Result<()> {
    use crate::cloud::client::CloudClient;
    use crate::cloud::jobs;

    // Read spec content
    let spec_content = std::fs::read_to_string(spec_path)?;

    // Get git remote URL
    let cwd = std::env::current_dir()?;
    let remote_output = std::process::Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(&cwd)
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to get git remote: {}", e))?;

    let git_repo_url = String::from_utf8_lossy(&remote_output.stdout).trim().to_string();
    if git_repo_url.is_empty() {
        return Err(anyhow::anyhow!(
            "No git remote 'origin' found. Cloud execution requires a remote repository."
        ));
    }

    // Get current branch
    let branch_output = std::process::Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(&cwd)
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to get current branch: {}", e))?;

    let git_base_branch = String::from_utf8_lossy(&branch_output.stdout).trim().to_string();
    if git_base_branch.is_empty() {
        return Err(anyhow::anyhow!("Could not determine current branch."));
    }

    if !json {
        println!("☁️  Dispatching job to Chakravarti Cloud...");
        println!("   Repository: {}", git_repo_url);
        println!("   Branch: {}", git_base_branch);
        if let Some(cred) = credential_name {
            println!("   Credential: {}", cred);
        }
    }

    // Create cloud client and dispatch job
    let client = CloudClient::new().map_err(|e| {
        if e.to_string().contains("Not authenticated") {
            anyhow::anyhow!("Not authenticated. Run 'ckrv cloud login' first.")
        } else {
            anyhow::anyhow!("{}", e)
        }
    })?;

    let job = client
        .create_job(&spec_content, &git_repo_url, &git_base_branch, credential_name)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to dispatch job: {}", e))?;

    if json {
        println!("{}", serde_json::to_string_pretty(&job)?);
    } else {
        ui.success("Job Dispatched", &format!("Job ID: {}", job.id));
        println!();
        println!("📋 Track progress:");
        println!("   ckrv status {}", job.id);
        println!();
        println!("📝 Stream logs:");
        println!("   ckrv logs {} --follow", job.id);
        println!();
        println!("📦 Pull results when complete:");
        println!("   ckrv pull {}", job.id);
    }

    Ok(())
}

#[derive(Debug, serde::Deserialize)]
struct OpenRouterConfigLite {
    model: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct AgentConfigLite {
    id: String,
    name: String,
    #[serde(default)]
    level: u8,
    #[serde(default)]
    enabled: bool,
    openrouter: Option<OpenRouterConfigLite>,
}

#[derive(Debug, serde::Deserialize)]
struct AgentsFileLite {
    agents: Vec<AgentConfigLite>,
}

/// Find an agent ID that matches a model string (e.g. "minimax/minimax-m2.1")
fn find_agent_for_model_string(cwd: &Path, model_string: &str) -> Option<String> {
    let agents_path = dirs::config_dir()
        .map(|d| d.join("chakravarti").join("agents.yaml"))
        .filter(|p| p.exists())
        .unwrap_or_else(|| cwd.join(".chakravarti").join("agents.yaml"));

    if !agents_path.exists() { return None; }

    let content = match std::fs::read_to_string(agents_path) {
        Ok(c) => c,
        Err(_) => return None,
    };

    let config: AgentsFileLite = match serde_yaml::from_str(&content) {
        Ok(c) => c,
        Err(_) => return None,
    };

    config.agents.iter()
        .filter(|a| a.enabled)
        .find(|a| {
            // Match ID, Name, or OpenRouter Model
            a.id == model_string || 
            a.name == model_string || 
            a.openrouter.as_ref().map(|o| o.model.as_deref() == Some(model_string)).unwrap_or(false)
        })
        .map(|a| a.id.clone())
}

/// Find the best agent for a given complexity level.
/// Strategy: Find the lowest level agent that is >= required level.
/// If no agent meets the requirement, return the highest available level.
fn find_best_agent_for_level(cwd: &Path, required_level: u8) -> Option<String> {
    // Check global path first
    let agents_path = dirs::config_dir()
        .map(|d| d.join("chakravarti").join("agents.yaml"))
        .filter(|p| p.exists())
        .unwrap_or_else(|| cwd.join(".chakravarti").join("agents.yaml"));

    if !agents_path.exists() {
        return None;
    }

    let content = match std::fs::read_to_string(agents_path) {
        Ok(c) => c,
        Err(_) => return None,
    };

    let config: AgentsFileLite = match serde_yaml::from_str(&content) {
        Ok(c) => c,
        Err(_) => return None,
    };

    let enabled_agents: Vec<&AgentConfigLite> = config.agents.iter().filter(|a| a.enabled).collect();
    if enabled_agents.is_empty() {
        return None;
    }

    // 1. Try to find agents with level >= required
    let mut sufficient_agents: Vec<&AgentConfigLite> = enabled_agents.iter()
        .filter(|a| a.level >= required_level)
        .map(|&a| a)
        .collect();

    if !sufficient_agents.is_empty() {
        // Sort by level ascending (pick the "cheapest" one that is good enough)
        sufficient_agents.sort_by_key(|a| a.level);
        return Some(sufficient_agents[0].id.clone());
    }

    // 2. Fallback: pick the strongest available agent
    let mut all_agents = enabled_agents;
    all_agents.sort_by_key(|a| a.level); // Sort ascending
    all_agents.last().map(|a| a.id.clone())
}
