---
last_commit: 5160ff1
last_updated: 2026-01-29
related_files:
  - src/lib.rs
  - src/orchestrator.rs
  - src/runner.rs
  - src/agent_task.rs
  - src/workflow.rs
---

# ckrv-core

Core domain primitives and orchestration engine for Chakravarti.

## Overview

This crate contains the fundamental types and traits that define the Chakravarti domain model. It implements the orchestration logic that transforms specs into executed code via multi-step agent workflows.

## Key Types

| Type | Size | Purpose |
|------|------|---------|
| `Spec` | 4KB | Feature specification with validation |
| `Plan` | 5KB | Generated implementation plan |
| `Job` | 8KB | Execution instance with attempts |
| `AgentTask` | 7KB | Workflow execution tracking |
| `Workflow` | 8KB | Step definitions and routing |
| `WorkflowRunner` | 18KB | Step executor with agent invocation |
| `Orchestrator` | 11KB | Main execution engine trait |

## Domain Model

```
Spec (what to build)
  ↓
Plan (how to build it)
  ↓
Job → Attempt → AttemptResult
  ↓
AgentTask (workflow execution)
  ↓
WorkflowStep → StepExecutionResult
  ↓
RunState (current status)
```

## Module Structure

```
src/
├── lib.rs            # Public exports (20+ types)
├── spec.rs           # Spec type, VerifyConfig, validation
├── plan.rs           # Plan type and parsing
├── job.rs            # Job, Attempt, AttemptResult, JobConfig
├── orchestrator.rs   # Orchestrator trait, DefaultOrchestrator
├── workflow.rs       # Workflow, WorkflowStep, WorkflowDefaults
├── runner.rs         # WorkflowRunner, RunnerConfig (18KB)
├── agent_task.rs     # AgentTask, AgentTaskStatus, TaskError
├── planner.rs        # Planner trait, DefaultPlanner
├── prompt.rs         # PromptRenderer, RenderContext
├── state.rs          # RunState execution state
├── step.rs           # Step, StepType, StepStatus
├── step_result.rs    # StepExecutionResult, StepExecutionStatus
├── config.rs         # Config type
├── events.rs         # JobEvent system
└── error.rs          # CoreError type
```

## Public API

All exports from `lib.rs`:

```rust
// Domain types
pub use spec::{Spec, VerifyConfig};
pub use plan::Plan;
pub use job::{Job, Attempt, AttemptResult, JobConfig, OptimizeMode};
pub use agent_task::{AgentTask, AgentTaskStatus, TaskError};
pub use state::RunState;
pub use step::{Step, StepType, StepStatus};
pub use step_result::{StepExecutionResult, StepExecutionStatus};

// Workflows
pub use workflow::{Workflow, WorkflowStep, WorkflowDefaults, StepOutput, OutputType, WorkflowError};

// Orchestration
pub use orchestrator::{Orchestrator, DefaultOrchestrator, OrchestratorResult, OrchestratorError, EventHandler};
pub use planner::{Planner, DefaultPlanner, PlanContext, PlanError};
pub use prompt::{PromptRenderer, RenderContext, RenderError, StepOutputs};

// Core
pub use config::Config;
pub use events::JobEvent;
pub use error::CoreError;
```

## Usage

### Creating Tasks

```rust
use ckrv_core::{AgentTask, AgentTaskStatus};
use std::path::PathBuf;

let mut task = AgentTask::new(
    AgentTask::generate_id(),  // e.g., "task-a1b2c3"
    "Create a new REST API endpoint",
    "swe",
    PathBuf::from("/path/to/worktree"),
);

task.set_status(AgentTaskStatus::Running);
task.save(base_dir)?;

// Later...
let loaded = AgentTask::load(base_dir, "task-a1b2c3")?;
```

### Orchestration

```rust
use ckrv_core::{DefaultOrchestrator, DefaultPlanner, Spec, JobConfig};

let planner = DefaultPlanner::new();
let orchestrator = DefaultOrchestrator::new(planner, repo_root);

let result = orchestrator.run(spec, JobConfig::default()).await?;

if result.success {
    println!("Job completed: {:?}", result.job);
}
```

### Workflow Execution

```rust
use ckrv_core::{WorkflowRunner, RunnerConfig, Workflow};

let workflow = Workflow::from_file("workflow.yaml")?;
let runner = WorkflowRunner::new(RunnerConfig::default());

let result = runner.run(&workflow, &mut task, base_dir)?;
for step in result.step_results {
    println!("{}: {:?}", step.step_id, step.status);
}
```

### RunnerConfig

The `RunnerConfig` struct configures workflow execution:

```rust
use ckrv_core::RunnerConfig;

let config = RunnerConfig {
    agent_binary: "claude".to_string(),
    use_sandbox: true,           // Use Docker sandbox (recommended)
    keep_container: false,       // Keep container after execution
    
    // OpenRouter configuration
    openrouter_api_key: Some("sk-or-...".to_string()),
    openrouter_model: Some("provider/model".to_string()),
    openrouter_base_url: None,   // Defaults to https://openrouter.ai/api
    
    // GLM Coding Plan configuration
    glm_api_key: Some("your-zai-key".to_string()),
    glm_model: Some("glm-4.7".to_string()),
    glm_timeout_ms: Some(3_000_000),  // 50 minutes
    
    ..Default::default()
};
```

| Field | Type | Description |
|-------|------|-------------|
| `agent_binary` | `String` | CLI binary name (usually "claude") |
| `use_sandbox` | `bool` | Run in Docker sandbox |
| `openrouter_api_key` | `Option<String>` | OpenRouter API key |
| `openrouter_model` | `Option<String>` | OpenRouter model ID |
| `glm_api_key` | `Option<String>` | Z.AI GLM API key |
| `glm_model` | `Option<String>` | GLM model ID (e.g., "glm-4.7") |
| `glm_timeout_ms` | `Option<u32>` | Timeout in ms (default: 3,000,000) |

## Traits

### Orchestrator

```rust
#[async_trait]
pub trait Orchestrator {
    async fn run(
        &self,
        spec: Spec,
        config: JobConfig,
    ) -> Result<OrchestratorResult, OrchestratorError>;
}
```

### Planner

```rust
pub trait Planner: Send + Sync {
    fn generate_plan(&self, spec: &Spec, context: PlanContext) -> Result<Plan, PlanError>;
}
```

### EventHandler

```rust
pub trait EventHandler: Send + Sync {
    fn handle(&self, event: JobEvent);
}
```

## Errors

| Error Type | Module | Purpose |
|------------|--------|---------|
| `CoreError` | error.rs | General core errors |
| `OrchestratorError` | orchestrator.rs | Orchestration failures |
| `PlanError` | planner.rs | Plan generation failures |
| `RunnerError` | runner.rs | Workflow execution failures |
| `TaskError` | agent_task.rs | Task persistence errors |
| `WorkflowError` | workflow.rs | Workflow parsing errors |

## Dependencies

| Crate | Purpose |
|-------|---------|
| `chrono` | Timestamps for tasks |
| `serde` | Serialization |
| `uuid` | Task ID generation |
| `tokio` | Async runtime |
| `async-trait` | Async trait support |
| `thiserror` | Error handling |
