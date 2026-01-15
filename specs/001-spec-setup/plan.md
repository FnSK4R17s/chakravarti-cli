# Implementation Plan: Improve Spec Setup with AI-Powered Workflow

**Branch**: `001-spec-setup` | **Date**: 2026-01-13 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-spec-setup/spec.md`

## Summary

Enhance `ckrv spec` to use Claude Code for generating comprehensive, multi-phase specifications similar to spec-kit. The current implementation creates a minimal YAML spec file; this enhancement adds a rich, interactive workflow with phases: specify → clarify → plan → tasks. Each phase generates detailed artifacts (research.md, design.md, data-model.md, tasks.yaml) using AI.

**Key Change**: Keep `spec.yaml` format (renders on UI) but make it rich with detailed user stories, requirements, and success criteria.

## Technical Context

**Language/Version**: Rust (current CLI) + Claude Code (AI execution)
**Primary Dependencies**: ckrv-sandbox (Docker execution), ckrv-git, clap (CLI), serde_yaml
**Storage**: Files in `.specs/<spec-id>/` directory (YAML + Markdown)
**Testing**: cargo test, manual integration testing with Claude
**Target Platform**: Linux/macOS CLI, Docker for sandboxed execution
**Project Type**: Single project (CLI extension)
**Performance Goals**: Spec generation in < 2 minutes, task generation in < 30 seconds
**Constraints**: Depends on Claude Code availability, Docker for sandboxed execution
**Scale/Scope**: Single-user CLI tool, local execution

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Requirement | Status |
|-----------|-------------|--------|
| I. Code Quality Excellence | Full typing, zero lint errors, single responsibility | ✅ Rust enforces |
| II. Testing Standards | TDD approach planned, coverage targets defined | ✅ Will add tests |
| III. Reliability First | Error handling strategy, idempotency considered | ✅ Existing patterns |
| IV. Security by Default | No hardcoded secrets, input validation planned | ✅ Uses Docker sandbox |
| V. Deterministic CLI Behavior | Machine-readable output, explicit exit codes | ✅ JSON mode supported |

## Project Structure

### Documentation (this feature - spec-kit structure)

```text
specs/001-spec-setup/            # THIS repo's spec-kit structure
├── plan.md                      # This file (spec-kit plan)
├── spec.md                      # Feature specification
├── research.md                  # Phase 0 output
├── data-model.md                # Phase 1 output
├── quickstart.md                # Phase 1 output
├── contracts/                   # Phase 1 output
└── tasks.md                     # Phase 2 output
```

### Enhanced ckrv spec output (user projects)

```text
.specs/<branch-name>/            # What ckrv spec creates in user projects
├── spec.yaml                    # Rich specification (renders on UI)
├── research.md                  # Research findings
├── design.md                    # Technical design (NOT plan.yaml!)
├── data-model.md                # Entity definitions
├── tasks.yaml                   # Implementation tasks
├── plan.yaml                    # Execution plan (existing - ckrv plan)
└── checklists/
    └── requirements.md
```

### Source Code (repository root)

**CLI Backend**:
```text
crates/ckrv-cli/src/
├── commands/
│   ├── spec.rs            # MODIFY: Add new subcommands (clarify, design, init)
│   └── spec_structs.rs    # MODIFY: Add new data structures
├── templates/             # NEW: Embedded spec templates
│   ├── spec-template.yaml
│   ├── design-template.md
│   └── tasks-template.yaml
└── lib/
    └── prompts.rs         # NEW: AI prompt builders

tests/
├── integration/
│   └── spec_workflow_test.rs  # NEW: End-to-end workflow tests
└── unit/
    └── prompts_test.rs        # NEW: Prompt generation tests
