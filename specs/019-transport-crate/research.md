# Research: Transport Crate for Dual Backend Support

**Feature**: 019-transport-crate
**Date**: 2026-02-04

## Existing Pattern Analysis

### Similar Feature Analysis

**Similar Feature**: API handlers in `ckrv-ui/src/api/`
**Search Commands**:
- `grep -r "axum" crates/ckrv-ui/src/api/ --include="*.rs" -l`
- `grep -r "IntoResponse" crates/ckrv-ui/src/api/ --include="*.rs"`

**Docs Consulted**:
- `crates/docs/architecture.md` (last_commit: 34d5c95, 2026-02-03) - System overview
- `crates/RUST_CONVENTIONS.md` - Rust coding standards
- `crates/ckrv-ui/FRONTEND_CONVENTIONS.md` - Frontend conventions

**Conventions Applied**:
- `crates/RUST_CONVENTIONS.md` - Module documentation, error handling with thiserror/anyhow, import grouping

### Implementation Locations

| Crate | File/Dir | Purpose |
|-------|----------|---------|
| `ckrv-ui` | `src/api/*.rs` (17 files) | Current Axum handlers - TO BE MIGRATED |
| `ckrv-ui` | `src/state.rs` | AppState with Hub, SystemStatus - TO BE SHARED |
| `ckrv-ui` | `src/lib.rs` | Main router setup - WILL USE ckrv-transport |
| `ckrv-core` | `src/` | Business logic - UNCHANGED |
| `ckrv-sandbox` | `src/` | Docker execution - UNCHANGED |

### Current Handler Pattern

From `crates/ckrv-ui/src/api/status.rs`:

```rust
// Current pattern: Axum-specific types in function signature
use axum::{extract::State, response::IntoResponse, Json};
use crate::state::AppState;

pub async fn get_status(State(state): State<AppState>) -> impl IntoResponse {
    // Business logic mixed with transport
    let status = state.status.read().await.clone();
    Json(status)
}
```

**Problem**: Handler logic is tightly coupled to Axum types (`State`, `Json`, `IntoResponse`).

**Solution**: Split into transport-agnostic handler + transport wrapper:

```rust
// In ckrv-transport/src/handlers/status.rs
pub async fn get_status_handler(state: &AppState) -> Result<SystemStatus, TransportError> {
    Ok(state.status.read().await.clone())
}

// In ckrv-transport/src/axum/status.rs
pub async fn get_status(State(state): State<AppState>) -> impl IntoResponse {
    match handlers::get_status_handler(&state).await {
        Ok(status) => Json(status).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
```

### CLI/UI Parity Check

- **CLI path**: Not applicable - this is a backend refactoring only
- **UI path**: `ckrv-ui/src/api/*.rs` → migrates to `ckrv-transport/src/handlers/`
- **Conclusion**: No CLI changes needed; CLI commands (`ckrv run`, `ckrv task`) are unaffected

---

## Technical Decisions

### Decision 1: Crate Location

**Decision**: Create `crates/ckrv-transport/` as a new workspace crate

**Rationale**: 
- Follows existing crate naming pattern (`ckrv-*`)
- Can be imported by both `ckrv-ui` and future `ckrv-tauri`
- Keeps transport abstraction separate from UI concerns

**Alternatives Rejected**:
- Putting in `ckrv-core`: Would couple core logic to transport concerns
- Creating `ckrv-api`: Name is ambiguous; "transport" better describes the abstraction

### Decision 2: Feature Flag Strategy

**Decision**: Use `axum` and `tauri` feature flags, not enabled by default

```toml
[features]
default = []
axum = ["dep:axum", "dep:tower-http"]
tauri = ["dep:tauri"]
```

**Rationale**:
- Consumer crates explicitly opt-in to their transport
- Crate compiles without features for testing handlers in isolation
- No accidentally including both transports

**Alternatives Rejected**:
- Default to `axum`: Would require tauri builds to explicitly exclude
- Mutual exclusion: Technically possible to use both (though not recommended)

### Decision 3: Error Type Strategy

**Decision**: Use `thiserror` for `TransportError` with `From` implementations

```rust
#[derive(Debug, Error)]
pub enum TransportError {
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

// For Axum: convert to HTTP status
impl IntoResponse for TransportError { ... }

// For Tauri: convert to String
impl From<TransportError> for String { ... }
```

**Rationale**:
- Follows Rust conventions (thiserror for libraries)
- Maps cleanly to HTTP status codes
- Tauri expects String or serde-serializable errors

