# Frontend Code Conventions

> Making TypeScript/React code self-documenting for humans and LLMs alike.

This document establishes patterns that keep documentation colocated with code, inspired by Elixir's `@moduledoc`/`@doc` system where context lives alongside implementation.

---

## Core Principle

**Every file should be self-contained context.** An LLM (or new developer) reading a single file should understand:
- What this module does (purpose)
- How to use it (interface)
- Why it exists (context)
- What it depends on (relationships)

---

## Documentation Checklist (Mandatory)

> **AI Agents: Follow this checklist every time you create or modify a `.tsx` or `.ts` file.**
> Documentation is not a separate step — it is part of writing code.
> If you write a component or hook without docs, the code is **incomplete**.

### For every new file:
- [ ] Add `@module` header with `@description`, `@context`, and `@dependencies`
- [ ] Add `// ===` section separators (IMPORTS, TYPES, STATE, EFFECTS, HANDLERS, RENDER at minimum)
- [ ] Organize imports: React → external → internal components → hooks → utils → relative → types

### For every component you create:
- [ ] Document the Props interface with a `/** */` block above the interface
- [ ] Document each individual prop with a `/** description */` comment
- [ ] Add `@default` tag for optional props with defaults
- [ ] Document the component function with visual states and keyboard nav (if applicable)

### For every hook you create:
- [ ] Document `@param` for each parameter
- [ ] Document `@returns` describing the return value/object
- [ ] Include `@example` showing typical usage

### For every state/effect:
- [ ] Every `useState` gets a `/** purpose */` comment above it
- [ ] Every `useEffect` gets a `/** what triggers it and why */` comment above it
- [ ] Group related state under `// ===` section headers (e.g., EXECUTION STATE, UI STATE)

### Self-check before finishing:
- [ ] Every `useState` has a `/** */` JSDoc comment (not bare `//`)
- [ ] Every `useEffect` has a `/** */` JSDoc comment
- [ ] Section separators use exactly 60 `=` characters with UPPERCASE labels
- [ ] All colors use semantic theme classes (no hardcoded `text-red-500` etc.)
- [ ] No `TODO: add docs` placeholders left behind

---

## File Structure

Every `.tsx` or `.ts` file follows this structure:

```typescript
/**
 * @module ModuleName
 * @description
 * Brief description of what this module does and why it exists.
 * This should be 2-4 sentences explaining the "what" and "why".
 *
 * @context
 * Where this fits in the application architecture.
 * What triggers this component/hook to be used.
 *
 * @dependencies
 * - ParentComponent: renders this component when X happens
 * - useSpecStore: provides spec data
 * - WebSocket: receives real-time updates
 *
 * @example
 * // Most common usage pattern
 * <TaskCard task={task} onRetry={handleRetry} />
 */

// ============================================================
// IMPORTS
// ============================================================
// Group 1: React and external libraries
import { useState, useEffect, useCallback } from 'react';
import { motion } from 'framer-motion';

// Group 2: Internal UI components
import { Button } from '@/components/ui/button';
import { Card } from '@/components/ui/card';

// Group 3: Hooks and utilities
import { useSpec } from '@/hooks/useSpec';
import { cn } from '@/lib/utils';

// Group 4: Types (always last)
import type { Task, ExecutionStatus } from '@/types';

// ============================================================
// TYPES
// ============================================================
// All types for this module, fully documented

/**
 * Props for the TaskCard component.
 *
 * @example
 * const props: TaskCardProps = {
 *   task: { id: 'task_1', title: 'Implement auth', status: 'running' },
 *   onRetry: () => executor.retry('task_1'),
 *   expanded: true
 * };
 */
interface TaskCardProps {
  /** The task to display. Must have id, title, and status at minimum. */
  task: Task;

  /**
   * Called when user clicks retry on a failed task.
   * Only shown when task.status === 'failed'.
   */
  onRetry?: () => void;

  /**
   * Whether to show expanded view with full logs.
   * @default false
   */
  expanded?: boolean;
}

// ============================================================
// CONSTANTS
// ============================================================
// Magic numbers and strings get names and explanations

/** Animation duration for status transitions (matches design system) */
const STATUS_TRANSITION_MS = 200;

/** Maximum log lines to show in collapsed view */
const COLLAPSED_LOG_LINES = 5;

/**
 * IMPORTANT: Import theme colors from the centralized theme module.
 * Never hardcode Tailwind color classes like 'text-red-500'.
 * 
 * @see src/lib/theme.ts for all available color constants
 */
import { STATUS_COLORS, STATUS_BG, LOG_COLORS } from '@/lib/theme';

// ============================================================
// COMPONENT
// ============================================================

/**
 * Displays a single task's status and controls within the execution view.
 *
 * ## Visual States
 * - **pending**: Gray, shows queue position
 * - **running**: Blue pulse animation, elapsed time, live log tail
 * - **completed**: Green checkmark, summary, expandable details
 * - **failed**: Red X, error message, retry button
 *
 * ## Keyboard Navigation
 * - Enter/Space: Toggle expanded view
 * - R: Retry (when failed and focused)
 *
 * @see ExecutionRunner - Parent component that renders task list
 * @see useTaskExecution - Hook that manages task state
 */
export function TaskCard({ task, onRetry, expanded = false }: TaskCardProps) {
  // -- State --
  // Group related state together with comments explaining purpose
  
  /** Whether user has manually expanded this card */
  const [isExpanded, setIsExpanded] = useState(expanded);
  
  /** Tracks elapsed time for running tasks */
  const [elapsedSeconds, setElapsedSeconds] = useState(0);

  // -- Effects --
  // Each effect gets a comment explaining what triggers it and why
  
  /**
   * Timer for running tasks.
   * Starts when task begins running, clears on completion/failure.
   */
  useEffect(() => {
    if (task.status !== 'running') return;
    
    const interval = setInterval(() => {
      setElapsedSeconds(s => s + 1);
    }, 1000);
    
    return () => clearInterval(interval);
  }, [task.status]);

  // -- Handlers --
  // Named functions with JSDoc, not inline arrows
  
  /** Toggle expanded state on user interaction */
  const handleToggleExpand = useCallback(() => {
    setIsExpanded(prev => !prev);
  }, []);

  /**
   * Retry failed task.
   * Shows confirmation if task failed due to timeout.
   */
  const handleRetry = useCallback(() => {
    if (!onRetry) return;
    onRetry();
  }, [onRetry]);

  // -- Render helpers --
  // Extract complex render logic into named functions
  
  /** Renders the appropriate icon based on task status */
  const renderStatusIcon = () => {
    switch (task.status) {
      case 'pending': return <ClockIcon className="w-4 h-4" />;
      case 'running': return <SpinnerIcon className="w-4 h-4 animate-spin" />;
      case 'completed': return <CheckIcon className="w-4 h-4" />;
      case 'failed': return <XIcon className="w-4 h-4" />;
    }
  };

  // -- Main render --
  return (
    <Card className={cn('p-4', STATUS_COLORS[task.status])}>
      {/* ... */}
    </Card>
  );
}
```

---

## Documentation Requirements

### Module Header (Required)

Every file must start with a module-level JSDoc block:

```typescript
/**
 * @module ExecutionRunner
 * @description
 * Main orchestration UI that manages parallel task execution.
 * Handles WebSocket connections, batch management, and log streaming.
 *
 * @context
 * Rendered by App.tsx when user navigates to /execute.
 * Requires an active spec to be selected.
 *
 * @dependencies
 * - useSpec: Current spec and tasks
 * - useWebSocket: Real-time execution updates
 * - useLogStore: Centralized log management
 */
```

### Props Interface (Required for Components)

```typescript
/**
 * Props for ComponentName.
 * Brief description of the component's purpose.
 */
interface ComponentNameProps {
  /** Clear description. Include valid values if constrained. */
  requiredProp: string;

  /**
   * Longer description for complex props.
   * Explain when/why you'd use different values.
   * @default defaultValue
   */
  optionalProp?: number;
}
```

### Function Documentation (Required for Exports)

```typescript
/**
 * Brief description of what this function does.
 *
 * @param input - Description of parameter
 * @returns Description of return value
 *
 * @example
 * const result = myFunction({ key: 'value' });
 * // result: { processed: true }
 *
 * @throws {ValidationError} When input is invalid
 * @see relatedFunction - For similar functionality
 */
export function myFunction(input: Input): Output {
  // ...
}
```

### Inline Comments

Use inline comments to explain **why**, not **what**:

```typescript
// ❌ Bad: describes what code does (obvious from reading it)
// Loop through tasks
for (const task of tasks) {

// ✅ Good: explains why this approach was chosen
// Process in reverse order so dependent tasks see their dependencies' results
for (const task of tasks.reverse()) {

// ✅ Good: explains business logic
// Cap at 10 parallel tasks to avoid overwhelming the git worktree manager
const batchSize = Math.min(tasks.length, 10);
```

