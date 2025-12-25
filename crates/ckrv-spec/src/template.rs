//! Spec template generation.

/// Default spec template content.
pub const SPEC_TEMPLATE: &str = r#"id: {id}
goal: {goal}

constraints:
  - Add constraint here

acceptance:
  - Define acceptance criterion here
"#;

/// Generate a spec file content from ID and optional goal.
#[must_use]
pub fn generate_spec_content(id: &str, goal: Option<&str>) -> String {
    SPEC_TEMPLATE
        .replace("{id}", id)
        .replace("{goal}", goal.unwrap_or("Describe your goal here"))
}
