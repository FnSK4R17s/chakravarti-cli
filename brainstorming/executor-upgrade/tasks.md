# Simplified Execution Runner - Tasks

**Brainstorm**: [notes.md](./notes.md)
**Created**: 2026-02-06
**Implemented**: 2026-02-06
**Status**: ✅ Implemented

## Task Overview

| Phase | Tasks | Estimate |
|-------|-------|----------|
| Phase 1: Backend API Enhancement | 2 | 2h |
| Phase 2: Axum WebSocket Fix | 2 | 3h |
| Phase 3: Tauri Execution Commands | 3 | 4h |
| Phase 3.5: Tauri Project Selection | 4 | 4h |
| Phase 4: Frontend Hook | 2 | 3h |
| Phase 5: Simplify Executor UI | 2 | 2h |
| **Total** | **15** | **18h** |

## Integration Points Discovered

From documentation research:

| Component | Imported By | Action Required |
|-----------|------------|-----------------|
| `BarebonesExecutor` | `CodePage.tsx` | Update if API changes |
| `SystemStatus` | `types.ts`, `StatusWidget`, `ChatDashboard`, `CommandPalette`, `useAutoSelectedSpec`, `useWorkflowProgress` | Add `project_root` field |
| Axum execution routes | `ckrv-transport/src/axum/execution.rs` | Fix WebSocket stub |
| LogStore | `services/engine.rs` | Already works, no changes |

---

## Phase 1: Backend API Enhancement

### Task 1.1: Add `project_root` to SystemStatus
**Priority**: P0
**Estimate**: 1h
**Files**: `crates/ckrv-transport/src/state.rs`, `crates/ckrv-transport/src/handlers/status.rs`

Add project path to SystemStatus so both web and Tauri can display it in Settings.

```rust
pub struct SystemStatus {
    pub active_branch: String,
    pub feature_number: Option<String>,
    pub is_ready: bool,
    pub mode: SystemMode,
    pub project_root: String,  // NEW
}
```

**Acceptance Criteria**:
- [ ] `project_root: String` added to `SystemStatus` struct
- [ ] `get_status_handler` populates it from `state.project_root`
- [ ] TypeScript types updated (run `cargo test -p ckrv-transport --features typescript`)
- [ ] `cargo check -p ckrv-transport` passes

---

### Task 1.2: Update Frontend SystemStatus Type
**Priority**: P0
**Estimate**: 1h
**Files**: `crates/ckrv-ui/frontend/src/types.ts`

Update the TypeScript interface to include `project_root`.

**Affected Importers** (from research):
- `StatusWidget.tsx` - Uses `SystemStatus` type
- `ChatDashboard.tsx` - Has duplicate interface (consolidate)
- `CommandPalette.tsx` - Has duplicate interface (consolidate)
- `useAutoSelectedSpec.ts` - Imports from types
- `useWorkflowProgress.ts` - Has duplicate interface (consolidate)

**Acceptance Criteria**:
- [ ] `project_root: string` added to `SystemStatus` in `types.ts`
- [ ] Duplicate `SystemStatus` interfaces consolidated to use shared type
- [ ] Settings page displays project path (add to SettingsPage.tsx)
- [ ] TypeScript compiles without errors

---

## Phase 2: Axum WebSocket Fix

### Task 2.1: Implement Execution WebSocket Streaming
**Priority**: P0
**Estimate**: 2h
**Files**: `crates/ckrv-transport/src/axum/execution.rs`, `crates/ckrv-transport/src/handlers/execution.rs`

Replace the TODO stub with real WebSocket log streaming.

```rust
async fn execution_ws(
    State(state): State<AppState>,
    Query(query): Query<WsQuery>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        // Subscribe to engine broadcast
        // Forward messages to WebSocket client
        // Handle disconnect gracefully
    })
}
```

**Acceptance Criteria**:
- [ ] WebSocket connects and streams log messages
- [ ] Batch status updates sent to client
- [ ] Execution complete event sent
- [ ] Client disconnect handled without crash
- [ ] Manual test: `wscat -c ws://localhost:3000/api/execution/ws?run_id=test`

---

### Task 2.2: Add Engine Broadcast Subscription
**Priority**: P0
**Estimate**: 1h
**Files**: `crates/ckrv-ui/src/services/engine.rs`

Expose a method for the WebSocket handler to subscribe to execution events.

**Acceptance Criteria**:
- [ ] `Engine::subscribe(run_id: String) -> broadcast::Receiver<LogMessage>` method added
- [ ] Receiver gets all log and status messages for the run
- [ ] Multiple subscribers supported (broadcast channel)
- [ ] Unit test for subscription

