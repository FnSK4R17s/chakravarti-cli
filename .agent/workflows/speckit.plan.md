---
description: Execute the implementation planning workflow using the plan template to generate design artifacts.
handoffs: 
  - label: Create Tasks
    agent: speckit.tasks
    prompt: Break the plan into tasks
    send: true
  - label: Create Checklist
    agent: speckit.checklist
    prompt: Create a checklist for the following domain...
---

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).

## Outline

1. **Setup**: Run `.specify/scripts/bash/setup-plan.sh --json` from repo root and parse JSON for FEATURE_SPEC, IMPL_PLAN, SPECS_DIR, BRANCH. For single quotes in args like "I'm Groot", use escape syntax: e.g 'I'\''m Groot' (or double-quote if possible: "I'm Groot").

2. **Load context**: Read FEATURE_SPEC and `.specify/memory/constitution.md`. Load IMPL_PLAN template (already copied).

3. **Execute plan workflow**: Follow the structure in IMPL_PLAN template to:
   - Fill Technical Context (mark unknowns as "NEEDS CLARIFICATION")
   - Fill Constitution Check section from constitution
   - Evaluate gates (ERROR if violations unjustified)
   - Phase 0: Generate research.md (resolve all NEEDS CLARIFICATION)
   - Phase 1: Generate data-model.md, contracts/, quickstart.md
   - Phase 1: Update agent context by running the agent script
   - Re-evaluate Constitution Check post-design

4. **Stop and report**: Command ends after Phase 2 planning. Report branch, IMPL_PLAN path, and generated artifacts.

## Phases

### Phase 0: Existing Pattern Analysis (NEW - MANDATORY)

**This phase prevents implementation location errors by analyzing existing codebase patterns.**

1. **Identify similar features** in the codebase:
   - Search documentation: Read `crates/docs/architecture.md` and relevant `crates/<crate>/docs/README.md`
   - Search for keywords: `grep -r "<feature_keyword>" crates/ --include="*.rs" -l`
   - Example: If adding GLM support, search for "openrouter" to find where similar feature lives

2. **Map implementation locations**:
   ```bash
   # Find all files where similar feature is implemented
   grep -rn "<similar_feature>" crates/ --include="*.rs" | cut -d: -f1 | sort -u
   ```
   
3. **Document findings** in `research.md`:
   ```markdown
   ### Existing Pattern Analysis
   
   **Similar Feature**: [name]
   **Search Command**: `grep -r "..." crates/`
   
   **Implementation Locations**:
   | Crate | File | Purpose |
   |-------|------|---------|
   | ckrv-core | runner.rs | CLI execution path |
   | ckrv-cli | commands/task.rs | Config loading |
   | ckrv-ui | services/engine.rs | UI execution path |
   
   **CLI/UI Parity Check**:
   - CLI path: [files that handle CLI execution]
   - UI path: [files that handle UI execution]
   - Conclusion: New feature MUST be added to [list all locations] for parity
   ```

4. **ERROR if pattern not found**: If no similar feature exists, document this and proceed with architecture review instead.

### Phase 1: Research & Unknowns

1. **Extract unknowns from Technical Context** above:
   - For each NEEDS CLARIFICATION → research task
   - For each dependency → best practices task
   - For each integration → patterns task

2. **Generate and dispatch research agents**:

   ```text
   For each unknown in Technical Context:
     Task: "Research {unknown} for {feature context}"
   For each technology choice:
     Task: "Find best practices for {tech} in {domain}"
   ```

3. **Consolidate findings** in `research.md` using format:
   - Decision: [what was chosen]
   - Rationale: [why chosen]
   - Alternatives considered: [what else evaluated]

**Output**: research.md with Existing Pattern Analysis + all NEEDS CLARIFICATION resolved

### Phase 2: Design & Contracts

**Prerequisites:** `research.md` complete with pattern analysis

1. **Extract entities from feature spec** → `data-model.md`:
   - Entity name, fields, relationships
   - Validation rules from requirements
   - State transitions if applicable

2. **Generate API contracts** from functional requirements:
   - For each user action → endpoint
   - Use standard REST/GraphQL patterns
   - Output OpenAPI/GraphQL schema to `/contracts/`

3. **Determine affected files** based on Pattern Analysis:
   - List ALL files that need modification (not just one crate)
   - For each file, specify: `[MODIFY]`, `[CREATE]`, or `[OPTIONAL]`
   - **OPTIONAL changes require justification** (see Key Rules)

4. **Agent context update**:
   - Run `.specify/scripts/bash/update-agent-context.sh claude`
   - These scripts detect which AI agent is in use
   - Update the appropriate agent-specific context file
   - Add only new technology from current plan
   - Preserve manual additions between markers

**Output**: data-model.md, /contracts/*, quickstart.md, agent-specific file

## Key Rules

- Use absolute paths
- ERROR on gate failures or unresolved clarifications
- **CLI/UI Parity**: If a feature works in UI, it MUST also work via CLI (unless documented exception)
- **Pattern Analysis**: NEVER decide implementation location without first searching for existing similar patterns
- **Optional Changes Require Justification**: Any change marked `[OPTIONAL]` MUST include:
  ```markdown
  **Why optional**: [Clear explanation of why this is not required]
  **Risk if omitted**: [What functionality will be missing]
  **Recommendation**: [Keep optional / Make mandatory]
  ```

