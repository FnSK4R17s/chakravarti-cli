---
generated: 2026-02-03
commit: a64990a
workflow: /docs.frontend
---

# Frontend Health Report

## Executive Summary

| Metric | Value | Status |
|--------|-------|--------|
| **Total Components** | 27 | - |
| **Total Lines of Code** | 12,011 | - |
| **Components > 500 LOC** | 8 | 🔴 Critical |
| **Components 200-500 LOC** | 10 | ⚠️ Warning |
| **`any` Usages** | 0 | ✅ Good |
| **Missing JSDoc** | 17/27 (63%) | 🔴 Critical |
| **Missing Props Interface** | 10/27 (37%) | ⚠️ Warning |
| **TODO/FIXME Items** | 1 | ✅ Good |
| **Console Statements** | 26 | ⚠️ Warning |
| **Inline Styles** | 26 | ⚠️ Warning |

---

## 🔴 Critical: Oversized Components

Components over 500 lines need **immediate splitting**:

| Component | Lines | useState | useEffect | Imports | Status |
|-----------|-------|----------|-----------|---------|--------|
| `ExecutionRunner.tsx` | **1,492** | 19 | 7 | 15 | 🔴 **Massive** |
| `AgentManager.tsx` | **991** | 8 | 1 | 14 | 🔴 Very Large |
| `PlanEditor.tsx` | **886** | 10 | 3 | 11 | 🔴 Very Large |
| `TestRunner.tsx` | **834** | 14 | 3 | 12 | 🔴 Very Large |
| `TaskEditor.tsx` | **788** | 10 | 2 | 13 | 🔴 Very Large |
| `SpecEditor.tsx` | **671** | 9 | 2 | 12 | 🔴 Large |
| `TaskDetailModal.tsx` | **596** | 4 | 3 | 12 | 🔴 Large |
| `QAReviewer.tsx` | **560** | 7 | 0 | 11 | 🔴 Large |

### Detailed Analysis: `ExecutionRunner.tsx`

This is the worst offender with **1,492 lines** and:
- **19 useState calls** - massive state complexity
- **7 useEffect calls** - multiple side effects interacting
- **15 imports** - high coupling

**Recommended Split:**
1. `ExecutionControls.tsx` - Start/Stop buttons, status display
2. `ExecutionBatchPanel.tsx` - Batch selection and carousel
3. `ExecutionWebSocket.tsx` - WebSocket connection (extract to hook)
4. `ExecutionLogs.tsx` - Log display and terminal embedding
5. `ExecutionHistory.tsx` - Historical runs panel
6. `ExecutionMerge.tsx` - Branch merging UI

---

## ⚠️ Warning: Medium Components (200-500 LOC)

These should be monitored and refactored when touched:

| Component | Lines | Notes |
|-----------|-------|-------|
| `BarebonesExecutor.tsx` | 496 | Consider splitting |
| `WorkflowPanel.tsx` | 399 | Acceptable |
| `LogViewer.tsx` | 378 | Could extract logic |
| `DiffViewer.tsx` | 352 | Acceptable |
| `CompletionSummary.tsx` | 338 | Acceptable |
| `TestFixModal.tsx` | 323 | Acceptable |
| `CommandPalette.tsx` | 320 | Acceptable |
| `AgentCliModal.tsx` | 310 | Acceptable |
| `ChatDashboard.tsx` | 307 | Acceptable |
| `SpecWorkflow.tsx` | 303 | Acceptable |

---

## ✅ Good: Small Components (< 200 LOC)

These are well-sized and maintainable:

| Component | Lines |
|-----------|-------|
| `RunHistoryPanel.tsx` | 273 |
| `StatusWidget.tsx` | 228 |
| `ClarifyModal.tsx` | 223 |
| `BatchLogCarousel.tsx` | 185 |
| `BatchLogTerminal.tsx` | 184 |
| `NewSpecDialog.tsx` | 172 |
| `LogTerminal.tsx` | 163 |
| `CodePage.tsx` | 134 |
| `ErrorBoundary.tsx` | 105 |

---

## Documentation Coverage

### JSDoc Status

| Status | Components |
|--------|------------|
| ❌ Missing | AgentCliModal, AgentManager, ChatDashboard, CommandPalette, DiffViewer, ExecutionRunner, LogTerminal, LogViewer, PlanEditor, QAReviewer, SpecEditor, StatusWidget, TaskDetailModal, TaskEditor, TestFixModal, TestRunner, WorkflowPanel |
| ✅ Has JSDoc | BarebonesExecutor, BatchLogCarousel, BatchLogTerminal, ClarifyModal, CodePage, CompletionSummary, ErrorBoundary, NewSpecDialog, RunHistoryPanel, SpecWorkflow |

