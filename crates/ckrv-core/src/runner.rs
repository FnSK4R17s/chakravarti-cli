//! Workflow runner for executing multi-step agent workflows.
//!
//! The Runner iterates through workflow steps, renders prompts,
//! invokes the agent, and collects outputs.

use std::path::Path;
use std::time::Instant;

use crate::agent_task::{AgentTask, AgentTaskStatus};
use crate::prompt::{PromptRenderer, RenderContext};
use crate::step_result::StepExecutionResult;
use crate::workflow::{OutputType, Workflow, WorkflowStep};

/// Configuration for the workflow runner.
#[derive(Debug, Clone)]
pub struct RunnerConfig {
    /// Timeout per step in seconds.
    pub step_timeout_secs: u64,
    /// Whether to continue on step failure.
    pub continue_on_failure: bool,
    /// Agent binary to use (e.g., "claude").
    pub agent_binary: String,
    /// Whether to run agent in Docker sandbox.
    pub use_sandbox: bool,
    /// Docker image to use for sandbox (defaults to ckrv-agent:latest).
    pub sandbox_image: Option<String>,
    /// Keep Docker container after execution (for debugging).
    pub keep_container: bool,
    /// OpenRouter API key (for Claude Code + OpenRouter mode).
    pub openrouter_api_key: Option<String>,
    /// OpenRouter model ID (for Claude Code + OpenRouter mode).
    pub openrouter_model: Option<String>,
    /// OpenRouter base URL (defaults to https://openrouter.ai/api).
    pub openrouter_base_url: Option<String>,
}

impl Default for RunnerConfig {
    fn default() -> Self {
        Self {
            step_timeout_secs: 300, // 5 minutes
            continue_on_failure: false,
            agent_binary: "claude".to_string(),
            use_sandbox: false,
            sandbox_image: None,
            keep_container: false,
            openrouter_api_key: None,
            openrouter_model: None,
            openrouter_base_url: None,
        }
    }
}

/// Result of running a complete workflow.
#[derive(Debug)]
pub struct WorkflowRunResult {
    /// Results for each step.
    pub step_results: Vec<StepExecutionResult>,
    /// Whether all steps succeeded.
    pub success: bool,
    /// Total duration in milliseconds.
    pub duration_ms: u64,
}

/// Errors from workflow execution.
#[derive(Debug, thiserror::Error)]
pub enum RunnerError {
    /// Step execution failed.
    #[error("Step '{step_id}' failed: {message}")]
    StepFailed {
        /// The step ID that failed.
        step_id: String,
        /// The error message.
        message: String,
    },

    /// Prompt rendering failed.
    #[error("Failed to render prompt for step '{step_id}': {message}")]
    PromptRenderError {
        /// The step ID.
        step_id: String,
        /// The error message.
        message: String,
    },

    /// Agent invocation failed.
    #[error("Failed to invoke agent: {0}")]
    AgentError(String),

    /// Task persistence failed.
    #[error("Failed to save task: {0}")]
    PersistenceError(String),
}

/// The workflow runner executes workflow steps sequentially.
pub struct WorkflowRunner {
    config: RunnerConfig,
    renderer: PromptRenderer<'static>,
}

impl WorkflowRunner {
    /// Create a new workflow runner.
    #[must_use]
    pub fn new(config: RunnerConfig) -> Self {
        Self {
            config,
            renderer: PromptRenderer::new(),
        }
    }

