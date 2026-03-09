# UI: Connection Badge Flips to Disconnected on Message Submit - Tasks

**Issue**: [#81](https://github.com/FnSK4R17s/chakravarti-cli/issues/81)
**Brainstorm**: [notes.md](./notes.md)
**Created**: 2026-03-09

## Task Overview

| Phase | Tasks | Estimate |
|-------|-------|----------|
| Phase 1: Backend — Unblock async runtime | 4 | 3h |
| Phase 2: Frontend — Connection resilience | 3 | 2h |
| Phase 3: Verification & cleanup | 2 | 1h |
| **Total** | **9** | **6h** |

## Dependencies

```
Phase 1 ─────────────────────────────────────────────────►
  Task 1.1 ──► Task 1.2 ──┬─► Task 1.3
                           │
                           └─► Task 1.4
                                  │
Phase 2 ──────────────────────────┼──────────────────────►
                                  │
  Task 2.1 ──► Task 2.2 ──► Task 2.3
                                  │
Phase 3 ──────────────────────────┼──────────────────────►
                                  │
                                  └─► Task 3.1 ──► Task 3.2
```

Phase 2 can start independently of Phase 1 (no code dependency), but testing (Phase 3) requires both.

---

## Phase 1: Backend — Unblock Async Runtime

### Task 1.1: Wrap command handlers with `spawn_blocking`
**Priority**: P0
**Estimate**: 1h
**Files**: `crates/ckrv-transport/src/axum/commands.rs`

All 10 route handlers in `commands.rs` call synchronous `run_*_handler()` functions that execute `std::process::Command::output()`. These block the Tokio executor. Wrap each handler with `tokio::task::spawn_blocking()`.

The `AppState` is already `Clone` with `Arc<RwLock<_>>` internals (`state.rs:97-107`), so the `move` closure works out of the box.

**Pattern to apply to each handler:**
```rust
async fn run_spec_new(
    State(state): State<AppState>,
    Json(request): Json<SpecNewRequest>,
) -> impl IntoResponse {
    match tokio::task::spawn_blocking(move || run_spec_new_handler(&state, request)).await {
        Ok(Ok(result)) => Json(result).into_response(),
        Ok(Err(e)) => e.into_response(),
        Err(e) => TransportError::Internal(format!("Task panicked: {e}")).into_response(),
    }
}
```

**Handlers to wrap (all 10):**
- `run_init`
- `run_git_init`
- `run_spec_new`
- `run_spec_tasks`
- `run_plan`
- `run_execute`
- `run_diff`
- `run_verify`
- `run_promote`
- `run_fix`

**Acceptance Criteria**:
- [ ] All 10 handlers use `spawn_blocking`
- [ ] `TransportError` import added (needed for `Err(JoinError)` arm)
- [ ] `cargo check -p ckrv-transport` passes
- [ ] No clippy warnings

---

### Task 1.2: Wrap status handler with `spawn_blocking`
**Priority**: P0
**Estimate**: 30m
**Files**: `crates/ckrv-transport/src/axum/status.rs`, `crates/ckrv-transport/src/handlers/status.rs`

The status handler (`get_status_handler`) is `async` but internally calls `detect_git_branch()` which runs 3 synchronous `std::process::Command` calls. Two options:

**Option A (simpler):** Wrap the `get_status` axum route with `spawn_blocking`:
```rust
async fn get_status(State(state): State<AppState>) -> impl IntoResponse {
    match tokio::task::spawn_blocking(move || {
        // get_status_handler needs to become sync, or we use block_on inside spawn_blocking
    }).await { ... }
}
```

**Option B (cleaner):** Make `get_status_handler` synchronous (remove `.await` on the `RwLock::read()` by switching to `std::sync::RwLock` for reading, or clone the status first). The only async call is `state.status.read().await` — this can be replaced with `state.status.blocking_read()` inside `spawn_blocking`.

Go with Option B: change `get_status_handler` to be sync, use `blocking_read()`, and wrap the axum route in `spawn_blocking`.

**Acceptance Criteria**:
- [ ] `get_status_handler` is synchronous (no `async`)
- [ ] Uses `state.status.blocking_read()` instead of `.read().await`
- [ ] Axum route wraps call in `spawn_blocking`
- [ ] `cargo check -p ckrv-transport` passes

---

### Task 1.3: Wrap specs handlers with `spawn_blocking`
**Priority**: P1
**Estimate**: 45m
**Files**: `crates/ckrv-transport/src/axum/specs.rs`

The `list_specs`, `get_spec_detail`, `create_spec`, and `save_spec` handlers call synchronous handler functions that perform filesystem I/O (reading YAML files, creating directories). Wrap them in `spawn_blocking`.

Placeholder handlers (`validate_spec`, `generate_design`, `generate_tasks`, `get_clarifications`, `clarify`) return static JSON — these do NOT need wrapping.

**Handlers to wrap (4):**
- `list_specs` → calls `list_specs_handler` (reads filesystem)
- `get_spec_detail` → calls `get_spec_handler` (reads YAML files)
- `create_spec` → calls `create_spec_handler` (writes files, may run `ckrv`)
- `save_spec` → calls `update_spec_handler` (writes files)

**Acceptance Criteria**:
- [ ] 4 non-placeholder handlers use `spawn_blocking`
- [ ] Placeholder handlers left untouched
- [ ] `cargo check -p ckrv-transport` passes

---

### Task 1.4: Wrap docker and cloud handlers with `spawn_blocking`
**Priority**: P1
**Estimate**: 30m
**Files**: `crates/ckrv-transport/src/axum/docker.rs`, `crates/ckrv-transport/src/axum/cloud.rs`

Both `check_docker` and `get_cloud_status` call synchronous handlers that spawn subprocesses (`docker info`, cloud auth checks). These poll every 10s/15s from the frontend and can compound the blocking problem.

**Acceptance Criteria**:
- [ ] `check_docker` handler uses `spawn_blocking`
- [ ] `get_cloud_status` handler uses `spawn_blocking`
- [ ] `cargo check -p ckrv-transport` passes

---

## Phase 2: Frontend — Connection Resilience

### Task 2.1: Switch `useConnection` to use `/health` endpoint
**Priority**: P1
**Estimate**: 30m
**Files**: `crates/ckrv-ui/frontend/src/hooks/useConnection.ts`, `crates/ckrv-ui/frontend/src/hooks/useConnection.test.ts`

The `/health` endpoint (`server.rs:131`) returns `"OK"` instantly with zero blocking I/O — it never touches git, filesystem, or subprocesses. Switch `useConnection` to ping `/health` instead of `/api/status`.

This decouples the connection heartbeat from the heavier status data fetch, which is the correct separation of concerns. The React Query `['status']` in `Dashboard.tsx` already handles the richer `/api/status` data.

Also increase the abort timeout from 3s to 5s for extra resilience.

**Acceptance Criteria**:
- [ ] `useConnection` fetches `/health` instead of `/api/status`
- [ ] Abort timeout increased from 3000ms to 5000ms
- [ ] Existing tests updated to mock `/health` instead of `/api/status`
- [ ] `npm run typecheck` passes (from `crates/ckrv-ui/frontend/`)

---

### Task 2.2: Add consecutive failure threshold before showing "disconnected"
**Priority**: P1
**Estimate**: 45m
**Files**: `crates/ckrv-ui/frontend/src/hooks/useConnection.ts`, `crates/ckrv-ui/frontend/src/hooks/useConnection.test.ts`

A single failed health check shouldn't flip the badge to "disconnected" — transient network hiccups or momentary load spikes happen. Add a consecutive failure counter: only transition to "disconnected" after 2+ consecutive failures.

**Implementation:**
```typescript
const [failureCount, setFailureCount] = useState(0);

// In checkConnection:
if (response.ok) {
    setFailureCount(0);
    setStatus('connected');
} else {
    setFailureCount(prev => prev + 1);
    if (failureCount + 1 >= 2) {
        setStatus('disconnected');
    }
}
```

**Acceptance Criteria**:
- [ ] New `failureCount` state variable
- [ ] Status only changes to `'disconnected'` after 2+ consecutive failures
- [ ] Single failure keeps status as `'connected'` (if previously connected)
- [ ] Successful response resets failure count to 0
- [ ] Tests cover: single failure stays connected, two failures show disconnected, recovery resets
- [ ] `npm run typecheck` passes

---

### Task 2.3: Remove duplicate `/api/status` polling from Dashboard
**Priority**: P2
**Estimate**: 30m
**Files**: `crates/ckrv-ui/frontend/src/layouts/Dashboard.tsx`

Currently `Dashboard.tsx` has two independent polls to `/api/status`:
1. `useConnection(5000)` — for the connection badge (line 58)
2. `useQuery(['status'], ...)` with `refetchInterval: 5000` — for branch name/is_ready (lines 65-72)

After Task 2.1, `useConnection` uses `/health`, so no duplication remains for `/api/status`. However, we should verify the React Query `['status']` query is the sole consumer of `/api/status` and remove any redundant invalidation.

Review `ChatDashboard.tsx:121` — the `invalidateQueries({ queryKey: ['status'] })` on spec creation success is fine (it refreshes `is_ready` status), but confirm it doesn't cause a cascade.

**Acceptance Criteria**:
- [ ] Only one source polls `/api/status` (the React Query `['status']` in Dashboard)
- [ ] `useConnection` polls `/health` (confirmed from Task 2.1)
- [ ] Network tab shows no duplicate `/api/status` requests at the same interval
- [ ] Branch name and `is_ready` still update correctly

---

## Phase 3: Verification & Cleanup

### Task 3.1: Build and test full stack
**Priority**: P0
**Estimate**: 30m
**Files**: N/A (integration testing)

Verify the fix end-to-end:

1. `cargo check -p ckrv-transport` — all backend changes compile
2. `cargo test -p ckrv-transport` — existing tests pass
3. `cd crates/ckrv-ui/frontend && npx tsc --noEmit` — frontend type checks
4. `cd crates/ckrv-ui/frontend && npm test` — frontend tests pass
5. `just install` — full build succeeds
6. `ckrv ui` — launch and manually verify:
   - Connection badge shows "Connected" on load
   - Submit a spec description
   - Badge stays green throughout spec creation
   - Badge recovers if server is briefly stopped and restarted

**Acceptance Criteria**:
- [ ] `cargo check -p ckrv-transport` passes
- [ ] `cargo test -p ckrv-transport` passes
- [ ] Frontend type check passes
- [ ] Frontend tests pass
- [ ] `just install` succeeds
- [ ] Manual test: badge stays "Connected" during spec creation

---

### Task 3.2: Update brainstorm status
**Priority**: P2
**Estimate**: 5m
**Files**: `brainstorming/issue-081-ui-connection-status/notes.md`

Update brainstorm status from "Draft" to "Implemented". Add any implementation notes or deviations discovered during implementation.

**Acceptance Criteria**:
- [ ] Status changed to "Implemented"
- [ ] Any deviations from plan documented

---

## Out of Scope (Future Work)

These items were discussed in the brainstorm but deferred:

| Item | Why Deferred | Follow-up |
|------|-------------|-----------|
| Convert to `tokio::process::Command` | Larger refactor, Option B from brainstorm | Could be a separate issue for all 10 handler files |
| Wrap remaining handlers (tasks, test, agents, console, diff, execution) | Not on the critical path for this bug | Same `spawn_blocking` pattern applies; do when those pages show issues |
| Add `/api/ping` endpoint | `/health` already exists and serves the purpose | Not needed |
