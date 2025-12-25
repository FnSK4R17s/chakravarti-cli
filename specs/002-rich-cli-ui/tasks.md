# Tasks: Rich CLI UI

**Feature**: Rich CLI UI
**Branch**: `002-rich-cli-ui`
**Spec**: [spec.md](./spec.md)

## Phase 1: Setup
**Goal**: Initialize dependencies and module structure without breaking the build.

- [x] T001 Add `indicatif`, `termimad`, `tabled`, `console`, `dialoguer` dependencies to `crates/ckrv-cli/Cargo.toml`
- [x] T002 Create `crates/ckrv-cli/src/ui/mod.rs` with empty modules and pub re-export
- [x] T003 Create `crates/ckrv-cli/src/ui/theme.rs` with `Theme` struct and color constants (hardcoded per Spec option A)

## Phase 2: Foundational
**Goal**: Core `UiContext` and testing infrastructure. Must pass TTY and Silent Mode checks.

- [x] T004 Create `crates/ckrv-cli/src/ui/terminal.rs` with TTY detection logic (is_terminal, NO_COLOR)
- [x] T005 [P] Implement `UiContext` struct in `crates/ckrv-cli/src/ui/mod.rs` (holds Theme and interactive state)
- [x] T006 Implement "Silent Mode" check in `UiContext::new()` (detect `--json` flag or generic silent config)
- [x] T007 Create `crates/ckrv-cli/src/ui/spinner.rs` wrapping `indicatif::ProgressBar` (ensures Stderr output per Spec)

## Phase 3: User Story 1 - Brand Identity & Core Styling
**Goal**: Banner and colors on startup.
**Priority**: P1

- [x] T008 [US1] Implement `Banner` component in `crates/ckrv-cli/src/ui/components.rs` (ASCII art "CHAKRAVARTI")
- [x] T009 [US1] Create `Renderable` trait in `crates/ckrv-cli/src/ui/mod.rs`
- [x] T010 [US1] Implement `render()` for `Banner` using `Theme` colors
- [x] T011 [US1] Integrate `UiContext` into `main.rs` and display banner on `help` or version command

## Phase 4: User Story 2 - Rich Data Presentation
**Goal**: Tables, Panels, and Markdown rendering.
**Priority**: P2

- [x] T012 [P] [US2] Implement `Table` helper in `crates/ckrv-cli/src/ui/components.rs` using `tabled` (apply Theme styles)
- [x] T013 [P] [US2] Implement `Panel` component in `crates/ckrv-cli/src/ui/components.rs` (Boxed success/error messages)
- [x] T014 [US2] Implement `markdown()` method in `UiContext` using `termimad` (configure Skin from Theme)
- [x] T015 [US2] Replace standard `println!` calls in 2-3 key commands (e.g., `list`, `status`) with `UiContext` methods

## Phase 5: User Story 3 - Interactive Feedback
**Goal**: Spinners for long running tasks.
**Priority**: P3

- [x] T016 [US3] Add `spinner()` method to `UiContext` returning `SpinnerGuard`
- [x] T017 [US3] Implement `SpinnerGuard` methods (`success`, `error`, `finish`) to swap spinner for static checkmark/X
- [x] T018 [US3] Integrate spinner into a mock long-running command (or existing one) to verify behavior

## Final Phase: Polish
**Goal**: Consistency check and Cleanup.

- [ ] T019 Audit all `eprintln!` and `println!` usage in `ckrv-cli` for consistency
- [ ] T020 Verify `NO_COLOR` and `--json` behavior across all modified commands (CI/CD check)

## Dependencies

- Phase 2 (Foundation) blocks all Story phases.
- Phase 3 (Branding), 4 (Tables), and 5 (Spinners) can theoretically proceed in parallel after Phase 2, but sequential is safer for consistent refactoring.

## Parallel Execution Examples

- **T012** (Tables) and **T013** (Panels) are independent components.
- **T008** (Banner) and **T009** (Renderable Trait) can be done by separate agents if coordination is tight.