```

**UI Frontend** (React):
```text
crates/ckrv-ui/frontend/src/
├── components/
│   ├── SpecManager.tsx        # NEW: Spec list with actions (new, delete, duplicate)
│   ├── SpecEditor.tsx         # MODIFY: Rich spec.yaml editor with sections
│   ├── SpecWorkflow.tsx       # NEW: Workflow controls (clarify, design, tasks)
│   ├── SpecViewer.tsx         # NEW: Read-only spec display with sections
│   ├── ClarifyModal.tsx       # NEW: Interactive clarification dialog
│   └── DesignPanel.tsx        # NEW: Design.md viewer with research findings
├── pages/
│   └── SpecPage.tsx           # NEW: Main spec management page
└── hooks/
    └── useSpec.ts             # NEW: Spec state management hook

crates/ckrv-ui/src/routes/
├── spec.rs                    # NEW: API routes for spec operations
└── mod.rs                     # MODIFY: Register spec routes
```

**Structure Decision**: Extend existing `spec.rs` module with new subcommands. Add new UI components following existing patterns (shadcn/ui, TanStack Query). API routes follow existing REST conventions.

## Architecture Overview

### Current State (ckrv spec new)
```
User Input → Simple YAML prompt → Claude (Docker) → spec.yaml (minimal)
```

### Target State (ckrv spec workflow)
```
User Input → Rich YAML template prompt → Claude (Docker) → spec.yaml (rich)
                                                         → research.md (if needed)
                                                         → design.md (optional)
                                                         → tasks.yaml
```

**Note**: `spec.yaml` is the primary output (renders on UI). `design.md` is technical design documentation (distinct from `plan.yaml` which is the execution plan).

## Phase Breakdown

### Phase 0: Research - AI Prompt Engineering

**Objective**: Determine optimal prompts for generating spec-kit-quality output

**Research Questions**:
1. What prompt structure produces consistent, well-formatted spec.md files?
2. How to handle [NEEDS CLARIFICATION] markers in AI output?
3. Should we use multi-turn conversation or single-shot prompts?

**Findings**:
- Single-shot prompts with explicit template structure work best
- Include example output in prompt for format consistency
- Use markdown for human-readable specs, YAML for machine-readable data
- Break complex features into multiple prompts (spec → clarify → plan → tasks)

### Phase 1: Design - Enhanced Spec Command

**New Subcommands**:

| Command | Purpose | Output |
|---------|---------|--------|
| `ckrv spec init` | Initialize spec directory with templates | Empty template files |
| `ckrv spec new` | Generate spec from description (enhanced) | spec.yaml with rich content |
| `ckrv spec clarify` | Identify and resolve ambiguities | Updated spec.yaml |
| `ckrv spec design` | Generate technical design | design.md, research.md |
| `ckrv spec tasks` | Generate implementation tasks (existing) | tasks.yaml |
| `ckrv spec validate` | Validate spec quality (existing) | Pass/fail report |

**Data Model**:

```rust
// Enhanced spec output format
struct SpecOutput {
    id: String,
    goal: String,
    user_stories: Vec<UserStory>,
    requirements: Vec<Requirement>,
    success_criteria: Vec<Criterion>,
    edge_cases: Vec<String>,
    assumptions: Vec<String>,
    clarifications: Vec<Clarification>,  // NEW: [NEEDS CLARIFICATION] items
}

