# Bug Audit: Chakravarti CLI Web UI

**Feature**: 006-bug-free-polished-ui  
**Date**: 2026-01-05  
**Status**: In Progress - Initial Discovery Complete

## Bug Catalog

### P1 - Critical (Blocks Core Workflow)

---

#### BUG-001: Execution Runner UI Freeze During Rapid Log Updates

**Component**: `ExecutionRunner.tsx`  
**Lines Affected**: 536-566 (WebSocket onmessage handler)

**Steps to Reproduce**:
1. Start a multi-batch execution with 5+ parallel batches
2. Observe WebSocket receiving rapid messages (>10/second)
3. UI becomes unresponsive or stutters significantly

**Expected Behavior**: UI remains responsive with smooth log streaming at 60fps

**Actual Behavior**: UI freezes intermittently, frame drops visible

**Root Cause Analysis**:
- Each WebSocket message triggers immediate state updates (`setBatches`, `setBatchLogs`, `addLog`)
- React re-renders synchronously for each message
- No message batching or throttling implemented
- `updateBatchFromLog` function runs expensive string matching operations on every message

**Proposed Fix**:
1. Batch WebSocket messages using `requestAnimationFrame`
2. Use `useTransition` for non-urgent updates
3. Debounce batch status updates (only update state once per 100ms)
4. Move regex matching outside render cycle

**Regression Test**: E2E test that sends 50+ messages in 1 second and verifies UI responsiveness

---

#### BUG-002: WebSocket Connection Not Reconnected on Network Failure

**Component**: `ExecutionRunner.tsx`  
**Lines Affected**: 568-577 (ws.onerror, ws.onclose handlers)

**Steps to Reproduce**:
1. Start execution
2. Disconnect network briefly (airplane mode)
3. Reconnect network
4. Observe connection status

**Expected Behavior**: Auto-retry with countdown indicator, automatic reconnection

**Actual Behavior**: Connection closes permanently, no retry mechanism, status shows "failed"

**Root Cause Analysis**:
- `ws.onerror` immediately sets `executionStatus('failed')` without retry
- `ws.onclose` only logs a message, doesn't attempt reconnection
- No reconnection logic exists in the codebase

**Proposed Fix**:
1. Implement retry logic with exponential backoff (3 attempts, 5s/10s/20s intervals)
2. Add visible countdown indicator showing retry status
3. Only set 'failed' status after all retries exhausted
4. Add 'reconnecting' status to ExecutionStatus type

**Regression Test**: E2E test that simulates network interruption and verifies reconnection

---

#### BUG-003: Terminal Not Cleared on Reset

**Component**: `ExecutionRunner.tsx`  
**Lines Affected**: 580-612 (handleRun function)

**Steps to Reproduce**:
1. Run an execution to completion
2. Click "Reset" or start a new run
3. Previous logs may still appear in terminal

**Expected Behavior**: Terminal cleared completely, fresh state

**Actual Behavior**: `terminalRef.current?.clear()` is called but old logs may persist if terminal reference is stale

**Root Cause Analysis**:
- The optional chaining (`?.`) may skip clear if terminal isn't mounted yet
- No guarantee terminal ref is current when clear is called
- Race condition between terminal mount and clear call

**Proposed Fix**:
1. Add explicit check that terminal is mounted before clearing
2. Use callback ref pattern for terminal initialization
3. Add retry logic for clear operation

**Regression Test**: Unit test verifying terminal.clear() is called successfully

---

### P2 - Major (Degraded Experience)

---

#### BUG-004: Inconsistent Loading States Across Components

**Components**: `CommandPalette.tsx`, `SpecEditor.tsx`, `TaskEditor.tsx`, `PlanEditor.tsx`

**Steps to Reproduce**:
1. Navigate between different pages rapidly
2. Observe loading indicators during data fetches
3. Some components show loaders, others don't

**Expected Behavior**: Consistent loading indicator pattern across all async operations

**Actual Behavior**: 
- Some mutations show `Loader2` spinning icon
- Some queries have no loading indicator
- Inconsistent placement (some inline, some overlay)

