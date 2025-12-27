//! Workflow definition and parsing for multi-step agent orchestration.
//!
//! This module provides YAML-based workflow definitions compatible with Rover's
//! `swe.yml` format, enabling multi-step AI agent workflows.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// A workflow defines a sequence of steps to be executed by an AI agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    /// Schema version (e.g., "1.0").
    pub version: String,
    /// Unique name of the workflow.
    pub name: String,
    /// Human-readable description.
    #[serde(default)]
    pub description: Option<String>,
    /// Default configuration.
    #[serde(default)]
    pub defaults: Option<WorkflowDefaults>,
    /// Steps to execute in order.
    pub steps: Vec<WorkflowStep>,
}

/// Default configuration for a workflow.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkflowDefaults {
    /// Default agent tool (e.g., "claude", "gemini").
    #[serde(default)]
    pub tool: Option<String>,
    /// Default model (e.g., "sonnet", "gpt-4").
    #[serde(default)]
    pub model: Option<String>,
}

/// A single step in a workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    /// Unique step identifier.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Step type (default: "agent").
    #[serde(default = "default_step_type")]
    #[serde(rename = "type")]
    pub step_type: String,
    /// Agent tool override for this step.
    #[serde(default)]
    pub agent: Option<String>,
    /// Prompt template (supports Handlebars syntax).
    pub prompt: String,
    /// Expected outputs from this step.
    #[serde(default)]
    pub outputs: Vec<StepOutput>,
}

fn default_step_type() -> String {
    "agent".to_string()
}

/// An expected output from a step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepOutput {
    /// Output variable name.
    pub name: String,
    /// Output type: "file" or "string".
    #[serde(rename = "type")]
    pub output_type: OutputType,
    /// Description of the output.
    #[serde(default)]
    pub description: Option<String>,
    /// For file type: the filename to look for.
    #[serde(default)]
    pub filename: Option<String>,
}

/// Type of step output.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OutputType {
    /// Output is a file created by the agent.
    File,
    /// Output is a string value (JSON key or parsed from response).
    String,
}

/// Errors from workflow operations.
#[derive(Debug, thiserror::Error)]
pub enum WorkflowError {
    /// File not found.
    #[error("Workflow file not found: {0}")]
    NotFound(String),

    /// YAML parsing failed.
    #[error("Failed to parse workflow YAML: {0}")]
    ParseError(String),

    /// Validation failed.
    #[error("Workflow validation failed: {0}")]
    ValidationError(String),

    /// IO error.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl Workflow {
    /// Load a workflow from a YAML file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, WorkflowError> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(WorkflowError::NotFound(path.display().to_string()));
        }

        let content = fs::read_to_string(path)?;
        Self::parse(&content)
    }

    /// Parse a workflow from YAML string.
    ///
    /// # Errors
    ///
    /// Returns an error if parsing fails.
    pub fn parse(yaml: &str) -> Result<Self, WorkflowError> {
        let workflow: Workflow =
            serde_yaml::from_str(yaml).map_err(|e| WorkflowError::ParseError(e.to_string()))?;

        workflow.validate()?;
        Ok(workflow)
    }

    /// Validate the workflow structure.
    fn validate(&self) -> Result<(), WorkflowError> {
        if self.name.is_empty() {
            return Err(WorkflowError::ValidationError(
                "Workflow name cannot be empty".to_string(),
            ));
        }

        if self.steps.is_empty() {
            return Err(WorkflowError::ValidationError(
                "Workflow must have at least one step".to_string(),
            ));
        }

        // Check for duplicate step IDs
        let mut seen_ids = std::collections::HashSet::new();
        for step in &self.steps {
            if !seen_ids.insert(&step.id) {
                return Err(WorkflowError::ValidationError(format!(
                    "Duplicate step ID: {}",
                    step.id
                )));
            }
        }

        Ok(())
    }

    /// Get the default agent tool for this workflow.
    #[must_use]
    pub fn default_tool(&self) -> Option<&str> {
        self.defaults.as_ref().and_then(|d| d.tool.as_deref())
    }

    /// Get a step by ID.
    #[must_use]
    pub fn get_step(&self, id: &str) -> Option<&WorkflowStep> {
        self.steps.iter().find(|s| s.id == id)
    }
}

impl WorkflowStep {
    /// Get the agent tool for this step (step-level or workflow default).
    #[must_use]
    pub fn tool_or_default<'a>(&'a self, workflow_default: Option<&'a str>) -> Option<&'a str> {
        self.agent.as_deref().or(workflow_default)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_WORKFLOW: &str = r#"
version: '1.0'
name: 'test-workflow'
description: 'A test workflow'
defaults:
  tool: claude
steps:
  - id: plan
    name: 'Planning'
    prompt: |
      Create a plan for: {{inputs.description}}
    outputs:
      - name: plan_file
        type: file
        filename: plan.md
  - id: implement
    name: 'Implementation'
    prompt: |
      Implement based on: {{steps.plan.outputs.plan_file}}
    outputs:
      - name: summary
        type: string
"#;

    #[test]
    fn test_parse_workflow() {
        let workflow = Workflow::parse(SAMPLE_WORKFLOW).expect("parse");

        assert_eq!(workflow.version, "1.0");
        assert_eq!(workflow.name, "test-workflow");
        assert_eq!(workflow.steps.len(), 2);
    }

    #[test]
    fn test_workflow_defaults() {
        let workflow = Workflow::parse(SAMPLE_WORKFLOW).expect("parse");

        assert_eq!(workflow.default_tool(), Some("claude"));
    }

    #[test]
    fn test_get_step() {
        let workflow = Workflow::parse(SAMPLE_WORKFLOW).expect("parse");

        let step = workflow.get_step("plan").expect("step exists");
        assert_eq!(step.name, "Planning");

        assert!(workflow.get_step("nonexistent").is_none());
    }

    #[test]
    fn test_step_outputs() {
        let workflow = Workflow::parse(SAMPLE_WORKFLOW).expect("parse");

        let plan_step = workflow.get_step("plan").expect("step");
        assert_eq!(plan_step.outputs.len(), 1);
        assert_eq!(plan_step.outputs[0].output_type, OutputType::File);
        assert_eq!(plan_step.outputs[0].filename, Some("plan.md".to_string()));
    }

    #[test]
    fn test_validation_empty_name() {
        let yaml = r#"
version: '1.0'
name: ''
steps:
  - id: test
    name: Test
    prompt: Do something
"#;
        let result = Workflow::parse(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_no_steps() {
        let yaml = r#"
version: '1.0'
name: 'empty'
steps: []
"#;
        let result = Workflow::parse(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_duplicate_ids() {
        let yaml = r#"
version: '1.0'
name: 'dupe'
steps:
  - id: same
    name: First
    prompt: First prompt
  - id: same
    name: Second
    prompt: Second prompt
"#;
        let result = Workflow::parse(yaml);
        assert!(result.is_err());
    }
}
