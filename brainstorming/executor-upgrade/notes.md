# Simplified Execution Runner

**Issue**: Internal refactor (no GitHub issue)
**Created**: 2026-02-05
**Status**: In Progress

## Problem Statement

The current execution runner infrastructure has several problems:

1. **Dead code**: Just deleted 1,569-line `ExecutionRunner.tsx` that wasn't even used
2. **Buggy WebSocket handling**: Connection drops, reconnection issues
3. **No Tauri support**: Only works in web mode (Axum WebSocket)
4. **No audit trail**: Logs disappear after execution ends
5. **Complex state management**: Hard to reason about execution status

## Current State

### What We Have
```
BarebonesExecutor.tsx (548 lines)
├── WebSocket connection to /api/execution/ws
├── Batch status pills
├── Log viewer (simple div with auto-scroll)
├── Run/Stop controls
└── Auto-spec selection via useAutoSelectedSpec hook
```

### Pain Points
| Issue | Impact |
|-------|--------|
| WebSocket-only | Can't work in Tauri desktop mode |
| Logs lost on close | No audit trail for debugging |
| No reconnection | Refresh = lost connection |
| State in component | Hard to share execution state |
| No run history | Can't see/replay past executions |

### Current Backend State

#### Axum Routes (ckrv-transport)
```
/execution/start        POST  - Start execution (works)
/execution/ws           GET   - WebSocket streaming (TODO STUB!)
/execution/stop         POST  - Stop execution (works)
/execution/status       GET   - Get status (works)
/execution/{id}/logs    GET   - Get historical logs (works)
/execution/{id}/logs/tail GET - Tail logs (works)
```

**Key Finding:** The WebSocket route in `ckrv-transport/src/axum/execution.rs` is a **TODO stub**:
```rust
async fn execution_ws(...) -> impl IntoResponse {
    ws.on_upgrade(move |_socket| async move {
        // TODO: Implement proper execution log streaming
        let _ = query.run_id;
    })
}
```

#### Execution Engine (ckrv-ui)
- **Location**: `crates/ckrv-ui/src/services/engine.rs` (~1,200 lines)
- **Responsibilities**:
  - Load/parse plan.yaml
  - Spawn Docker containers for each batch
  - Stream output via broadcast channel
  - Persist logs to JSONL files
  - Track batch status

#### Log Storage (Already Shared!)
- **Location**: `{project_root}/.ckrv/logs/{execution_id}/log.jsonl`
- **Format**: JSONL (one JSON object per line)
- **Service**: `crates/ckrv-ui/src/services/log_store.rs` (354 lines, well-tested)
- **Key APIs**:
  - `append()` - Write log entry
  - `read_all()` - Get all logs
  - `read_range()` - Pagination
  - `read_tail()` - Last N entries
  - `read_since()` - For reconnection
  - `list_executions()` - List all runs

**✅ Good news**: Logs are already in the project directory, so both Web and Tauri can read them!
- Web (Axum): Uses LogStore directly
- Tauri: Can use LogStore since it's the same project root

#### What Actually Works Today
| Feature | Web (Axum) | Tauri |
|---------|------------|-------|
| Start execution | ✅ | ❌ |
| Stop execution | ✅ | ❌ |
| WebSocket stream | ❌ (stub) | ❌ |
| HTTP log polling | ✅ | ❌ |
| Log persistence | ✅ | ❌ |

### Frontend → Backend Communication

**Current Flow (Web):**
```
Frontend                    Axum                      Engine
   │                         │                           │
   ├── POST /execution/start─┼───────────────────────────┤
   │                         │                           │
   ├── WS /execution/ws ─────┼── (stub - does nothing!) ─┤
   │                         │                           │
   └── GET /execution/{id}/logs/tail ────────────────────┤
           (polling fallback currently used)
```

### Project Root Selection (Tauri-Only Requirement)

**Problem**: Axum mode starts via `ckrv ui` which inherits the CWD as project root. Tauri is a standalone desktop app - it doesn't know which project to work with.

