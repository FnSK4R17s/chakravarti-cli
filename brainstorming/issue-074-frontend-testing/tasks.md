# Automate Frontend Testing for ckrv-ui - Tasks

**Issue**: [#74](https://github.com/FnSK4R17s/chakravarti-cli/issues/74)
**Brainstorm**: [notes.md](./notes.md)
**Created**: 2026-03-02

## Task Overview

| Phase | Tasks | Estimate |
|-------|-------|----------|
| Phase 1: Test Infrastructure | 4 | 3h |
| Phase 2: CI Integration | 3 | 2h |
| Phase 3: Component Tests | 5 | 6h |
| Phase 4: Hook Tests | 3 | 4h |
| Phase 5: Conventions & Coverage | 2 | 1.5h |
| **Total** | **17** | **16.5h** |

## Dependencies

```
Phase 1 ─────────────────────────────────────────────────────────────►
  1.1 ──► 1.2 ──► 1.3 ──► 1.4
                     │
Phase 2 ─────────────┼───────────────────────────────────────────────►
                     └──► 2.1 ──► 2.2 ──► 2.3
                     │
Phase 3 ─────────────┼───────────────────────────────────────────────►
                     └──► 3.1 ──┬─► 3.2
                                ├─► 3.3  (3.1-3.5 run in parallel)
                                ├─► 3.4
                                └─► 3.5
Phase 4 ─────────────┼───────────────────────────────────────────────►
                     └──► 4.1 ──┬─► 4.2  (4.1-4.3 run in parallel)
                                └─► 4.3
Phase 5 (after 3 + 4) ──────────────────────────────────────────────►
                         5.1 ──► 5.2
```

**Key dependency**: Phase 1 (infrastructure) blocks everything else. Phases 2, 3, 4 can run in parallel after Phase 1 is complete. Phase 5 runs after 3 and 4 to set accurate coverage thresholds.

---

## Phase 1: Test Infrastructure

### Task 1.1: Install test dependencies
**Priority**: P0
**Estimate**: 30m
**Files**: `crates/ckrv-ui/frontend/package.json`

Install Vitest, React Testing Library, and MSW as dev dependencies.

```bash
cd crates/ckrv-ui/frontend
npm install -D vitest @vitest/coverage-v8 jsdom
npm install -D @testing-library/react @testing-library/jest-dom @testing-library/user-event
npm install -D msw
```

Add scripts to `package.json`:
```json
"test": "vitest",
"test:run": "vitest --run",
"test:coverage": "vitest --run --coverage"
```

**Acceptance Criteria**:
- [ ] `npm run test` launches Vitest in watch mode
- [ ] `npm run test:run` runs once and exits
- [ ] `npm run test:coverage` produces a coverage report
- [ ] No changes to existing scripts (`dev`, `build`, `lint`, `test:e2e`)
- [ ] `npm run build` still succeeds (no side effects from new deps)

---

### Task 1.2: Create Vitest configuration
**Priority**: P0
**Estimate**: 30m
**Files**: `crates/ckrv-ui/frontend/vitest.config.ts`

Create `vitest.config.ts` that extends the existing Vite config. Must configure:
- `jsdom` environment for DOM simulation
- `@/` path alias (mirrors `vite.config.ts`)
- Coverage provider (`v8`) with reporters and thresholds
- Setup file reference
- Include/exclude patterns (skip `tests/e2e/`, `src/components/ui/`, `src/types/`)

Use `mergeConfig` from `vitest/config` to inherit the React plugin and path aliases from `vite.config.ts`.

**Acceptance Criteria**:
- [ ] `vitest.config.ts` extends `vite.config.ts` via `mergeConfig`
- [ ] `@/` import alias works in test files
- [ ] Coverage excludes shadcn/ui primitives (`src/components/ui/**`) and types (`src/types/**`)
- [ ] Coverage includes business components, hooks, lib, and services
- [ ] `npx vitest --run` exits cleanly (no test files yet = pass)

---

### Task 1.3: Create test setup and utilities
**Priority**: P0
**Estimate**: 1h
**Files**:
- `crates/ckrv-ui/frontend/src/test/setup.ts`
- `crates/ckrv-ui/frontend/src/test/test-utils.tsx`
- `crates/ckrv-ui/frontend/src/test/mocks/handlers.ts`
- `crates/ckrv-ui/frontend/src/test/mocks/server.ts`
- `crates/ckrv-ui/frontend/src/test/mocks/fixtures.ts`
- `crates/ckrv-ui/frontend/src/test/mocks/websocket.ts`

**setup.ts** - Global test lifecycle:
- Import `@testing-library/jest-dom/vitest` for DOM matchers
- Start/stop/reset MSW server in `beforeAll`/`afterEach`/`afterAll`
- Auto-cleanup RTL after each test
- Mock `window.matchMedia` (required by some Radix components)
- Mock `ResizeObserver` (used by xterm.js, scroll areas)

**test-utils.tsx** - Custom render:
- Wrap all renders in `QueryClientProvider` (TanStack Query)
- Use a fresh `QueryClient` per test (`retry: false`, `gcTime: 0`)
- Re-export everything from `@testing-library/react` for single import
- Export `renderWithProviders` as `render`

**mocks/handlers.ts** - MSW request handlers:
- Cover the main API endpoints from `src/lib/api.ts`:
  - `GET /api/status` -> `{ project_root, initialized }`
  - `GET /api/agents` -> array of `AgentConfig`
  - `GET /api/specs` -> array of spec summaries
  - `GET /api/docker` -> `DockerStatus`
  - `POST /api/specs/new` -> success response
  - `POST /api/command` -> command result
- Use typed factories from `fixtures.ts` for response data

**mocks/server.ts** - MSW server instance:
- `setupServer(...handlers)` for Node environment

**mocks/fixtures.ts** - Typed factory functions:
- `createAgent(overrides?)` -> `AgentConfig`
- `createSpec(overrides?)` -> spec summary
- `createDockerStatus(overrides?)` -> `DockerStatus`
- All types imported from `@/types/api.generated`

**mocks/websocket.ts** - Mock WebSocket class:
- `send`, `close` as `vi.fn()`
- `simulateOpen()`, `simulateMessage(data)`, `simulateClose()`, `simulateError()` helpers
- Tracks `readyState` transitions

**Acceptance Criteria**:
- [ ] `setup.ts` configures jest-dom matchers globally
- [ ] MSW server starts/stops cleanly in test lifecycle
- [ ] `render()` from `test-utils.tsx` wraps in `QueryClientProvider`
- [ ] Factory functions produce valid typed objects
- [ ] `MockWebSocket` can simulate open/message/close events
- [ ] `window.matchMedia` and `ResizeObserver` mocks prevent runtime errors

---

### Task 1.4: Write smoke test to validate infrastructure
**Priority**: P0
**Estimate**: 30m
**Files**: `crates/ckrv-ui/frontend/src/components/StatusWidget.test.tsx`

Write a minimal test for `StatusWidget` (one of the simpler components) to prove the entire setup works end-to-end:
- Imports from `@/test/test-utils`
- MSW intercepts `/api/status`
- Component renders and shows connection status
- Uses `findByText` (async) since data comes from TanStack Query

This is a "canary" test. If it passes, the infrastructure works.

**Acceptance Criteria**:
- [ ] `npm run test:run` finds and runs the test
- [ ] Test passes: StatusWidget renders with mocked API data
- [ ] Coverage report includes `StatusWidget.tsx`
- [ ] No console warnings about unhandled requests (MSW catches them)

---

## Phase 2: CI Integration

### Task 2.1: Add frontend-test job to ci.yml
**Priority**: P1
**Estimate**: 30m
**Files**: `.github/workflows/ci.yml`

Add a new `frontend-test` job alongside the existing frontend jobs:

```yaml
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
    - name: Run unit tests with coverage
      run: npx vitest --run --coverage
      working-directory: crates/ckrv-ui/frontend
    - name: Upload coverage report
      uses: actions/upload-artifact@v4
      if: always()
      with:
        name: coverage-report
        path: crates/ckrv-ui/frontend/coverage/
```

**Acceptance Criteria**:
- [ ] `frontend-test` job appears in CI pipeline
- [ ] Job runs in parallel with other frontend jobs (no `needs:` dependency)
- [ ] Coverage report uploaded as artifact
- [ ] Job fails if any unit test fails
- [ ] Job fails if coverage drops below configured thresholds

---

### Task 2.2: Add frontend-e2e job to ci.yml
**Priority**: P1
**Estimate**: 45m
**Files**: `.github/workflows/ci.yml`, `crates/ckrv-ui/frontend/playwright.config.ts`

Add a `frontend-e2e` job that runs existing Playwright tests against a mocked backend. This requires:

1. **Update `playwright.config.ts`** to support a "mocked" mode:
   - Uncomment or add a `webServer` block that runs `npx vite dev` (not the Rust backend)
   - The Vite dev server serves the frontend; MSW in the browser intercepts API calls
   - Or: create a `tests/e2e/global-setup.ts` that starts Vite programmatically

2. **CI job**: Install Playwright browsers (Chromium only for speed), run tests.

```yaml
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
    - name: Install Playwright Chromium
      run: npx playwright install --with-deps chromium
      working-directory: crates/ckrv-ui/frontend
    - name: Run E2E tests (Chromium)
      run: npx playwright test --project=chromium
      working-directory: crates/ckrv-ui/frontend
    - name: Upload test report
      uses: actions/upload-artifact@v4
      if: always()
      with:
        name: playwright-report
        path: crates/ckrv-ui/frontend/playwright-report/
```

**Note**: The existing E2E tests expect a running backend (`baseURL: localhost:3000`). Two options:
- **Option A**: Add MSW browser integration to E2E tests (service worker intercept). More work but no backend needed.
- **Option B**: Build `ckrv` binary in CI and use the `webServer` block. Slower but tests real integration.

Recommend **Option B** initially since E2E tests already use `data-testid` selectors against real UI. Gate this job behind `needs: [rust-build]` to reuse the build artifact.

**Acceptance Criteria**:
- [ ] `frontend-e2e` job appears in CI pipeline
- [ ] Playwright runs against a backend (real or mocked)
- [ ] Only Chromium browser tested in CI (speed)
- [ ] HTML report uploaded as artifact on failure
- [ ] Retries configured (`retries: 2` in Playwright config when `CI=true`)

---

### Task 2.3: Verify CI pipeline end-to-end
**Priority**: P1
**Estimate**: 30m
**Files**: None (validation task)

Open a test PR that includes the new jobs and at least one passing unit test (from Task 1.4). Verify:
- All existing jobs still pass
- New `frontend-test` job passes and shows coverage
- New `frontend-e2e` job passes (or is documented as expected-to-skip if backend isn't available)
- Artifacts (coverage, Playwright report) are downloadable

**Acceptance Criteria**:
- [ ] PR CI shows all green (or expected skips documented)
- [ ] Coverage artifact downloadable from `frontend-test` job
- [ ] Playwright report downloadable from `frontend-e2e` job
- [ ] No increase in total CI time beyond ~3 min for new jobs

---

## Phase 3: Component Tests

> All component tests use the `render` from `@/test/test-utils` and `userEvent` from `@testing-library/user-event`. Tests should focus on **behavior** (what the user sees and does) not **implementation** (internal state, refs).

### Task 3.1: AgentManager tests
**Priority**: P1
**Estimate**: 1.5h
**Files**: `crates/ckrv-ui/frontend/src/components/AgentManager.test.tsx`

`AgentManager.tsx` (1447 lines) is the largest component. Key behaviors to test:

**Render & data loading:**
- Renders agent list from `/api/agents` (MSW)
- Shows loading state while fetching
- Shows empty state when no agents configured

**CRUD operations:**
- "Add Agent" button opens the `AgentCliModal` dialog
- Deleting an agent calls `POST /api/agents/delete` and removes from list
- Setting default agent calls `POST /api/agents/set-default`

**Agent type selection:**
- Provider dropdown shows all agent types
- Selecting a provider filters available models
- OpenRouter agents show model picker

**Error handling:**
- API failure shows error toast/message
- Network timeout shows retry option

MSW handlers needed: `GET /api/agents`, `POST /api/agents/upsert`, `POST /api/agents/delete`, `POST /api/agents/set-default`, `GET /api/agents/models`

**Acceptance Criteria**:
- [ ] Renders agent list with mocked data
- [ ] Tests add, delete, and set-default flows
- [ ] Tests loading and error states
- [ ] No direct testing of internal state (`useState` values)
- [ ] All assertions use accessible queries (`getByRole`, `getByText`, `findByText`)

---

### Task 3.2: SpecEditor tests
**Priority**: P1
**Estimate**: 1h
**Files**: `crates/ckrv-ui/frontend/src/components/SpecEditor.test.tsx`

`SpecEditor.tsx` (815 lines) handles spec creation and editing.

**Key behaviors:**
- Renders spec content when a spec is selected (via `useSpec` hook / MSW)
- "New Spec" button opens `NewSpecDialog`
- Editing spec content and saving calls `POST /api/specs/save`
- Spec list shows all available specs from API
- Selecting a spec loads its content

MSW handlers needed: `GET /api/specs`, `GET /api/specs/:name`, `POST /api/specs/new`, `POST /api/specs/save`

**Acceptance Criteria**:
- [ ] Renders spec list from mocked API
- [ ] Tests spec selection and content display
- [ ] Tests new spec creation flow
- [ ] Tests save operation
- [ ] Tests empty state (no specs)

---

### Task 3.3: TaskEditor tests
**Priority**: P1
**Estimate**: 1h
**Files**: `crates/ckrv-ui/frontend/src/components/TaskEditor.test.tsx`

`TaskEditor.tsx` (968 lines) displays tasks generated from a spec.

**Key behaviors:**
- Renders task list with status indicators
- "Generate Tasks" button calls `POST /api/command` with `tasks` command
- Expanding a task shows detail content
- Task status badges render correctly (pending, running, done, failed)
- `TaskDetailModal` opens on task click

MSW handlers needed: `POST /api/command` (for task generation), task data via spec endpoints

**Acceptance Criteria**:
- [ ] Renders task list with various statuses
- [ ] Tests task generation trigger
- [ ] Tests task detail modal open/close
- [ ] Tests status badge rendering for all states

---

### Task 3.4: CodePage tests
**Priority**: P1
**Estimate**: 1h
**Files**: `crates/ckrv-ui/frontend/src/components/CodePage.test.tsx`

`CodePage.tsx` is the unified workflow page with tab navigation (Spec, Tasks, Plan, Run).

**Key behaviors:**
- Renders with 4 tabs visible
- Default tab is "Spec"
- Clicking each tab switches content area
- Tab state persists via `useCodeTab` hook
- Workflow progress indicator shows current step
- `data-testid` attributes are present (already verified: 6 occurrences)

**Acceptance Criteria**:
- [ ] Renders all 4 tabs
- [ ] Tests tab switching via click
- [ ] Tests default tab selection
- [ ] Tests that correct content panel renders per tab
- [ ] Tests workflow progress indicator states

---

### Task 3.5: ErrorBoundary tests
**Priority**: P2
**Estimate**: 30m
**Files**: `crates/ckrv-ui/frontend/src/components/ErrorBoundary.test.tsx`

`ErrorBoundary.tsx` is a critical safety net. Test that:
- It renders children normally when no error
- It catches thrown errors and shows fallback UI
- The "Try Again" / reset button clears the error state
- It logs error info (console.error)

**Acceptance Criteria**:
- [ ] Renders children when no error
- [ ] Catches component errors and shows fallback
- [ ] Reset button recovers from error state
- [ ] Error details visible in fallback UI

---

## Phase 4: Hook Tests

> Hook tests use `renderHook` from `@testing-library/react` wrapped in `QueryClientProvider` where needed. For WebSocket hooks, use the `MockWebSocket` from `@/test/mocks/websocket`.

### Task 4.1: useSpec hook tests
**Priority**: P1
**Estimate**: 1.5h
**Files**: `crates/ckrv-ui/frontend/src/hooks/useSpec.test.ts`

`useSpec.ts` (391 lines) is the most complex hook - handles spec CRUD via TanStack Query.

**Key behaviors:**
- `useSpecs()` fetches spec list from `/api/specs`
- `useSpec(name)` fetches individual spec content
- `useCreateSpec()` mutation calls `/api/specs/new`
- `useSaveSpec()` mutation calls `/api/specs/save`
- Auto-selection: `useAutoSelectedSpec` picks the first spec when none selected
- Cache invalidation: mutations invalidate the spec list query

**Test approach:** Use `renderHook` with the `QueryClientProvider` wrapper. MSW handles API responses.

**Acceptance Criteria**:
- [ ] Tests `useSpecs` returns spec list from API
- [ ] Tests `useSpec(name)` returns spec content
- [ ] Tests `useCreateSpec` mutation calls correct endpoint
- [ ] Tests `useSaveSpec` mutation invalidates query cache
- [ ] Tests loading and error states
- [ ] Tests auto-selection logic

---

### Task 4.2: useExecutionStream hook tests
**Priority**: P1
**Estimate**: 1.5h
**Files**: `crates/ckrv-ui/frontend/src/hooks/useExecutionStream.test.ts`

`useExecutionStream.ts` (298 lines) processes WebSocket events during task execution.

**Key behaviors:**
- Opens WebSocket connection to `/ws/execution`
- Processes event types: `batch_start`, `task_start`, `task_complete`, `task_error`, `batch_complete`, `execution_complete`
- Updates state progressively (current batch, task statuses, logs)
- Handles connection loss and reconnection
- Cleans up WebSocket on unmount

**Test approach:** Mock `WebSocket` globally with `MockWebSocket`. Simulate events via `simulateMessage()`.

**Acceptance Criteria**:
- [ ] Opens WebSocket on mount
- [ ] Processes each event type and updates state correctly
- [ ] Handles connection close and triggers reconnect
- [ ] Cleans up WebSocket on unmount
- [ ] Tests error event handling

---

### Task 4.3: useWebSocketReconnect hook tests
**Priority**: P2
**Estimate**: 1h
**Files**: `crates/ckrv-ui/frontend/src/hooks/useWebSocketReconnect.test.ts`

`useWebSocketReconnect.ts` handles exponential backoff reconnection.

**Key behaviors:**
- Starts in disconnected state
- Connects and transitions to `connected` state
- On connection loss, waits with exponential backoff before retry
- Caps retry count and backoff duration
- Exposes `reconnect()` for manual trigger
- Reports `status` (connected, connecting, disconnected, failed)

**Test approach:** Use `vi.useFakeTimers()` to control backoff delays. Mock `WebSocket`.

**Acceptance Criteria**:
- [ ] Tests initial disconnected state
- [ ] Tests connection lifecycle (connecting -> connected)
- [ ] Tests backoff timing increases exponentially
- [ ] Tests max retry limit
- [ ] Tests manual reconnect trigger
- [ ] Tests cleanup on unmount

---

## Phase 5: Conventions & Coverage

### Task 5.1: Set coverage thresholds
**Priority**: P2
**Estimate**: 30m
**Files**: `crates/ckrv-ui/frontend/vitest.config.ts`

After Phases 3 and 4 are complete, measure actual coverage and set thresholds slightly below current levels to prevent regression:

- Run `npm run test:coverage` and note actual numbers
- Set `thresholds` in vitest config to ~5% below current (gives buffer for new untested code)
- Configure `thresholds.perFile` to false initially (global thresholds only)
- Add comment documenting the ratchet-up schedule (review quarterly)

**Acceptance Criteria**:
- [ ] Coverage thresholds set in `vitest.config.ts`
- [ ] `npm run test:coverage` fails if coverage drops below thresholds
- [ ] Thresholds are realistic based on actual measured coverage
- [ ] Comment documents the ratchet plan

---

### Task 5.2: Document test conventions in FRONTEND_CONVENTIONS.md
**Priority**: P2
**Estimate**: 1h
**Files**: `crates/ckrv-ui/FRONTEND_CONVENTIONS.md`

Add a "Testing" section to the existing conventions doc. Cover:

**File organization:**
- Colocated tests: `Component.test.tsx` next to `Component.tsx`
- Test utilities: import from `@/test/test-utils` (not `@testing-library/react` directly)
- Mock data: use factories from `@/test/mocks/fixtures.ts`

**Writing tests:**
- Always use accessible queries (`getByRole`, `getByLabelText`, `getByText`) over `getByTestId`
- Use `findBy*` (async) for data that loads via TanStack Query
- Use `userEvent` (not `fireEvent`) for user interactions
- Don't test shadcn/ui primitives - only custom wrappers
- Don't test internal state - test visible behavior

**MSW patterns:**
- Default handlers cover happy paths
- Override per-test for error scenarios: `server.use(http.get('/api/agents', () => HttpResponse.error()))`
- Clean up: `server.resetHandlers()` in `afterEach` (handled by setup.ts)

**Naming:**
- `describe('ComponentName')` at top level
- `it('renders X when Y')` for render tests
- `it('calls API when user clicks Z')` for interaction tests

**Acceptance Criteria**:
- [ ] Testing section added to `FRONTEND_CONVENTIONS.md`
- [ ] Covers file organization, query patterns, MSW usage, naming
- [ ] Includes example test showing the preferred pattern
- [ ] Linked from the documentation checklist at the top

---

## Summary

After all 17 tasks are complete:

| What | Before | After |
|------|--------|-------|
| Unit test framework | None | Vitest + RTL + MSW |
| Component test coverage | 0% | Top 5 components tested |
| Hook test coverage | 0% | Top 3 hooks tested |
| CI unit test job | None | `frontend-test` with coverage |
| CI E2E job | None | `frontend-e2e` (Chromium) |
| Coverage thresholds | None | Configured and enforced |
| Test conventions | Undocumented | Section in FRONTEND_CONVENTIONS.md |
