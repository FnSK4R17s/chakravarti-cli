# Research: Unified Code Page

**Branch**: `015-unified-code-page`  
**Date**: 2026-01-24  
**Status**: Complete

## Research Summary

This feature is a UI refactoring that consolidates existing pages. No new technologies or significant research was required as all building blocks already exist in the codebase.

## Decisions

### 1. Tab Component Implementation

**Decision**: Use existing Radix UI Tabs component (`@radix-ui/react-tabs`)

**Rationale**:
- Already installed and used in: `SpecEditor.tsx` (view toggle: visual/outline/code)
- Provides accessibility out-of-the-box (keyboard navigation, ARIA)
- Styled consistently with existing shadcn theme
- No additional dependencies needed

**Alternatives Considered**:
- Custom tab implementation: Rejected - unnecessary when Radix already available
- React Router nested routes: Rejected - adds complexity for simple tab switching

### 2. Tab State Management

**Decision**: Use React `useState` within `CodePage` component, not global navigation context

**Rationale**:
- Tab state is local to the Code page, not needed globally
- Existing navigation context (`NavigationContext`) handles page-level routing
- Simple state management sufficient for 4 tabs
- State naturally resets when navigating away (acceptable per spec P2)

**Alternatives Considered**:
- URL query params (`?tab=tasks`): Could add for deep-linking, but not required for MVP
- Global context: Over-engineering for local tab state

### 3. Component Mounting Strategy

**Decision**: Keep all 4 tab components mounted, use CSS visibility to show/hide

**Rationale**:
- Prevents data loss when switching tabs (e.g., unsaved form state in SpecEditor)
- Faster tab switching (no remounting/re-fetching)
- TanStack Query caching already handles API calls efficiently

**Alternatives Considered**:
- Conditional rendering (unmount inactive tabs): Risk losing component state
- Lazy loading: Unnecessary, components are already loaded at Code page mount

### 4. Navigation Reduction

**Decision**: Reduce sidebar from 9 items to 5 items

**Current Navigation (9 items)**:
1. Dashboard
2. Agents
3. Specs ❌ (merged into Code)
4. Tasks ❌ (merged into Code)
5. Plan ❌ (merged into Code)
6. Runner ❌ (merged into Code)
7. Diff
8. Test
9. QA

**New Navigation (5 items)**:
1. Dashboard
2. Agents
3. **Code** ✅ (new unified page)
4. Test
5. QA

**Note**: Diff page is being removed from top-level navigation. It could be:
- Accessed from within the Code page (Run tab shows diffs)
- Added as a 6th item if users need direct access
- Moved to a command palette action

**Rationale**: 
- Diff is typically accessed after execution, so can be linked from Runner/ExecutionRunner
- Reduces cognitive load significantly (44% fewer nav items)

## No Clarifications Needed

All technical decisions were straightforward based on existing codebase patterns.

## References

- Existing Tabs usage: `SpecEditor.tsx:12, 214-229`
- Navigation context: `App.tsx:35-45`
- Dashboard layout: `layouts/Dashboard.tsx:78-143`
- Radix Tabs docs: https://www.radix-ui.com/primitives/docs/components/tabs