---

## Naming Conventions

### Files

| Type | Pattern | Example |
|------|---------|---------|
| Component | PascalCase.tsx | `TaskCard.tsx` |
| Hook | camelCase starting with 'use' | `useTaskExecution.ts` |
| Utility | camelCase | `formatDuration.ts` |
| Types | camelCase or PascalCase | `types.ts`, `TaskTypes.ts` |
| Constants | SCREAMING_SNAKE_CASE in file | `config.ts` |

### Variables and Functions

```typescript
// Components: PascalCase
function TaskCard() {}

// Hooks: camelCase starting with 'use'
function useTaskExecution() {}

// Event handlers: handle + Event + Element (optional)
const handleClick = () => {};
const handleSubmitForm = () => {};
const handleTaskRetry = () => {};

// Boolean variables: is/has/should/can prefix
const isLoading = true;
const hasError = false;
const shouldRetry = true;
const canEdit = false;

// Arrays: plural nouns
const tasks: Task[] = [];
const selectedIds: string[] = [];

// Maps/Records: noun + 'Map' or 'By' + Key
const taskMap: Map<string, Task>;
const tasksById: Record<string, Task>;
```

---

## State Management Patterns

### State Grouping

Group related state and document the relationship:

```typescript
// ============================================================
// EXECUTION STATE
// ============================================================
// These states are tightly coupled - changes to one often affect others

/** Current execution status */
const [status, setStatus] = useState<ExecutionStatus>('idle');

/** Error from last execution attempt, cleared on retry */
const [error, setError] = useState<Error | null>(null);

/** Tasks currently being executed */
const [runningTasks, setRunningTasks] = useState<string[]>([]);

// ============================================================
// UI STATE
// ============================================================
// Ephemeral UI state, not persisted

/** Which panel is currently expanded */
const [expandedPanel, setExpandedPanel] = useState<string | null>(null);

/** Search/filter text */
const [filterText, setFilterText] = useState('');
```

### Complex State

When a component has 5+ useState calls, consider extracting to a reducer or custom hook:

```typescript
// ❌ Too many individual states
const [tasks, setTasks] = useState([]);
const [status, setStatus] = useState('idle');
const [error, setError] = useState(null);
const [progress, setProgress] = useState(0);
const [logs, setLogs] = useState([]);
const [selectedTask, setSelectedTask] = useState(null);

// ✅ Extract to custom hook with documented interface
/**
 * Manages execution state for the runner.
 *
 * @returns Execution state and control functions
 *
 * @example
 * const { tasks, status, start, stop, retry } = useExecution(specId);
 */
const { tasks, status, error, progress, logs, selectedTask, actions } = useExecution(specId);
```

---

## Component Size Guidelines

| Lines | Status | Action |
|-------|--------|--------|
| < 200 | ✅ Good | Maintain |
| 200-400 | ⚠️ Monitor | Document well, consider splitting on next change |
| 400-600 | 🟠 Warning | Plan to split into subcomponents |
| > 600 | 🔴 Critical | Must split before adding features |

### Splitting Strategy

When splitting a large component:

1. **Extract hooks first** - Move state + effects into `useComponentName.ts`
2. **Extract subcomponents** - Identify render sections that can be isolated
3. **Keep related code together** - Don't split just to hit a line count

```typescript
// Before: ExecutionRunner.tsx (1400 lines)

// After:
// - useExecutionState.ts (state + effects, ~200 lines)
// - useExecutionWebSocket.ts (WS logic, ~150 lines)
// - ExecutionRunner.tsx (orchestration, ~200 lines)
// - ExecutionControls.tsx (buttons/status, ~150 lines)
// - ExecutionBatchPanel.tsx (batch UI, ~200 lines)
// - ExecutionLogPanel.tsx (log display, ~200 lines)
```

---

## Import Organization

```typescript
// 1. React (always first)
import { useState, useEffect } from 'react';

// 2. External libraries (alphabetical)
import { motion } from 'framer-motion';
import { z } from 'zod';

// 3. Internal absolute imports - components
import { Button } from '@/components/ui/button';
import { Card } from '@/components/ui/card';

// 4. Internal absolute imports - hooks
import { useSpec } from '@/hooks/useSpec';
import { useToast } from '@/hooks/use-toast';

// 5. Internal absolute imports - utilities
import { cn } from '@/lib/utils';
import { formatDuration } from '@/lib/format';

// 6. Relative imports (parent directories)
import { ParentContext } from '../context';

// 7. Relative imports (same directory)
import { helperFunction } from './helpers';

// 8. Type imports (always last, using 'import type')
import type { Task, Spec, ExecutionStatus } from '@/types';
```

