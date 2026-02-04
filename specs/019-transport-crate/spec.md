# Feature Specification: Transport Crate for Dual Backend Support

**Feature Branch**: `019-transport-crate`  
**Created**: 2026-02-04  
**Status**: Draft  
**Input**: User description: "Move frontend API handling code to a separate ckrv-transport crate for dual backend support"
**Related**: [Issue #42 - Tauri Desktop App](https://github.com/FnSK4R17s/chakravarti-cli/issues/42), [Brainstorming Notes](../../brainstorming/issue-042-tauri-desktop-app/notes.md)

## Overview

Create a new `ckrv-transport` crate that consolidates all API handler logic currently in `ckrv-ui/src/api/`. This crate will serve as the single source of truth for backend API handlers, enabling:

1. **Dual backend support**: Same handler logic works for both Axum (web) and Tauri (desktop)
2. **Compile-time feature selection**: Use Rust feature flags to include only the relevant transport layer
3. **Type generation**: Generate TypeScript types from Rust using `ts-rs` for frontend type safety
4. **Single maintenance point**: Add/modify endpoints in one place instead of multiple files

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Developer Adds New API Endpoint (Priority: P1)

A developer needs to add a new API endpoint that will work in both the web UI (Axum) and the future desktop app (Tauri). Instead of implementing the handler twice, they add it once in `ckrv-transport/src/handlers/` and expose it through thin wrappers.

**Why this priority**: This is the core value proposition - reducing duplicate code and ensuring parity between backends.

**Independent Test**: Developer can add a new endpoint by modifying only `ckrv-transport` files, and it automatically becomes available in both Axum and Tauri builds.

**Acceptance Scenarios**:

1. **Given** a developer wants to add a new endpoint, **When** they implement the handler in `handlers.rs` and add exports in `axum.rs` and `tauri.rs`, **Then** the endpoint works identically in both Axum (`ckrv ui`) and Tauri builds
2. **Given** a handler function exists in `ckrv-transport`, **When** it is used by both Axum and Tauri wrappers, **Then** both return identical responses for identical inputs

---

### User Story 2 - Web UI Continues Working (Priority: P1)

Existing `ckrv ui` users experience no change in functionality. The web interface continues to work exactly as before, with all current API endpoints available.

**Why this priority**: Cannot break existing functionality during refactoring.

**Independent Test**: Run `ckrv ui` and verify all current features work (specs, execution, agents, history, etc.)

**Acceptance Scenarios**:

1. **Given** an existing `ckrv ui` installation, **When** the crate is refactored to use `ckrv-transport`, **Then** all existing API endpoints return the same responses
2. **Given** the frontend makes API calls to `/api/*` endpoints, **When** the backend uses `ckrv-transport` handlers, **Then** responses are backward-compatible

---

### User Story 3 - TypeScript Types Stay In Sync (Priority: P2)

Frontend developers have access to auto-generated TypeScript types that match the Rust types exactly. When a Rust type changes, the TypeScript types update automatically during build.

**Why this priority**: Type safety prevents runtime errors and reduces debugging time.

**Independent Test**: Modify a Rust type in `ckrv-transport/src/types.rs`, run type generation, verify TypeScript types update correspondingly.

**Acceptance Scenarios**:

1. **Given** a Rust type with `#[derive(TS)]` annotation, **When** `cargo test` or type generation script runs, **Then** corresponding TypeScript types are generated in `frontend/src/types/api.generated.ts`
2. **Given** a new field is added to a Rust struct, **When** type generation runs, **Then** the TypeScript interface includes the new field

---

### User Story 4 - Future Tauri Integration (Priority: P3)

When the Tauri desktop app is implemented (Issue #42), it can import `ckrv-transport` with the `tauri` feature flag and immediately have access to all API handlers as Tauri commands.

**Why this priority**: Enables future Tauri implementation but not blocking current functionality.

**Independent Test**: Build `ckrv-transport` with `--features tauri` and verify Tauri command exports compile correctly.

**Acceptance Scenarios**:

1. **Given** `ckrv-transport` is built with `features = ["tauri"]`, **When** the Tauri app imports it, **Then** all handlers are available as `#[tauri::command]` functions
2. **Given** a Tauri app uses `ckrv-transport`, **When** the frontend calls `invoke('list_agents')`, **Then** it returns the same data as `GET /api/agents` would in web mode

---

### Edge Cases

- **What happens when both `axum` and `tauri` features are enabled?** Both modules compile; consumer crate picks which to use.
- **How does error handling work across transports?** Common `TransportError` type converts to appropriate format (HTTP status for Axum, String/Error for Tauri).
- **What about WebSocket/SSE endpoints like terminal and events?** These require transport-specific implementations but share core logic where possible.
- **What if a handler needs different behavior per transport?** Use trait methods or conditional compilation within the handler.

## Requirements *(mandatory)*

### Functional Requirements

#### Crate Structure

- **FR-001**: System MUST create a new crate `ckrv-transport` at `crates/ckrv-transport/`
- **FR-002**: System MUST support feature flags `axum` and `tauri` for conditional compilation
- **FR-003**: Crate MUST compile with no features enabled (for testing handlers in isolation)
- **FR-004**: Crate MUST compile with `--features axum` for web deployment
- **FR-005**: Crate MUST compile with `--features tauri` for desktop deployment

#### Handler Migration

- **FR-006**: System MUST migrate all 17 API modules from `ckrv-ui/src/api/` to `ckrv-transport/src/handlers/`:
  - `agents.rs` - Agent configuration CRUD
  - `cloud.rs` - Cloud connection status
  - `commands.rs` - CLI command execution
  - `console.rs` - Interactive command console
  - `diff.rs` - Git diff viewing
  - `docker.rs` - Docker status checks
  - `events.rs` - Server-Sent Events stream
  - `execution.rs` - Batch execution control
  - `history.rs` - Run history management
  - `plans.rs` - Execution plan management
  - `qa.rs` - QA command handlers
  - `session.rs` - Docker session management
  - `specs.rs` - Specification CRUD
  - `status.rs` - System status endpoint
  - `tasks.rs` - Task management
  - `terminal.rs` - Interactive terminal WebSocket
  - `test.rs` - Test command handlers

- **FR-007**: Handler functions MUST be transport-agnostic (no Axum or Tauri types in signatures)
- **FR-008**: All existing API behavior MUST be preserved after migration

#### Transport Layer

- **FR-009**: Axum module MUST provide `create_router() -> Router<AppState>` function
- **FR-010**: Tauri module MUST provide `get_invoke_handlers()` macro invocation
- **FR-011**: Transport wrappers MUST be thin adapters that delegate to handler functions
- **FR-012**: Error types MUST convert appropriately for each transport (HTTP status for Axum, String for Tauri)

#### Type Generation

- **FR-013**: Request/Response types MUST be defined in `ckrv-transport/src/types/`
- **FR-014**: Types MUST use `#[derive(TS)]` from `ts-rs` for TypeScript generation
- **FR-015**: Generated TypeScript types MUST be placed in `crates/ckrv-ui/frontend/src/types/api.generated.ts`
- **FR-016**: Type generation MUST run as part of build or test process

#### Consumer Integration

- **FR-017**: `ckrv-ui` MUST depend on `ckrv-transport` with `features = ["axum"]`
- **FR-018**: `ckrv-ui/src/api/` MUST be removed or reduced to re-exports only
- **FR-019**: Future `ckrv-tauri` MUST be able to depend on `ckrv-transport` with `features = ["tauri"]`

### Key Entities

- **Handler**: A transport-agnostic async function that processes a request and returns a response or error
- **Transport Wrapper**: A thin adapter function that converts transport-specific input/output to handler format
- **TransportError**: A unified error type that can convert to HTTP status codes (Axum) or error strings (Tauri)
- **AppState**: Shared application state containing Orchestrator, Config, and other runtime dependencies

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: All 17 API modules successfully migrated to `ckrv-transport` with 100% test coverage preserved
- **SC-002**: `ckrv ui` functionality is identical before and after migration (zero user-visible changes)
- **SC-003**: Adding a new endpoint requires changes to only 1 crate (ckrv-transport) instead of 2+ crates
- **SC-004**: Build time for `ckrv-ui` is within 10% of pre-migration baseline
- **SC-005**: `cargo build --features axum` and `cargo build --features tauri` both succeed without errors
- **SC-006**: TypeScript types are generated from Rust sources with zero manual synchronization required
- **SC-007**: All existing integration tests pass without modification

## Assumptions

1. **No behavioral changes**: This is a pure refactoring - no new features or behavior changes to existing APIs
2. **Feature flag exclusivity at use site**: Consumer crates will enable exactly one of `axum` or `tauri`, not both
3. **TypeScript generation via ts-rs**: We use the `ts-rs` crate for TypeScript generation (alternatives like `specta` could be considered)
4. **Async runtime compatibility**: Both Axum and Tauri use Tokio, so async handlers work in both contexts
5. **WebSocket/SSE differences**: Some endpoints (terminal, events) may require transport-specific implementations alongside shared logic
6. **CLI-only for now**: This feature is a backend refactoring; CLI commands (`ckrv run`, `ckrv task`) are unaffected and continue working
