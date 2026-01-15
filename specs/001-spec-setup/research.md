# Research: AI-Powered Spec Workflow

**Feature**: 001-spec-setup
**Date**: 2026-01-13
**Status**: Complete

## Research Questions

### RQ1: What prompt structure produces consistent, well-formatted spec.md files?

**Decision**: Use a structured template-based prompt with explicit section markers

**Rationale**:
- Including the exact template structure in the prompt ensures consistent output
- Using markdown format (`spec.md`) is more readable than YAML for human review
- Example output in the prompt helps Claude understand expected format

**Alternatives Considered**:
- Free-form prompts: Inconsistent output format
- YAML-only: Less readable for stakeholders
- Multi-turn conversation: Slower, more complex to implement

**Implementation**:
```rust
let prompt = format!(r#"
Generate a feature specification following this EXACT template:

# Feature Specification: {short_name}

## User Scenarios & Testing

### User Story 1 - [Title] (Priority: P1)
[Description]

**Acceptance Scenarios**:
1. **Given** [context], **When** [action], **Then** [outcome]

[Continue with template...]

FEATURE DESCRIPTION:
{description}
"#);
```

### RQ2: How to handle [NEEDS CLARIFICATION] markers?

**Decision**: Two-phase approach - generate with markers, then interactive resolve

**Rationale**:
- AI can identify ambiguities during generation
- User resolves ambiguities interactively
- Updated spec is then complete and unambiguous

**Alternatives Considered**:
- Ask all questions upfront: Overwhelming for users
- Ignore ambiguities: Poor spec quality
- AI guesses everything: May not match user intent

**Implementation**:
1. `ckrv spec new` generates spec with [NEEDS CLARIFICATION: question] markers
2. `ckrv spec clarify` parses markers and prompts user
3. User selects from options or provides custom answer
4. Spec is updated with resolved values

### RQ3: Single-shot vs multi-turn prompts?

**Decision**: Single-shot prompts for each command

**Rationale**:
- Simpler implementation
- Each command is idempotent
- No need to maintain conversation state
- Faster execution (no back-and-forth)

**Alternatives Considered**:
- Multi-turn: More natural but complex to implement
- Streaming: Good for progress feedback but not needed for spec generation

### RQ4: Template embedding strategy?

**Decision**: Use `include_str!` macro to embed templates at compile time

**Rationale**:
- Templates are version-locked with the binary
- No external file dependencies
- Can be overridden with project-local templates if needed

**Implementation**:
```rust
const SPEC_TEMPLATE: &str = include_str!("templates/spec-template.md");
const PLAN_TEMPLATE: &str = include_str!("templates/plan-template.md");
```

### RQ5: Output format - spec.yaml vs spec.md?

**Decision**: Keep spec.yaml as primary format (renders on UI), use design.md for technical documentation

**Rationale**:
- YAML is machine-parseable and renders well in the ckrv UI
- Rich YAML structure with nested user_stories, requirements, etc.
- Markdown (design.md) is better for technical prose
- Keep `plan.yaml` distinct - it's the execution plan for task orchestration

**Implementation**:
1. AI generates rich `spec.yaml` (primary, renders on UI)
2. `ckrv spec design` generates `design.md` (technical design document)
3. `ckrv plan` generates `plan.yaml` (execution plan - existing)
4. `tasks.yaml` for machine-readable task list

## Technology Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Primary spec format | Markdown (spec.md) | Human readable |
| Task format | YAML (tasks.yaml) | Machine parseable |
| Template storage | Embedded (include_str!) | Version locked |
| AI execution | Docker sandbox | Security, isolation |
| Clarification UX | Interactive CLI | Immediate feedback |

## Prompt Templates

### Spec Generation Prompt

Key elements for consistent output:
1. Explicit template structure with all sections
2. Clear instructions for each section
3. Feature description at the end (not beginning) to avoid influencing format
4. Examples of good/bad requirements

### Clarify Prompt

Key elements:
1. List of [NEEDS CLARIFICATION] markers found
2. For each, suggest 2-3 options
3. Ask for selection or custom input

### Plan Prompt

Key elements:
1. Full spec.md content as context
2. Technical context (language, dependencies)
3. Architecture decision guidance
4. Output structure for plan.md

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Claude unavailable | Graceful error with retry guidance |
| Poor AI output | Validation before saving, user can regenerate |
| Long generation time | Progress indicators, timeout handling |
| Template drift | Version templates with CLI binary |