### Decision 4: TypeScript Generation

**Decision**: Use `ts-rs` crate for TypeScript type generation

**Rationale**:
- Well-maintained, widely used
- Integrates with `#[derive]` workflow
- Outputs clean TypeScript interfaces

**Alternatives Rejected**:
- `specta`: More complex, tied to rspc framework
- Manual types: Prone to drift, maintenance burden
- OpenAPI: Overkill for internal frontend

### Decision 5: State Management

**Decision**: Keep `AppState` in `ckrv-transport`, re-export to consumers

```rust
// ckrv-transport/src/state.rs
pub struct AppState {
    pub status: Arc<RwLock<SystemStatus>>,
    pub hub: Arc<Hub>,
    pub project_root: PathBuf,
}

// ckrv-ui can import: use ckrv_transport::AppState;
```

**Rationale**:
- State is shared between Axum and Tauri
- Avoids duplication
- Single source of truth for state shape

---

## Dependency Analysis

### Required Dependencies

| Dependency | Purpose | Feature-Gated |
|------------|---------|---------------|
| `serde` | JSON serialization | No |
| `serde_json` | JSON handling | No |
| `thiserror` | Error types | No |
| `tokio` | Async runtime | No |
| `ts-rs` | TypeScript generation | Optional |
| `axum` | HTTP framework | Yes (`axum`) |
| `tower-http` | HTTP middleware | Yes (`axum`) |
| `tauri` | Desktop framework | Yes (`tauri`) |

### Workspace Crate Dependencies

```toml
[dependencies]
ckrv-core = { workspace = true }     # Business logic
ckrv-sandbox = { workspace = true }  # Docker execution
ckrv-git = { workspace = true }      # Git operations
ckrv-metrics = { workspace = true }  # Metrics tracking
```

---

## Migration Strategy

### Phase 1: Create Crate Structure
1. Create `crates/ckrv-transport/Cargo.toml`
2. Create module structure (`handlers/`, `types/`, `axum/`)
3. Define `TransportError` and `AppState`

### Phase 2: Migrate Handlers (Ordered by Complexity)

| Priority | Module | Lines | Complexity | Notes |
|----------|--------|-------|------------|-------|
| 1 | `status.rs` | 101 | Low | Simple read-only, good test case |
| 2 | `docker.rs` | 50 | Low | Simple status check |
| 3 | `cloud.rs` | 100 | Low | Status endpoint |
| 4 | `agents.rs` | 709 | Medium | CRUD operations |
| 5 | `specs.rs` | 850 | Medium | File operations |
| 6 | `plans.rs` | 250 | Medium | CRUD operations |
| 7 | `tasks.rs` | 250 | Medium | CRUD operations |
| 8 | `history.rs` | 420 | Medium | File + state operations |
| 9 | `execution.rs` | 730 | High | Long-running jobs, state machine |
| 10 | `commands.rs` | 170 | Medium | CLI command dispatch |
| 11 | `console.rs` | 200 | Medium | Interactive console |
| 12 | `diff.rs` | 200 | Medium | Git diff viewing |
| 13 | `qa.rs` | 360 | Medium | QA command handlers |
| 14 | `test.rs` | 990 | High | Test command handlers |
| 15 | `session.rs` | 215 | Medium | Docker session |
| 16 | `terminal.rs` | 540 | High | WebSocket, PTY |
| 17 | `events.rs` | 30 | Special | SSE → Tauri events |

### Phase 3: Update ckrv-ui
1. Add `ckrv-transport = { features = ["axum"] }` dependency
2. Replace local `api/` with re-exports from transport
3. Update router to use transport's `create_router()`

### Phase 4: Add TypeScript Generation
1. Add `#[derive(TS)]` to response types
2. Configure ts-rs output path
3. Add npm script for type generation

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Breaking existing API endpoints | High | Comprehensive integration tests before migration |
| WebSocket/SSE handling differs | Medium | Keep transport-specific impl for these modules initially |
| Build time increase | Low | Feature flags keep compile times minimal |
| Type generation drift | Medium | CI check that generated types are up-to-date |

---

## Open Questions (Resolved)

All questions from spec have been resolved through research:

1. ✅ **Crate structure**: `crates/ckrv-transport/` with feature flags
2. ✅ **Error handling**: `TransportError` with transport-specific conversions
3. ✅ **TypeScript generation**: `ts-rs` crate
4. ✅ **Migration order**: Low complexity first, WebSocket/SSE last