**Root Cause Analysis**:
- Each component implements loading states differently
- No shared loading component or pattern
- `isLoading` from `useQuery` not consistently checked

**Proposed Fix**:
1. Create reusable `LoadingOverlay` component
2. Create reusable `LoadingButton` component
3. Standardize loading indicator placement and animation

**Regression Test**: Visual regression test for loading states

---

#### BUG-005: Auto-Collapse Timer Creates Memory Leak

**Component**: `ExecutionRunner.tsx`  
**Lines Affected**: 495-498

**Steps to Reproduce**:
1. Run execution with multiple batches
2. Let batches complete
3. Navigate away from Runner page before 5s collapse timer fires
4. Memory leak occurs due to orphaned setTimeout

**Expected Behavior**: Timer cleaned up on component unmount

**Actual Behavior**: 
```javascript
setTimeout(() => {
    setBatchCompletedAt(prev => ({ ...prev })); // Force re-render
}, 5100);
```
This timeout is not cleared on unmount.

**Root Cause Analysis**:
- setTimeout created inside `updateBatchFromLog` callback
- No reference stored for cleanup
- Not part of useEffect cleanup pattern

**Proposed Fix**:
1. Track timeout IDs in a ref
2. Clear all timeouts in component cleanup
3. Consider using `useTimeout` custom hook

**Regression Test**: Unit test verifying cleanup on unmount

---

#### BUG-006: Missing Error Boundary for Component Crashes

**Components**: All components

**Steps to Reproduce**:
1. Trigger any unhandled error in a component (e.g., malformed API response)
2. Entire app crashes with white screen

**Expected Behavior**: Graceful error display with option to retry

**Actual Behavior**: React error boundary not implemented, app crashes completely

**Root Cause Analysis**:
- No `ErrorBoundary` component wrapping the app or individual pages
- Unhandled promise rejections in API calls can crash components
- Some `try/catch` blocks only log errors, don't update UI state

**Proposed Fix**:
1. Create `ErrorBoundary` component
2. Wrap each page-level component
3. Show user-friendly error message with retry option
4. Log errors to console for debugging

**Regression Test**: Test that simulates component error and verifies error UI displays

---

#### BUG-007: Batch Log Panel Log Scroll Issues

**Component**: `ExecutionRunner.tsx` → `BatchLogPanel`  
**Lines Affected**: 180-273

**Steps to Reproduce**:
1. Expand a batch log panel during active execution
2. Logs stream in rapidly
3. Scroll position jumps unexpectedly or doesn't auto-scroll

**Expected Behavior**: Auto-scroll to bottom for new logs, manual scroll pauses auto-scroll

**Actual Behavior**: 
- No explicit auto-scroll implementation in BatchLogPanel
- Logs may render outside visible area
- User scroll position not preserved

**Root Cause Analysis**:
- BatchLogPanel uses `overflow-y-auto` but no scroll management
- No `scrollIntoView` or similar call when logs update
- No detection of user scroll to pause auto-scroll

**Proposed Fix**:
1. Add ref to log container
2. Implement auto-scroll to bottom on new entries
3. Detect user scroll up and pause auto-scroll
4. Add "scroll to bottom" button when not at bottom

**Regression Test**: E2E test verifying scroll behavior during log streaming

---

#### BUG-008: XTerm.js FitAddon Race Condition

**Component**: `LogTerminal.tsx`  
**Lines Affected**: 89-94

**Steps to Reproduce**:
1. Load page with terminal
2. Rapidly resize browser window
3. Terminal may not fit container properly

**Expected Behavior**: Terminal always fits container after resize

**Actual Behavior**: 
```javascript
useEffect(() => {
    const timer = setTimeout(() => {
        fitAddonRef.current?.fit();
    }, 100);
    return () => clearTimeout(timer);
});
```
This useEffect runs on every render (no dependency array), causing excessive fit() calls.

**Root Cause Analysis**:
- Missing dependency array causes effect to run every render
- 100ms delay adds up with rapid re-renders
- Race conditions between multiple fit() calls

