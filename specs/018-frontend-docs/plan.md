# Implementation Plan: Frontend Code Documentation

**Branch**: `018-frontend-docs` | **Date**: 2026-02-03 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/018-frontend-docs/spec.md`

## Summary

Add comprehensive documentation to all frontend React components and hooks following FRONTEND_CONVENTIONS.md patterns. This includes @module headers, Props JSDoc with @example blocks, section comments for large files, and inline comment improvements. The goal is enabling AI agents to understand component context from a single file read.

## Technical Context

**Language/Version**: TypeScript 5.x with React 18  
**Primary Dependencies**: React, Zustand (state), Framer Motion (animations), shadcn/ui  
**Storage**: N/A (documentation only)  
**Testing**: Vitest for any doc-related tests  
**Target Platform**: Web (Vite dev server)
**Project Type**: Web application (frontend only for this feature)  
**Performance Goals**: N/A (documentation changes only)  
**Constraints**: No code logic changes; documentation additive only  
**Scale/Scope**: 27 components + 12 hooks + 1 layout + 1 README

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Requirement | Status |
|-----------|-------------|--------|
| I. Code Quality Excellence | Self-documenting code, comments explain "why" not "what" | ✅ FR-011 aligns |
| II. Testing Standards | N/A - documentation only | ✅ Pass |
| III. Reliability First | N/A - no runtime changes | ✅ Pass |
| IV. Security by Default | N/A - no security impact | ✅ Pass |
| V. Deterministic CLI Behavior | Workflow produces consistent output | ✅ FR-010 ensures this |

## Project Structure

### Documentation (this feature)

```text
specs/018-frontend-docs/
├── plan.md              # This file
├── research.md          # Phase 0 output (existing pattern analysis)
├── spec.md              # Feature specification
├── checklists/
│   ├── conventions.md   # Documentation conventions checklist (40/44 ✓)
│   └── requirements.md  # Basic requirements checklist (16/16 ✓)
└── tasks.md             # Phase 2 output (to be generated)
```

### Source Code (target files)

```text
crates/ckrv-ui/frontend/src/
├── components/               # 27 .tsx files to document
│   ├── ExecutionRunner.tsx   # 1492 lines - CRITICAL (largest)
│   ├── AgentManager.tsx      # 991 lines - CRITICAL
│   ├── PlanEditor.tsx        # 886 lines
│   ├── TestRunner.tsx        # 834 lines
│   ├── TaskEditor.tsx        # 788 lines
│   ├── SpecEditor.tsx        # 671 lines
│   ├── TaskDetailModal.tsx   # 596 lines
│   ├── QAReviewer.tsx        # 560 lines
│   ├── BarebonesExecutor.tsx # 496 lines
│   ├── WorkflowPanel.tsx     # 399 lines (200-400 range)
│   ├── LogViewer.tsx         # 378 lines (200-400 range)
│   ├── DiffViewer.tsx        # 352 lines (200-400 range)
│   ├── CompletionSummary.tsx # 338 lines (200-400 range)
│   ├── TestFixModal.tsx      # 323 lines (200-400 range)
│   ├── CommandPalette.tsx    # 320 lines (200-400 range)
│   ├── AgentCliModal.tsx     # 310 lines (200-400 range)
│   ├── ChatDashboard.tsx     # 307 lines (200-400 range)
│   ├── SpecWorkflow.tsx      # 303 lines (200-400 range)
│   ├── RunHistoryPanel.tsx   # 273 lines (200-400 range)
│   ├── StatusWidget.tsx      # 228 lines (200-400 range)
│   ├── ClarifyModal.tsx      # 223 lines (200-400 range)
│   ├── BatchLogCarousel.tsx  # 185 lines
│   ├── BatchLogTerminal.tsx  # 184 lines
│   ├── NewSpecDialog.tsx     # 172 lines
│   ├── LogTerminal.tsx       # 163 lines
│   ├── CodePage.tsx          # 134 lines
│   └── ErrorBoundary.tsx     # 105 lines
│
├── hooks/                    # 12 .ts files to document
│   ├── useSpec.ts            # 331 lines - largest hook
│   ├── useLogStore.ts        # 290 lines
│   ├── useWebSocketReconnect.ts # 220 lines
│   ├── use-toast.ts          # 186 lines
│   ├── useRunHistory.ts      # 183 lines
│   ├── useFocusTrap.ts       # 134 lines
│   ├── useWorkflowProgress.ts # 98 lines
│   ├── useTimeout.ts         # 92 lines
│   ├── useAutoSelectedSpec.ts # 90 lines
│   ├── useCommand.ts         # 72 lines
│   ├── useConnection.ts      # 61 lines
│   └── useCodeTab.ts         # 47 lines
│
├── layouts/
│   └── Dashboard.tsx         # 1 layout file to document
│
├── services/
│   └── logService.ts         # Service file (optional scope)
│
├── types/                    # Type files (optional scope)
│   ├── history.ts
│   ├── log.ts
│   └── websocket.ts
│
├── App.tsx                   # Main app entry
├── main.tsx                  # Vite entry
├── types.ts                  # Shared types
└── lib/utils.ts              # Utility functions

