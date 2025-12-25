//! Orchestrator for coordinating the execution lifecycle.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;

use crate::{
    events::JobEvent,
    job::{AttemptResult, Job, JobConfig},
    planner::{PlanContext, PlanError, Planner},
    Plan, RunState, Spec, Step, StepStatus,
};

/// Orchestrator coordinates the full job lifecycle.
#[async_trait]
pub trait Orchestrator: Send + Sync {
    /// Run a job from a specification.
    ///
    /// # Errors
    ///
    /// Returns an error if orchestration fails.
    async fn run(
        &self,
        spec: Spec,
        config: JobConfig,
    ) -> Result<OrchestratorResult, OrchestratorError>;
}

/// Result of orchestration.
#[derive(Debug, Clone)]
pub struct OrchestratorResult {
    /// The job that was executed.
    pub job: Job,
    /// Path to the diff file if successful.
    pub diff_path: Option<PathBuf>,
    /// Total execution time in milliseconds.
    pub duration_ms: u64,
    /// Number of attempts made.
    pub attempts: u32,
}

/// Errors from orchestration.
#[derive(Debug, thiserror::Error)]
pub enum OrchestratorError {
    /// Planning failed.
    #[error("Planning failed: {0}")]
    PlanningFailed(#[from] PlanError),

    /// Execution failed.
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    /// Verification failed.
    #[error("Verification failed: {0}")]
    VerificationFailed(String),

    /// Max attempts exceeded.
    #[error("Max attempts exceeded ({attempts})")]
    MaxAttemptsExceeded {
        /// Number of attempts made.
        attempts: u32,
    },

    /// Git error.
    #[error("Git error: {0}")]
    GitError(String),

    /// Sandbox error.
    #[error("Sandbox error: {0}")]
    SandboxError(String),
}

/// Event handler for orchestrator progress updates.
pub trait EventHandler: Send + Sync {
    /// Handle a job event.
    fn handle(&self, event: JobEvent);
}

/// Default event handler that logs events.
pub struct LoggingEventHandler;

impl EventHandler for LoggingEventHandler {
    fn handle(&self, event: JobEvent) {
        tracing::info!(?event, "Job event");
    }
}

/// Default orchestrator implementation.
pub struct DefaultOrchestrator<P: Planner> {
    planner: P,
    event_handler: Arc<dyn EventHandler>,
    repo_root: PathBuf,
}

impl<P: Planner> DefaultOrchestrator<P> {
    /// Create a new orchestrator.
    pub fn new(planner: P, repo_root: PathBuf) -> Self {
        Self {
            planner,
            event_handler: Arc::new(LoggingEventHandler),
            repo_root,
        }
    }

    /// Set the event handler.
    pub fn with_event_handler(mut self, handler: Arc<dyn EventHandler>) -> Self {
        self.event_handler = handler;
        self
    }

    fn emit(&self, event: JobEvent) {
        self.event_handler.handle(event);
    }

    async fn execute_step(
        &self,
        step: &mut Step,
        _worktree_path: &PathBuf,
    ) -> Result<(), OrchestratorError> {
        let start = Instant::now();

        self.emit(JobEvent::StepStarted {
            step_id: step.id.clone(),
        });

        // For now, simulate step execution
        // In full implementation, this would:
        // 1. For Analyze: gather repo context
        // 2. For Generate: call model to generate code
        // 3. For Execute: apply changes via sandbox
        // 4. For Test: run verification

        // Simulate success
        step.status = StepStatus::Completed;
        step.duration_ms = Some(start.elapsed().as_millis() as u64);

        self.emit(JobEvent::StepCompleted {
            step_id: step.id.clone(),
            duration_ms: step.duration_ms.unwrap_or(0),
        });

        Ok(())
    }

    async fn execute_plan(
        &self,
        plan: &mut Plan,
        worktree_path: &PathBuf,
    ) -> Result<(), OrchestratorError> {
        // Get steps in execution order
        let step_order: Vec<String> = plan.steps.iter().map(|s| s.id.clone()).collect();

        for step_id in step_order {
            // Find and execute step
            if let Some(step) = plan.steps.iter_mut().find(|s| s.id == step_id) {
                self.execute_step(step, worktree_path).await?;
            }
        }

        Ok(())
    }

