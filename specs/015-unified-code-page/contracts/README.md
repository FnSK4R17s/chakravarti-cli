# Contracts: Unified Code Page

**Branch**: `015-unified-code-page`  
**Date**: 2026-01-24

## Summary

No new API contracts are required for this feature. The Unified Code Page is a frontend-only refactoring that reorganizes existing components into a tabbed interface.

## Existing API Contracts (Unchanged)

The following existing API endpoints continue to be used by their respective components:

### Used by SpecEditor (Spec tab)
- `GET /api/specs` - List specifications
- `GET /api/specs/detail?name={name}` - Get spec detail

### Used by TaskEditor (Tasks tab)
- `GET /api/tasks?spec={name}` - Get tasks for spec
- `POST /api/tasks/execute` - Execute task

### Used by PlanEditor (Plan tab)
- `GET /api/plans?spec={name}` - Get plan for spec
- `POST /api/plans/generate` - Generate plan

### Used by ExecutionRunner (Run tab)
- `GET /api/execution/status` - Get execution status
- `POST /api/execution/run` - Start execution
- `GET /api/execution/logs` - Stream execution logs

## No New Endpoints Required

The tab switching is purely client-side state management. No server communication is needed for:
- Switching between tabs
- Remembering active tab state
- Rendering the unified Code page

## Type Definitions

For completeness, here are the TypeScript types used for tab navigation (frontend only):

```typescript
// Tab type for Code page
export type CodeTabType = 'spec' | 'tasks' | 'plan' | 'run';

// Updated page type (removes specs, tasks, plan, runner; adds code)
export type PageType = 'dashboard' | 'agents' | 'code' | 'test' | 'qa';

// Tab metadata for rendering
export interface CodeTab {
  id: CodeTabType;
  label: string;
  icon: string; // Lucide icon name
}

export const CODE_TABS: CodeTab[] = [
  { id: 'spec', label: 'Spec', icon: 'FileText' },
  { id: 'tasks', label: 'Tasks', icon: 'ListTodo' },
  { id: 'plan', label: 'Plan', icon: 'Workflow' },
  { id: 'run', label: 'Run', icon: 'Rocket' },
];
```