---

## Phase 3: Tauri Execution Commands

### Task 3.1: Create Execution Commands Module
**Priority**: P1
**Estimate**: 1h
**Files**: `crates/ckrv-tauri/src/commands/execution.rs`, `crates/ckrv-tauri/src/commands/mod.rs`

Create Tauri commands for starting/stopping execution.

```rust
#[tauri::command]
async fn start_execution(spec: String, run_id: String, app: AppHandle) -> Result<String, String>

#[tauri::command]
async fn stop_execution(run_id: String) -> Result<(), String>

#[tauri::command]
async fn get_execution_status() -> Result<ExecutionStatus, String>
```

**Acceptance Criteria**:
- [ ] Commands registered in `lib.rs`
- [ ] Start execution spawns engine task
- [ ] Events emitted: `execution:log`, `execution:batch_status`, `execution:complete`
- [ ] Stop execution cancels running task

---

### Task 3.2: Emit Execution Events to Frontend
**Priority**: P1
**Estimate**: 2h
**Files**: `crates/ckrv-tauri/src/commands/execution.rs`

Forward engine broadcast messages as Tauri events.

```rust
app.emit("execution:log", LogPayload { ... })?;
app.emit("execution:batch_status", BatchPayload { ... })?;
app.emit("execution:complete", CompletePayload { ... })?;
```

**Acceptance Criteria**:
- [ ] Log messages emitted as events
- [ ] Batch status changes emitted
- [ ] Completion status emitted (success/error)
- [ ] Events include run_id for filtering
- [ ] Frontend can listen with `listen("execution:log", ...)`

---

### Task 3.3: Add Log Access Commands
**Priority**: P1
**Estimate**: 1h
**Files**: `crates/ckrv-tauri/src/commands/execution.rs`

Allow frontend to fetch historical logs.

```rust
#[tauri::command]
async fn get_execution_logs(run_id: String, since: Option<String>) -> Result<Vec<LogEntry>, String>

#[tauri::command]
async fn list_executions() -> Result<Vec<RunSummary>, String>
```

**Acceptance Criteria**:
- [ ] `get_execution_logs` returns logs from LogStore
- [ ] `since` parameter filters by timestamp (for reconnection)
- [ ] `list_executions` returns run summaries

---

## Phase 3.5: Tauri Project Selection

### Task 3.5.1: Create Project Commands
**Priority**: P1
**Estimate**: 1h
**Files**: `crates/ckrv-tauri/src/commands/project.rs`, `crates/ckrv-tauri/src/commands/mod.rs`

Add commands for project selection.

```rust
#[tauri::command]
async fn get_project_root(state: State<'_, AppState>) -> Result<Option<PathBuf>, String>

#[tauri::command]
async fn set_project_root(path: PathBuf, state: State<'_, AppState>) -> Result<(), String>

#[tauri::command]
async fn get_recent_projects() -> Result<Vec<ProjectInfo>, String>

#[tauri::command]
async fn open_project_dialog(app: AppHandle) -> Result<Option<PathBuf>, String>
```

**Acceptance Criteria**:
- [ ] Commands registered in `lib.rs`
- [ ] `open_project_dialog` uses `tauri-plugin-dialog`
- [ ] Recent projects persisted to `~/.ckrv/tauri-config.json`
- [ ] Project root stored in AppState

---

### Task 3.5.2: Create Project Selection Screen
**Priority**: P1
**Estimate**: 1.5h
**Files**: `crates/ckrv-ui/frontend/src/components/ProjectSelector.tsx`

Create a full-page project selection component for first launch.

**Acceptance Criteria**:
- [ ] Shows "Select a project to begin" header
- [ ] "Choose folder" button opens native dialog
- [ ] Recent projects list (clickable)
- [ ] On selection, saves project and reloads app

---

### Task 3.5.3: Add Project Selection to App Entry
**Priority**: P1
**Estimate**: 1h
**Files**: `crates/ckrv-ui/frontend/src/App.tsx` or router

Conditionally show project selector on first launch.

**Acceptance Criteria**:
- [ ] If `projectRoot === null`, show ProjectSelector
- [ ] If `projectRoot !== null`, show normal app
- [ ] Project root persisted across app restarts
- [ ] Works correctly in Tauri mode only (web mode skips this)

---

### Task 3.5.4: Display Project Path in Settings
**Priority**: P2
**Estimate**: 30m
**Files**: `crates/ckrv-ui/frontend/src/pages/SettingsPage.tsx`

