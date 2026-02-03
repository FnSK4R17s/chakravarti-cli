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

2. **Check for checklists** (Spec Quality Gate):
   
   - Check if `SPECS_DIR/checklists/` directory exists
   - If directory exists, list all `.md` files in it
   
   **If NO checklists exist**:
   
   ```text
   ⚠️  No checklists found in specs/<feature>/checklists/
   
   Before planning, you should validate your spec quality by running:
   
     /speckit.checklist requirements
     /speckit.checklist guiding_docs alignment   (ensures vision/principle alignment)
     /speckit.checklist <domain>                 (e.g., security, ux, api)
   
   Then review and check off items to ensure your spec is complete.
   
   This step helps you:
   • Catch gaps early (cheaper to fix in text than in code)
   • Build the "spec designer muscle" (learn what complete specs look like)
   • Reduce rework (plans built on solid specs don't need revision)
   • Ensure alignment with project vision and principles
   
   Do you want to proceed with planning anyway? (yes/no)
   ```
   
   - Wait for user response
   - If user says "no" or "stop", halt execution and suggest running `/speckit.checklist`
   - If user says "yes" or "proceed", continue to step 3 with a warning note
   
   **If checklists exist**, check their completion status:
   
   - Scan all checklist files in the checklists/ directory
   - For each checklist, count:
     - Total items: All lines matching `- [ ]` or `- [X]` or `- [x]`
     - Completed items: Lines matching `- [X]` or `- [x]`
     - Incomplete items: Lines matching `- [ ]`
   - Create a status table:

     ```text
     | Checklist        | Total | Completed | Incomplete | Status   |
     |------------------|-------|-----------|------------|----------|
     | requirements.md  | 8     | 8         | 0          | ✓ PASS   |
     | guiding_docs.md  | 5     | 5         | 0          | ✓ PASS   |
     | ux.md            | 12    | 10        | 2          | ✗ FAIL   |
     | security.md      | 6     | 6         | 0          | ✓ PASS   |
     ```

   - Calculate overall status:
     - **PASS**: All checklists have 0 incomplete items
     - **FAIL**: One or more checklists have incomplete items
   
   - **Check for guiding_docs alignment checklist**:
     - If `guiding_docs/` directory exists in the project root BUT no `guiding_docs.md` or `guiding_docs_alignment.md` checklist exists:
       
       ```text
       💡 Tip: This project has guiding documents (vision, principles) in guiding_docs/
       
       Consider running:
         /speckit.checklist guiding_docs alignment
       
       This ensures your spec aligns with project vision and principles before planning.
       ```

   **If any checklist is incomplete**:
   
   ```text
   ⚠️  Some checklists are incomplete!
   
   Before creating a technical plan, please review your spec against the 
   checklist items and either:
   
   1. Check off items that your spec already covers: - [x]
   2. Update your spec.md to address any gaps you find
   
   Incomplete items may indicate missing requirements that would lead to
   an incomplete plan and rework later.
   
   Tip: Open each checklist file and review items one by one. This is 
   where you build the "spec designer muscle"!
   
   Do you want to proceed with planning anyway? (yes/no)
   ```
   
   - Wait for user response
   - If user says "no" or "stop", halt execution and list the incomplete checklists
   - If user says "yes" or "proceed", continue to step 3 with a warning note
   
   **If all checklists are complete**:
   
   - Display: `✓ All checklists passed! Your spec has been reviewed.`
   - Automatically proceed to step 3

3. **Load context**: Read FEATURE_SPEC and `.specify/memory/constitution.md`. Load IMPL_PLAN template (already copied).

4. **Execute plan workflow**: Follow the structure in IMPL_PLAN template to:
   - Fill Technical Context (mark unknowns as "NEEDS CLARIFICATION")
   - Fill Constitution Check section from constitution
   - Evaluate gates (ERROR if violations unjustified)
   - Phase 0: Generate research.md (resolve all NEEDS CLARIFICATION)
   - Phase 1: Generate data-model.md, contracts/, quickstart.md
   - Phase 1: Update agent context by running the agent script
   - Re-evaluate Constitution Check post-design

5. **Stop and report**: Command ends after Phase 2 planning. Report branch, IMPL_PLAN path, and generated artifacts.

## Phases

### Phase 0: Existing Pattern Analysis (NEW - MANDATORY)

**This phase prevents implementation location errors by analyzing existing codebase patterns.**

1. **Consult auto-generated documentation** (PRIMARY REFERENCE):
   
   > **Important**: The project maintains auto-generated docs that are tagged with git commits for freshness tracking. These are your best source of truth for understanding the codebase.
   
   - Read `crates/docs/architecture.md` for system overview and crate responsibilities
   - Read relevant `crates/<crate>/docs/README.md` for crate-specific details
   - Check the `last_commit:` frontmatter in each doc to verify freshness
   - If docs are stale (commit doesn't match recent changes), note this in research.md

2. **Identify similar features** in the codebase:
   - Use the docs from step 1 to understand where similar functionality lives
   - Search for keywords: `grep -r "<feature_keyword>" crates/ --include="*.rs" -l`
   - Example: If adding GLM support, search for "openrouter" to find where similar feature lives

3. **Check for conventions files** (MUST RESPECT):
   
   ```bash
   # Find all convention files in the project
   find . -name "*_CONVENTIONS.md" -o -name "*_CONVENTIONS.txt" 2>/dev/null
   ```
   
   - Read any `*_CONVENTIONS.md` files found (e.g., `RUST_CONVENTIONS.md`, `API_CONVENTIONS.md`)
   - These define coding standards that MUST be followed when creating code changes
   - Document which conventions apply to this feature in research.md

4. **Map implementation locations**:
   ```bash
   # Find all files where similar feature is implemented
   grep -rn "<similar_feature>" crates/ --include="*.rs" | cut -d: -f1 | sort -u
   ```
   
5. **Document findings** in `research.md`:
   ```markdown
   ### Existing Pattern Analysis
   
   **Similar Feature**: [name]
   **Search Command**: `grep -r "..." crates/`
   **Docs Consulted**: [list of docs read with their commit tags]
   **Conventions Applied**: [list of *_CONVENTIONS.md files that apply]
   
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

6. **ERROR if pattern not found**: If no similar feature exists, document this and proceed with architecture review instead.

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

