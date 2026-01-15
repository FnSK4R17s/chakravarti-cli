//! AI Prompt Builders for Spec Generation
//!
//! This module provides prompt construction utilities for Claude Code
//! to generate rich specifications, clarifications, and designs.

/// Embedded spec template
pub const SPEC_TEMPLATE: &str = include_str!("templates/spec-template.yaml");

/// Embedded design template  
pub const DESIGN_TEMPLATE: &str = include_str!("templates/design-template.md");

/// Embedded tasks template
pub const TASKS_TEMPLATE: &str = include_str!("templates/tasks-template.yaml");

/// Build a prompt for generating a rich spec.yaml from a description
pub fn build_spec_prompt(description: &str, spec_id: &str) -> String {
    format!(
        r#"Generate a comprehensive YAML specification for this feature.

FEATURE DESCRIPTION:
{description}

SPEC ID: {spec_id}

OUTPUT REQUIREMENTS:
Generate a YAML file following this EXACT structure. Output ONLY raw YAML - no markdown code fences, no explanations.

TEMPLATE STRUCTURE:
{template}

INSTRUCTIONS:
1. Replace all placeholder text in brackets with concrete content based on the feature description
2. Generate at least 3 user stories with realistic priorities (P1, P2, P3)
3. Each user story must have at least 2 acceptance scenarios in Given/When/Then format
4. Generate at least 5 functional requirements that are testable
5. Success criteria must include specific, measurable targets (numbers, percentages, time limits)
6. Edge cases should cover error scenarios, boundary conditions, and unusual usage
7. If something is unclear, add a clarification entry with options
8. Focus on WHAT and WHY, not HOW to implement
9. Keep it technology-agnostic - no specific languages, frameworks, or tools in requirements

QUALITY CHECKS:
- Every user story must be independently testable
- Every requirement must be verifiable  
- Every success criterion must be measurable
- Edge cases should inspire defensive implementation

Output the complete YAML now:"#,
        description = description,
        spec_id = spec_id,
        template = SPEC_TEMPLATE
    )
}

/// Build a prompt for resolving clarifications in a spec
pub fn build_clarify_prompt(spec_yaml: &str, clarifications: &[ClarificationItem]) -> String {
    let clarification_list: String = clarifications
        .iter()
        .enumerate()
        .map(|(i, c)| {
            format!(
                "{}. Topic: {}\n   Question: {}\n   Options: {:?}",
                i + 1,
                c.topic,
                c.question,
                c.options
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    format!(
        r#"Review this specification and help resolve the clarifications.

CURRENT SPEC:
{spec_yaml}

CLARIFICATIONS NEEDED:
{clarification_list}

For each clarification:
1. Analyze the context from the spec
2. Recommend the best option based on common patterns and best practices
3. Explain the implications of each choice

Format your response as:
CLARIFICATION 1:
  Recommended: [Option letter]
  Reasoning: [Why this is the best choice]
  
CLARIFICATION 2:
  Recommended: [Option letter]
  Reasoning: [Why this is the best choice]

[Continue for all clarifications]"#,
        spec_yaml = spec_yaml,
        clarification_list = clarification_list
    )
}

/// Build a prompt for generating a technical design document
pub fn build_design_prompt(spec_yaml: &str, spec_id: &str) -> String {
    format!(
        r#"Generate a technical design document for this feature specification.

SPECIFICATION:
{spec_yaml}

SPEC ID: {spec_id}

OUTPUT REQUIREMENTS:
Generate a markdown design document following this structure:
{template}

INSTRUCTIONS:
1. Analyze the user stories and requirements to determine architecture
2. Identify components needed for each user story
3. Define data model if the feature involves data
4. List technical decisions with rationale
5. Identify risks and mitigations
6. Keep the design practical and implementable

Output the complete markdown document now:"#,
        spec_yaml = spec_yaml,
        spec_id = spec_id,
        template = DESIGN_TEMPLATE
    )
}

/// Strip markdown code fences from AI output
pub fn strip_yaml_fences(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();
    let mut in_fence = false;
    let mut first_fence_skipped = false;

    for line in lines {
        if line.starts_with("```") {
            if !first_fence_skipped {
                // Skip the opening fence
                first_fence_skipped = true;
                in_fence = true;
                continue;
            } else if in_fence {
                // Skip the closing fence
                in_fence = false;
                continue;
            }
        }
        result.push(line);
    }

    result.join("\n")
}

/// A clarification item that needs user input
#[derive(Debug, Clone)]
pub struct ClarificationItem {
    pub topic: String,
    pub question: String,
    pub options: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_yaml_fences_with_fences() {
        let input = "```yaml\nkey: value\n```";
        let result = strip_yaml_fences(input);
        assert_eq!(result, "key: value");
    }

    #[test]
    fn test_strip_yaml_fences_without_fences() {
        let input = "key: value\nanother: thing";
        let result = strip_yaml_fences(input);
        assert_eq!(result, "key: value\nanother: thing");
    }

    #[test]
    fn test_build_spec_prompt_contains_description() {
        let prompt = build_spec_prompt("Add user authentication", "001-auth");
        assert!(prompt.contains("Add user authentication"));
        assert!(prompt.contains("001-auth"));
    }
}
