//! AI Prompt Builders for Spec Generation
//!
//! This module provides prompt construction utilities for Claude Code
//! to generate rich specifications, clarifications, and designs.

// ============================================================
// TYPES
// ============================================================

/// Embedded spec template.
pub const SPEC_TEMPLATE: &str = include_str!("templates/spec-template.yaml");

/// Embedded design template.
pub const DESIGN_TEMPLATE: &str = include_str!("templates/design-template.md");

/// Embedded tasks template.
#[allow(dead_code)]
pub const TASKS_TEMPLATE: &str = include_str!("templates/tasks-template.yaml");

// ============================================================
// IMPLEMENTATION
// ============================================================

/// Build a prompt for generating a rich spec.yaml from a description.
pub fn build_spec_prompt(description: &str, spec_id: &str) -> String {
    format!(
        r"Generate a comprehensive YAML specification for this feature.

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
9. If the feature description mentions specific technologies, languages, or frameworks, PRESERVE them in the overview and requirements — they are explicit user choices, not suggestions. Only generalize aspects the user left unspecified.
10. The `input_prompt` field MUST contain the EXACT text from the FEATURE DESCRIPTION above — copy it verbatim, do not rephrase.

QUALITY CHECKS:
- Every user story must be independently testable
- Every requirement must be verifiable  
- Every success criterion must be measurable
- Edge cases should inspire defensive implementation

Output the complete YAML now:",
        description = description,
        spec_id = spec_id,
        template = SPEC_TEMPLATE
    )
}

/// Build a prompt for resolving clarifications in a spec
#[allow(dead_code)]
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
        r"Review this specification and help resolve the clarifications.

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

[Continue for all clarifications]",
        spec_yaml = spec_yaml,
        clarification_list = clarification_list
    )
}

/// Build a prompt for generating a technical design document
pub fn build_design_prompt(spec_yaml: &str, spec_id: &str) -> String {
    // Detect project context by scanning for common config files
    let cwd = std::env::current_dir().unwrap_or_default();
    let mut project_context = String::new();

    // Check for language/framework indicators
    let indicators: Vec<(&str, &str)> = vec![
        ("Cargo.toml", "Rust project (Cargo)"),
        ("package.json", "Node.js/JavaScript project"),
        ("go.mod", "Go project"),
        ("pyproject.toml", "Python project"),
        ("pom.xml", "Java (Maven) project"),
        ("build.gradle", "Java/Kotlin (Gradle) project"),
        ("Gemfile", "Ruby project"),
        ("composer.json", "PHP project"),
        ("mix.exs", "Elixir project"),
        ("tsconfig.json", "TypeScript configured"),
        ("vite.config.ts", "Vite frontend build tool"),
        ("vite.config.js", "Vite frontend build tool"),
        ("next.config.js", "Next.js framework"),
        ("next.config.ts", "Next.js framework"),
        ("angular.json", "Angular framework"),
        ("vue.config.js", "Vue.js framework"),
        ("tailwind.config.js", "Tailwind CSS"),
        ("tailwind.config.ts", "Tailwind CSS"),
        ("Dockerfile", "Docker containerization"),
        ("docker-compose.yml", "Docker Compose"),
        ("Makefile", "Make build system"),
        ("justfile", "Just command runner"),
    ];

    let mut detected: Vec<String> = Vec::new();
    for (file, desc) in &indicators {
        if cwd.join(file).exists() {
            detected.push(format!("- `{}` detected: {}", file, desc));
        }
    }

    if !detected.is_empty() {
        project_context = format!(
            "\n\nPROJECT CONTEXT (existing files in repository):\n{}\n\nYou MUST design for compatibility with the existing project setup above.",
            detected.join("\n")
        );
    }

    format!(
        r#"Generate a technical design document for this feature specification.

SPECIFICATION (this is the AUTHORITATIVE source of truth — follow it exactly):
{spec_yaml}

SPEC ID: {spec_id}
{project_context}

CRITICAL RULES:
- The spec contains an `input_prompt` field with the user's ORIGINAL request. This is the highest-priority source of intent. If it mentions specific technologies, you MUST use them.
- If the spec overview mentions specific technologies, languages, or frameworks, you MUST use them. Do NOT substitute alternatives.
- If clarifications have been resolved (resolved: field is not null), treat each resolved answer as a HARD REQUIREMENT in your design.
- If no tech stack is specified in either the input_prompt or the overview, AND no project context files are detected above, only then may you choose appropriate technologies — and you must justify each choice.
- Every design decision must trace back to a user story, requirement, or resolved clarification in the spec.

OUTPUT REQUIREMENTS:
Generate a markdown design document following this structure:
{template}

INSTRUCTIONS:
1. Read the spec overview carefully — extract any technology or architecture constraints mentioned
2. Read ALL resolved clarifications — each is a binding design decision
3. Analyze user stories and requirements to determine architecture
4. Identify components needed for each user story
5. Define data model if the feature involves data
6. List technical decisions with rationale (tied to spec requirements)
7. Identify risks and mitigations
8. Keep the design practical and implementable

Output the complete markdown document now:"#,
        spec_yaml = spec_yaml,
        spec_id = spec_id,
        project_context = project_context,
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

/// A clarification item that needs user input.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ClarificationItem {
    /// Topic area the clarification addresses.
    pub topic: String,
    /// Question to present to the user.
    pub question: String,
    /// Available answer options for the user to choose from.
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