    /// Run a workflow for the given task.
    ///
    /// # Errors
    ///
    /// Returns an error if execution fails and `continue_on_failure` is false.
    pub async fn run(
        &self,
        workflow: &Workflow,
        task: &mut AgentTask,
        base_dir: &Path,
    ) -> Result<WorkflowRunResult, RunnerError> {
        let start = Instant::now();
        let mut step_results = Vec::new();
        let mut all_success = true;

        // Ensure workspace directory exists
        let workspace_dir = if task.worktree_path.exists() {
            task.worktree_path.clone()
        } else {
            // Create workspace directory or use base_dir as fallback
            let workspace = &task.worktree_path;
            if let Err(e) = std::fs::create_dir_all(workspace) {
                tracing::warn!(error = %e, "Could not create workspace, using base_dir");
                base_dir.to_path_buf()
            } else {
                workspace.clone()
            }
        };

        tracing::info!(workspace = %workspace_dir.display(), "Running workflow in workspace");

        // Build initial render context from task
        let mut context = RenderContext::new()
            .with_input("description", &task.original_prompt)
            .with_input("prompt", &task.original_prompt);

        // Update task status
        task.set_status(AgentTaskStatus::Running);
        task.save(base_dir)
            .map_err(|e| RunnerError::PersistenceError(e.to_string()))?;

        // Execute each step
        for step in &workflow.steps {
            let step_result = self
                .execute_step(step, &context, task, &workspace_dir)
                .await;

            match &step_result {
                Ok(result) => {
                    // Record outputs in context for next steps
                    for (name, value) in &result.outputs {
                        context.record_output(&step.id, name, value.clone());
                        task.record_step_output(&step.id, name, value.clone());
                    }
                    step_results.push(result.clone());
                }
                Err(e) => {
                    all_success = false;
                    let failed_result = StepExecutionResult::failed(&step.id, e.to_string());
                    step_results.push(failed_result);

                    if !self.config.continue_on_failure {
                        task.set_status(AgentTaskStatus::Failed);
                        task.save(base_dir)
                            .map_err(|e| RunnerError::PersistenceError(e.to_string()))?;
                        return Err(RunnerError::StepFailed {
                            step_id: step.id.clone(),
                            message: e.to_string(),
                        });
                    }
                }
            }
        }

        // Update final status
        if all_success {
            task.set_status(AgentTaskStatus::Completed);
        } else {
            task.set_status(AgentTaskStatus::Failed);
        }
        task.save(base_dir)
            .map_err(|e| RunnerError::PersistenceError(e.to_string()))?;

        Ok(WorkflowRunResult {
            step_results,
            success: all_success,
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }

    /// Execute a single workflow step.
    async fn execute_step(
        &self,
        step: &WorkflowStep,
        context: &RenderContext,
        _task: &AgentTask,
        workspace_dir: &Path,
    ) -> Result<StepExecutionResult, RunnerError> {
        let start = Instant::now();

        // Render the prompt
        let prompt = self.renderer.render(&step.prompt, context).map_err(|e| {
            RunnerError::PromptRenderError {
                step_id: step.id.clone(),
                message: e.to_string(),
            }
        })?;

        tracing::info!(step_id = %step.id, "Executing step with prompt length: {}", prompt.len());

        // Invoke the agent CLI
        let (stdout, stderr, success) = self.invoke_agent(&prompt, workspace_dir).await?;

        // Build result
        let mut result = if success {
            StepExecutionResult::success(&step.id, start.elapsed().as_millis() as u64)
        } else {
            StepExecutionResult::failed(&step.id, &stderr)
        };

        result = result.with_stdout(&stdout).with_stderr(&stderr);

        // Parse outputs based on step output definitions
        for output_def in &step.outputs {
            match output_def.output_type {
                OutputType::File => {
                    // For file outputs, check if file exists in workspace
                    if let Some(filename) = &output_def.filename {
                        let file_path = workspace_dir.join(filename);
                        if file_path.exists() {
                            // Read file content for use in subsequent steps
                            if let Ok(content) = std::fs::read_to_string(&file_path) {
                                result = result.with_output(&output_def.name, content);
                            } else {
                                result = result.with_output(&output_def.name, filename.clone());
                            }
                        } else {
                            result = result.with_output(&output_def.name, filename.clone());
                        }
                    }
                }
                OutputType::String => {
                    // For string outputs, try to extract from stdout
                    // Look for JSON output or use the full stdout
                    let output_value = self.extract_string_output(&stdout, &output_def.name);
                    result = result.with_output(&output_def.name, output_value);
                }
            }
        }

        Ok(result)
    }

    async fn invoke_agent(
        &self,
        prompt: &str,
        workdir: &std::path::Path,
    ) -> Result<(String, String, bool), RunnerError> {
        if self.config.use_sandbox {
            self.invoke_agent_sandboxed(prompt, workdir).await
        } else {
            self.invoke_agent_local(prompt, workdir).await
        }
    }

    /// Invoke agent locally (no Docker).
    async fn invoke_agent_local(
        &self,
        prompt: &str,
        workdir: &std::path::Path,
    ) -> Result<(String, String, bool), RunnerError> {
        use std::process::Command;

        // Resolve the agent binary path
        let agent_path = self.resolve_agent_path();

        tracing::debug!(agent_path = %agent_path, "Invoking agent locally");

        // Build the claude command with tools enabled for actual file operations
        // Using --dangerously-skip-permissions because we're running in a controlled task workspace
        let mut cmd = Command::new(&agent_path);
        cmd.args([
            "-p",
            prompt,
            "--output-format",
            "text",
            "--dangerously-skip-permissions",
        ]);
        cmd.current_dir(workdir);

        // Set OpenRouter environment variables if configured
        // Per https://openrouter.ai/docs/guides/guides/claude-code-integration
        if let Some(ref api_key) = self.config.openrouter_api_key {
            let base_url = self.config.openrouter_base_url
                .as_deref()
                .unwrap_or("https://openrouter.ai/api");
            
            tracing::info!(
                base_url = %base_url,
                model = ?self.config.openrouter_model,
                "Using OpenRouter for Claude Code"
            );

            // Required env vars for OpenRouter
            cmd.env("ANTHROPIC_BASE_URL", base_url);
            cmd.env("ANTHROPIC_AUTH_TOKEN", api_key);
            cmd.env("ANTHROPIC_API_KEY", ""); // Must be explicitly empty!

            // Optional: override default model
            if let Some(ref model) = self.config.openrouter_model {
                // Set all tiers to the same model for consistency
                cmd.env("ANTHROPIC_DEFAULT_SONNET_MODEL", model);
                cmd.env("ANTHROPIC_DEFAULT_OPUS_MODEL", model);
                cmd.env("ANTHROPIC_DEFAULT_HAIKU_MODEL", model);
            }
        }

        let output = cmd.output().map_err(|e| {
            RunnerError::AgentError(format!("Failed to spawn {}: {}", agent_path, e))
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let success = output.status.success();

        tracing::debug!(
            agent = %agent_path,
            success = success,
            stdout_len = stdout.len(),
            stderr_len = stderr.len(),
            "Agent invocation complete"
        );

        Ok((stdout, stderr, success))
    }

    /// Invoke agent inside Docker sandbox.
    async fn invoke_agent_sandboxed(
        &self,
        prompt: &str,
        workdir: &std::path::Path,
    ) -> Result<(String, String, bool), RunnerError> {
        use ckrv_sandbox::{DefaultAllowList, DockerSandbox, ExecuteConfig, Sandbox};
        use std::time::Duration;

        tracing::info!("Invoking agent in Docker sandbox");

        // Create sandbox
        let sandbox = DockerSandbox::new(DefaultAllowList::default()).map_err(|e| {
            RunnerError::AgentError(format!("Failed to create Docker sandbox: {}", e))
        })?;

        // Build command: claude -p "prompt" --dangerously-skip-permissions --output-format text
        // We use --dangerously-skip-permissions because we're in a controlled sandbox
        let command = format!(
            "{} -p {} --dangerously-skip-permissions --output-format text",
            self.config.agent_binary,
            shell_escape::escape(prompt.into())
        );

        // Configure execution
        let mut config = ExecuteConfig::new("", workdir.to_path_buf())
            .shell(&command)
            .with_timeout(Duration::from_secs(self.config.step_timeout_secs))
            .with_keep_container(self.config.keep_container);

        // Add OpenRouter environment variables if configured
        if let Some(ref api_key) = self.config.openrouter_api_key {
            let base_url = self.config.openrouter_base_url
                .as_deref()
                .unwrap_or("https://openrouter.ai/api");

            tracing::info!(
                base_url = %base_url,
                model = ?self.config.openrouter_model,
                "Using OpenRouter for Claude Code in sandbox"
            );

            config = config
                .env("ANTHROPIC_BASE_URL", base_url)
                .env("ANTHROPIC_AUTH_TOKEN", api_key)
                .env("ANTHROPIC_API_KEY", ""); // Must be explicitly empty!

            if let Some(ref model) = self.config.openrouter_model {
                config = config
                    .env("ANTHROPIC_DEFAULT_SONNET_MODEL", model)
                    .env("ANTHROPIC_DEFAULT_OPUS_MODEL", model)
                    .env("ANTHROPIC_DEFAULT_HAIKU_MODEL", model);
            }
        }

        // Execute in sandbox
        let result = sandbox
            .execute(config)
            .await
            .map_err(|e| RunnerError::AgentError(format!("Sandbox execution failed: {}", e)))?;

        let success = result.success();

        tracing::debug!(
            exit_code = result.exit_code,
            stdout_len = result.stdout.len(),
            stderr_len = result.stderr.len(),
            duration_ms = result.duration_ms,
            "Sandbox execution complete"
        );

        Ok((result.stdout, result.stderr, success))
    }

    /// Resolve the agent binary path, checking common installation locations.
    fn resolve_agent_path(&self) -> String {
        use std::path::PathBuf;

        // If already an absolute path, use it directly
        if self.config.agent_binary.starts_with('/') {
            return self.config.agent_binary.clone();
        }

        // Common locations to check
        let home = std::env::var("HOME").unwrap_or_default();
        let candidates = [
            format!("{}/.local/bin/{}", home, self.config.agent_binary),
            format!("/usr/local/bin/{}", self.config.agent_binary),
            format!("/usr/bin/{}", self.config.agent_binary),
            format!("{}/.npm/bin/{}", home, self.config.agent_binary),
            format!("{}/.cargo/bin/{}", home, self.config.agent_binary),
        ];

        for candidate in &candidates {
            if PathBuf::from(candidate).exists() {
                tracing::debug!(path = %candidate, "Found agent binary");
                return candidate.clone();
            }
        }

        // Fallback to bare name (rely on PATH)
        self.config.agent_binary.clone()
    }

    /// Extract a string output from agent response.
    fn extract_string_output(&self, stdout: &str, output_name: &str) -> String {
        // Try to parse as JSON and extract the named field
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(stdout) {
            if let Some(value) = json.get(output_name) {
                return value.as_str().unwrap_or(&value.to_string()).to_string();
            }
        }

        // Otherwise, return a summary of the output
        if stdout.len() > 200 {
            format!("{}...", &stdout[..200])
        } else {
            stdout.to_string()
        }
    }
}

impl Default for WorkflowRunner {
    fn default() -> Self {
        Self::new(RunnerConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::Workflow;
    use std::path::PathBuf;
    use tempfile::TempDir;

    const TEST_WORKFLOW: &str = r#"
version: '1.0'
name: 'test'
steps:
  - id: step1
    name: 'Step 1'
    prompt: 'Do: {{inputs.description}}'
    outputs:
      - name: result
        type: string
"#;

    #[tokio::test]
    async fn test_runner_executes_workflow() {
        let dir = TempDir::new().expect("temp dir");
        let workflow = Workflow::parse(TEST_WORKFLOW).expect("parse");

        let mut task = AgentTask::new(
            "test-run",
            "Test task",
            "test",
            PathBuf::from("/tmp/worktree"),
        );

        let runner = WorkflowRunner::default();
        let result = runner.run(&workflow, &mut task, dir.path()).await;

        assert!(result.is_ok());
        let run_result = result.unwrap();
        assert!(run_result.success);
        assert_eq!(run_result.step_results.len(), 1);
    }

    #[tokio::test]
    async fn test_runner_records_outputs() {
        let dir = TempDir::new().expect("temp dir");
        let workflow = Workflow::parse(TEST_WORKFLOW).expect("parse");

        let mut task = AgentTask::new("test-outputs", "Test", "test", PathBuf::from("/tmp"));

        let runner = WorkflowRunner::default();
        let _ = runner.run(&workflow, &mut task, dir.path()).await;

        // Task should have recorded outputs
        assert!(task.get_step_output("step1", "result").is_some());
    }
}