    async fn run_attempt(
        &self,
        job: &mut Job,
        plan: &mut Plan,
    ) -> Result<AttemptResult, OrchestratorError> {
        let attempt_num = job.attempt_count() + 1;

        self.emit(JobEvent::AttemptStarted {
            number: attempt_num,
        });

        // Create worktree for this attempt
        let worktree_path = self
            .repo_root
            .join(".chakravarti")
            .join("worktrees")
            .join(&job.id)
            .join(format!("attempt-{attempt_num}"));

        // For now, just create the directory
        // In full implementation, this would use ckrv-git to create actual worktree
        std::fs::create_dir_all(&worktree_path)
            .map_err(|e| OrchestratorError::GitError(e.to_string()))?;

        // Execute plan
        match self.execute_plan(plan, &worktree_path).await {
            Ok(()) => {
                let result = AttemptResult::success("All steps completed");
                self.emit(JobEvent::AttemptCompleted {
                    number: attempt_num,
                    result: result.clone(),
                });
                Ok(result)
            }
            Err(e) => {
                let result = AttemptResult::failure(&e.to_string());
                self.emit(JobEvent::AttemptCompleted {
                    number: attempt_num,
                    result: result.clone(),
                });
                Err(e)
            }
        }
    }
}

#[async_trait]
impl<P: Planner + 'static> Orchestrator for DefaultOrchestrator<P> {
    async fn run(
        &self,
        spec: Spec,
        config: JobConfig,
    ) -> Result<OrchestratorResult, OrchestratorError> {
        let start = Instant::now();
        let mut job = Job::new(spec.id.clone(), config.clone());

        self.emit(JobEvent::StateChanged {
            state: RunState::Planning,
        });

        // Generate plan
        let context = PlanContext::from_repo(&self.repo_root);
        let mut plan = self.planner.plan(&spec, &context).await?;

        self.emit(JobEvent::StateChanged {
            state: RunState::Executing {
                attempt: 1,
                step: "starting".to_string(),
            },
        });

        // Retry loop
        let mut last_error = None;
        for attempt in 1..=config.max_attempts {
            match self.run_attempt(&mut job, &mut plan).await {
                Ok(result) => {
                    job.add_attempt(result);

                    // Success!
                    let duration_ms = start.elapsed().as_millis() as u64;
                    let diff_path = self
                        .repo_root
                        .join(".chakravarti")
                        .join("runs")
                        .join(&job.id)
                        .join("diff.patch");

                    // Create runs directory
                    if let Some(parent) = diff_path.parent() {
                        std::fs::create_dir_all(parent).ok();
                    }

                    self.emit(JobEvent::StateChanged {
                        state: RunState::Succeeded {
                            attempt,
                            diff_path: diff_path.clone(),
                        },
                    });

                    return Ok(OrchestratorResult {
                        job,
                        diff_path: Some(diff_path),
                        duration_ms,
                        attempts: attempt,
                    });
                }
                Err(e) => {
                    job.add_attempt(AttemptResult::failure(&e.to_string()));
                    last_error = Some(e);

                    // Check if we should retry
                    if attempt < config.max_attempts {
                        tracing::info!(attempt, max = config.max_attempts, "Retrying...");
                        // Could implement replan logic here for T103
                    }
                }
            }
        }

        // All attempts failed
        self.emit(JobEvent::StateChanged {
            state: RunState::Failed {
                attempts: config.max_attempts,
                last_error: last_error
                    .as_ref()
                    .map_or("Unknown".to_string(), |e| e.to_string()),
            },
        });

        Err(OrchestratorError::MaxAttemptsExceeded {
            attempts: config.max_attempts,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planner::DefaultPlanner;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_orchestrator_basic_run() {
        let dir = TempDir::new().expect("temp dir");
        std::fs::create_dir_all(dir.path().join(".chakravarti")).ok();

        let planner = DefaultPlanner::new();
        let orchestrator = DefaultOrchestrator::new(planner, dir.path().to_path_buf());

        let spec = Spec {
            id: "test-spec".to_string(),
            goal: "Test goal".to_string(),
            constraints: vec![],
            acceptance: vec![],
            verify: None,
            source_path: None,
        };

        let config = JobConfig::default();

        let result = orchestrator.run(spec, config).await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.attempts, 1);
        assert!(result.diff_path.is_some());
    }

    #[tokio::test]
    async fn test_orchestrator_creates_directories() {
        let dir = TempDir::new().expect("temp dir");

        let planner = DefaultPlanner::new();
        let orchestrator = DefaultOrchestrator::new(planner, dir.path().to_path_buf());

        let spec = Spec {
            id: "dir-test".to_string(),
            goal: "Test".to_string(),
            constraints: vec![],
            acceptance: vec![],
            verify: None,
            source_path: None,
        };

        let config = JobConfig::default();

        let result = orchestrator.run(spec, config).await.expect("run");

        // Check worktree directory was created (using actual job id)
        let worktree_dir = dir
            .path()
            .join(".chakravarti")
            .join("worktrees")
            .join(&result.job.id)
            .join("attempt-1");
        assert!(worktree_dir.exists());

        // Check runs directory was created
        let runs_dir = dir
            .path()
            .join(".chakravarti")
            .join("runs")
            .join(&result.job.id);
        assert!(runs_dir.exists());
    }

    #[test]
    fn test_orchestrator_result_structure() {
        let result = OrchestratorResult {
            job: Job::new("test".to_string(), JobConfig::default()),
            diff_path: Some(PathBuf::from("/tmp/diff.patch")),
            duration_ms: 1000,
            attempts: 2,
        };

        assert_eq!(result.attempts, 2);
        assert!(result.diff_path.is_some());
    }
}
