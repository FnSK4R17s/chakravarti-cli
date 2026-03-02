//! # Tasks Handler
//!
//! Handlers for task management.

use crate::error::TransportError;
use crate::state::AppState;
use crate::types::{ListTasksResponse, TaskDetail, TaskStatus, TaskSummary, UpdateTaskRequest};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

// ============================================================================
// Internal Types (matching tasks.yaml structure)
// ============================================================================

/// Full task structure from tasks.yaml.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct TaskFile {
    id: String,
    #[serde(default)]
    phase: String,
    title: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    file: String,
    #[serde(default)]
    user_story: Option<String>,
    #[serde(default)]
    parallel: bool,
    #[serde(default = "default_complexity")]
    complexity: i32,
    #[serde(default = "default_model_tier")]
    model_tier: String,
    #[serde(default)]
    estimated_tokens: i32,
    #[serde(default = "default_risk")]
    risk: String,
    #[serde(default)]
    context_required: Vec<String>,
    #[serde(default = "default_status")]
    status: String,
}

fn default_complexity() -> i32 {
    1
}
fn default_model_tier() -> String {
    "light".to_string()
}
fn default_risk() -> String {
    "low".to_string()
}
fn default_status() -> String {
    "pending".to_string()
}

/// Tasks file structure.
#[derive(Debug, Deserialize)]
struct TasksFile {
    tasks: Option<Vec<TaskFile>>,
}

/// Tasks output structure.
#[derive(Debug, Serialize)]
struct TasksOutput {
    tasks: Vec<TaskFile>,
}

// ============================================================================
// Path Utilities
// ============================================================================

/// Get the specs directory path for a project.
fn get_specs_dir(project_root: &Path) -> PathBuf {
    project_root.join(".specs")
}

/// Get path to a specific spec within a project.
fn get_spec_path(project_root: &Path, name: &str) -> PathBuf {
    get_specs_dir(project_root).join(name)
}

/// Get current git branch for a project.
fn get_current_branch(project_root: &Path) -> String {
    Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(project_root)
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout)
                    .ok()
                    .map(|s| s.trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_default()
}

// ============================================================================
// Handlers
// ============================================================================

/// List tasks for a spec.
pub fn list_tasks_handler(
    state: &AppState,
    spec_name: Option<String>,
) -> Result<ListTasksResponse, TransportError> {
    let project_root = &state.project_root;

    // If no spec provided, try to detect from current branch
    let spec = spec_name.unwrap_or_else(|| {
        let branch = get_current_branch(project_root);
        if branch.is_empty() || branch == "main" || branch == "master" {
            String::new()
        } else {
            branch
        }
    });

    if spec.is_empty() {
        return Ok(vec![]);
    }

    let tasks_path = get_spec_path(project_root, &spec).join("tasks.yaml");

    if !tasks_path.exists() {
        return Ok(vec![]);
    }

    let content = fs::read_to_string(&tasks_path)
        .map_err(|e| TransportError::Internal(format!("Failed to read tasks: {e}")))?;

    let tasks = parse_tasks_yaml(&content);

    Ok(tasks
        .into_iter()
        .map(|t| TaskSummary {
            id: t.id,
            title: t.title,
            status: parse_status(&t.status),
            complexity: t.complexity as u8,
            phase: if t.phase.is_empty() {
                None
            } else {
                Some(t.phase)
            },
        })
        .collect())
}

/// Get a single task.
pub fn get_task_handler(
    state: &AppState,
    spec_name: String,
    task_id: String,
) -> Result<TaskDetail, TransportError> {
    let tasks_path = get_spec_path(&state.project_root, &spec_name).join("tasks.yaml");

    if !tasks_path.exists() {
        return Err(TransportError::NotFound(format!(
            "Tasks not found for spec: {spec_name}"
        )));
    }

    let content = fs::read_to_string(&tasks_path)
        .map_err(|e| TransportError::Internal(format!("Failed to read tasks: {e}")))?;

    let tasks = parse_tasks_yaml(&content);

    let task = tasks
        .into_iter()
        .find(|t| t.id == task_id)
        .ok_or_else(|| TransportError::NotFound(format!("Task not found: {task_id}")))?;

    Ok(TaskDetail {
        id: task.id,
        title: task.title,
        description: if task.description.is_empty() {
            None
        } else {
            Some(task.description)
        },
        status: parse_status(&task.status),
        complexity: task.complexity as u8,
        phase: if task.phase.is_empty() {
            None
        } else {
            Some(task.phase)
        },
        file_path: if task.file.is_empty() {
            None
        } else {
            Some(task.file)
        },
        user_story: task.user_story,
        parallel: task.parallel,
        model_tier: if task.model_tier.is_empty() {
            None
        } else {
            Some(task.model_tier)
        },
        estimated_tokens: if task.estimated_tokens > 0 {
            Some(task.estimated_tokens as u32)
        } else {
            None
        },
        context_required: if task.context_required.is_empty() {
            None
        } else {
            Some(task.context_required)
        },
    })
}

