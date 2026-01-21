---
last_commit: c1bb442
last_updated: 2026-01-21
related_files:
  - src/lib.rs
  - src/orchestrator.rs
  - src/job.rs
  - src/spec.rs
  - src/plan.rs
---

# ckrv-core

Core domain primitives and orchestration engine for Chakravarti.

## Overview

This crate contains the fundamental types and traits that define the Chakravarti domain model. It implements the orchestration logic that transforms specs into executed code.

## Key Types

| Type | Purpose |
|------|---------|
| `Spec` | Feature specification to implement |
| `Plan` | Generated implementation plan |
| `Job` | Execution instance with attempts |
| `Attempt` | Single execution run |
| `Orchestrator` | Main execution engine |
| `Workflow` | Step-by-step execution plan |

## Domain Model

```
Spec (what to build)
  ↓
Plan (how to build it)
  ↓
Job (execution instance)
  ↓
Attempt (single run with result)
  ↓
RunState (current status)
```

## Module Structure

```
src/
├── spec.rs           # Spec type and loading
├── plan.rs           # Plan type and parsing
├── job.rs            # Job execution model
├── orchestrator.rs   # Main execution engine
├── workflow.rs       # Workflow step definitions
├── runner.rs         # Step runner implementation
├── planner.rs        # Plan generation
├── prompt.rs         # Prompt rendering
├── state.rs          # Execution state
├── step.rs           # Step types
└── events.rs         # Event system
```

## Usage

```rust
use ckrv_core::{Spec, Plan, Orchestrator, DefaultOrchestrator};

// Load a spec
let spec = Spec::from_file("spec.md")?;

// Create orchestrator and run
let mut orchestrator = DefaultOrchestrator::new(config);
let result = orchestrator.run(spec)?;
```

## Traits

### Orchestrator

```rust
pub trait Orchestrator {
    fn run(&mut self, spec: Spec) -> OrchestratorResult;
    fn on_event(&self, handler: impl EventHandler);
}
```

### Planner

```rust
pub trait Planner {
    fn generate_plan(&self, spec: &Spec, context: PlanContext) -> Result<Plan>;
}
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `ckrv-spec` | Spec parsing |
| `ckrv-sandbox` | Execution environment |
| `tokio` | Async runtime |
