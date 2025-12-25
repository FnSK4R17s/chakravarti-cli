//! Core domain primitives and orchestration for Chakravarti CLI.
//!
//! This crate contains the fundamental types and traits that define
//! the Chakravarti domain model: Spec, Plan, Job, Attempt, and RunState.

pub mod config;
pub mod error;
pub mod events;
pub mod job;
pub mod orchestrator;
pub mod plan;
pub mod planner;
pub mod spec;
pub mod state;
pub mod step;

pub use config::Config;
pub use error::CoreError;
pub use events::JobEvent;
pub use job::{Attempt, AttemptResult, Job, JobConfig, OptimizeMode};
pub use orchestrator::{
    DefaultOrchestrator, EventHandler, Orchestrator, OrchestratorError, OrchestratorResult,
};
pub use plan::Plan;
pub use planner::{DefaultPlanner, PlanContext, PlanError, Planner};
pub use spec::{Spec, VerifyConfig};
pub use state::RunState;
pub use step::{Step, StepStatus, StepType};