**Solution**: Project selection screen on first launch, then normal Dashboard

**First Launch (no project selected)**:
```
┌─────────────────────────────────────────────────────────────────┐
│                            $ ckrv                                │
│                                                                  │
│                   Select a project to begin                      │
│                                                                  │
│         ┌─────────────────────────────────────────┐             │
│         │ 📁 Choose folder...                     │             │
│         └─────────────────────────────────────────┘             │
│                                                                  │
│                     Recent Projects:                             │
│                  • my-repo                                       │
│                  • another-project                               │
│                  • old-project                                   │
└─────────────────────────────────────────────────────────────────┘
```

**After selection → Normal Dashboard** (no path clutter):
```
┌─────────────────────────────────────────────────────────────────┐
│                            $ ckrv                                │
│                                                                  │
│              What would you like to build?                       │
│    Describe your feature and AI will generate a specification   │
│                                                                  │
│    ┌───────────────────────────────────────────────────────┐    │
│    │ Describe your feature...                               │    │
│    └───────────────────────────────────────────────────────┘    │
│                                                                  │
│    [REST API] [CLI Tool] [Web App] [Refactor] [Add Feature]     │
└─────────────────────────────────────────────────────────────────┘
```

**Settings Page** (shows current project):
```
┌─────────────────────────────────────────────────────────────────┐
│  Repository Status                                    [Ready]    │
│  ─────────────────────────────────────────────────────────────  │
│  📁 Project Path    /Users/dev/my-repo                          │
│  🔀 Branch          master                                       │
│  ☑  Initialized     Yes                                         │
│                                                                  │
│  To switch projects, restart the app.                           │
└─────────────────────────────────────────────────────────────────┘
```

**Implementation**:
1. **Backend API (both modes)**:
   - Add `project_root: String` to `SystemStatus` struct in `ckrv-transport/src/state.rs`
   - Update `get_status_handler` to include `project_root` from `AppState`
   - Frontend Settings page displays it in Repository Status section
   
   ```rust
   // ckrv-transport/src/state.rs
   pub struct SystemStatus {
       pub active_branch: String,
       pub feature_number: Option<String>,
       pub is_ready: bool,
       pub mode: SystemMode,
       pub project_root: String,  // NEW: Add for Settings display
   }
   ```

2. **Tauri-specific commands**:
   - `get_project_root()` - Returns current project path (or null)
   - `set_project_root(path)` - Changes project context (requires app restart)
   - `get_recent_projects()` - List recently opened
   - `open_project_dialog()` - Native folder picker
3. **Storage (Tauri only)**: 
   - Persist to `~/.ckrv/tauri-config.json`
   - Store last 10 recent projects
4. **Project Selection Flow (Tauri only)**:
   - **App launch**: If no project saved, show "Select a project" screen
   - **App launch**: If project saved, load it and proceed to Dashboard
   - **Switching**: Require app restart (like VSCode workspace switching)

### Why No Mid-Session Switching?

**Disaster scenario**: User switches project during execution run
```
1. Execution running on Project A (Docker containers active)
2. User switches to Project B
3. UI shows Project B's specs (no execution)
4. User clicks "Stop" → Nothing happens (wrong project)
5. Project A execution continues unmonitored
6. Logs get mixed/lost
7. User confused, data corrupted
```

**Simple rule**: Project is locked for the session. To switch:
1. Stop any running executions
2. Close app
3. Reopen and select new project

This is exactly what VSCode does - you can't switch workspace mid-editing without a reload.

**Proposed Flow (both modes):**
```
Frontend                    Backend (Axum or Tauri IPC)
   │                              │
   ├── start_execution ───────────┤
   │                              │
   │◄── stream: log, batch_status, complete
   │    (WebSocket or Tauri events)
   │                              │
   └── (auto-persist to disk) ────┤
```

## Proposed Solution