frontend/
└── README.md                 # MUST update (currently Vite boilerplate)
```

**Structure Decision**: Frontend-only documentation feature. No backend changes. All modifications are additive documentation in existing files.

## File Classification

### By Priority (implement in this order)

| Priority | Category | Files | Rationale |
|----------|----------|-------|-----------|
| **P1** | Critical (>800 lines) | 4 files | Most complex, most AI benefit |
| **P2** | Large (400-800 lines) | 5 files | High complexity |
| **P3** | Medium (200-400 lines) | 12 files | SHOULD have section comments |
| **P4** | Small (<200 lines) | 6 files | Need @module + Props only |
| **P5** | Hooks | 12 files | Different pattern (add @param/@returns) |
| **P6** | README | 1 file | Update from boilerplate |

### By Required Changes

| Requirement | Applicable Files | Count |
|-------------|-----------------|-------|
| @module header (FR-001) | All components | 27 |
| @module header (FR-002) | All hooks | 12 |
| @param/@returns (FR-002a) | All hooks | 12 |
| Props @example (FR-003) | Components with Props | ~25 |
| Section comments (FR-006) | Files >400 lines | 9 |
| Section comments (FR-006a) | Files 200-400 lines | 12 |
| State grouping (FR-007) | Files with >5 useState | TBD |
| README update (FR-008) | frontend/README.md | 1 |
| Error handling docs (FR-012) | ErrorBoundary.tsx + others | ~5 |
| Naming check (FR-013) | All files | 39 |

## Implementation Phases

### Phase 1: Critical Components (9 files, >400 lines)

Files requiring full documentation with section comments:

1. **ExecutionRunner.tsx** (1492 lines) - Execution orchestration
2. **AgentManager.tsx** (991 lines) - Agent configuration  
3. **PlanEditor.tsx** (886 lines) - Plan editing
4. **TestRunner.tsx** (834 lines) - Test execution
5. **TaskEditor.tsx** (788 lines) - Task management
6. **SpecEditor.tsx** (671 lines) - Spec editing
7. **TaskDetailModal.tsx** (596 lines) - Task details
8. **QAReviewer.tsx** (560 lines) - QA review
9. **BarebonesExecutor.tsx** (496 lines) - Simple executor

**Each file needs**:
- [ ] @module header (5 sections)
- [ ] Props interface JSDoc with @example
- [ ] Section comments (STATE, EFFECTS, HANDLERS, RENDER)
- [ ] State grouping comments if >5 useState
- [ ] Error handling comments if applicable

### Phase 2: Medium Components (12 files, 200-400 lines)

Files with recommended (SHOULD) section comments:

1. WorkflowPanel.tsx (399)
2. LogViewer.tsx (378)
3. DiffViewer.tsx (352)
4. CompletionSummary.tsx (338)
5. TestFixModal.tsx (323)
6. CommandPalette.tsx (320)
7. AgentCliModal.tsx (310)
8. ChatDashboard.tsx (307)
9. SpecWorkflow.tsx (303)
10. RunHistoryPanel.tsx (273)
11. StatusWidget.tsx (228)
12. ClarifyModal.tsx (223)

**Each file needs**:
- [ ] @module header (5 sections)
- [ ] Props interface JSDoc with @example
- [ ] Section comments (RECOMMENDED)

### Phase 3: Small Components (6 files, <200 lines)

1. BatchLogCarousel.tsx (185)
2. BatchLogTerminal.tsx (184)
3. NewSpecDialog.tsx (172)
4. LogTerminal.tsx (163)
5. CodePage.tsx (134)
6. ErrorBoundary.tsx (105)

**Each file needs**:
- [ ] @module header (5 sections)
- [ ] Props interface JSDoc with @example
- [ ] Error handling docs for ErrorBoundary.tsx (FR-012)

### Phase 4: Hooks (12 files)

1. useSpec.ts (331)
2. useLogStore.ts (290)
3. useWebSocketReconnect.ts (220)
4. use-toast.ts (186)
5. useRunHistory.ts (183)
6. useFocusTrap.ts (134)
7. useWorkflowProgress.ts (98)
8. useTimeout.ts (92)
9. useAutoSelectedSpec.ts (90)
10. useCommand.ts (72)
11. useConnection.ts (61)
12. useCodeTab.ts (47)

**Each file needs**:
- [ ] @module header (5 sections)
- [ ] @param for each argument
- [ ] @returns describing return value

### Phase 5: Layout & README

1. **Dashboard.tsx** - Layout component
2. **frontend/README.md** - Update from Vite boilerplate

### Phase 6: Verification

Run `/docs.frontend` workflow verification:
- [ ] All @module headers present
- [ ] All Props have @example
- [ ] All >400 line files have section comments
- [ ] README updated
- [ ] 100% compliance (SC-008)

## Affected Files Summary

| Action | File Type | Count |
|--------|-----------|-------|
| [MODIFY] | Component files (.tsx) | 27 |
| [MODIFY] | Hook files (.ts) | 12 |
| [MODIFY] | Layout files (.tsx) | 1 |
| [MODIFY] | README.md | 1 |
| **TOTAL** | | **41 files** |

## CLI/UI Parity Check

N/A - This is a documentation-only feature. The `/docs.frontend` workflow handles both automated checks (CLI-like) and manual editing (UI-like) with identical outcomes per FR-010.

## Conventions Applied

- **FRONTEND_CONVENTIONS.md** - Primary reference for all documentation patterns
- **Constitution III (Reliability)** - Error handling documentation (FR-012)
- **Constitution I (Code Quality)** - Self-documenting code, "why" not "what" (FR-011)

## Complexity Tracking

No constitution violations. All requirements align with project principles.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| (none) | N/A | N/A |
