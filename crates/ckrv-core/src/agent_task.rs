//! Task entity for tracking workflow execution instances.
//!
//! A Task represents a single user request flowing through a workflow.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// A task instance representing a workflow execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTask {
    /// Unique task identifier (e.g., "task-001").
    pub id: String,
    /// Original prompt from the user.
    pub original_prompt: String,
    /// Name of the workflow being executed.
    pub workflow_name: String,
    /// Current status of the task.
    pub status: AgentTaskStatus,
    /// Path to the git worktree for this task.
    pub worktree_path: PathBuf,
    /// When the task was created.
    pub created_at: DateTime<Utc>,
    /// When the task was last updated.
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
    /// Outputs collected from completed steps.
    #[serde(default)]
    pub step_outputs: HashMap<String, HashMap<String, String>>,
}

/// Status of a task.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AgentTaskStatus {
    /// Task created but not started.
    Pending,
    /// Task is currently running.
    Running,
    /// Task completed successfully.
    Completed,
    /// Task failed.
    Failed,
    /// Task was cancelled.
    Cancelled,
}

/// Errors from task operations.
#[derive(Debug, thiserror::Error)]
pub enum TaskError {
    /// Task not found.
    #[error("Task not found: {0}")]
    NotFound(String),

    /// IO error.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

impl AgentTask {
    /// Create a new task.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        prompt: impl Into<String>,
        workflow_name: impl Into<String>,
        worktree_path: PathBuf,
    ) -> Self {
        Self {
            id: id.into(),
            original_prompt: prompt.into(),
            workflow_name: workflow_name.into(),
            status: AgentTaskStatus::Pending,
            worktree_path,
            created_at: Utc::now(),
            updated_at: None,
            step_outputs: HashMap::new(),
        }
    }

    /// Generate a new task ID.
    #[must_use]
    pub fn generate_id() -> String {
        format!(
            "task-{}",
            uuid::Uuid::new_v4()
                .to_string()
                .split('-')
                .next()
                .unwrap_or("unknown")
        )
    }

    /// Get the task directory path (where metadata and artifacts are stored).
    #[must_use]
    pub fn task_dir(base_dir: &Path, task_id: &str) -> PathBuf {
        base_dir.join(".ckrv").join("tasks").join(task_id)
    }

    /// Get the metadata file path.
    #[must_use]
    pub fn metadata_path(base_dir: &Path, task_id: &str) -> PathBuf {
        Self::task_dir(base_dir, task_id).join("metadata.json")
    }

    /// Save the task to disk.
    ///
    /// # Errors
    ///
    /// Returns an error if saving fails.
    pub fn save(&self, base_dir: &Path) -> Result<(), TaskError> {
        let task_dir = Self::task_dir(base_dir, &self.id);
        fs::create_dir_all(&task_dir)?;

        let metadata_path = task_dir.join("metadata.json");
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| TaskError::SerializationError(e.to_string()))?;
        fs::write(metadata_path, json)?;

        Ok(())
    }

    /// Load a task from disk.
    ///
    /// # Errors
    ///
    /// Returns an error if loading fails.
    pub fn load(base_dir: &Path, task_id: &str) -> Result<Self, TaskError> {
        let metadata_path = Self::metadata_path(base_dir, task_id);

        if !metadata_path.exists() {
            return Err(TaskError::NotFound(task_id.to_string()));
        }

        let json = fs::read_to_string(metadata_path)?;
        let task: Self = serde_json::from_str(&json)
            .map_err(|e| TaskError::SerializationError(e.to_string()))?;

        Ok(task)
    }

    /// Update the task status.
    pub fn set_status(&mut self, status: AgentTaskStatus) {
        self.status = status;
        self.updated_at = Some(Utc::now());
    }

    /// Record an output from a step.
    pub fn record_step_output(&mut self, step_id: &str, output_name: &str, value: String) {
        let step_outputs = self.step_outputs.entry(step_id.to_string()).or_default();
        step_outputs.insert(output_name.to_string(), value);
        self.updated_at = Some(Utc::now());
    }

    /// Get output from a previous step.
    #[must_use]
    pub fn get_step_output(&self, step_id: &str, output_name: &str) -> Option<&String> {
        self.step_outputs
            .get(step_id)
            .and_then(|outputs| outputs.get(output_name))
    }
}

impl Default for AgentTaskStatus {
    fn default() -> Self {
        Self::Pending
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_task_creation() {
        let task = AgentTask::new(
            "test-001",
            "Create hello.txt",
            "swe",
            PathBuf::from("/tmp/worktree"),
        );

        assert_eq!(task.id, "test-001");
        assert_eq!(task.status, AgentTaskStatus::Pending);
        assert_eq!(task.workflow_name, "swe");
    }

    #[test]
    fn test_task_save_and_load() {
        let dir = TempDir::new().expect("temp dir");
        let task = AgentTask::new(
            "test-002",
            "Test prompt",
            "swe",
            PathBuf::from("/tmp/worktree"),
        );

        task.save(dir.path()).expect("save");

        let loaded = AgentTask::load(dir.path(), "test-002").expect("load");
        assert_eq!(loaded.id, task.id);
        assert_eq!(loaded.original_prompt, task.original_prompt);
    }

    #[test]
    fn test_task_status_update() {
        let mut task = AgentTask::new("test-003", "Test", "swe", PathBuf::from("/tmp"));

        assert!(task.updated_at.is_none());

        task.set_status(AgentTaskStatus::Running);
        assert_eq!(task.status, AgentTaskStatus::Running);
        assert!(task.updated_at.is_some());
    }

    #[test]
    fn test_step_outputs() {
        let mut task = AgentTask::new("test-004", "Test", "swe", PathBuf::from("/tmp"));

        task.record_step_output("plan", "plan_file", "plan.md content".to_string());

        let output = task.get_step_output("plan", "plan_file");
        assert_eq!(output, Some(&"plan.md content".to_string()));

        assert!(task.get_step_output("nonexistent", "foo").is_none());
    }

    #[test]
    fn test_generate_id() {
        let id = AgentTask::generate_id();
        assert!(id.starts_with("task-"));
        assert!(id.len() > 5);
    }
}