### Architecture
```
┌─────────────────────────────────────────────────────────────────┐
│                      useExecutionStream Hook                     │
│  - Abstracts WebSocket (web) vs Tauri Events (desktop)          │
│  - Handles reconnection automatically                           │
│  - Maintains log buffer with persistence                        │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      ExecutorPanel Component                     │
│  - Simple UI that consumes hook                                 │
│  - Batch pills, log viewer, controls                            │
│  - No complex state - just renders what hook provides           │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Log Persistence Layer                       │
│  - Web: IndexedDB or localStorage                               │
│  - Tauri: JSONL files in ~/.ckrv/runs/                          │
│  - Queryable run history                                        │
└─────────────────────────────────────────────────────────────────┘
```

### Key Insight
The executor doesn't need PTY/interactive terminal. It just needs:
1. **Start execution** → Returns run_id
2. **Stream logs** → JSON messages with type, message, batch_id
3. **Track status** → pending/running/done/error per batch
4. **Persist logs** → Save for audit/debugging

## User Stories

### US1: Cross-Platform Execution
**As a** developer using the Tauri desktop app,
**I want** to run spec executions,
**So that** I can use the desktop app with full functionality.

### US2: Execution Audit Trail
**As a** developer debugging a failed run,
**I want** to view logs from past executions,
**So that** I can understand what went wrong.

### US3: Reliable Log Streaming
**As a** developer running a long execution,
**I want** log streaming to survive page refreshes,
**So that** I don't lose visibility into the run.

## Technical Approach

### Options Considered

| Option | Pros | Cons |
|--------|------|------|
| **A: Shared Hook** | Clean separation, testable, reusable | Need to design good API |
| B: Refactor in-place | Less work upfront | Still coupled, hard to test |
| C: Full rewrite | Clean slate | Risk of regression, more work |

### Decision

**Option A: Shared Hook** - Create `useExecutionStream` that:
- Detects environment (Tauri vs Web)
- Provides unified API for log streaming
- Handles persistence internally
- Component just renders state from hook

## Implementation Phases

### Phase 1: Create `useExecutionStream` Hook
**Files**: `src/hooks/useExecutionStream.ts`

```typescript
interface UseExecutionStreamReturn {
  logs: LogEntry[];
  batches: BatchStatus[];
  status: 'idle' | 'running' | 'done' | 'error';
  startRun: (spec: string) => Promise<string>; // returns run_id
  stopRun: () => Promise<void>;
  error: string | null;
}

function useExecutionStream(runId?: string): UseExecutionStreamReturn {
  // Detect Tauri vs Web
  // Connect appropriate transport
  // Handle reconnection
  // Persist logs
}
```

### Phase 2: Implement Axum WebSocket (Fix the Stub!)
**Files**: `crates/ckrv-transport/src/axum/execution.rs`, `crates/ckrv-transport/src/handlers/execution.rs`

The current WebSocket is a stub! Need to:
1. Subscribe to the engine's broadcast channel
2. Forward messages to WebSocket client
3. Handle client disconnect gracefully

```rust
async fn execution_ws(
    State(state): State<AppState>,
    Query(query): Query<WsQuery>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        // Get broadcast receiver from engine
        let rx = state.engine.subscribe(query.run_id);
        
        // Forward engine messages to WebSocket
        while let Ok(msg) = rx.recv().await {
            if socket.send(Message::Text(serde_json::to_string(&msg)?)).await.is_err() {
                break; // Client disconnected
            }
        }
    })
}
```

### Phase 3: Add Tauri Execution Commands
**Files**: `crates/ckrv-tauri/src/commands/execution.rs`

```rust
#[tauri::command]
async fn start_execution(spec: String, run_id: String, app: AppHandle) -> Result<...> {
    // Spawn execution in background
    // Emit events: execution:log, execution:batch_status, execution:complete
}

#[tauri::command]
async fn stop_execution(run_id: String) -> Result<...> {
    // Stop the running execution
}
```

### Phase 3.5: Project Root Selection (Tauri)
**Files**: `crates/ckrv-tauri/src/commands/project.rs`, Settings UI component

