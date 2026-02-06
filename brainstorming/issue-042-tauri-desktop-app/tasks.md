# Create Tauri Desktop App - Tasks

**Issue**: [#42](https://github.com/FnSK4R17s/chakravarti-cli/issues/42)
**Brainstorm**: [notes.md](./notes.md)
**Created**: 2026-02-05

## Task Overview

| Phase | Description | Tasks | Estimate |
|-------|-------------|-------|----------|
| Phase 1 | Transport Crate Foundation | 4 | 3h |
| Phase 2 | Frontend Restructuring | 4 | 2h |
| Phase 3 | Tauri Crate Scaffolding | 5 | 3h |
| Phase 4 | Core Commands | 6 | 6h |
| Phase 5 | Full Feature Commands | 5 | 8h |
| Phase 6 | Real-time Features | 4 | 8h |
| Phase 7 | Frontend Integration | 4 | 4h |
| Phase 8 | Polish & CI/CD | 5 | 6h |
| **Total** | | **37** | **40h** |

---

## Phase 1: Transport Crate Foundation

> **Status**: ✅ Complete (ckrv-transport already exists)
> 
> The `ckrv-transport` crate has been created and migrated. Tauri commands should use the shared handlers from this crate.

### Task 1.1: Verify Transport Crate Tauri Feature
**Priority**: P0
**Estimate**: 30m
**Files**: `crates/ckrv-transport/Cargo.toml`, `crates/ckrv-transport/src/tauri/mod.rs`

Verify the `tauri` feature flag is properly configured and the Tauri wrapper modules are ready for use.

**Acceptance Criteria**:
- [ ] `cargo check -p ckrv-transport --features tauri` compiles
- [ ] `src/tauri/mod.rs` exports `get_invoke_handlers()` function
- [ ] All handlers in `src/handlers/` can be called from Tauri wrappers

---

### Task 1.2: Complete Tauri Wrapper Coverage
**Priority**: P0
**Estimate**: 2h
**Files**: `crates/ckrv-transport/src/tauri/*.rs`

Ensure all handlers have corresponding Tauri command wrappers matching the Axum routes.

**Acceptance Criteria**:
- [ ] Tauri commands exist for all 17 API modules
- [ ] Each Tauri command calls shared handler from `handlers/*.rs`
- [ ] Error types properly convert to Tauri-compatible strings
- [ ] `get_invoke_handlers()` returns all commands

---

### Task 1.3: TypeScript Type Generation
**Priority**: P1
**Estimate**: 30m
**Files**: `crates/ckrv-transport/src/types/*.rs`, `crates/ckrv-frontend/src/types/api.generated.ts`

Verify ts-rs generates TypeScript types for all request/response types.

**Acceptance Criteria**:
- [ ] `cargo test -p ckrv-transport --features typescript export_typescript_types -- --ignored` generates types
- [ ] Generated types include all agent, spec, execution, and status types
- [ ] No compilation errors in frontend after type regeneration

---

## Phase 2: Frontend Restructuring

> **Note**: Brainstorm proposes moving `crates/ckrv-ui/frontend/` → `crates/ckrv-frontend/`.
> This enables both Tauri and Axum to consume the same frontend.

### Task 2.1: Create Standalone Frontend Crate
**Priority**: P0
**Estimate**: 30m
**Files**: `crates/ckrv-frontend/` (new directory)

Move the frontend to a standalone location accessible by both ckrv-ui and ckrv-tauri.

**Acceptance Criteria**:
- [ ] `crates/ckrv-frontend/` contains all React code from `ckrv-ui/frontend/`
- [ ] `npm install && npm run build` works in new location
- [ ] Old `crates/ckrv-ui/frontend/` removed (or symlinked temporarily)

---

### Task 2.2: Update ckrv-ui rust-embed Path
**Priority**: P0
**Estimate**: 15m
**Files**: `crates/ckrv-ui/src/server.rs`

Update the rust-embed macro to point to the new frontend location.

**Acceptance Criteria**:
- [ ] `#[folder = "../ckrv-frontend/dist"]` in server.rs
- [ ] `cargo build -p ckrv-ui` succeeds
- [ ] `ckrv ui` serves frontend correctly

---

### Task 2.3: Update Type Generation Path
**Priority**: P1
**Estimate**: 15m
**Files**: `crates/ckrv-transport/tests/export_types.rs`

Update ts-rs export path to generate types in new frontend location.

**Acceptance Criteria**:
- [ ] Types export to `crates/ckrv-frontend/src/types/api.generated.ts`
- [ ] Regenerating types updates correct file

---

### Task 2.4: Update Documentation
**Priority**: P2
**Estimate**: 30m
**Files**: `README.md`, `CONTRIBUTING.md`, `crates/docs/getting-started.md`

Update all documentation to reflect new frontend location.

**Acceptance Criteria**:
- [ ] README shows correct path for frontend development
- [ ] CONTRIBUTING.md updated with new structure
- [ ] Architecture diagrams show ckrv-frontend as separate

---

## Phase 3: Tauri Crate Scaffolding

### Task 3.1: Create ckrv-tauri Crate Directory
**Priority**: P0
**Estimate**: 30m
**Files**: `crates/ckrv-tauri/` (new)

Scaffold the Tauri crate with proper directory structure.

**Acceptance Criteria**:
- [ ] Directory structure matches spec: `src/`, `icons/`, `capabilities/`
- [ ] Cargo.toml with Tauri v2 and plugin dependencies
- [ ] build.rs calls `tauri_build::build()`

---

### Task 3.2: Configure tauri.conf.json
**Priority**: P0
**Estimate**: 30m
**Files**: `crates/ckrv-tauri/tauri.conf.json`

Create Tauri configuration with correct frontend paths and bundle settings.

**Acceptance Criteria**:
- [ ] `frontendDist` points to `../../ckrv-frontend/dist`
- [ ] `devUrl` set to `http://localhost:5173`
- [ ] `beforeDevCommand` and `beforeBuildCommand` run npm in ckrv-frontend
- [ ] Bundle settings for all platforms (macOS, Windows, Linux)

---

### Task 3.3: Add Capability Configuration
**Priority**: P1
**Estimate**: 15m
**Files**: `crates/ckrv-tauri/capabilities/default.json`

Configure Tauri security capabilities for plugins.

**Acceptance Criteria**:
- [ ] Shell plugin permissions for opening URLs
- [ ] Dialog plugin permissions for file/folder pickers
- [ ] FS plugin permissions for reading/writing specs

---

### Task 3.4: Create Main Entry Point
**Priority**: P0
**Estimate**: 1h
**Files**: `crates/ckrv-tauri/src/main.rs`, `crates/ckrv-tauri/src/state.rs`

Create Tauri main.rs with plugin initialization and command registration.

**Acceptance Criteria**:
- [ ] `cargo tauri dev` opens empty window with frontend
- [ ] Tauri plugins initialized (shell, dialog, fs)
- [ ] AppState managed with ckrv-core orchestrator
- [ ] DevTools open in debug builds

---

### Task 3.5: Add Placeholder Icons
**Priority**: P2
**Estimate**: 30m
**Files**: `crates/ckrv-tauri/icons/`

Add placeholder icons for all platforms (will be refined in Phase 8).

**Acceptance Criteria**:
- [ ] `icon.ico` for Windows
- [ ] `icon.icns` for macOS
- [ ] `icon.png` 512x512 for Linux
- [ ] Retina variants (128x128@2x.png)

---

## Phase 4: Core Commands

### Task 4.1: Implement Spec Commands
**Priority**: P0
**Estimate**: 1h
**Files**: `crates/ckrv-transport/src/tauri/specs.rs`

Wire Tauri commands for spec management using shared handlers.

**Commands**:
- `list_specs`, `get_spec`, `create_spec`, `update_spec`, `delete_spec`
- `generate_tasks`, `get_tasks`

**Acceptance Criteria**:
- [ ] All spec commands callable from frontend
- [ ] Commands use handlers from ckrv-transport
- [ ] Error messages properly propagated

---

### Task 4.2: Implement Execution Commands
**Priority**: P0
**Estimate**: 1.5h
**Files**: `crates/ckrv-transport/src/tauri/execution.rs`

Wire execution commands for running plans and tracking status.

**Commands**:
- `start_execution`, `stop_execution`, `get_execution_status`
- `get_batch_status`, `get_task_output`, `retry_task`

**Acceptance Criteria**:
- [ ] Can start execution from Tauri frontend
- [ ] Status updates received (via Tauri events)
- [ ] Stop/cancel works correctly

---

### Task 4.3: Implement Agent Commands
**Priority**: P0
**Estimate**: 1h
**Files**: `crates/ckrv-transport/src/tauri/agents.rs`

Wire agent management commands (already partially implemented).

**Commands**:
- `list_agents`, `upsert_agent`, `delete_agent`
- `set_default_agent`, `set_qa_agent`, `set_test_writer_agent`
- `test_agent`, `get_openrouter_models`

**Acceptance Criteria**:
- [ ] All 8 agent commands work
- [ ] Agent config persists correctly
- [ ] OpenRouter model fetch works

---

### Task 4.4: Implement Status Commands
**Priority**: P0
**Estimate**: 30m
**Files**: `crates/ckrv-transport/src/tauri/status.rs`, `crates/ckrv-transport/src/tauri/docker.rs`

Wire system and Docker status commands.

**Commands**:
- `get_system_status`, `get_docker_status`

**Acceptance Criteria**:
- [ ] Docker connection status detected
- [ ] System resource info available

---

### Task 4.5: Implement Plan Commands
**Priority**: P1
**Estimate**: 1h
**Files**: `crates/ckrv-transport/src/tauri/plans.rs`

Wire plan management commands.

**Commands**:
- `list_plans`, `get_plan`, `create_plan`, `regenerate_plan`

**Acceptance Criteria**:
- [ ] Plans load and save correctly
- [ ] Plan generation uses AI (via ckrv-core)

---

### Task 4.6: Implement History Commands
**Priority**: P1
**Estimate**: 1h
**Files**: `crates/ckrv-transport/src/tauri/history.rs`

Wire run history commands.

**Commands**:
- `list_runs`, `get_run`, `delete_run`, `export_run`

**Acceptance Criteria**:
- [ ] Run history displays in UI
- [ ] Can delete old runs
- [ ] Run details show task outputs

---

## Phase 5: Full Feature Commands

### Task 5.1: Implement Diff Commands
**Priority**: P1
**Estimate**: 1.5h
**Files**: `crates/ckrv-transport/src/tauri/diff.rs`

Wire Git diff viewing commands.

**Commands**:
- `get_diff_summary`, `get_file_diff`, `get_branch_diff`

**Acceptance Criteria**:
- [ ] Diff view shows changes correctly
- [ ] File-by-file navigation works
- [ ] Syntax highlighting preserved

---

### Task 5.2: Implement QA Commands
**Priority**: P1
**Estimate**: 1.5h
**Files**: `crates/ckrv-transport/src/tauri/qa.rs`

Wire QA review and bug analysis commands.

**Commands**:
- `run_qa_review`, `get_qa_report`, `run_bug_analysis`

**Acceptance Criteria**:
- [ ] QA agent can be invoked
- [ ] Reports display in UI
- [ ] Issues logged with severity

---

### Task 5.3: Implement Test Commands
**Priority**: P1
**Estimate**: 2h
**Files**: `crates/ckrv-transport/src/tauri/test.rs`

Wire test running and writing commands.

**Commands**:
- `run_tests`, `get_test_results`, `analyze_coverage`
- `write_tests`, `get_test_writer_status`

**Acceptance Criteria**:
- [ ] Tests run in sandbox
- [ ] Results displayed with pass/fail
- [ ] Test writer agent can generate tests

---

### Task 5.4: Implement Console Commands
**Priority**: P1
**Estimate**: 1.5h
**Files**: `crates/ckrv-transport/src/tauri/console.rs`, `crates/ckrv-transport/src/tauri/commands.rs`

Wire CLI command execution and console interaction.

**Commands**:
- `run_command`, `get_command_output`, `list_available_commands`

**Acceptance Criteria**:
- [ ] CLI commands can be run from UI
- [ ] Output streams to frontend
- [ ] Allow-listed commands only

---

### Task 5.5: Implement Cloud Commands
**Priority**: P2
**Estimate**: 1.5h
**Files**: `crates/ckrv-transport/src/tauri/cloud.rs`

Wire cloud connection and sync commands.

**Commands**:
- `get_cloud_status`, `connect_cloud`, `sync_to_cloud`

**Acceptance Criteria**:
- [ ] Cloud status visible in UI
- [ ] Connection flow works
- [ ] Sync uploads correctly

---

## Phase 6: Real-time Features

### Task 6.1: Implement Tauri Event System
**Priority**: P0
**Estimate**: 2h
**Files**: `crates/ckrv-transport/src/tauri/events.rs`

Replace SSE with Tauri's native event system for real-time updates.

**Events to emit**:
- `execution:started`, `execution:progress`, `execution:completed`
- `task:started`, `task:completed`, `task:failed`
- `agent:status`

**Acceptance Criteria**:
- [ ] Events emit during execution
- [ ] Frontend receives and displays updates
- [ ] No polling required for status

---

### Task 6.2: Create Frontend Event Listener
**Priority**: P0
**Estimate**: 1h
**Files**: `crates/ckrv-frontend/src/lib/transport/tauri.ts`

Create Tauri event listener layer in frontend to replace SSE EventSource.

**Acceptance Criteria**:
- [ ] `listen()` calls from @tauri-apps/api/event
- [ ] Same event interface as HTTP SSE
- [ ] Cleanup on component unmount

---

### Task 6.3: Implement Terminal Support
**Priority**: P1
**Estimate**: 3h
**Files**: `crates/ckrv-transport/src/tauri/terminal.rs`, `crates/ckrv-frontend/src/components/Terminal.tsx`

Implement PTY/terminal support for interactive agent sessions.

**Approach**: Use `tauri-plugin-shell` or custom PTY bridge

**Acceptance Criteria**:
- [ ] xterm.js renders in Tauri WebView
- [ ] Input/output flows to Docker container PTY
- [ ] Terminal themes work (light/dark)
- [ ] Special characters render correctly

---

### Task 6.4: Add Session Management
**Priority**: P1
**Estimate**: 2h
**Files**: `crates/ckrv-transport/src/tauri/session.rs`

Wire Docker session management for agent terminals.

**Commands**:
- `create_session`, `attach_session`, `detach_session`, `kill_session`

**Acceptance Criteria**:
- [ ] Sessions persist across navigation
- [ ] Can have multiple terminal tabs
- [ ] Sessions properly cleaned up on close

---

## Phase 7: Frontend Integration

### Task 7.1: Create Unified API Layer
**Priority**: P0
**Estimate**: 1.5h
**Files**: `crates/ckrv-frontend/src/lib/api.ts`

Create the unified API layer that auto-switches between HTTP and Tauri IPC.

**Acceptance Criteria**:
- [ ] `IS_TAURI` detection works
- [ ] All API functions have both HTTP and Tauri paths
- [ ] Type-safe with generated types
- [ ] No component changes needed

---

### Task 7.2: Add Transport Layer Abstraction
**Priority**: P1
**Estimate**: 1h
**Files**: `crates/ckrv-frontend/src/lib/transport/`

Create transport layer with http.ts, tauri.ts, and mock.ts implementations.

**Acceptance Criteria**:
- [ ] Transport interface with get/post/delete/stream
- [ ] HTTP transport uses fetch
- [ ] Tauri transport uses invoke
- [ ] Mock transport for testing

---

### Task 7.3: Add Tauri NPM Dependencies
**Priority**: P0
**Estimate**: 15m
**Files**: `crates/ckrv-frontend/package.json`

Add @tauri-apps dependencies for API and plugins.

**Dependencies**:
- `@tauri-apps/api@^2.0.0`
- `@tauri-apps/plugin-dialog@^2.0.0`
- `@tauri-apps/plugin-fs@^2.0.0`
- `@tauri-apps/plugin-shell@^2.0.0`

**Acceptance Criteria**:
- [ ] `npm install` succeeds
- [ ] Imports work in api.ts
- [ ] No bundler errors

---

### Task 7.4: Implement Native File Dialogs
**Priority**: P1
**Estimate**: 1h
**Files**: `crates/ckrv-frontend/src/lib/api.ts`, various components

Use Tauri's native file dialog instead of HTML5 file picker where applicable.

**Acceptance Criteria**:
- [ ] `openFileDialog()` uses native picker in Tauri mode
- [ ] Falls back to null in web mode
- [ ] Project directory selection works

---

## Phase 8: Polish & CI/CD

### Task 8.1: Design Production Icons
**Priority**: P2
**Estimate**: 1h
**Files**: `crates/ckrv-tauri/icons/`

Create proper branded icons for all platforms.

**Acceptance Criteria**:
- [ ] Icons match Chakravarti branding
- [ ] All required sizes present
- [ ] Icons display correctly on all platforms

---

### Task 8.2: Configure Bundle Metadata
**Priority**: P2
**Estimate**: 30m
**Files**: `crates/ckrv-tauri/tauri.conf.json`

Finalize bundle metadata for release.

**Acceptance Criteria**:
- [ ] Version matches Cargo.toml
- [ ] Description and category set
- [ ] Windows certificate placeholder ready
- [ ] macOS minimum version correct

---

### Task 8.3: Set Up GitHub Actions for Desktop Releases
**Priority**: P1
**Estimate**: 2h
**Files**: `.github/workflows/release-desktop.yml`

Create CI/CD pipeline for building and releasing desktop apps.

**Acceptance Criteria**:
- [ ] Builds on push to tag `v*`
- [ ] Matrix builds for Linux, macOS, Windows
- [ ] ARM64 support for Apple Silicon
- [ ] Artifacts uploaded to release

---

### Task 8.4: Add API Parity Check to CI
**Priority**: P1
**Estimate**: 1h
**Files**: `.github/workflows/ci.yml`, `scripts/check-api-parity.sh`

Add script to CI that verifies Axum and Tauri have same number of endpoints.

**Acceptance Criteria**:
- [ ] Script counts Axum handlers and Tauri commands
- [ ] CI fails if counts differ
- [ ] Warning output shows which are missing

---

### Task 8.5: Update Documentation
**Priority**: P2
**Estimate**: 1.5h
**Files**: `README.md`, `AGENTS.md`, `crates/docs/`

Add desktop app documentation and distribution instructions.

**Acceptance Criteria**:
- [ ] README has desktop installation section
- [ ] AGENTS.md documents Tauri conventions
- [ ] API parity checklist document created
- [ ] Getting started includes Tauri dev workflow

---

## Dependencies

```
Phase 1 ─────────────────────────────────────────────────────────────►
  Task 1.1 ──► Task 1.2 ──► Task 1.3
                   │
Phase 2 ───────────┼─────────────────────────────────────────────────►
                   │
  Task 2.1 ────────┴──► Task 2.2 ──► Task 2.3 ──► Task 2.4
       │
       │
Phase 3 ▼────────────────────────────────────────────────────────────►
       │
  Task 3.1 ──► Task 3.2 ──► Task 3.4
                   │           │
                   ▼           │
              Task 3.3         │
                               │
Phase 4 ───────────────────────┼─────────────────────────────────────►
                               │
  Task 4.1 ◄───────────────────┘
  Task 4.2 ──────────────────────► (depends on 4.1)
  Task 4.3
  Task 4.4
  Task 4.5
  Task 4.6

Phase 5 ─────────────────────────────────────────────────────────────►
  Task 5.1 through 5.5 (can parallelize)

Phase 6 ─────────────────────────────────────────────────────────────►
  Task 6.1 ──► Task 6.2
  Task 6.3 ──► Task 6.4

Phase 7 ─────────────────────────────────────────────────────────────►
  Task 7.1 ──► Task 7.2
  Task 7.3
  Task 7.4 (depends on 7.3)

Phase 8 ─────────────────────────────────────────────────────────────►
  Task 8.1 through 8.5 (can parallelize after Phase 7)
```

## Blockers

- [ ] ckrv-transport crate must be stable before Phase 4+
- [ ] Frontend restructure (Phase 2) blocks Tauri dev workflow
- [ ] Tauri v2 requires Rust 1.70+ (verify toolchain)

## Success Criteria (from Brainstorm)

| Metric | Target |
|--------|--------|
| Installer size (macOS) | < 20MB |
| Bundle size (Windows) | < 25MB |
| Cold start time | < 2 seconds |
| Memory usage (idle) | < 50MB |
| All existing UI features | Functional parity |
| Platform coverage | macOS, Windows, Linux |

## Notes

### Transport Crate Already Complete
Phase 1 is marked as mostly complete because `ckrv-transport` has been created and migrated. The remaining work is verifying Tauri feature coverage and completing any missing Tauri wrappers.

### Frontend Restructure Decision
The brainstorm proposes moving the frontend to `crates/ckrv-frontend/`. This is recommended but not mandatory for v1—Tauri can use relative paths to `ckrv-ui/frontend/` with appropriate `beforeBuildCommand` configuration.

### Tauri vs ckrv-ui Relationship
Tauri and `ckrv ui` (Axum) are **mutually exclusive at runtime**. They never run simultaneously. The parity concern is about keeping both backends in sync during development.

### Real-time Features Complexity
Phase 6 (real-time features) is the most complex. Terminal support in particular requires careful testing across platforms. Consider shipping without terminal in v1 if blocked.
