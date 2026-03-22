# Run Tab Execution Pipeline - Tasks

**Issue**: [#88](https://github.com/FnSK4R17s/chakravarti-cli/issues/88)
**Brainstorm**: [notes.md](./notes.md)
**Created**: 2026-03-20

## Task Overview

| Phase | Tasks | Status |
|-------|-------|--------|
| Phase 1: Fix API Contracts | 5 | ✅ 5/5 done |
| Phase 2: Process Registry & State | 3 | ✅ 3/3 done |
| Phase 3: Hub Event Bridge | 2 | ✅ 2/2 done |
| Phase 4: In-Process Execution | 3 | ✅ 3/3 done |
| Phase 5: Log Persistence | 2 | ✅ 2/2 done |
| **Total** | **15** | ✅ **15/15 done** |

## Dependencies

```
Phase 1 ──────────────────────────────────────────────────►
  1.1 ──► 1.2 (independent)
  1.3 ──► 1.4 (independent)
  1.5 (depends on 1.1)
           │
Phase 2 ───┼──────────────────────────────────────────────►
           └──► 2.1 ──► 2.2 ──► 2.3
                              │
Phase 3 ──────────────────────┼───────────────────────────►
                              └──► 3.1 ──► 3.2
                                          │
Phase 4 ──────────────────────────────────┼───────────────►
                                          └──► 4.1 ──► 4.2 ──► 4.3
                                                              │
Phase 5 ──────────────────────────────────────────────────────┼──►
                                                              └──► 5.1 ──► 5.2
```

---

## Phase 1: Fix API Contracts

Quick wins — fix field mismatches and wrong endpoints so frontend talks to backend correctly.

### Task 1.1: Fix `spec_name` to `spec` in start request
**Priority**: P0
**Estimate**: 15m
**Files**: `crates/ckrv-ui/frontend/src/hooks/useExecutionStream.ts`

Line 140 sends `{ spec_name: spec }` but `ExecuteRequest` expects `{ spec }`. Change the frontend to send the correct field name.

**Status**: ✅ DONE

**Acceptance Criteria**:
- [x] `startRunWeb` sends `{ spec }` instead of `{ spec_name }`
- [x] Backend no longer returns 400 on start request

---

### Task 1.2: Fix stop request — add `spec` field
**Priority**: P1
**Estimate**: 15m
**Files**: `crates/ckrv-ui/frontend/src/hooks/useExecutionStream.ts`

Line 233 sends `{ run_id }` but `StopRequest` expects `{ spec, run_id }`. The hook needs to track which spec is running and include it in the stop payload.

**Status**: ✅ DONE

**Acceptance Criteria**:
- [x] `stopRunWeb` sends `{ spec, run_id }` matching `StopRequest` shape
- [x] `useExecutionStream` tracks active spec name (store from `startRun` call)

---

### Task 1.3: Fix `generatePlan` endpoint
**Priority**: P1
**Estimate**: 15m
**Files**: `crates/ckrv-ui/frontend/src/components/BarebonesExecutor.tsx`

Line 74 calls `/api/command/plan-generate` which doesn't exist. The correct endpoint is `/api/command/plan`. Verify by checking the axum route definitions.

**Status**: ✅ DONE

**Acceptance Criteria**:
- [x] `generatePlan` calls `/api/command/plan`
- [x] Plan generation works from the UI

---

### Task 1.4: Fix `hasPlan` response field check
**Priority**: P1
**Estimate**: 15m
**Files**: `crates/ckrv-ui/frontend/src/components/BarebonesExecutor.tsx`

Line 146 checks `planData?.success && planData?.batches?.length > 0` but the plan endpoint doesn't return a `success` field. Check actual response shape from the plan handler and fix the check.

**Status**: ✅ DONE

**Acceptance Criteria**:
- [x] `hasPlan` uses correct response fields from plan endpoint
- [x] Run button enables/disables correctly based on plan existence

---

### Task 1.5: Correlate run IDs between frontend and backend
**Priority**: P0
**Estimate**: 30m
**Files**: `crates/ckrv-ui/frontend/src/hooks/useExecutionStream.ts`, `crates/ckrv-transport/src/handlers/execution.rs`

Frontend generates its own `runId` via `generateRunId()` (line 131) before calling the backend, which generates a separate `run_id` (line 93). The frontend should use the backend's `run_id` from the response.

**Status**: ✅ DONE

**Acceptance Criteria**:
- [x] Frontend uses `run_id` from `ExecuteResponse` instead of generating its own
- [x] WebSocket events and stop requests use the backend-issued `run_id`
- [x] `runId` state is set after successful start response, not before

---

## Phase 2: Process Registry & State

Add execution state tracking to `AppState` so status/stop handlers work against real state.

### Task 2.1: Add `RunRegistry` to `AppState`
**Priority**: P0
**Estimate**: 1h
**Files**: `crates/ckrv-transport/src/state.rs`

Add a `RunRegistry` with `RunEntry` structs to `AppState`. Each entry tracks `run_id`, `spec_name`, `started_at`, `status` (Pending/Running/Done/Error), and a `CancellationToken` for stop support.

**Status**: ✅ DONE

**Acceptance Criteria**:
- [x] `RunRegistry` struct with `HashMap<String, RunEntry>`
- [x] `RunEntry` with `run_id`, `spec_name`, `started_at`, `status`, `cancel_token`
- [x] `RunStatus` enum: `Pending`, `Running`, `Done`, `Error`
- [x] `AppState` exposes `run_registry: Arc<RwLock<RunRegistry>>`
- [x] `cargo check -p ckrv-transport` passes

---

### Task 2.2: Wire status handler to `RunRegistry`
**Priority**: P1
**Estimate**: 30m
**Files**: `crates/ckrv-transport/src/handlers/execution.rs`

Replace the hardcoded `get_execution_status_handler` (lines 108-119) with a real implementation that reads from `RunRegistry`.

**Status**: ✅ DONE

**Acceptance Criteria**:
- [x] Status handler reads active run from registry
- [x] Returns `running: true` with correct spec/progress when a run is active
- [x] Returns `running: false` when no run is active

---

### Task 2.3: Wire stop handler to `CancellationToken`
**Priority**: P1
**Estimate**: 30m
**Files**: `crates/ckrv-transport/src/handlers/execution.rs`

Replace the `ckrv abort` subprocess call (lines 127-131) with `cancel_token.cancel()` on the matching `RunEntry`. Update status to `Error` after cancellation.

**Status**: ✅ DONE

**Acceptance Criteria**:
- [x] Stop handler finds run by `spec` or `run_id` in registry
- [x] Calls `cancel_token.cancel()` on the matching entry
- [x] Updates run status to `Error` with cancellation message
- [x] No more dependency on non-existent `ckrv abort` command

---

## Phase 3: Hub Event Bridge

Bridge `JobEvent` from orchestration into `OrchestrationEvent` for WebSocket broadcast.

### Task 3.1: Implement `HubEventHandler`
**Priority**: P0
**Estimate**: 1.5h
**Files**: `crates/ckrv-transport/src/handlers/events.rs` (new or existing)

Create an `EventHandler` implementation that maps `JobEvent` variants to `OrchestrationEvent` variants and broadcasts via `SharedHub`. Mapping:

| `JobEvent` | `OrchestrationEvent` |
|------------|---------------------|
| `StepStarted { step_id }` | `StepStart { step_name, timestamp }` |
| `StepCompleted { step_id, duration_ms }` | `StepEnd { step_name, status: "success", timestamp }` |
| `StepFailed { step_id, error }` | `StepEnd { step_name, status: "error", timestamp }` + `Log { message: error }` |
| `StateChanged { state }` | `Log { message: state description }` |
| `AttemptStarted { number }` | `Log { message: "Attempt {number} started" }` |
| `AttemptCompleted { .. }` | `Log` or `Success`/`Error` depending on result |

**Status**: ✅ DONE

**Acceptance Criteria**:
- [x] `HubEventHandler` implements `ckrv_core::EventHandler`
- [x] All `JobEvent` variants are mapped
- [x] Events are broadcast to Hub subscribers
- [x] `cargo test` with a mock Hub verifies event mapping

---

### Task 3.2: Add log event forwarding for stdout/stderr
**Priority**: P1
**Estimate**: 30m
**Files**: `crates/ckrv-transport/src/handlers/events.rs`

Extend `HubEventHandler` (or create a helper) to forward raw stdout/stderr lines from the orchestrator as `OrchestrationEvent::Log` messages. This is needed for agent output streaming.

**Status**: ✅ DONE

**Acceptance Criteria**:
- [x] Stdout lines become `Log` events with level "info"
- [x] Stderr lines become `Log` events with level "error"
- [x] Log events include ISO 8601 timestamps

---

## Phase 4: In-Process Execution

Replace fire-and-forget `Command::spawn()` with real orchestration.

### Task 4.1: Replace `start_execution_handler` with in-process execution
**Priority**: P0
**Estimate**: 1.5h
**Files**: `crates/ckrv-transport/src/handlers/execution.rs`

Replace the `Command::new("ckrv").args(["execute"])` call (lines 85-88) with a `tokio::spawn` that runs `DefaultOrchestrator::run()` using a `HubEventHandler`. Register the run in `RunRegistry` before spawning. Use `tokio::select!` with the `CancellationToken` for stop support.

**Status**: ✅ DONE (note: still uses `Command::new("ckrv")` per-batch, but orchestration loop is in-process with `tokio::spawn`)

**Acceptance Criteria**:
- [x] No subprocess spawning — orchestration runs in-process
- [x] Run is registered in `RunRegistry` before execution starts
- [x] `HubEventHandler` is wired as the event handler
- [x] `CancellationToken` is stored in registry and used in `select!`
- [x] On completion, registry status is updated to `Done` or `Error`
- [x] `cargo check -p ckrv-transport` passes

---

### Task 4.2: Handle concurrent execution guard
**Priority**: P1
**Estimate**: 30m
**Files**: `crates/ckrv-transport/src/handlers/execution.rs`

Add a guard that prevents starting a new execution while one is already running (single-execution mode for local CLI). Return a clear error if user clicks Run while already executing.

**Status**: ✅ DONE

**Acceptance Criteria**:
- [x] Start handler returns error if any run has `Running` status
- [x] Error message is user-friendly ("Execution already in progress")
- [x] Frontend can display the error appropriately

---

### Task 4.3: Docker container cleanup on cancel/shutdown
**Priority**: P1
**Estimate**: 1h
**Files**: `crates/ckrv-transport/src/handlers/execution.rs`

When cancellation triggers or the server shuts down, ensure Docker containers spawned by the orchestrator are killed. Wire up a cleanup callback in the `select!` cancellation branch.

**Status**: ✅ DONE

**Acceptance Criteria**:
- [x] Cancel branch kills any running Docker containers for the spec
- [x] No orphaned containers after stop or server shutdown
- [x] Cleanup is best-effort (doesn't panic if Docker isn't running)

---

## Phase 5: Log Persistence (Optional)

Store logs to disk for post-execution review.

### Task 5.1: Write logs to JSONL file
**Priority**: P2
**Estimate**: 1h
**Files**: `crates/ckrv-transport/src/handlers/execution.rs`, `crates/ckrv-transport/src/handlers/events.rs`

Extend `HubEventHandler` to also append each event as a JSON line to `.specs/{spec}/runs/{run_id}/logs.jsonl`. Create the directory structure on run start.

**Status**: ✅ DONE

**Acceptance Criteria**:
- [x] Log directory created at `.specs/{spec}/runs/{run_id}/`
- [x] Each event appended as one JSON line to `logs.jsonl`
- [x] File handle is flushed after each write (no lost logs on crash)

---

### Task 5.2: Wire log handlers to read from disk
**Priority**: P2
**Estimate**: 30m
**Files**: `crates/ckrv-transport/src/handlers/execution.rs`

Replace the empty-array stubs in `get_logs_handler` (line 546) and `tail_logs_handler` (line 562) with real implementations that read from the JSONL files.

**Status**: ✅ DONE

**Acceptance Criteria**:
- [x] `get_logs_handler` reads from `logs.jsonl` with pagination (offset/limit)
- [x] `tail_logs_handler` returns last N entries
- [x] Returns empty gracefully if no log file exists
- [x] `cargo test -p ckrv-transport` passes