```rust
// Tauri commands
#[tauri::command]
async fn get_project_root(state: State<'_, AppState>) -> Result<Option<PathBuf>> {
    Ok(state.project_root.lock().await.clone())
}

#[tauri::command]
async fn set_project_root(path: PathBuf, state: State<'_, AppState>) -> Result<()> {
    *state.project_root.lock().await = Some(path.clone());
    // Persist to config
    save_recent_project(&path)?;
    Ok(())
}

#[tauri::command]
async fn open_project_dialog(app: AppHandle) -> Result<Option<PathBuf>> {
    use tauri_plugin_dialog::DialogExt;
    let folder = app.dialog().file().pick_folder().await?;
    Ok(folder)
}
```

**Settings UI** (`SettingsPage.tsx`):
- Folder picker using Tauri dialog
- Recent projects list
- Current project indicator in header

### Phase 4: Unified Log Access via API
**Files**: Frontend use `/api/execution/{id}/logs` endpoints

Since logs are already persisted by the backend (LogStore), the frontend doesn't need its own persistence layer. Instead:

1. **Real-time**: Subscribe to WebSocket/Tauri events during execution
2. **Historical**: Fetch via REST API after execution ends
3. **Reconnection**: Use `/execution/{id}/logs?since={timestamp}` to catch up

```typescript
// In useExecutionStream hook
async function loadHistoricalLogs(runId: string, since?: Date) {
    const url = since 
        ? `/api/execution/${runId}/logs?since=${since.toISOString()}`
        : `/api/execution/${runId}/logs`;
    const res = await fetch(url);
    return res.json();
}
```

**Both modes share the same logs** because:
- Web: Backend reads from `.ckrv/logs/`
- Tauri: Tauri commands read from same `.ckrv/logs/`

### Phase 5: Simplify ExecutorPanel
**Files**: Refactor `BarebonesExecutor.tsx` to use hook

- Remove WebSocket logic (moved to hook)
- Remove complex state management
- Keep UI: batch pills, log viewer, controls
- Target: ~200 lines

### Phase 6: Run History Panel (Optional)
**Files**: `src/components/RunHistoryPanel.tsx`

- Show past runs with timestamps
- Click to view logs
- Filter by spec, status, date

## Open Questions

- [x] Do we need PTY for executor? **No, just log streaming**
- [ ] How much run history to keep? (7 days? 50 runs?)
- [ ] Should logs be searchable?
- [ ] Real-time log filtering by batch?

## Success Criteria

| Metric | Target |
|--------|--------|
| BarebonesExecutor lines | < 250 (from 548) |
| Axum WebSocket implemented | ✅ (currently stub) |
| Works in Tauri | ✅ |
| Works in Web | ✅ |
| Logs persisted (shared between modes) | ✅ |
| Can reconnect | ✅ |
| Tauri project selection | ✅ |

## Next Steps

- [ ] Phase 1: Create `useExecutionStream` hook skeleton
- [ ] Phase 2: Fix Axum WebSocket stub - implement real streaming
- [ ] Phase 3: Add Tauri execution commands with event emitting
- [ ] Phase 3.5: Add Tauri project root selection (Settings page)
- [ ] Phase 4: Add log access to hook (uses backend LogStore via API)
- [ ] Phase 5: Refactor BarebonesExecutor to use hook
- [ ] Test in both web and Tauri modes
- [ ] (Optional) Phase 6: Add run history panel

## References

- `crates/ckrv-ui/frontend/src/components/BarebonesExecutor.tsx` - Current frontend
- `crates/ckrv-transport/src/axum/execution.rs` - Axum routes (WebSocket stub here!)
- `crates/ckrv-transport/src/handlers/execution.rs` - Execution handlers
- `crates/ckrv-ui/src/services/engine.rs` - Execution engine with broadcast
- `crates/ckrv-tauri/src/commands/terminal.rs` - Pattern for Tauri commands
- `.agent/skills/tauri-pty-terminal/SKILL.md` - PTY skill (for reference, different use case)
- Tauri Event System: https://tauri.app/develop/calling-frontend/
