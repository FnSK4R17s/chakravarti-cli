# Automate Frontend Testing for ckrv-ui

**Issue**: [#74](https://github.com/FnSK4R17s/chakravarti-cli/issues/74)
**Created**: 2026-03-02
**Status**: Tasks Generated

## Problem Statement

The ckrv-ui frontend has **77 source files** (28 business components, 13 hooks, 4 type files, 3 services/utils, plus 22 shadcn/ui primitives) with **zero unit/component test coverage**. The only automated quality gates are ESLint, TypeScript typecheck, and Vite build. E2E tests exist (6 Playwright suites, ~1032 lines) but are **not integrated into CI** and require a running backend server.

This means:
- Regressions in component logic (state, effects, event handlers) are caught only by humans
- Hook behavior (WebSocket reconnection, execution streaming, workflow progress) is untested
- The CI gives a false sense of confidence - a passing build doesn't mean working features
- Refactoring is risky without a safety net

### Vision Alignment

From vision.md: *"When you return from a run: your feature is implemented, tested against existing tests, with new tests written for the generated code."* The UI is the primary interface for the orchestration workflow. If the UI itself lacks tests, the "fire and forget" promise breaks down - users come back to a broken dashboard instead of a working review screen.

## Current State

### What exists

| Layer | Tool | Status | Files |
|-------|------|--------|-------|
| **Lint** | ESLint 9 | In CI | `npm run lint` |
| **Types** | TypeScript 5.9 strict | In CI | `npx tsc -b` |
| **Build** | Vite 7 | In CI | `npm run build` |
| **E2E** | Playwright 1.40 | NOT in CI | 6 suites in `tests/e2e/` |
| **Unit** | None | Missing | - |
| **Component** | None | Missing | - |
| **Coverage** | None | Missing | - |

### E2E test suites (existing, not in CI)

| Suite | Lines | Coverage Area |
|-------|-------|---------------|
| `code-page.spec.ts` | 164 | Tab navigation, sidebar, project selector |
| `execution-runner.spec.ts` | 205 | Run workflow, progress, agent selection |
| `accessibility.spec.ts` | 199 | ARIA roles, keyboard nav, axe-core |
| `error-handling.spec.ts` | 169 | Error boundaries, network failures |
| `visual-consistency.spec.ts` | 156 | Theme colors, layout consistency |
| `responsive.spec.ts` | 139 | Mobile/tablet viewports |

These use a well-designed isolation fixture (`tests/helpers/test-project.ts`) that creates temp git repos per test. The Playwright config has a commented-out `webServer` block ready to auto-start the Rust backend.

### CI pipeline (`ci.yml`)

```
rust-fmt ─┐
rust-clippy ─┤  (all run in parallel)
rust-test ───┤
rust-build ──┤
frontend-lint ─┤
frontend-typecheck ─┤
frontend-build ─┘
```

No frontend test job. No E2E job. No coverage reporting.

### Component inventory by complexity

**Large (600+ lines) - highest test priority:**
- `AgentManager.tsx` (1447 lines) - agent CRUD, model selection, defaults
- `PlanEditor.tsx` (1028) - plan display/editing
- `TestRunner.tsx` (983) - test execution UI
- `TaskEditor.tsx` (968) - task display/editing
- `SpecEditor.tsx` (815) - spec creation/editing
- `TaskDetailModal.tsx` (692) - task detail overlay
- `QAReviewer.tsx` (689) - quality review UI

**Medium (300-600 lines):**
- `WorkflowPanel.tsx` (478), `AgentCliModal.tsx` (466), `LogViewer.tsx` (427)
- `BarebonesExecutor.tsx` (417), `DiffViewer.tsx` (414), `TestFixModal.tsx` (389)
- `CompletionSummary.tsx` (386), `CommandPalette.tsx` (385)
- `SpecWorkflow.tsx` (350), `ChatDashboard.tsx` (342), `RunHistoryPanel.tsx` (315)

**Hooks by complexity:**
- `useSpec.ts` (391) - spec CRUD, auto-selection, mutations
- `useLogStore.ts` (317) - log aggregation, search, filtering
- `useExecutionStream.ts` (298) - WebSocket event processing
- `useWorkflowProgress.ts` - state machine for multi-step workflow
- `useWebSocketReconnect.ts` - exponential backoff, reconnection
- `useCommand.ts` - CLI command execution via API
- `useConnection.ts` - backend health checks

### API layer

`src/lib/api.ts` patches global `fetch` to route through Tauri IPC when in desktop mode, otherwise hits HTTP endpoints. This dual-path architecture needs careful mocking:
- **Browser mode**: standard fetch to `localhost:3000/api/*`
- **Tauri mode**: `invoke()` to Rust commands

## Proposed Solution

### Layer 1: Unit Tests with Vitest

**Why Vitest over Jest:** Vite-native (shares config/plugins), ESM-first, faster startup, compatible API. The project already uses Vite 7 so Vitest is zero-friction.

**Setup:**
```
crates/ckrv-ui/frontend/
├── vitest.config.ts          # Test config (jsdom, path aliases, coverage)
├── src/
│   ├── test/
│   │   ├── setup.ts          # Global test setup (DOM mocks, providers)
│   │   ├── test-utils.tsx    # Custom render with QueryClient, etc.
│   │   ├── mocks/
│   │   │   ├── handlers.ts   # MSW request handlers
│   │   │   ├── server.ts     # MSW server setup
│   │   │   └── fixtures.ts   # Typed mock data factories
│   │   └── matchers.ts       # Custom vitest matchers (optional)
│   ├── components/
│   │   ├── AgentManager.tsx
│   │   ├── AgentManager.test.tsx   # <-- colocated
│   │   └── ...
│   └── hooks/
│       ├── useSpec.ts
│       ├── useSpec.test.ts         # <-- colocated
│       └── ...
```

**Key decisions:**
- **Colocated tests** (`*.test.tsx` next to source) - easier to find, moves with refactors
- **jsdom environment** - lightweight DOM simulation for component rendering
- **MSW for API mocking** - intercepts fetch/XHR at network level, works with both browser and Tauri paths
- **React Testing Library** - tests behavior not implementation, encourages accessible markup

### Layer 2: E2E in CI

Two strategies (not mutually exclusive):

**Option A: Mocked E2E (recommended for CI)**
- Run Playwright against Vite dev server
- MSW intercepts all `/api/*` calls with fixture data
- No Rust build required, fast (~2-3 min)
- Tests UI behavior in isolation

**Option B: Full-stack E2E (nightly/on-demand)**
- Build the Rust binary, spin up `ckrv ui`
- Playwright tests against real backend
- Slow (~10-15 min with Rust build), but catches integration bugs
- Use the commented-out `webServer` block in `playwright.config.ts`

### Layer 3: CI Integration

```
existing jobs ──┐
                │
frontend-unit ──┤  NEW: vitest --run
frontend-e2e ───┘  NEW: playwright (mocked backend via MSW)
```

## User Stories

### US1: Developer adds a new component
**As a** contributor,
**I want** `npm run test` to catch regressions in component behavior,
**So that** I know my changes don't break existing functionality before pushing.

### US2: CI catches regressions
**As a** maintainer,
**I want** the CI pipeline to fail when frontend tests break,
**So that** PRs with regressions don't get merged.

### US3: E2E validates user flows
**As a** maintainer,
**I want** E2E tests to run in CI against a mocked backend,
**So that** critical user workflows (spec creation, task execution, agent config) are validated on every PR.

### US4: Coverage visibility
**As a** contributor,
**I want** to see test coverage in CI output,
**So that** I know which areas need more tests.

## Technical Approach

### Options Considered

| Option | Pros | Cons |
|--------|------|------|
| **Vitest + RTL + MSW** | Vite-native, fast, ESM, shares config | New deps (~4-5 packages) |
| **Jest + RTL + MSW** | Industry standard, huge ecosystem | Needs ESM transform config, slower, fights Vite |
| **Vitest only (no RTL)** | Fewer deps | Harder to test React components idiomatically |
| **Playwright component testing** | Reuses existing Playwright | Heavier, slower for unit-level tests |

### Decision

**Vitest + React Testing Library + MSW.** Rationale:
- Vitest shares Vite's plugin pipeline and config (path aliases, React plugin)
- RTL's "test behavior, not implementation" philosophy matches our accessible-first conventions
- MSW provides network-level mocking that works regardless of Tauri vs browser mode
- This is the most widely adopted stack for Vite+React projects in 2026

### Dependencies to install

```bash
# Test runner + DOM environment
npm install -D vitest @vitest/coverage-v8 jsdom

# React component testing
npm install -D @testing-library/react @testing-library/jest-dom @testing-library/user-event

# API mocking
npm install -D msw
```

### vitest.config.ts

```typescript
import { defineConfig, mergeConfig } from 'vitest/config';
import viteConfig from './vite.config';

export default mergeConfig(viteConfig, defineConfig({
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: ['./src/test/setup.ts'],
    include: ['src/**/*.test.{ts,tsx}'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'lcov', 'html'],
      include: ['src/components/**', 'src/hooks/**', 'src/lib/**', 'src/services/**'],
      exclude: ['src/components/ui/**', 'src/types/**', 'src/test/**'],
      thresholds: {
        // Start low, ratchet up over time
        lines: 30,
        functions: 30,
        branches: 20,
      },
    },
  },
}));
```

### Test setup file (src/test/setup.ts)

```typescript
import '@testing-library/jest-dom/vitest';
import { cleanup } from '@testing-library/react';
import { afterEach, beforeAll, afterAll } from 'vitest';
import { server } from './mocks/server';

// MSW server lifecycle
beforeAll(() => server.listen({ onUnhandledRequest: 'warn' }));
afterEach(() => {
  cleanup();
  server.resetHandlers();
});
afterAll(() => server.close());
```

### Custom render wrapper (src/test/test-utils.tsx)

```typescript
import { render, RenderOptions } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { ReactElement } from 'react';

function createTestQueryClient() {
  return new QueryClient({
    defaultOptions: {
      queries: { retry: false, gcTime: 0 },
      mutations: { retry: false },
    },
  });
}

function AllProviders({ children }: { children: React.ReactNode }) {
  const queryClient = createTestQueryClient();
  return (
    <QueryClientProvider client={queryClient}>
      {children}
    </QueryClientProvider>
  );
}

export function renderWithProviders(
  ui: ReactElement,
  options?: Omit<RenderOptions, 'wrapper'>
) {
  return render(ui, { wrapper: AllProviders, ...options });
}

export * from '@testing-library/react';
export { renderWithProviders as render };
```

### MSW handlers (src/test/mocks/handlers.ts)

```typescript
import { http, HttpResponse } from 'msw';

// Typed fixture factories matching api.generated.ts types
export const handlers = [
  http.get('/api/status', () =>
    HttpResponse.json({ project_root: '/tmp/test', initialized: true })
  ),
  http.get('/api/specs', () =>
    HttpResponse.json([{ name: 'test-feature', status: 'ready' }])
  ),
  http.get('/api/agents', () =>
    HttpResponse.json([{ name: 'claude', provider: 'claude-code', is_default: true }])
  ),
  // ... more handlers matching API endpoints in src/lib/api.ts
];
```

### Example component test

```typescript
// src/components/AgentManager.test.tsx
import { render, screen, within } from '@/test/test-utils';
import userEvent from '@testing-library/user-event';
import { AgentManager } from './AgentManager';

describe('AgentManager', () => {
  it('renders the agent list', async () => {
    render(<AgentManager />);
    expect(await screen.findByText('claude')).toBeInTheDocument();
  });

  it('opens the add agent modal', async () => {
    const user = userEvent.setup();
    render(<AgentManager />);
    await user.click(screen.getByRole('button', { name: /add agent/i }));
    expect(screen.getByRole('dialog')).toBeInTheDocument();
  });
});
```

### Example hook test

```typescript
// src/hooks/useWebSocketReconnect.test.ts
import { renderHook, act } from '@testing-library/react';
import { useWebSocketReconnect } from './useWebSocketReconnect';

describe('useWebSocketReconnect', () => {
  it('starts in disconnected state', () => {
    const { result } = renderHook(() => useWebSocketReconnect('ws://localhost:3000'));
    expect(result.current.status).toBe('disconnected');
  });

  it('increments retry count on failure', async () => {
    // Test exponential backoff logic
  });
});
```

### CI workflow additions

```yaml
# New job: Frontend Unit Tests
frontend-test:
  name: Frontend Unit Tests
  runs-on: ubuntu-latest
  timeout-minutes: 10
  steps:
    - uses: actions/checkout@v4
    - uses: actions/setup-node@v4
      with:
        node-version: "20"
    - name: Install dependencies
      run: npm install
      working-directory: crates/ckrv-ui/frontend
    - name: Run unit tests
      run: npx vitest --run --coverage
      working-directory: crates/ckrv-ui/frontend
    - name: Upload coverage
      uses: actions/upload-artifact@v4
      if: always()
      with:
        name: coverage-report
        path: crates/ckrv-ui/frontend/coverage/

# New job: Frontend E2E Tests (mocked)
frontend-e2e:
  name: Frontend E2E
  runs-on: ubuntu-latest
  timeout-minutes: 15
  steps:
    - uses: actions/checkout@v4
    - uses: actions/setup-node@v4
      with:
        node-version: "20"
    - name: Install dependencies
      run: npm install
      working-directory: crates/ckrv-ui/frontend
    - name: Install Playwright browsers
      run: npx playwright install --with-deps chromium
      working-directory: crates/ckrv-ui/frontend
    - name: Run E2E tests
      run: npm run test:e2e -- --project=chromium
      working-directory: crates/ckrv-ui/frontend
    - name: Upload test report
      uses: actions/upload-artifact@v4
      if: always()
      with:
        name: playwright-report
        path: crates/ckrv-ui/frontend/playwright-report/
```

### Package.json script additions

```json
{
  "scripts": {
    "test": "vitest",
    "test:run": "vitest --run",
    "test:coverage": "vitest --run --coverage",
    "test:e2e": "playwright test",
    "test:e2e:ui": "playwright test --ui",
    "test:e2e:headed": "playwright test --headed"
  }
}
```

## Implementation Notes

### Tauri mocking

The `api.ts` module checks `window.__TAURI_INTERNALS__` to decide fetch routing. In tests, this will be `undefined` (jsdom), so all requests go through standard fetch -> MSW intercepts. No special handling needed.

### WebSocket mocking

For hooks like `useExecutionStream` and `useWebSocketReconnect`, use a mock WebSocket class:

```typescript
// src/test/mocks/websocket.ts
import { vi } from 'vitest';

export class MockWebSocket {
  onopen: (() => void) | null = null;
  onmessage: ((event: MessageEvent) => void) | null = null;
  onclose: (() => void) | null = null;
  onerror: ((event: Event) => void) | null = null;
  readyState = WebSocket.CONNECTING;

  send = vi.fn();
  close = vi.fn();

  // Helpers for test control
  simulateOpen() { this.readyState = WebSocket.OPEN; this.onopen?.(); }
  simulateMessage(data: unknown) { this.onmessage?.(new MessageEvent('message', { data: JSON.stringify(data) })); }
  simulateClose() { this.readyState = WebSocket.CLOSED; this.onclose?.(); }
}
```

### xterm.js mocking

Components like `LogTerminal` and `BatchLogTerminal` use xterm.js which requires a real DOM. Mock the Terminal class:

```typescript
vi.mock('@xterm/xterm', () => ({
  Terminal: vi.fn().mockImplementation(() => ({
    open: vi.fn(),
    write: vi.fn(),
    dispose: vi.fn(),
    onData: vi.fn(),
  })),
}));
```

### shadcn/ui components

Don't test shadcn/ui primitives (`src/components/ui/`) - they're third-party. Only test custom wrappers like `LoadingButton.tsx` and `LoadingOverlay.tsx`.

### Test data factories

Use typed factories that match `api.generated.ts`:

```typescript
// src/test/mocks/fixtures.ts
import type { AgentConfig, SpecSummary } from '@/types/api.generated';

export function createAgent(overrides?: Partial<AgentConfig>): AgentConfig {
  return {
    name: 'test-agent',
    provider: 'claude-code',
    is_default: false,
    ...overrides,
  };
}

export function createSpec(overrides?: Partial<SpecSummary>): SpecSummary {
  return {
    name: 'test-spec',
    status: 'ready',
    ...overrides,
  };
}
```

## Open Questions

- [x] **Vitest vs Jest?** -> Vitest (Vite-native, faster, ESM-first)
- [x] **Colocated vs `__tests__` dirs?** -> Colocated (`*.test.tsx` next to source)
- [x] **E2E CI strategy?** -> Mocked (MSW) for PRs, full-stack optional for nightly
- [ ] **Coverage thresholds?** Start at 30% lines/functions, ratchet up quarterly
- [ ] **Snapshot testing?** Skip for now - fragile with frequent UI changes. Reconsider once components stabilize.
- [ ] **Visual regression in CI?** Playwright already has `visual-consistency.spec.ts` with screenshot comparison. Enable in CI once baseline images are committed.
- [ ] **E2E backend strategy for full-stack?** The Playwright config has a commented-out `webServer` block. Need to decide: build `ckrv` in CI (expensive) or provide a lightweight mock server binary.

## Success Criteria

| Metric | Target |
|--------|--------|
| Unit test framework | Vitest installed and configured |
| `npm run test` works locally | Pass |
| CI runs unit tests on every PR | `frontend-test` job in `ci.yml` |
| CI runs E2E on every PR | `frontend-e2e` job (Chromium only) |
| Business components with tests | Top 5 by complexity (AgentManager, PlanEditor, TestRunner, TaskEditor, SpecEditor) |
| Hooks with tests | Top 3 (useSpec, useExecutionStream, useWebSocketReconnect) |
| Coverage visible in CI | Upload artifact + text summary |
| Coverage threshold | 30% lines minimum (ratchet up over time) |

## Phased Rollout

### Phase 1: Infrastructure (1 task)
- Install deps, create `vitest.config.ts`, setup file, test utils, MSW handlers
- Add `npm run test` / `npm run test:coverage` scripts
- Write 1 smoke test to validate the setup works

### Phase 2: CI Integration (1 task)
- Add `frontend-test` job to `ci.yml`
- Add `frontend-e2e` job to `ci.yml` (Chromium only, mocked)
- Verify both pass on a PR

### Phase 3: Critical Component Tests (5 tasks)
- `AgentManager.test.tsx` - renders, CRUD operations, default agent selection
- `SpecEditor.test.tsx` - renders, creates spec, validates input
- `TaskEditor.test.tsx` - renders, task list display, status filtering
- `PlanEditor.test.tsx` - renders, plan display, step navigation
- `CodePage.test.tsx` - tab switching, workflow state transitions

### Phase 4: Hook Tests (3 tasks)
- `useSpec.test.ts` - CRUD mutations, auto-selection logic, error handling
- `useExecutionStream.test.ts` - WebSocket messages, state transitions, cleanup
- `useWebSocketReconnect.test.ts` - backoff timing, max retries, reconnect lifecycle

### Phase 5: Coverage Ratchet & Conventions (1 task)
- Set initial thresholds in `vitest.config.ts`
- Add test conventions section to `FRONTEND_CONVENTIONS.md`
- Document MSW handler patterns for future contributors

## References

- [Vitest docs](https://vitest.dev/)
- [React Testing Library docs](https://testing-library.com/docs/react-testing-library/intro/)
- [MSW docs](https://mswjs.io/docs/)
- Existing Playwright config: `crates/ckrv-ui/frontend/playwright.config.ts`
- Existing test helpers: `crates/ckrv-ui/frontend/tests/helpers/test-project.ts`
- Frontend conventions: `crates/ckrv-ui/FRONTEND_CONVENTIONS.md`
- CI workflow: `.github/workflows/ci.yml`
