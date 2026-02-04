# Implementation Plan: Transport Crate for Dual Backend Support

**Branch**: `019-transport-crate` | **Date**: 2026-02-04 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/019-transport-crate/spec.md`

## Summary

Create a new `ckrv-transport` crate that consolidates all 17 API handler modules from `ckrv-ui/src/api/` into a single source of truth. The crate uses Rust feature flags (`axum`, `tauri`) to conditionally compile transport-specific wrappers around shared handler logic. This enables the same handler code to serve both the web UI (Axum) and future desktop app (Tauri) without duplication.

## Technical Context

**Language/Version**: Rust 1.75+  
**Primary Dependencies**: axum 0.8, tauri 2.0, ts-rs 7.x, thiserror 1.x  
**Storage**: File-based (YAML configs, spec files) - unchanged  
**Testing**: cargo test (unit), integration tests with tempfile  
**Target Platform**: Linux, macOS, Windows (via consumer crates)  
**Project Type**: Rust workspace crate (library)  
**Performance Goals**: No regression from current Axum performance  
**Constraints**: Zero breaking changes to existing API responses  
**Scale/Scope**: 17 API modules, ~4500 lines of handler code to migrate

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-checked after Phase 1 design.*

| Principle | Requirement | Status |
|-----------|-------------|--------|
| I. Code Quality Excellence | Full typing, zero lint errors, single responsibility | ✅ All handlers have clear single responsibility; Rust ensures full typing |
| II. Testing Standards | TDD approach planned, coverage targets defined | ✅ Migrate tests alongside handlers; maintain 80%+ coverage |
| III. Reliability First | Error handling strategy, idempotency considered | ✅ TransportError type with explicit error cases |
| IV. Security by Default | No hardcoded secrets, input validation planned | ✅ No secrets in transport layer; validation in handlers |
| V. Deterministic CLI Behavior | Machine-readable output, explicit exit codes | N/A - This is backend-only; CLI unchanged |

## Project Structure

### Documentation (this feature)

```text
specs/019-transport-crate/
├── spec.md              # Feature specification
├── plan.md              # This file
├── research.md          # Existing pattern analysis
├── data-model.md        # Type definitions
├── quickstart.md        # Developer guide
├── contracts/           # API contracts
│   └── transport-api.md # Handler signatures
├── checklists/
│   └── requirements.md  # Spec quality checklist
└── tasks.md             # Implementation tasks (created by /speckit.tasks)
```

### Source Code (repository root)

```text
crates/
├── ckrv-transport/              # [CREATE] NEW CRATE
│   ├── Cargo.toml               # [CREATE] With feature flags
│   ├── src/
│   │   ├── lib.rs               # [CREATE] Module exports, feature gates
│   │   ├── error.rs             # [CREATE] TransportError type
│   │   ├── state.rs             # [CREATE] AppState (migrated from ckrv-ui)
│   │   ├── types/               # [CREATE] Request/Response types
│   │   │   ├── mod.rs
│   │   │   ├── agents.rs
│   │   │   ├── specs.rs
│   │   │   ├── execution.rs
│   │   │   └── ... (one per domain)
│   │   ├── handlers/            # [CREATE] Transport-agnostic handlers
│   │   │   ├── mod.rs
│   │   │   ├── agents.rs
│   │   │   ├── cloud.rs
│   │   │   ├── commands.rs
│   │   │   ├── console.rs
│   │   │   ├── diff.rs
│   │   │   ├── docker.rs
│   │   │   ├── events.rs
│   │   │   ├── execution.rs
│   │   │   ├── history.rs
│   │   │   ├── plans.rs
│   │   │   ├── qa.rs
│   │   │   ├── session.rs
│   │   │   ├── specs.rs
│   │   │   ├── status.rs
│   │   │   ├── tasks.rs
│   │   │   ├── terminal.rs
│   │   │   └── test.rs
│   │   ├── axum/                # [CREATE] Axum wrappers (feature = "axum")
│   │   │   ├── mod.rs           # create_router() function
│   │   │   └── ...              # Thin wrappers calling handlers
│   │   └── tauri/               # [CREATE] Tauri wrappers (feature = "tauri")
│   │       ├── mod.rs           # get_invoke_handlers() function
│   │       └── ...              # #[tauri::command] exports
│   └── tests/
│       └── handler_tests.rs     # [CREATE] Unit tests for handlers
│
├── ckrv-ui/
│   ├── Cargo.toml               # [MODIFY] Add ckrv-transport dependency
│   ├── src/
│   │   ├── lib.rs               # [MODIFY] Use transport's create_router()
│   │   ├── api/                 # [MODIFY] Reduce to re-exports or remove
│   │   │   └── mod.rs           # Re-export from ckrv-transport
│   │   └── state.rs             # [MODIFY] Remove (moved to transport)
│   └── frontend/
│       └── src/
│           └── types/
│               └── api.generated.ts  # [CREATE] ts-rs generated types
│
└── Cargo.toml                   # [MODIFY] Add ckrv-transport to workspace
```

**Structure Decision**: New library crate with feature-gated transport implementations. Consumer crates (ckrv-ui, ckrv-tauri) import with appropriate feature flag.

## Affected Files Summary

### New Files (CREATE)

| File | Purpose |
|------|---------|
| `crates/ckrv-transport/Cargo.toml` | Crate manifest with feature flags |
| `crates/ckrv-transport/src/lib.rs` | Module exports and feature gates |
| `crates/ckrv-transport/src/error.rs` | TransportError enum |
| `crates/ckrv-transport/src/state.rs` | AppState struct |
| `crates/ckrv-transport/src/types/*.rs` | Request/Response types |
| `crates/ckrv-transport/src/handlers/*.rs` | Transport-agnostic handlers |
| `crates/ckrv-transport/src/axum/*.rs` | Axum route wrappers |
| `crates/ckrv-transport/src/tauri/*.rs` | Tauri command wrappers (stub) |
| `crates/ckrv-ui/frontend/src/types/api.generated.ts` | Generated TypeScript types |

### Modified Files (MODIFY)

| File | Change |
|------|--------|
| `Cargo.toml` (root) | Add ckrv-transport to workspace members |
| `crates/ckrv-ui/Cargo.toml` | Add ckrv-transport dependency |
| `crates/ckrv-ui/src/lib.rs` | Import router from ckrv-transport |
| `crates/ckrv-ui/src/api/mod.rs` | Re-export or remove |

### Deleted Files (DELETE) - Optional

| File | Reason |
|------|--------|
| `crates/ckrv-ui/src/api/*.rs` (17 files) | [OPTIONAL] Can keep as re-exports initially |
| `crates/ckrv-ui/src/state.rs` | [OPTIONAL] Can keep as re-export initially |

**Why optional**: Keeping files as re-exports maintains backward compatibility for any internal imports. Can be fully removed in follow-up cleanup.

## Complexity Tracking

> No Constitution violations. All principles are satisfied by design.

| Decision | Justification |
|----------|---------------|
| New crate instead of modifying ckrv-ui | Separation of concerns; ckrv-ui is web-specific, transport is shared |
| Feature flags over runtime detection | Compile-time guarantees; no dead code in final binary |
| ts-rs over manual types | Eliminates drift between Rust and TypeScript types |

## Implementation Phases

### Phase 1: Crate Foundation (Tasks 1-5)
- Create crate structure and Cargo.toml
- Define TransportError and AppState
- Set up feature flags
- Add to workspace

### Phase 2: Type Definitions (Tasks 6-10)
- Migrate request/response types from ckrv-ui
- Add ts-rs derive macros
- Generate TypeScript types

### Phase 3: Handler Migration (Tasks 11-27)
- Migrate handlers in complexity order (see research.md)
- Start with `status.rs`, `docker.rs`, `cloud.rs`
- End with `terminal.rs`, `events.rs`

### Phase 4: Axum Integration (Tasks 28-32)
- Create Axum wrappers
- Build `create_router()` function
- Update ckrv-ui to use transport

### Phase 5: Testing & Cleanup (Tasks 33-38)
- Migrate and update tests
- Verify all endpoints work
- Clean up old api/ files

### Phase 6: TypeScript Generation (Tasks 39-42)
- Configure ts-rs output
- Add npm script for generation
- Update frontend to use generated types