**Coverage: 37%** (10/27)

### Props Interface Status

| Status | Components |
|--------|------------|
| ⚠️ Missing | BarebonesExecutor, ChatDashboard, DiffViewer, ExecutionRunner, LogViewer, PlanEditor, QAReviewer, SpecEditor, TaskEditor, TestRunner |
| ✅ Has Props | AgentCliModal, AgentManager, BatchLogCarousel, BatchLogTerminal, ClarifyModal, CodePage, CommandPalette, CompletionSummary, ErrorBoundary, LogTerminal, NewSpecDialog, RunHistoryPanel, SpecWorkflow, StatusWidget, TaskDetailModal, TestFixModal, WorkflowPanel |

**Coverage: 63%** (17/27)

---

## Hooks Analysis

| Hook | Lines | Complexity |
|------|-------|------------|
| `useSpec.ts` | 331 | ⚠️ Large |
| `useLogStore.ts` | 290 | ⚠️ Large |
| `useWebSocketReconnect.ts` | 220 | ⚠️ Medium |
| `use-toast.ts` | 186 | Acceptable |
| `useRunHistory.ts` | 183 | Acceptable |
| `useFocusTrap.ts` | 134 | Acceptable |
| `useWorkflowProgress.ts` | 98 | ✅ Good |
| `useTimeout.ts` | 92 | ✅ Good |
| `useAutoSelectedSpec.ts` | 90 | ✅ Good |
| `useCommand.ts` | 72 | ✅ Good |
| `useConnection.ts` | 61 | ✅ Good |
| `useCodeTab.ts` | 47 | ✅ Good |

**Total: 1,804 lines across 12 hooks**

---

## Code Quality Issues

### Console Statements (26 total)

| File | Count | Type |
|------|-------|------|
| `ExecutionRunner.tsx` | 11 | Mixed (log, error, warn) |
| `ErrorBoundary.tsx` | 2 | console.error (acceptable) |
| `LogViewer.tsx` | 1 | console.error |
| `PlanEditor.tsx` | 1 | console.error |
| `TestFixModal.tsx` | 1 | console.error |
| `LogTerminal.tsx` | 1 | console.warn |

**Action:** Replace debug `console.log` with proper logging or remove before production.

### Inline Styles (26 total)

Inline styles should be migrated to Tailwind classes where possible.

### TODO/FIXME (1 total)

- `TestRunner.tsx:770` - `// TODO: Send prompt to agent`

---

## Recommendations

### 🔴 Immediate Actions (Critical)

1. **Split `ExecutionRunner.tsx`** - This 1,492-line component is unmaintainable
   - Extract WebSocket logic to `useExecutionWebSocket` hook
   - Extract batch panel to `ExecutionBatchPanel.tsx`
   - Extract controls to `ExecutionControls.tsx`
   - Extract log view to `ExecutionLogPanel.tsx`

2. **Split `AgentManager.tsx`** (991 lines)
   - Extract agent list to `AgentList.tsx`
   - Extract agent form to `AgentForm.tsx`
   - Extract role management to `AgentRoles.tsx`

3. **Split `PlanEditor.tsx`** (886 lines)
   - Extract plan tree to `PlanTree.tsx`
   - Extract task panel to `PlanTaskPanel.tsx`

### ⚠️ Short-term Actions (This Week)

4. **Add JSDoc to all components** - 17 components missing documentation
5. **Add Props interfaces** - 10 components missing proper typing
6. **Remove debug console.log** - 11 statements in ExecutionRunner alone

### 📋 Long-term Technical Debt

7. **Consider state management** - Components with 10+ useState calls need better state management (Zustand, Jotai)
8. **Extract common patterns** - Similar loading/error states could be abstracted
9. **Add Storybook** - For component testing and documentation
10. **Add component tests** - No unit tests exist for components

---

## Component-by-Component Priority

| Priority | Component | Action Required |
|----------|-----------|-----------------|
| P0 | ExecutionRunner | Split into 5+ components |
| P0 | AgentManager | Split into 3+ components |
| P1 | PlanEditor | Split or extract hooks |
| P1 | TestRunner | Split or extract hooks |
| P1 | TaskEditor | Split or extract hooks |
| P2 | SpecEditor | Extract hooks |
| P2 | TaskDetailModal | Extract hooks |
| P2 | QAReviewer | Add Props interface |
| P3 | All | Add JSDoc documentation |

---

## Metrics Tracking

| Date | Components > 500 | Missing JSDoc | Missing Props | Console.log |
|------|------------------|---------------|---------------|-------------|
| 2026-02-03 | 8 | 17 | 10 | 26 |
| Target | 0 | 0 | 0 | < 5 |

---

*Generated by `/docs.frontend` workflow*
