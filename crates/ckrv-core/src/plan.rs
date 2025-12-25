//! Plan and step dependencies.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::Step;

/// A plan is a deterministic DAG of execution steps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    /// Unique plan identifier.
    pub id: String,

    /// ID of the source specification.
    pub spec_id: String,

    /// Ordered list of steps respecting dependencies.
    pub steps: Vec<Step>,

    /// When the plan was generated.
    pub created_at: DateTime<Utc>,
}

impl Plan {
    /// Create a new plan for a specification.
    #[must_use]
    pub fn new(spec_id: String, steps: Vec<Step>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            spec_id,
            steps,
            created_at: Utc::now(),
        }
    }

    /// Get steps that are ready to execute (no pending dependencies).
    #[must_use]
    pub fn ready_steps(&self) -> Vec<&Step> {
        self.steps
            .iter()
            .filter(|step| {
                step.dependencies.iter().all(|dep_id| {
                    self.steps
                        .iter()
                        .find(|s| &s.id == dep_id)
                        .map_or(true, |dep| dep.status.is_complete())
                })
            })
            .filter(|step| !step.status.is_complete())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{StepStatus, StepType};

    fn make_step(id: &str, deps: Vec<&str>, status: StepStatus) -> Step {
        Step {
            id: id.to_string(),
            name: format!("Step {id}"),
            step_type: StepType::Execute,
            dependencies: deps.into_iter().map(String::from).collect(),
            status,
            output: None,
            duration_ms: None,
        }
    }

    #[test]
    fn test_plan_new_generates_uuid() {
        let plan = Plan::new("spec_123".to_string(), vec![]);
        assert!(!plan.id.is_empty());
        assert_eq!(plan.spec_id, "spec_123");
    }

    #[test]
    fn test_plan_with_no_steps_has_no_ready() {
        let plan = Plan::new("spec".to_string(), vec![]);
        assert!(plan.ready_steps().is_empty());
    }

    #[test]
    fn test_step_with_no_deps_is_ready() {
        let step = make_step("s1", vec![], StepStatus::Pending);
        let plan = Plan::new("spec".to_string(), vec![step]);

        let ready = plan.ready_steps();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, "s1");
    }

    #[test]
    fn test_completed_step_is_not_ready() {
        let step = make_step("s1", vec![], StepStatus::Completed);
        let plan = Plan::new("spec".to_string(), vec![step]);

        assert!(plan.ready_steps().is_empty());
    }

    #[test]
    fn test_step_with_pending_dep_is_not_ready() {
        let s1 = make_step("s1", vec![], StepStatus::Pending);
        let s2 = make_step("s2", vec!["s1"], StepStatus::Pending);
        let plan = Plan::new("spec".to_string(), vec![s1, s2]);

        let ready = plan.ready_steps();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, "s1"); // Only s1 is ready, s2 depends on s1
    }

    #[test]
    fn test_step_with_completed_dep_is_ready() {
        let s1 = make_step("s1", vec![], StepStatus::Completed);
        let s2 = make_step("s2", vec!["s1"], StepStatus::Pending);
        let plan = Plan::new("spec".to_string(), vec![s1, s2]);

        let ready = plan.ready_steps();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, "s2"); // s1 complete, so s2 is ready
    }

    #[test]
    fn test_parallel_steps_both_ready() {
        let s1 = make_step("s1", vec![], StepStatus::Pending);
        let s2 = make_step("s2", vec![], StepStatus::Pending);
        let plan = Plan::new("spec".to_string(), vec![s1, s2]);

        let ready = plan.ready_steps();
        assert_eq!(ready.len(), 2);
    }

    #[test]
    fn test_diamond_dependency() {
        //   s1
        //  /  \
        // s2  s3
        //  \  /
        //   s4
        let s1 = make_step("s1", vec![], StepStatus::Completed);
        let s2 = make_step("s2", vec!["s1"], StepStatus::Completed);
        let s3 = make_step("s3", vec!["s1"], StepStatus::Pending);
        let s4 = make_step("s4", vec!["s2", "s3"], StepStatus::Pending);
        let plan = Plan::new("spec".to_string(), vec![s1, s2, s3, s4]);

        let ready = plan.ready_steps();
        // Only s3 is ready (s4 waits for s3)
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, "s3");
    }

    #[test]
    fn test_failed_dep_counts_as_complete() {
        let s1 = make_step(
            "s1",
            vec![],
            StepStatus::Failed {
                error: "oops".to_string(),
            },
        );
        let s2 = make_step("s2", vec!["s1"], StepStatus::Pending);
        let plan = Plan::new("spec".to_string(), vec![s1, s2]);

        let ready = plan.ready_steps();
        // s1 failed counts as complete for dependency purposes
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, "s2");
    }

    #[test]
    fn test_plan_serialization() {
        let plan = Plan::new("spec_123".to_string(), vec![]);
        let json = serde_json::to_string(&plan).expect("serialize");
        let parsed: Plan = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(plan.id, parsed.id);
        assert_eq!(plan.spec_id, parsed.spec_id);
    }
}
