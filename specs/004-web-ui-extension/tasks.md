---
description: "Task list template for feature implementation"
---

# Tasks: web-ui-extension

**Input**: Design documents from `/specs/004-web-ui-extension/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: TDD is mandated by the Constitution. Tests are included.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2)
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [x] T001 Create `crates/ckrv-ui` library with `cargo new --lib`
- [x] T002 Initialize `crates/ckrv-ui/frontend` with `npm create vite@latest` (React/TS)
- [x] T003 [P] Add backend dependencies (`axum`, `tokio`, `rust-embed`) to `crates/ckrv-ui/Cargo.toml`
- [x] T004 [P] Add frontend dependencies (`radix-ui`, `lucide-react`, `tanstack-query`, `zod`) to `crates/ckrv-ui/frontend/package.json`
- [x] T005 Configure `crates/ckrv-ui/build.rs` to build frontend assets
- [x] T006 Register `ckrv-ui` in root `Cargo.toml` workspace members

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T007 Create `AppState` struct in `crates/ckrv-ui/src/state.rs` (holds `SystemStatus`)
- [x] T008 [P] Implement `ckrv-ui/src/hub.rs` (SSE event broadcaster)
- [x] T009 [P] Create `ckrv-ui/src/server.rs` with basic Axum router setup
- [x] T010 Implement `ui` command in `crates/ckrv-cli/src/commands/ui.rs` to start the server

**Checkpoint**: Foundation ready - `ckrv ui` starts a server (even if empty).

---

## Phase 3: User Story 1 - Visual Dashboard (Priority: P1) 🎯 MVP

**Goal**: View system status and logs.

**Independent Test**: Open `http://localhost:3000` and see "Online" status and stream of logs.

### Tests for User Story 1
- [x] T011 [P] [US1] Create unit test for status endpoint in `crates/ckrv-ui/src/api/status.rs`
- [ ] T012 [P] [US1] Create frontend test for StatusBadge component

### Implementation for User Story 1
- [x] T013 [P] [US1] Implement `/api/status` endpoint in `crates/ckrv-ui/src/api/status.rs`
- [x] T014 [P] [US1] Implement `/api/events` SSE endpoint in `crates/ckrv-ui/src/api/events.rs`
- [x] T015 [US1] Create `DashboardLayout` in `crates/ckrv-ui/frontend/src/layouts/Dashboard.tsx`
- [x] T016 [US1] Implement `StatusWidget` component in `crates/ckrv-ui/frontend/src/components/StatusWidget.tsx`
- [x] T017 [US1] Implement `LogViewer` component with virtual scrolling in `crates/ckrv-ui/frontend/src/components/LogViewer.tsx`

**Checkpoint**: User Story 1 fully functional.

---

## Phase 4: User Story 2 - Full Command Interface (Priority: P2)

**Goal**: Trigger CLI commands from UI.

**Independent Test**: Click "Init" or "Spec" in UI and see CLI action happen.

### Tests for User Story 2
- [x] T018 [P] [US2] Create unit test for command dispatcher in `crates/ckrv-ui/src/services/command.rs`

### Implementation for User Story 2
- [x] T019 [P] [US2] Implement `CommandService` in `crates/ckrv-ui/src/services/command.rs` (wraps `ckrv-core`)
- [x] T020 [P] [US2] Create API endpoints for commands in `crates/ckrv-ui/src/api/commands.rs`
- [x] T021 [US2] Create `CommandPalette` component in `crates/ckrv-ui/frontend/src/components/CommandPalette.tsx`
- [ ] T022 [US2] Create `SpecForm` component in `crates/ckrv-ui/frontend/src/features/SpecForm.tsx` (for `spec new`)
- [x] T023 [US2] Wire up API mutations with `TanStack Query` hooks in `crates/ckrv-ui/frontend/src/hooks/useCommand.ts`

**Checkpoint**: User Story 2 complete.

---

## Phase 5: Polish & Cross-Cutting Concerns

- [ ] T024 [P] Add error boundaries and toast notifications to Frontend
- [ ] T025 [P] Optimize `rust-embed` compression settings in `Cargo.toml`
- [x] T026 Update root `README.md` with `ckrv ui` documentation
- [x] T027 Verify strictly typed responses across all API endpoints

## Dependencies & Execution Order

### Phase Dependencies
- **Setup**: Blocking.
- **Foundational**: Blocking.
- **US1 & US2**: US1 should theoretically come first for the "Dashboard" container, but US2's API work can happen in parallel.

### Parallel Opportunities
- Frontend components can be built in parallel with Backend API handlers.
- `ckrv-cli` integration (T010) is independent of internal UI logic.
