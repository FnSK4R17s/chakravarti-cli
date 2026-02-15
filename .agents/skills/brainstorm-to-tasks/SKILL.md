---
name: brainstorm-to-tasks
description: Generate a structured tasks.md file from a brainstorming document. Use when a brainstorm has matured and is ready for implementation planning.
license: MIT
compatibility: Requires shell and view_file capabilities
metadata:
  author: FnSK4R17s
  version: "1.0"
---

# Brainstorm to Tasks

Generate a structured `tasks.md` file from a brainstorming document. This skill extracts implementation phases, tasks, acceptance criteria, and dependencies from free-form brainstorming notes.

## When to Use

- Brainstorm status is "Ready for Implementation" or similar
- The notes contain enough detail to define concrete tasks
- You want to break down the work into phases with estimates

## Prerequisites

1. A brainstorming document exists (e.g., `brainstorming/issue-042-*/notes.md`)
2. The document contains implementation details, phases, or a technical approach section

## Execution Steps

### 1. Read the Brainstorming Document

// turbo
```bash
cat brainstorming/issue-${ISSUE_NUM}-*/notes.md
```

Or use `view_file` to read the document in chunks if large.

### 2. Extract Key Sections

Identify these sections in the brainstorming document:

| Section to Find | Maps To |
|-----------------|---------|
| Technical Approach / Architecture | Task breakdown structure |
| Implementation Phases / Steps | Phase definitions |
| User Stories | Acceptance criteria sources |
| Open Questions (resolved) | Implementation decisions |
| API Audit / Component List | Scope definition |
| Migration Steps | Ordered task list |
| Benefits / Goals | Success criteria |

### 2.5. Research Generated Documentation

**Critical Step**: Before defining tasks, research the generated documentation to find where components are imported and used. This reveals integration points that need updating.

**Documentation locations to check:**

| Docs Path | What to Find |
|-----------|--------------|
| `crates/<crate>/docs/README.md` | Component overview, dependency graph |
| `crates/<crate>/docs/<module>.md` | Function references, imports |
| `crates/ckrv-ui/docs/frontend-health-report.md` | Component import locations |
| `crates/ckrv-ui/FRONTEND_CONVENTIONS.md` | Component patterns and examples |
| `crates/docs/crate-overview.md` | Cross-crate dependencies |

**Research commands:**

```bash
# Find where a component is imported
grep -r "import.*ComponentName" crates/ckrv-ui/frontend/src/

# Find Rust module imports
grep -r "use crate_name::" crates/

# Check component references in docs
grep -r "ComponentName" crates/*/docs/

# List all files that import a module
grep -l "from.*module_path" crates/ckrv-ui/frontend/src/**/*.tsx
```

**What to extract:**
1. **Direct importers** → Files that import the component you're modifying
2. **Re-exporters** → Index files that re-export the component
3. **Documented usage** → Examples in docs that show integration patterns
4. **Test files** → Tests that exercise the component

**Add to tasks:** For each component being modified, create a sub-task to update importers if the API changes.

### 3. Define Phases

Based on the brainstorming content, organize work into logical phases:

**Common Phase Patterns:**

| Pattern | Phases |
|---------|--------|
| **New Crate** | 1. Scaffolding → 2. Core Types → 3. Implementation → 4. Integration → 5. Testing |
| **New Feature** | 1. Research → 2. API Design → 3. Backend → 4. Frontend → 5. Polish |
| **Refactor** | 1. Analysis → 2. Extract → 3. Migrate → 4. Cleanup → 5. Verify |
| **Migration** | 1. Prepare → 2. Create New → 3. Port Logic → 4. Switch Over → 5. Remove Old |

### 4. Extract Tasks from Content

For each phase, identify discrete tasks by looking for:

- **Action items**: "Create", "Add", "Implement", "Migrate", "Update"
- **Component lists**: Each component becomes a task
- **API endpoints**: Each endpoint group becomes a task
- **File changes**: Major file changes become tasks
- **Dependencies**: "After X, do Y" suggests task ordering

### 5. Estimate Tasks

Use this estimation guide:

| Complexity | Estimate | Examples |
|------------|----------|----------|
| Trivial | 15m | Config change, rename |
| Simple | 30m-1h | Single file, clear pattern |
| Medium | 2-4h | Multiple files, some research |
| Complex | 4-8h | New patterns, integration work |
| Large | 1-2d | Major feature, many touchpoints |

### 6. Define Acceptance Criteria

For each task, extract acceptance criteria from:

- User stories in the brainstorm
- Technical requirements mentioned
- Integration points listed
- Test cases implied

**Format:**
```markdown
**Acceptance Criteria**:
- [ ] Criterion 1 (testable condition)
- [ ] Criterion 2 (measurable outcome)
```

### 7. Generate tasks.md

Create the tasks file using the template structure:

