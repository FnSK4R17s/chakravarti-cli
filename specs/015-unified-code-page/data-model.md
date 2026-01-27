# Data Model: Unified Code Page

**Branch**: `015-unified-code-page`  
**Date**: 2026-01-24

## Overview

This feature introduces minimal new state - primarily tab navigation within the Code page. No new backend entities or API changes are required.

## Frontend State Model

### CodeTabType

Represents the active tab within the Code page.

```typescript
type CodeTabType = 'spec' | 'tasks' | 'plan' | 'run';
```

| Value | Component Rendered | Icon | Label |
|-------|-------------------|------|-------|
| `spec` | `<SpecEditor />` | `FileText` | Spec |
| `tasks` | `<TaskEditor />` | `ListTodo` | Tasks |
| `plan` | `<PlanEditor />` | `Workflow` | Plan |
| `run` | `<ExecutionRunner />` | `Rocket` | Run |

### Updated PageType

The existing navigation context page type is updated.

```typescript
// Before (9 pages)
type PageType = 'dashboard' | 'agents' | 'specs' | 'tasks' | 'plan' | 'runner' | 'diff' | 'test' | 'qa';

// After (5 pages + Code tabs internally)
type PageType = 'dashboard' | 'agents' | 'code' | 'test' | 'qa';
```

### CodePageState

Local state within the `CodePage` component.

```typescript
interface CodePageState {
  activeTab: CodeTabType;
}
```

**Default Value**: `{ activeTab: 'spec' }`

## State Transitions

### Tab Navigation

```
User clicks tab → setActiveTab(newTab) → Re-render with new active content
```

- No API calls on tab switch
- Existing component state preserved (kept mounted)
- TanStack Query cache handles data freshness

### Page Navigation

```
User clicks "Code" nav item → setCurrentPage('code') → Render CodePage
User clicks other nav item → setCurrentPage('dashboard'|'agents'|etc) → Render other page
```

- Tab state resets to default when leaving and returning to Code page
- This is acceptable per spec (P2 story addresses session persistence if needed later)

## Relationships

```
NavigationContext (global)
    └── currentPage: PageType
            └── 'code' → CodePage (local state)
                            └── activeTab: CodeTabType
                                    ├── 'spec' → SpecEditor
                                    ├── 'tasks' → TaskEditor
                                    ├── 'plan' → PlanEditor
                                    └── 'run' → ExecutionRunner
```

## Validation Rules

1. `activeTab` MUST be one of: `'spec'`, `'tasks'`, `'plan'`, `'run'`
2. Default tab MUST be `'spec'` (first stage in workflow)
3. All tabs MUST be keyboard accessible (handled by Radix Tabs)

## No Backend Changes

- No new API endpoints required
- No database schema changes
- All existing API calls continue to work within their respective components
