//! # Plans Handler
//!
//! Handlers for execution plan management.

use crate::error::TransportError;
use crate::state::AppState;
use crate::types::{BatchSummary, ListPlansResponse, PlanDetail, PlanSummary};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

// ============================================================
// Internal Types (matching plan.yaml structure)
// ============================================================

/// Model assignment configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ModelAssignment {
    default: String,
    #[serde(default)]
    overrides: HashMap<String, String>,
}

/// Batch from plan.yaml.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PlanBatch {
    id: String,
    name: String,
    task_ids: Vec<String>,
    #[serde(default)]
    depends_on: Vec<String>,
    model_assignment: ModelAssignment,
    #[serde(default)]
    execution_strategy: String,
    #[serde(default)]
    estimated_cost: f64,
    #[serde(default)]
    estimated_time: String,
    #[serde(default)]
    reasoning: String,
    #[serde(default)]
    status: String,
    #[serde(default)]
    branch: Option<String>,
}

/// Plan file structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PlanFile {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    spec_id: Option<String>,
    batches: Vec<PlanBatch>,
}

// ============================================================
// Path Utilities
// ============================================================

/// Get the specs directory path for a project.
fn get_specs_dir(project_root: &Path) -> PathBuf {
    project_root.join(".specs")
}

/// Get path to a specific spec within a project.
fn get_spec_path(project_root: &Path, name: &str) -> PathBuf {
    get_specs_dir(project_root).join(name)
}

// ============================================================
// Handlers
// ============================================================

/// List all plans.
pub fn list_plans_handler(state: &AppState) -> Result<ListPlansResponse, TransportError> {
    let specs_dir = get_specs_dir(&state.project_root);
    let mut plans = Vec::new();

    if specs_dir.exists() && specs_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&specs_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let plan_path = path.join("plan.yaml");
                    if plan_path.exists() {
                        let spec_name = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("")
                            .to_string();

                        // Try to load and count batches
                        let (batch_count, total_cost) =
                            fs::read_to_string(&plan_path)
                                .ok()
                                .map_or((0, None), |content| {
                                    serde_yaml::from_str::<PlanFile>(&content).ok().map_or(
                                        (0, None),
                                        |plan| {
                                            let cost: f64 =
                                                plan.batches.iter().map(|b| b.estimated_cost).sum();
                                            (plan.batches.len(), Some(cost))
                                        },
                                    )
                                });

                        plans.push(PlanSummary {
                            spec_name,
                            batch_count,
                            total_estimated_cost: total_cost,
                            status: None,
                        });
                    }
                }
            }
        }
    }

    plans.sort_by(|a, b| a.spec_name.cmp(&b.spec_name));
    Ok(plans)
}

/// Get a plan for a spec.
pub fn get_plan_handler(state: &AppState, spec_name: String) -> Result<PlanDetail, TransportError> {
    let plan_path = get_spec_path(&state.project_root, &spec_name).join("plan.yaml");

    if !plan_path.exists() {
        return Err(TransportError::NotFound(format!(
            "Plan not found for spec: {spec_name}"
        )));
    }

    let content = fs::read_to_string(&plan_path)
        .map_err(|e| TransportError::Internal(format!("Failed to read plan: {e}")))?;

    let mut plan: PlanFile = serde_yaml::from_str(&content)
        .map_err(|e| TransportError::Internal(format!("Failed to parse plan: {e}")))?;

    // Reset stale running batches to pending
    let mut needs_save = false;
    for batch in &mut plan.batches {
        if batch.status == "running" {
            batch.status = "pending".to_string();
            needs_save = true;
        }
    }

    if needs_save {
        if let Ok(yaml) = serde_yaml::to_string(&plan) {
            let _ = fs::write(&plan_path, &yaml);
        }
    }

    // Convert to response type
    let batches: Vec<BatchSummary> = plan
        .batches
        .into_iter()
        .map(|b| {
            let task_count = b.task_ids.len();
            BatchSummary {
                id: b.id,
                name: b.name,
                task_count,
                task_ids: b.task_ids,
                depends_on: b.depends_on,
                model_assignment: crate::types::BatchModelAssignment {
                    default: b.model_assignment.default,
                    overrides: b.model_assignment.overrides,
                },
                execution_strategy: b.execution_strategy,
                status: if b.status.is_empty() {
                    "pending".to_string()
                } else {
                    b.status
                },
                estimated_cost: Some(b.estimated_cost),
                estimated_time: b.estimated_time,
                reasoning: b.reasoning,
            }
        })
        .collect();

    Ok(PlanDetail {
        spec_name,
        batches,
        raw_yaml: Some(content),
    })
}

/// Update a plan.
pub fn update_plan_handler(
    state: &AppState,
    spec_name: String,
    raw_yaml: String,
) -> Result<PlanDetail, TransportError> {
    let plan_path = get_spec_path(&state.project_root, &spec_name).join("plan.yaml");

    // Validate YAML before saving
    let _: PlanFile = serde_yaml::from_str(&raw_yaml)
        .map_err(|e| TransportError::BadRequest(format!("Invalid plan YAML: {e}")))?;

    fs::write(&plan_path, &raw_yaml)
        .map_err(|e| TransportError::Internal(format!("Failed to write plan: {e}")))?;

    get_plan_handler(state, spec_name)
}

/// Delete a plan.
pub fn delete_plan_handler(state: &AppState, spec_name: String) -> Result<(), TransportError> {
    let plan_path = get_spec_path(&state.project_root, &spec_name).join("plan.yaml");

    if !plan_path.exists() {
        return Err(TransportError::NotFound(format!(
            "Plan not found for spec: {spec_name}"
        )));
    }

    fs::remove_file(&plan_path)
        .map_err(|e| TransportError::Internal(format!("Failed to delete plan: {e}")))?;

    Ok(())
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_list_plans_handler() {
        let state = AppState::new(PathBuf::from("/tmp/test-plans"));
        let result = list_plans_handler(&state);
        assert!(result.is_ok());
    }
}