/// Update a task.
pub fn update_task_handler(
    state: &AppState,
    spec_name: String,
    task_id: String,
    request: UpdateTaskRequest,
) -> Result<TaskDetail, TransportError> {
    let tasks_path = get_spec_path(&state.project_root, &spec_name).join("tasks.yaml");

    if !tasks_path.exists() {
        return Err(TransportError::NotFound(format!(
            "Tasks not found for spec: {spec_name}"
        )));
    }

    let content = fs::read_to_string(&tasks_path)
        .map_err(|e| TransportError::Internal(format!("Failed to read tasks: {e}")))?;

    let mut tasks = parse_tasks_yaml(&content);

    // Find and update the task
    let task = tasks
        .iter_mut()
        .find(|t| t.id == task_id)
        .ok_or_else(|| TransportError::NotFound(format!("Task not found: {task_id}")))?;

    if let Some(status) = request.status {
        task.status = status_to_string(&status);
    }

    // Write back
    let output = TasksOutput {
        tasks: tasks.clone(),
    };
    let yaml = serde_yaml::to_string(&output)
        .map_err(|e| TransportError::Internal(format!("Failed to serialize tasks: {e}")))?;

    fs::write(&tasks_path, yaml)
        .map_err(|e| TransportError::Internal(format!("Failed to write tasks: {e}")))?;

    // Return updated task
    let task = tasks
        .into_iter()
        .find(|t| t.id == task_id)
        .ok_or_else(|| TransportError::NotFound(format!("Task not found: {task_id}")))?;

    Ok(TaskDetail {
        id: task.id,
        title: task.title,
        description: if task.description.is_empty() {
            None
        } else {
            Some(task.description)
        },
        status: parse_status(&task.status),
        complexity: task.complexity as u8,
        phase: if task.phase.is_empty() {
            None
        } else {
            Some(task.phase)
        },
        file_path: if task.file.is_empty() {
            None
        } else {
            Some(task.file)
        },
        user_story: task.user_story,
        parallel: task.parallel,
        model_tier: if task.model_tier.is_empty() {
            None
        } else {
            Some(task.model_tier)
        },
        estimated_tokens: if task.estimated_tokens > 0 {
            Some(task.estimated_tokens as u32)
        } else {
            None
        },
        context_required: if task.context_required.is_empty() {
            None
        } else {
            Some(task.context_required)
        },
    })
}

// ============================================================================
// Helpers
// ============================================================================

/// Parse tasks from YAML content.
fn parse_tasks_yaml(content: &str) -> Vec<TaskFile> {
    serde_yaml::from_str::<TasksFile>(content)
        .ok()
        .and_then(|y| y.tasks)
        .unwrap_or_default()
}

/// Parse status string to enum.
fn parse_status(status: &str) -> TaskStatus {
    match status.to_lowercase().as_str() {
        "running" | "in_progress" => TaskStatus::InProgress,
        "completed" | "done" => TaskStatus::Completed,
        "failed" => TaskStatus::Failed,
        "skipped" => TaskStatus::Skipped,
        // "pending" and any unrecognized status
        _ => TaskStatus::Pending,
    }
}

/// Convert status enum to string.
fn status_to_string(status: &TaskStatus) -> String {
    match status {
        TaskStatus::Pending => "pending".to_string(),
        TaskStatus::Queued => "queued".to_string(),
        TaskStatus::Running | TaskStatus::InProgress => "running".to_string(),
        TaskStatus::Completed => "completed".to_string(),
        TaskStatus::Failed => "failed".to_string(),
        TaskStatus::Skipped => "skipped".to_string(),
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_status() {
        assert!(matches!(parse_status("pending"), TaskStatus::Pending));
        assert!(matches!(parse_status("running"), TaskStatus::InProgress));
        assert!(matches!(parse_status("completed"), TaskStatus::Completed));
        assert!(matches!(parse_status("failed"), TaskStatus::Failed));
    }

    #[test]
    fn test_status_to_string() {
        assert_eq!(status_to_string(&TaskStatus::Pending), "pending");
        assert_eq!(status_to_string(&TaskStatus::InProgress), "running");
    }
}