```bash
ISSUE_NUM=42
FOLDER=$(ls -d brainstorming/issue-$(printf '%03d' $ISSUE_NUM)-* 2>/dev/null | head -1)
TITLE=$(head -1 "$FOLDER/notes.md" | sed 's/# //')

cat > "$FOLDER/tasks.md" << 'EOF'
# ${TITLE} - Tasks

**Issue**: [#${ISSUE_NUM}](https://github.com/FnSK4R17s/chakravarti-cli/issues/${ISSUE_NUM})
**Brainstorm**: [notes.md](./notes.md)
**Created**: $(date +%Y-%m-%d)

## Task Overview

| Phase | Tasks | Estimate |
|-------|-------|----------|
| Phase 1: Setup | 3 | 2h |
| Phase 2: Core | 5 | 6h |
| Phase 3: Integration | 4 | 4h |
| **Total** | 12 | 12h |

---

## Phase 1: Setup

### Task 1.1: ...
...
EOF
```

## Task Template

Use this structure for each task:

```markdown
### Task X.Y: {{Task Title}}
**Priority**: P0 | P1 | P2
**Estimate**: Xh
**Files**: `path/to/file.rs`, `another/file.ts`

<!-- Description extracted from brainstorm -->
Brief description of what this task accomplishes.

**Acceptance Criteria**:
- [ ] Criterion 1
- [ ] Criterion 2
- [ ] Tests pass: `cargo test -p <crate>`

---
```

## Priority Definitions

| Priority | Meaning | When to Use |
|----------|---------|-------------|
| P0 | Blocker | Other tasks depend on this |
| P1 | Critical | Core functionality |
| P2 | Important | Polish, optimization, testing |

## Dependency Mapping

Create a dependency diagram from task ordering mentioned in the brainstorm:

```markdown
## Dependencies

```
Phase 1 ─────────────────────────────────────────────────►
  Task 1.1 ──► Task 1.2 ──► Task 1.3
                   │
Phase 2 ───────────┼──────────────────────────────────────►
                   │
                   └───► Task 2.1 ──┬─► Task 2.2
                                    │
Phase 3 ────────────────────────────┼─────────────────────►
                                    │
                                    └─► Task 3.1 ──► Task 3.2
```
```

## Example Extraction

### From Brainstorm:
```markdown
## Technical Approach

### Phase 1: Create ckrv-transport crate
- Set up Cargo.toml with feature flags
- Define shared types in types.rs
- Add ts-rs for TypeScript generation

### Phase 2: Migrate handlers
- Move handlers from ckrv-ui to ckrv-transport
- Create Axum wrapper layer
- Create Tauri wrapper layer (stubs)
```

### Generated Tasks:
```markdown
## Phase 1: Create Transport Crate

### Task 1.1: Scaffold ckrv-transport crate
**Priority**: P0
**Estimate**: 30m
**Files**: `crates/ckrv-transport/Cargo.toml`, `crates/ckrv-transport/src/lib.rs`

Create the new crate with feature flags for axum and tauri.

**Acceptance Criteria**:
- [ ] `cargo check -p ckrv-transport` passes
- [ ] Feature flags `axum` and `tauri` defined
- [ ] Added to workspace Cargo.toml

---

### Task 1.2: Define shared types
**Priority**: P0
**Estimate**: 1h
**Files**: `crates/ckrv-transport/src/types/*.rs`

Create request/response types with ts-rs annotations.

**Acceptance Criteria**:
- [ ] All API request types defined
- [ ] All API response types defined
- [ ] `#[derive(TS)]` on exportable types
- [ ] `cargo test -p ckrv-transport --features typescript` generates types

---
```

## Output Location

Tasks file goes in the same folder as the brainstorm:

```
brainstorming/
└── issue-042-tauri-desktop-app/
    ├── notes.md      # Original brainstorm
    └── tasks.md      # Generated tasks (THIS OUTPUT)
```

## Validation Checklist

After generating tasks.md, verify:

- [ ] All phases from brainstorm are represented
- [ ] Each task has clear acceptance criteria
- [ ] Estimates are realistic (not too optimistic)
- [ ] Dependencies form a valid DAG (no cycles)
- [ ] P0 tasks are truly blocking
- [ ] File paths are accurate
- [ ] Total estimate aligns with scope

## Post-Generation

After creating tasks.md:

1. **Review with user**: Share the generated tasks for feedback
2. **Update brainstorm status**: Change to "Tasks Generated"
3. **Consider spec**: If feature is complex, run `/speckit.specify` next
4. **Track progress**: Tasks can be converted to GitHub issues with `/speckit.taskstoissues`

## Related Skills

| Skill | Relationship |
|-------|--------------|
| `brainstorming` | Source of input for this skill |
| `speckit.specify` | Next step for formal specification |
| `speckit.tasks` | Alternative for spec-first approach |
| `speckit.taskstoissues` | Convert tasks to GitHub issues |
