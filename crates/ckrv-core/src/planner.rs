//! Planner for generating execution plans from specs.

use std::path::Path;

use async_trait::async_trait;

use crate::{Plan, Spec, Step, StepType};

/// Context for planning, including repository information.
#[derive(Debug, Clone)]
pub struct PlanContext {
    /// Root path of the repository.
    pub repo_root: std::path::PathBuf,
    /// List of files in the repository.
    pub files: Vec<String>,
    /// Primary programming language detected.
    pub primary_language: Option<String>,
    /// Build system detected (cargo, npm, make, etc.).
    pub build_system: Option<String>,
    /// Test command if known.
    pub test_command: Option<String>,
}

impl PlanContext {
    /// Create context by analyzing a repository.
    #[must_use]
    pub fn from_repo(repo_root: &Path) -> Self {
        let mut ctx = Self {
            repo_root: repo_root.to_path_buf(),
            files: Vec::new(),
            primary_language: None,
            build_system: None,
            test_command: None,
        };

        // Detect language and build system
        if repo_root.join("Cargo.toml").exists() {
            ctx.primary_language = Some("rust".to_string());
            ctx.build_system = Some("cargo".to_string());
            ctx.test_command = Some("cargo test".to_string());
        } else if repo_root.join("package.json").exists() {
            ctx.primary_language = Some("javascript".to_string());
            ctx.build_system = Some("npm".to_string());
            ctx.test_command = Some("npm test".to_string());
        } else if repo_root.join("pyproject.toml").exists() || repo_root.join("setup.py").exists() {
            ctx.primary_language = Some("python".to_string());
            ctx.build_system = Some("pip".to_string());
            ctx.test_command = Some("pytest".to_string());
        } else if repo_root.join("go.mod").exists() {
            ctx.primary_language = Some("go".to_string());
            ctx.build_system = Some("go".to_string());
            ctx.test_command = Some("go test ./...".to_string());
        } else if repo_root.join("Makefile").exists() {
            ctx.build_system = Some("make".to_string());
            ctx.test_command = Some("make test".to_string());
        }

        ctx
    }

    /// Get a summary for the planner model.
    #[must_use]
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();

        if let Some(lang) = &self.primary_language {
            parts.push(format!("Language: {lang}"));
        }
        if let Some(build) = &self.build_system {
            parts.push(format!("Build: {build}"));
        }
        if let Some(test) = &self.test_command {
            parts.push(format!("Test: {test}"));
        }

        if parts.is_empty() {
            "Unknown project type".to_string()
        } else {
            parts.join(", ")
        }
    }
}

/// Trait for generating execution plans from specifications.
#[async_trait]
pub trait Planner: Send + Sync {
    /// Generate a plan from a specification.
    ///
    /// # Errors
    ///
    /// Returns an error if plan generation fails.
    async fn plan(&self, spec: &Spec, context: &PlanContext) -> Result<Plan, PlanError>;
}

/// Errors from planning.
#[derive(Debug, thiserror::Error)]
pub enum PlanError {
    /// Model API error.
    #[error("Model error: {0}")]
    ModelError(String),

    /// Invalid spec for planning.
    #[error("Invalid spec: {0}")]
    InvalidSpec(String),

    /// Plan parsing error.
    #[error("Failed to parse plan: {0}")]
    ParseError(String),
}

/// Default planner that creates a standard 4-step plan.
pub struct DefaultPlanner;

impl DefaultPlanner {
    /// Create a new default planner.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultPlanner {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Planner for DefaultPlanner {
    async fn plan(&self, spec: &Spec, context: &PlanContext) -> Result<Plan, PlanError> {
        // Create a standard 4-step plan with dependencies
        let steps = vec![
            Step::new("analyze", "Analyze codebase", StepType::Analyze),
            Step::new("generate", "Generate changes", StepType::Generate)
                .with_dependency("analyze"),
            Step::new("execute", "Apply changes", StepType::Execute).with_dependency("generate"),
            Step::new("test", "Run tests", StepType::Test).with_dependency("execute"),
        ];

        let plan = Plan::new(spec.id.clone(), steps);

        // Add context metadata
        tracing::info!(
            spec_id = %spec.id,
            context = %context.summary(),
            steps = plan.steps.len(),
            "Generated plan"
        );

        Ok(plan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_plan_context_rust_project() {
        let dir = TempDir::new().expect("temp dir");
        std::fs::write(dir.path().join("Cargo.toml"), "[package]").ok();

        let ctx = PlanContext::from_repo(dir.path());

        assert_eq!(ctx.primary_language, Some("rust".to_string()));
        assert_eq!(ctx.build_system, Some("cargo".to_string()));
        assert_eq!(ctx.test_command, Some("cargo test".to_string()));
    }

    #[test]
    fn test_plan_context_node_project() {
        let dir = TempDir::new().expect("temp dir");
        std::fs::write(dir.path().join("package.json"), "{}").ok();

        let ctx = PlanContext::from_repo(dir.path());

        assert_eq!(ctx.primary_language, Some("javascript".to_string()));
        assert_eq!(ctx.build_system, Some("npm".to_string()));
    }

    #[test]
    fn test_plan_context_summary() {
        let ctx = PlanContext {
            repo_root: std::path::PathBuf::from("/tmp"),
            files: Vec::new(),
            primary_language: Some("rust".to_string()),
            build_system: Some("cargo".to_string()),
            test_command: Some("cargo test".to_string()),
        };

        let summary = ctx.summary();
        assert!(summary.contains("rust"));
        assert!(summary.contains("cargo"));
    }

    #[tokio::test]
    async fn test_default_planner() {
        let planner = DefaultPlanner::new();
        let spec = Spec {
            id: "test".to_string(),
            goal: "Test goal".to_string(),
            constraints: vec![],
            acceptance: vec![],
            verify: None,
            source_path: None,
        };
        let ctx = PlanContext {
            repo_root: std::path::PathBuf::from("/tmp"),
            files: Vec::new(),
            primary_language: None,
            build_system: None,
            test_command: None,
        };

        let plan = planner.plan(&spec, &ctx).await.expect("plan");

        assert_eq!(plan.steps.len(), 4);
        assert_eq!(plan.steps[0].id, "analyze");
    }
}