Add project path row to Repository Status section.

**Acceptance Criteria**:
- [ ] Project path shown below branch
- [ ] "To switch projects, restart the app" hint (Tauri only)
- [ ] Path truncated with ellipsis if too long

---

## Phase 4: Frontend Hook

### Task 4.1: Create `useExecutionStream` Hook
**Priority**: P0
**Estimate**: 2h
**Files**: `crates/ckrv-ui/frontend/src/hooks/useExecutionStream.ts`

Create unified hook for execution streaming in both modes.

```typescript
interface UseExecutionStreamReturn {
  logs: LogEntry[];
  batches: BatchStatus[];
  status: 'idle' | 'running' | 'done' | 'error';
  runId: string | null;
  startRun: (spec: string) => Promise<string>;
  stopRun: () => Promise<void>;
  error: string | null;
}

function useExecutionStream(): UseExecutionStreamReturn
```

**Acceptance Criteria**:
- [ ] Detects Tauri vs Web mode
- [ ] Web: Connects to WebSocket `/api/execution/ws`
- [ ] Tauri: Listens to `execution:*` events
- [ ] Logs accumulated in state
- [ ] Batch status tracked
- [ ] Reconnection loads historical logs

---

### Task 4.2: Add Historical Log Loading
**Priority**: P1
**Estimate**: 1h
**Files**: `crates/ckrv-ui/frontend/src/hooks/useExecutionStream.ts`

Allow loading logs from past runs.

```typescript
// In hook:
async function loadHistoricalLogs(runId: string, since?: Date) {
  // Web: GET /api/execution/{runId}/logs
  // Tauri: invoke('get_execution_logs', { runId, since })
}
```

**Acceptance Criteria**:
- [ ] Historical logs loadable by run ID
- [ ] Reconnection uses `since` timestamp
- [ ] Works in both web and Tauri modes

---

## Phase 5: Simplify Executor UI

### Task 5.1: Refactor BarebonesExecutor to Use Hook
**Priority**: P1
**Estimate**: 1.5h
**Files**: `crates/ckrv-ui/frontend/src/components/BarebonesExecutor.tsx`

Replace WebSocket logic with the new hook.

**Current state**: 548 lines with embedded WebSocket handling
**Target state**: ~200 lines, pure UI

**Acceptance Criteria**:
- [ ] All WebSocket code removed (moved to hook)
- [ ] Uses `useExecutionStream()` hook
- [ ] Batch pills render from hook state
- [ ] Log viewer renders from hook logs
- [ ] Run/Stop buttons call hook methods
- [ ] Line count < 250

---

### Task 5.2: Update CodePage Integration
**Priority**: P2
**Estimate**: 30m
**Files**: `crates/ckrv-ui/frontend/src/components/CodePage.tsx`

Verify BarebonesExecutor still works in CodePage.

**Acceptance Criteria**:
- [ ] Execute tab renders BarebonesExecutor
- [ ] No prop changes required (hook manages all state)
- [ ] Manual test: Run an execution end-to-end

---

## Dependencies

```
Phase 1 ────────────────────────────────────────────────────►
  Task 1.1 ──► Task 1.2
                    │
Phase 2 ────────────┼───────────────────────────────────────►
                    │
  Task 2.2 ──► Task 2.1
                    │
Phase 3 ────────────┼───────────────────────────────────────►
                    │
  Task 3.1 ──► Task 3.2 ──► Task 3.3
                    │
Phase 3.5 ──────────┼───────────────────────────────────────►
                    │
  Task 3.5.1 ──► Task 3.5.2 ──► Task 3.5.3 ──► Task 3.5.4
                    │
Phase 4 ────────────┴───────────────────────────────────────►
  (depends on 2.1 and 3.2)
  Task 4.1 ──► Task 4.2
                    │
Phase 5 ────────────┼───────────────────────────────────────►
                    │
  Task 5.1 ──► Task 5.2
```

## Validation Checklist

After completing all tasks:

- [ ] `cargo check --workspace` passes
- [ ] Frontend builds: `cd frontend && npm run build`
- [ ] Web mode: Execution streams logs via WebSocket
- [ ] Tauri mode: Execution streams logs via events
- [ ] Logs persisted to `.ckrv/logs/`
- [ ] Historical logs viewable in both modes
- [ ] Tauri: Project selection works on first launch
- [ ] Settings: Project path displayed in both modes
- [ ] BarebonesExecutor < 250 lines