---

## Error Handling

```typescript
// ============================================================
// ERROR BOUNDARIES
// ============================================================
// Wrap major sections in error boundaries with descriptive fallbacks

<ErrorBoundary
  fallback={<ErrorFallback section="task-list" onRetry={refetch} />}
  onError={(error) => logError('TaskList', error)}
>
  <TaskList tasks={tasks} />
</ErrorBoundary>

// ============================================================
// ASYNC ERROR HANDLING
// ============================================================
// Always handle errors explicitly, never let them disappear

try {
  const result = await executeTask(task);
  setResult(result);
} catch (error) {
  // Log with context for debugging
  console.error('[TaskExecution] Failed to execute task:', {
    taskId: task.id,
    error: error instanceof Error ? error.message : error,
  });
  
  // User-facing error state
  setError(error instanceof Error ? error : new Error('Unknown error'));
  
  // Toast for immediate feedback
  toast.error(`Failed to execute ${task.title}`);
}
```

---

## Testing Conventions

### Stack

- **Vitest** — Test runner (configured via `vitest.config.ts`)
- **React Testing Library** — Component rendering and queries
- **MSW (Mock Service Worker)** — API mocking at the network level
- **@testing-library/user-event** — Realistic user interaction simulation

### File Organization

Tests are **colocated** with source files:

```text
src/
  components/
    StatusWidget.tsx
    StatusWidget.test.tsx     ← colocated test
  hooks/
    useSpec.ts
    useSpec.test.tsx           ← colocated test
  test/
    setup.ts                   ← global setup (jest-dom, MSW lifecycle, browser mocks)
    test-utils.tsx             ← custom render with providers (QueryClientProvider)
    mocks/
      handlers.ts              ← base MSW handlers (happy-path defaults)
      server.ts                ← MSW server instance
      fixtures.ts              ← factory functions for test data
      websocket.ts             ← MockWebSocket class for WS hooks
```

### Import Pattern

Always import from `@/test/test-utils` instead of `@testing-library/react`:

```typescript
import { describe, it, expect, vi } from 'vitest';
import { render, screen, waitFor } from '@/test/test-utils';
import { userEvent } from '@testing-library/user-event';
import { http, HttpResponse } from 'msw';
import { server } from '@/test/mocks/server';
```

### Query Priority

Prefer accessible queries over test IDs:

```typescript
// ✅ Best - accessible queries
screen.getByRole('button', { name: /submit/i });
screen.getByText('Repository Status');
screen.getByLabelText('Email address');

// ✅ Good - for async data
await screen.findByText('Loading complete');

// ⚠️ Acceptable - when no accessible alternative exists
screen.getByTestId('code-tab-spec');

// ❌ Avoid - brittle
document.querySelector('.my-class');
```

### Async Data

Use `findBy*` queries (which internally use `waitFor`) for content that appears after API calls:

```typescript
// ✅ Correct
render(<StatusWidget />);
expect(await screen.findByText('Repository Status')).toBeInTheDocument();

// ❌ Wrong - may fail on timing
render(<StatusWidget />);
expect(screen.getByText('Repository Status')).toBeInTheDocument();
```

### User Interactions

Use `userEvent` (not `fireEvent`) for realistic event simulation:

```typescript
const user = userEvent.setup();
await user.click(screen.getByRole('button', { name: /delete/i }));
await user.type(screen.getByRole('textbox'), 'hello');
```

### MSW Patterns

**Base handlers** in `handlers.ts` provide happy-path defaults. Override per-test with `server.use()`:

```typescript
it('shows error when API fails', async () => {
  server.use(
    http.get('/api/status', () => {
      return HttpResponse.error();
    })
  );
  render(<StatusWidget />);
  await waitFor(() => {
    expect(screen.getByText('Connection Error')).toBeInTheDocument();
  });
});
```

### Hook Testing

Use `renderHook` with a QueryClientProvider wrapper:

```typescript
import { renderHook, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  return function Wrapper({ children }: { children: React.ReactNode }) {
    return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
  };
}

it('fetches specs', async () => {
  const { result } = renderHook(() => useSpecs(), { wrapper: createWrapper() });
  await waitFor(() => expect(result.current.isSuccess).toBe(true));
  expect(result.current.data?.specs).toHaveLength(2);
});
```

### Naming Conventions