struct Clarification {
    topic: String,
    question: String,
    options: Vec<String>,
    resolved: Option<String>,
}
```

**Embedded Templates**:
- Include spec-kit templates directly in the binary using `include_str!`
- Templates define expected sections and format
- AI fills in content based on user description

### Phase 2: CLI Implementation Strategy

See [tasks.md](./tasks.md) for detailed task breakdown.

**Key CLI Implementation Points**:

1. **Enhance `execute_generate()`**:
   - Use richer prompt with spec-kit template structure
   - Generate rich spec.yaml (not minimal)
   - Include all required sections (user stories, requirements, success criteria)

2. **Add `execute_clarify()`**:
   - Parse existing spec for [NEEDS CLARIFICATION] markers
   - Present options to user interactively
   - Update spec with resolved clarifications

3. **Add `execute_init()`**:
   - Create spec directory structure
   - Copy empty template files
   - No AI required

4. **Add `execute_design()`** (new command):
   - Read spec.yaml
   - Generate research.md with decisions
   - Generate design.md with architecture
   - Store in spec directory

### Phase 3: UI Implementation

**New UI Components**:

| Component | Purpose | Features |
|-----------|---------|----------|
| `SpecManager.tsx` | Spec list view | List all specs, status badges, quick actions |
| `SpecEditor.tsx` | Rich spec viewer/editor | Collapsible sections, syntax highlighting |
| `SpecWorkflow.tsx` | Workflow controls | Buttons for clarify, design, tasks |
| `ClarifyModal.tsx` | Clarification dialog | Interactive Q&A with options |
| `DesignPanel.tsx` | Design viewer | Render design.md with research |
| `NewSpecDialog.tsx` | Create spec dialog | Description input, AI generation |

**API Routes** (Rust backend):

| Route | Method | Purpose |
|-------|--------|---------|
| `/api/specs` | GET | List all specs with status |
| `/api/specs/{id}` | GET | Get spec details |
| `/api/specs` | POST | Create new spec (triggers AI) |
| `/api/specs/{id}/clarify` | POST | Submit clarification answers |
| `/api/specs/{id}/design` | POST | Generate design.md |
| `/api/specs/{id}/tasks` | POST | Generate tasks.yaml |
| `/api/specs/{id}/validate` | GET | Run validation |

**UI Workflow**:

```
┌─────────────────────────────────────────────────────────────┐
│                     SPECS PAGE                              │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ [+ New Spec]  [Refresh]                             │   │
│  ├─────────────────────────────────────────────────────┤   │
│  │ 001-user-auth     ✅ Ready      [View] [Tasks]     │   │
│  │ 002-payment       ⚠️ Clarify    [View] [Clarify]   │   │
│  │ 003-dashboard     📝 Draft      [View] [Design]    │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼ Click "View"
┌─────────────────────────────────────────────────────────────┐
│                   SPEC VIEWER                               │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ 001-user-auth                    [Clarify] [Tasks]  │   │
│  ├─────────────────────────────────────────────────────┤   │
│  │ ▼ User Stories (3)                                   │   │
│  │   ├─ US1: Login with OAuth2 (P1) ✅                  │   │
│  │   ├─ US2: Password reset (P2)                        │   │
│  │   └─ US3: Remember me (P3)                           │   │
│  │ ▼ Requirements (5)                                   │   │
│  │   ├─ FR-001: System MUST support OAuth2...           │   │
│  │   └─ ...                                             │   │
│  │ ▼ Success Criteria (3)                               │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

## Complexity Tracking

> No Constitution violations anticipated. This is an extension of existing patterns.

| Aspect | Complexity | Justification |
|--------|------------|---------------|
| Prompt engineering | Medium | Need iterative refinement |
| Interactive clarify flow | Medium | New UX pattern |
| Template embedding | Low | Standard Rust pattern |
| Multi-phase workflow | Low | Sequential command execution |
| UI spec list/viewer | Medium | New components, follows existing patterns |
| UI workflow controls | Medium | API integration, progress states |
| UI clarify modal | High | Interactive Q&A, state management |

## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Spec quality | 90% pass validation | Automated check |
| Generation time | < 2 minutes | CLI timing |
| User adoption | Preferred over manual | User feedback |
| Task coverage | 100% user stories | Automated check |
| UI usability | Complete workflow in UI | E2E test |
| UI responsiveness | < 500ms for list/view | Performance test |

## Next Steps

1. Run `/speckit.tasks` to generate implementation tasks
2. Start with P1 (enhanced CLI spec generation) for MVP
3. Add clarify and design commands
4. Implement UI components (Phase 3)
5. Add API routes for UI integration