**Proposed Fix**:
1. Add proper dependency array `[]` if only on mount, or proper deps
2. Use ResizeObserver instead of window resize event for container-level changes
3. Debounce fit() calls during rapid resizes

**Regression Test**: Test terminal resize behavior

---

### P3 - Minor (Polish Issues)

---

#### BUG-009: Inconsistent Animation Durations

**Components**: Various

**Steps to Reproduce**:
1. Open various modals and panels across the app
2. Observe animation timing differences

**Expected Behavior**: Consistent 200-300ms animation duration per spec

**Actual Behavior**: 
- Some animations are 150ms
- Some are 300ms
- Some have no animation at all

**Root Cause Analysis**:
- Animation durations hardcoded in individual components
- No shared animation timing constants
- `transition-all` used without explicit duration in some places

**Proposed Fix**:
1. Define animation timing CSS custom properties
2. Standardize on 200ms for micro-interactions, 300ms for modals/panels
3. Audit all components for consistent timing

**Regression Test**: Visual regression test for animation consistency

---

#### BUG-010: Focus Trap Missing in Modals

**Components**: `AgentManager.tsx`, `CommandPalette.tsx`, `TaskDetailModal.tsx`

**Steps to Reproduce**:
1. Open any modal dialog
2. Press Tab repeatedly
3. Focus escapes modal and goes to background elements

**Expected Behavior**: Focus trapped within modal, cycles through modal elements only

**Actual Behavior**: Focus escapes modal, can interact with background content

**Root Cause Analysis**:
- No focus trap library or implementation
- Modal backdrop clickable but focus not managed
- Escape key handler exists but focus not returned to trigger

**Proposed Fix**:
1. Implement focus trap using `focus-trap-react` or custom hook
2. Return focus to triggering element on close
3. Prevent scroll on body when modal open

**Regression Test**: Accessibility test for focus management

---

#### BUG-011: Missing ARIA Labels on Interactive Elements

**Components**: Various

**Steps to Reproduce**:
1. Navigate app with screen reader
2. Encounter unlabeled buttons and icons

**Expected Behavior**: All interactive elements have appropriate ARIA labels

**Actual Behavior**: 
- Icon-only buttons have no labels
- Some status indicators lack aria-label
- Progress elements missing role and value attributes

**Root Cause Analysis**:
- Accessibility not audited during development
- Lucide icons render as SVG without accessible labels
- Focus indicators partially implemented

**Proposed Fix**:
1. Add aria-label to all icon-only buttons
2. Add role and aria-* attributes to custom widgets
3. Ensure visible focus indicators per spec FR-012

**Regression Test**: Accessibility audit using axe-core

---

#### BUG-012: Hardcoded Colors Instead of CSS Variables

**Components**: `LogTerminal.tsx`, possibly others

**Steps to Reproduce**:
1. Review LogTerminal theme configuration
2. Colors are hardcoded: `background: '#1e1e1e'`

**Expected Behavior**: Use CSS custom properties for theme consistency

**Actual Behavior**: XTerm.js theme colors hardcoded, won't adapt if design system changes

**Root Cause Analysis**:
- XTerm.js requires explicit color values in theme config
- CSS custom properties can't be directly passed to JS
- No mechanism to sync JS theme with CSS variables

**Proposed Fix**:
1. Define terminal theme colors as JS constants derived from design system
2. Use `getComputedStyle` to read CSS variables and apply to XTerm theme
3. Or accept hardcoded values if XTerm.js theming is standalone

**Regression Test**: Visual check for theme consistency

---

## Summary

| Priority | Count | Description |
|----------|-------|-------------|
| P1 - Critical | 3 | Blocks core workflow, must fix |
| P2 - Major | 5 | Degraded experience, high priority |
| P3 - Minor | 4 | Polish issues, lower priority |
| **Total** | **12** | |

## Next Steps

1. Create tasks for each bug fix
2. Implement fixes in priority order (P1 first)
3. Add regression test for each fixed bug
4. Re-run bug audit after fixes to verify resolution