```text
describe('ComponentName', () => {
  it('renders loading state initially', ...);
  it('displays data after loading', ...);
  it('shows error when API fails', ...);
  it('calls onSubmit when form is submitted', ...);
});
```

### Test File Header

Every test file follows the standard `@module` header format:

```typescript
/**
 * @module StatusWidget.test
 * @description
 * Unit tests for the StatusWidget component.
 *
 * @context
 * Tests status display, loading/error states, and git init flow.
 *
 * @dependencies
 * - @/test/test-utils: Custom render with QueryClientProvider
 * - msw: API mocking for /api/status
 */
```

### Coverage Thresholds

Coverage is enforced in `vitest.config.ts`. Current thresholds:

| Metric    | Threshold |
|-----------|-----------|
| Lines     | 20%       |
| Functions | 18%       |
| Branches  | 15%       |

Thresholds will increase as coverage grows.

### Running Tests

```bash
npm run test          # Watch mode
npm run test:run      # Single run (CI)
npm run test:coverage # With coverage report
```

---

## Theme System

The frontend uses **shadcn/ui CSS variables** for theming, enabling easy theme swapping with community themes from [tweakcn.com](https://tweakcn.com).

### Swapping Themes

To change the theme, run:

```bash
npx shadcn@latest add https://tweakcn.com/r/themes/<theme-name>.json
```

**Current theme:** `darkmatter`

**Available themes at tweakcn.com:**
- darkmatter (current)
- catppuccin
- rosepine
- dracula
- nord
- tokyo-night
- and many more...

### Color Palette

The theme provides these semantic colors:

| Variable | Usage | Dark Mode Example |
|----------|-------|-------------------|
| `--primary` | Main action, active states | Orange |
| `--secondary` | Secondary actions | Teal |
| `--muted` | Inactive backgrounds | Dark gray |
| `--muted-foreground` | Muted text | Gray |
| `--destructive` | Destructive actions | Teal (theme-specific) |
| `--success` | Success states | Green |
| `--warning` | Warning states | Amber |
| `--error` | Error states | Red |
| `--info` | Info/waiting states | Teal |

### Using Theme Colors (Tailwind Classes)

**ALWAYS use semantic Tailwind classes:**

```typescript
// ✅ GOOD - uses theme semantic colors
<span className="text-error">Error message</span>
<span className="text-success">Completed</span>
<span className="text-primary">Active</span>
<span className="text-muted-foreground">Muted text</span>
<div className="bg-card border-border">Card</div>

// ❌ BAD - hardcoded colors break theming
<span className="text-red-500">Error</span>
<span className="text-green-400">Success</span>
```

### Theme Helper Functions

For dynamic status-based styling, use the helpers in `@/lib/theme.ts`:

```typescript
import { getStatusClass, getLogClass, getAgentClasses } from '@/lib/theme';

// Status-based coloring
<span className={getStatusClass(task.status)}>{task.status}</span>

// Log type coloring
<span className={getLogClass('error')}>Error log</span>

// Agent provider styling
const agent = getAgentClasses('anthropic');
<span className={agent.text}>Claude</span>
```

### Available Theme Classes

| Category | Classes |
|----------|---------|
| **Text** | `text-primary`, `text-secondary`, `text-muted-foreground`, `text-foreground`, `text-success`, `text-warning`, `text-error`, `text-info` |
| **Background** | `bg-background`, `bg-card`, `bg-muted`, `bg-primary`, `bg-success`, `bg-warning`, `bg-error`, `bg-info` |
| **Border** | `border-border`, `border-primary`, `border-success`, `border-warning`, `border-error`, `border-info` |

### Custom Semantic Extensions

We extend the base shadcn theme with app-specific semantic colors:

| Variable | Purpose |
|----------|---------|
| `--success` | Green for completed states |
| `--warning` | Amber for warning/running states |
| `--error` | Red for failed/error states |
| `--info` | Teal for info/waiting states |

These are defined in `src/index.css` and work like any other theme color.

---

## Changelog

| Date | Change | Author |
|------|--------|--------|
| 2026-03-02 | Added comprehensive Testing Conventions section (Vitest + RTL + MSW) | Claude |
| 2026-02-03 | Migrated to shadcn/tweakcn theme system (darkmatter). Replaced all hardcoded colors. | Antigravity |
| 2026-02-03 | Added centralized theme.ts with OKLCH-based color tokens | Antigravity |
| 2026-02-03 | Initial version | Claude + Shikhar |

---

*This document should be updated when patterns evolve or new conventions are established.*

