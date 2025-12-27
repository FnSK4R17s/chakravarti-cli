//! Prompt rendering using Handlebars templating.
//!
//! This module provides template rendering for workflow step prompts,
//! supporting variable substitution like `{{inputs.description}}` and
//! `{{steps.plan.outputs.plan_file}}`.

use handlebars::Handlebars;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

/// Prompt renderer using Handlebars templates.
pub struct PromptRenderer<'a> {
    handlebars: Handlebars<'a>,
}

/// Context for rendering a prompt template.
#[derive(Debug, Clone, Serialize)]
pub struct RenderContext {
    /// Input variables (from task initiation).
    pub inputs: HashMap<String, String>,
    /// Outputs from previous steps.
    pub steps: HashMap<String, StepOutputs>,
}

/// Outputs from a completed step.
#[derive(Debug, Clone, Serialize)]
pub struct StepOutputs {
    /// Named outputs from the step.
    pub outputs: HashMap<String, String>,
}

/// Errors from prompt rendering.
#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    /// Template rendering failed.
    #[error("Template rendering failed: {0}")]
    TemplateError(String),

    /// Missing required variable.
    #[error("Missing required variable: {0}")]
    MissingVariable(String),
}

impl<'a> PromptRenderer<'a> {
    /// Create a new prompt renderer.
    #[must_use]
    pub fn new() -> Self {
        let mut handlebars = Handlebars::new();
        // Strict mode: fail on missing variables
        handlebars.set_strict_mode(true);
        Self { handlebars }
    }

    /// Render a prompt template with the given context.
    ///
    /// # Errors
    ///
    /// Returns an error if rendering fails.
    pub fn render(&self, template: &str, context: &RenderContext) -> Result<String, RenderError> {
        self.handlebars
            .render_template(template, context)
            .map_err(|e| RenderError::TemplateError(e.to_string()))
    }

    /// Render a prompt with raw JSON context.
    ///
    /// # Errors
    ///
    /// Returns an error if rendering fails.
    pub fn render_with_json(&self, template: &str, context: &Value) -> Result<String, RenderError> {
        self.handlebars
            .render_template(template, context)
            .map_err(|e| RenderError::TemplateError(e.to_string()))
    }
}

impl Default for PromptRenderer<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderContext {
    /// Create a new empty render context.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inputs: HashMap::new(),
            steps: HashMap::new(),
        }
    }

    /// Add an input variable.
    #[must_use]
    pub fn with_input(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.inputs.insert(name.into(), value.into());
        self
    }

    /// Add outputs from a completed step.
    #[must_use]
    pub fn with_step_outputs(mut self, step_id: impl Into<String>, outputs: StepOutputs) -> Self {
        self.steps.insert(step_id.into(), outputs);
        self
    }

    /// Set the inputs map.
    pub fn set_inputs(&mut self, inputs: HashMap<String, String>) {
        self.inputs = inputs;
    }

    /// Record an output from a step.
    pub fn record_output(&mut self, step_id: &str, output_name: &str, value: String) {
        let step = self
            .steps
            .entry(step_id.to_string())
            .or_insert_with(|| StepOutputs {
                outputs: HashMap::new(),
            });
        step.outputs.insert(output_name.to_string(), value);
    }
}

impl Default for RenderContext {
    fn default() -> Self {
        Self::new()
    }
}

impl StepOutputs {
    /// Create new step outputs.
    #[must_use]
    pub fn new() -> Self {
        Self {
            outputs: HashMap::new(),
        }
    }

    /// Add an output.
    #[must_use]
    pub fn with_output(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.outputs.insert(name.into(), value.into());
        self
    }
}

impl Default for StepOutputs {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_simple_template() {
        let renderer = PromptRenderer::new();
        let context = RenderContext::new().with_input("description", "Add a login page");

        let template = "Create: {{inputs.description}}";
        let result = renderer.render(template, &context).expect("render");

        assert_eq!(result, "Create: Add a login page");
    }

    #[test]
    fn test_render_with_step_outputs() {
        let renderer = PromptRenderer::new();

        let plan_outputs = StepOutputs::new().with_output("plan_file", "plan.md content here");

        let context = RenderContext::new()
            .with_input("description", "Build feature")
            .with_step_outputs("plan", plan_outputs);

        let template = "Implement: {{steps.plan.outputs.plan_file}}";
        let result = renderer.render(template, &context).expect("render");

        assert_eq!(result, "Implement: plan.md content here");
    }

    #[test]
    fn test_render_multiline() {
        let renderer = PromptRenderer::new();
        let context = RenderContext::new().with_input("goal", "Add tests");

        let template = r#"
# Task
Goal: {{inputs.goal}}

Please proceed.
"#;
        let result = renderer.render(template, &context).expect("render");

        assert!(result.contains("Goal: Add tests"));
    }

    #[test]
    fn test_missing_variable_error() {
        let renderer = PromptRenderer::new();
        let context = RenderContext::new();

        let template = "Missing: {{inputs.nonexistent}}";
        let result = renderer.render(template, &context);

        assert!(result.is_err());
    }

    #[test]
    fn test_record_output() {
        let mut context = RenderContext::new();
        context.record_output("analyze", "context_file", "analysis.md".to_string());

        assert!(context.steps.contains_key("analyze"));
        assert_eq!(
            context.steps["analyze"].outputs["context_file"],
            "analysis.md"
        );
    }
}
