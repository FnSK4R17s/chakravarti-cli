//! Core domain primitives and orchestration for Chakravarti CLI.
//!
//! This crate contains the fundamental types and traits that define
//! the Chakravarti domain model: Spec, Plan, Job, Attempt, and RunState.

// ============================================================
// MODULES
// ============================================================

/// Task entity for tracking workflow execution instances.
pub mod agent_task;
/// Configuration types for project settings.
pub mod config;
/// Core error types shared across the domain.
pub mod error;
/// Job events for progress tracking and observation.
pub mod events;
/// Job and Attempt lifecycle types.
pub mod job;
/// Orchestrator trait and default implementation for execution coordination.
pub mod orchestrator;
/// Execution plan as a directed acyclic graph of steps.
pub mod plan;
/// Planner trait and default implementation for generating plans from specs.
pub mod planner;
/// Prompt rendering using Handlebars templates.
pub mod prompt;
/// Workflow runner for executing multi-step agent workflows.
pub mod runner;
/// Specification type defining desired code changes.
pub mod spec;
/// Job run state machine with lifecycle transitions.
pub mod state;
/// Step types for plan execution.
pub mod step;
/// Step execution result for tracking workflow step outcomes.
pub mod step_result;
/// Workflow definition and parsing for multi-step agent orchestration.
pub mod workflow;

// ============================================================
// RE-EXPORTS
// ============================================================

/// Agent task types for workflow execution tracking.
pub use agent_task::{AgentTask, AgentTaskStatus, TaskError};
/// Project configuration.
pub use config::Config;
/// Core error enum.
pub use error::CoreError;
/// Job lifecycle events.
pub use events::JobEvent;
/// Job, attempt, and configuration types.
pub use job::{Attempt, AttemptResult, Job, JobConfig, OptimizeMode};
/// Orchestrator trait and supporting types.
pub use orchestrator::{
    DefaultOrchestrator, EventHandler, Orchestrator, OrchestratorError, OrchestratorResult,
};
/// Execution plan type.
pub use plan::Plan;
/// Planner trait and context types.
pub use planner::{DefaultPlanner, PlanContext, PlanError, Planner};
/// Prompt renderer and context types.
pub use prompt::{PromptRenderer, RenderContext, RenderError, StepOutputs};
/// Specification and verification config types.
pub use spec::{Spec, VerifyConfig};
/// Job run state machine.
pub use state::RunState;
/// Step, status, and type enums.
pub use step::{Step, StepStatus, StepType};
/// Step execution result and status types.
pub use step_result::{StepExecutionResult, StepExecutionStatus};
/// Workflow, step, and output types for YAML-based workflows.
pub use workflow::{
    OutputType, StepOutput, Workflow, WorkflowDefaults, WorkflowError, WorkflowStep,
};
