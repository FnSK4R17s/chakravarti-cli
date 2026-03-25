# Run Tab Execution Pipeline is Non-Functional

**Issue**: [#88](https://github.com/FnSK4R17s/chakravarti-cli/issues/88)
**Created**: 2026-03-10
**Status**: In Progress

## Problem Statement

The Run tab UI renders but clicking Run produces no real execution. The backend spawns a fire-and-forget process with no output capture, the WebSocket never receives events, and most endpoints are stubs. There are 10 distinct bugs spanning API contract mismatches, missing process management, and stub handlers.

This is the **core value proposition** of ckrv — "fire and forget" execution from the UI. If this doesn't work, the UI is just a pretty dashboard with no teeth.

## Current State

### What works
- **Hub broadcast channel** (`hub.rs`) — properly implemented pub-sub with bounded channel
- **WebSocket forwarding** (`axum/execution.rs`) — correctly subscribes to Hub and forwards events
- **Frontend UI** (`BarebonesExecutor.tsx`) — renders batch pills, log terminal, Run/Stop buttons
- **CLI `ckrv run`** (`commands/run.rs`) — full local execution pipeline with Docker sandboxes
- **Job/Attempt types** (`job.rs`) — proper data structures for tracking execution

### What's broken (10 issues from #88)

| # | Severity | Issue | File |
|---|----------|-------|------|
| 1 | CRITICAL | Frontend sends `spec_name`, backend expects `spec` | `useExecutionStream.ts:140` |
| 2 | CRITICAL | `start_execution_handler` is fire-and-forget, no Hub integration | `handlers/execution.rs:85-88` |
| 3 | CRITICAL | Run IDs are uncorrelated (frontend vs backend generate separately) | Both sides |
| 4 | HIGH | `generatePlan` calls non-existent `/api/command/plan-generate` | `BarebonesExecutor.tsx:74` |
| 5 | HIGH | `hasPlan` check uses wrong response field (`success` doesn't exist) | `BarebonesExecutor.tsx:146` |
| 6 | HIGH | Stop handler calls non-existent `ckrv abort` command | `handlers/execution.rs:127` |
| 7 | MEDIUM | Frontend sends wrong fields for stop request (missing `spec`) | `useExecutionStream.ts:231` |
| 8 | MEDIUM | Execution status handler is hardcoded stub (always `running: false`) | `handlers/execution.rs:108-119` |
| 9 | MEDIUM | Log handlers always return empty arrays | `handlers/execution.rs:546-573` |
| 10 | LOW | Hub <-> Frontend event types align but Hub never receives events | `hub.rs` / `useExecutionStream.ts` |

### Data flow (current — broken)

```
User clicks "Run"
  → POST /api/execution/start {spec_name: "my-spec"}  ← WRONG FIELD
  → Backend: 400 Bad Request (spec field empty)
  → Even if fixed: spawn("ckrv execute") → child handle DROPPED
  → No stdout/stderr captured
  → No events published to Hub
  → WebSocket connected but silent forever
  → Frontend shows "Running..." with no logs, no progress, no completion
```

## Proposed Solution

### Architecture: In-Process Execution with Hub Bridge

Instead of shelling out to `ckrv execute`, the UI backend should run execution **in-process** using the orchestration engine directly. This gives us:
- Direct access to stdout/stderr pipes
- Ability to publish events to Hub in real-time
- Process handle for stop/status tracking
- No need for a `ckrv abort` CLI command

### Data flow (target)

```
User clicks "Run"
  → POST /api/execution/start {spec: "my-spec"}
  → Backend: validate spec exists, create run_id
  → Store run state in AppState (Arc<RwLock<RunRegistry>>)
  → Spawn tokio task:
      → Run orchestration (tasks → Docker sandbox → agent)
      → Pipe stdout/stderr lines → Hub.broadcast(Log{...})
      → On step start/end → Hub.broadcast(StepStart/StepEnd{...})
      → On completion → Hub.broadcast(Success/Error{...})
  → Return {started: true, run_id}
  → Frontend: connect WebSocket → receive real-time events
  → Batch pills update as steps start/end
  → Logs stream in terminal area
  → Completion: status badge updates, WebSocket closes
```

## User Stories

### US1: Start Execution from Run Tab
**As a** developer using the ckrv UI,
**I want** to click "Run" and see real-time execution progress,
**So that** I can monitor my spec being implemented without switching to the terminal.

### US2: Stop a Running Execution
**As a** developer who notices something wrong mid-execution,
**I want** to click "Stop" and have execution halt immediately,
**So that** I don't waste agent tokens on a bad run.

### US3: View Execution Logs
**As a** developer reviewing execution results,
**I want** to see all log output from agent execution,
**So that** I can debug failures and understand what agents did.

### US4: Track Batch Progress
**As a** developer running a multi-batch spec,
**I want** to see which batches are pending/running/done/failed,
**So that** I know where execution is at a glance.

## Technical Approach

### Options Considered

| Option | Pros | Cons |
|--------|------|------|
| **A: In-process execution** — Run orchestration directly in the axum server process | Real-time Hub access, no IPC overhead, direct process control | Ties execution to server lifecycle, server restart kills execution |
| **B: Subprocess with pipe bridge** — Shell out to `ckrv run` but capture stdout/stderr and parse into Hub events | Reuses existing CLI pipeline, execution survives server restart | Fragile output parsing, lossy log capture, no structured events |
| **C: Shared file/socket bridge** — `ckrv run` writes events to a Unix socket or file, server reads and broadcasts | Decoupled processes, resilient | Complex IPC, ordering issues, platform-specific |

### Decision

**Option A: In-process execution** — aligns with the vision's "fire and forget" model. The server IS the orchestration engine. If the server dies, execution should stop (it's running Docker containers on behalf of the user — orphaned containers are worse than stopped execution).

The CLI `ckrv run` remains for terminal-only users. The UI server uses the same orchestration code but with a Hub-connected event handler.

### Implementation Plan

#### Phase 1: Fix API Contracts (Quick Wins)
Fix the 5 field mismatches and wrong endpoints so the frontend can at least talk to the backend correctly:
1. `spec_name` → `spec` in `useExecutionStream.ts`
2. `/api/command/plan-generate` → `/api/command/plan` in `BarebonesExecutor.tsx`
3. Fix `hasPlan` check to use correct response fields
4. Add `spec` field to stop request
5. Correlate run IDs (backend returns run_id, frontend uses it)

#### Phase 2: Process Registry & State Tracking
Add execution state tracking to `AppState`:
```rust
pub struct RunRegistry {
    pub runs: HashMap<String, RunEntry>,
}
pub struct RunEntry {
    pub run_id: String,
    pub spec_name: String,
    pub started_at: Instant,
    pub status: RunStatus, // Pending | Running | Done | Error
    pub cancel_token: CancellationToken,
}
```
- Status handler reads from registry instead of returning hardcoded values
- Stop handler triggers `cancel_token.cancel()`

#### Phase 3: Hub Event Bridge
Create an `EventHandler` implementation that bridges `JobEvent` → `OrchestrationEvent` → `Hub.broadcast()`:
```rust
struct HubEventHandler {
    hub: SharedHub,
}
impl EventHandler for HubEventHandler {
    fn handle(&self, event: JobEvent) {
        let orch_event = match event {
            JobEvent::StepStarted { step_id } =>
                OrchestrationEvent::StepStart { step_name: step_id, .. },
            JobEvent::StepCompleted { step_id, .. } =>
                OrchestrationEvent::StepEnd { step_name: step_id, .. },
            // ... map all variants
        };
        self.hub.broadcast(orch_event);
    }
}
```

#### Phase 4: In-Process Execution
Replace the fire-and-forget `Command::spawn()` with actual orchestration:
```rust
// In start_execution_handler:
let hub = state.hub.clone();
let registry = state.run_registry.clone();
let cancel = CancellationToken::new();

tokio::spawn(async move {
    let handler = HubEventHandler::new(hub);
    let orchestrator = DefaultOrchestrator::new(handler);

    select! {
        result = orchestrator.run(spec, config) => {
            // Update registry with result
        }
        _ = cancel.cancelled() => {
            // Cleanup: kill Docker containers
        }
    }
});
```

#### Phase 5: Log Persistence (Optional)
Store logs to disk for post-execution review:
- Write to `.specs/{spec}/runs/{run_id}/logs.jsonl`
- `get_logs_handler` reads from disk
- `tail_logs_handler` returns last N entries

## Open Questions

- [ ] Should the WebSocket filter events by `run_id`? Currently all subscribers get all events. Fine for single-user, but breaks with concurrent runs.
- [ ] Should we support multiple concurrent executions? Vision says "multi-spec parallelism" is a cloud feature. For now, single execution with a "busy" guard seems right.
- [ ] How does `DefaultOrchestrator.execute_step()` connect to the real Docker sandbox execution? Currently it's simulated — need to wire up `ckrv-sandbox` crate.
- [ ] Should log persistence be per-run or per-spec? Per-run makes more sense for history.

## Success Criteria

| Metric | Target |
|--------|--------|
| Click "Run" → first log appears | < 3 seconds |
| Batch status pills update in real-time | On every step start/end |
| "Stop" kills execution | Within 2 seconds |
| Status endpoint reflects actual state | Always accurate |
| No orphaned Docker containers | On stop or server shutdown |

## Files Involved

| File | Role | Changes Needed |
|------|------|----------------|
| `crates/ckrv-ui/frontend/src/hooks/useExecutionStream.ts` | WebSocket/execution hook | Fix `spec_name` → `spec`, fix stop request fields |
| `crates/ckrv-ui/frontend/src/components/BarebonesExecutor.tsx` | Run tab UI | Fix plan endpoint, fix `hasPlan` check |
| `crates/ckrv-transport/src/handlers/execution.rs` | Execution handlers | Replace stubs with real implementation |
| `crates/ckrv-transport/src/axum/execution.rs` | Route wrappers + WebSocket | Add run_id filtering (optional) |
| `crates/ckrv-transport/src/hub.rs` | Broadcast channel | No changes needed |
| `crates/ckrv-transport/src/state.rs` | App state | Add RunRegistry |
| `crates/ckrv-core/src/orchestrator.rs` | Orchestration engine | Wire up real step execution |

## Next Steps

- [ ] Phase 1: Fix API contracts (quick wins, ~30 min)
- [ ] Phase 2: Add RunRegistry to AppState (~1 hour)
- [ ] Phase 3: Implement HubEventHandler bridge (~1 hour)
- [ ] Phase 4: Replace fire-and-forget with in-process execution (~2 hours)
- [ ] Phase 5: Log persistence (optional, ~1 hour)

## References

- [Issue #88](https://github.com/FnSK4R17s/chakravarti-cli/issues/88) — Full bug report with 10 issues
- [PR #87](https://github.com/FnSK4R17s/chakravarti-cli/pull/87) — Code page overhaul (where these issues were discovered)
- `guiding_docs/vision.md` — "Fire and forget" execution model
- `crates/ckrv-core/src/orchestrator.rs` — EventHandler trait that needs Hub bridge
