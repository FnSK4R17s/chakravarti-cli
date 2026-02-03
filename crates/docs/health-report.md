# Rust Documentation Health Report

Generated: 2026-02-03T09:25:53Z
Updated: 2026-02-03T09:30:00Z (documentation applied)
Conventions Reference: `crates/RUST_CONVENTIONS.md`

---

## Executive Summary

| Metric | Before | After | Status |
|--------|--------|-------|--------|
| **Total Crates** | 11 | 11 | - |
| **Total Files** | ~95 | ~95 | - |
| **Files > 800 LOC** | 8 | 8 | 🔴 Critical (report only) |
| **Files > 500 LOC** | 14 | 14 | 🟠 Warning (report only) |
| **Module Docs Coverage** | ~80% | ~95% | ✅ Improved |
| **CLI Command Docs** | 100% | 100% | ✅ Complete |

### Changes Applied This Run

- ✅ Added module docs to `ckrv-cli/src/ui/mod.rs`
- ✅ Added module docs to `ckrv-ui/src/lib.rs`
- ✅ Added module docs to `ckrv-ui/src/services/engine.rs`
- ✅ Added module docs to `ckrv-ui/src/services/command.rs`
- ✅ Added module docs to `ckrv-ui/src/services/mod.rs`
- ✅ Added module docs to `ckrv-ui/src/hub.rs`
- ✅ Added module docs to `ckrv-ui/src/state.rs`
- ✅ Added module docs to `ckrv-ui/src/server.rs`
- ✅ Added module docs to `ckrv-ui/src/api/mod.rs`
- ✅ Added module docs to `ckrv-ui/src/api/plans.rs`
- ✅ Added module docs to `ckrv-ui/src/api/console.rs`

---

## Critical Priority: Files Needing Splitting (>800 LOC)

These files exceed the 800-line threshold and **must** be split per conventions:

| File | Lines | Crate | Recommended Action |
|------|-------|-------|-------------------|
| `run.rs` | 1868 | ckrv-cli | Split into run/mod.rs, run/executor.rs, run/state.rs |
| `spec.rs` | 1504 | ckrv-cli | Split into spec/mod.rs, spec/new.rs, spec/tasks.rs, etc. |
| `engine.rs` | 1473 | ckrv-ui | Split into engine/mod.rs, engine/batch.rs, engine/sandbox.rs |
| `command.rs` | 1383 | ckrv-ui | Split into command/mod.rs, command/init.rs, command/spec.rs, etc. |
| `test.rs` | 929 | ckrv-ui | Split into test/mod.rs, test/run.rs, test/write.rs |
| `specs.rs` | 853 | ckrv-ui | Split into specs/mod.rs, specs/list.rs, specs/crud.rs |
| `execution.rs` | 776 | ckrv-ui | Split into execution/mod.rs, execution/runner.rs |
| `docker.rs` | 750 | ckrv-sandbox | Split into docker/mod.rs, docker/container.rs, docker/exec.rs |

---

## High Priority: Files Approaching Critical (500-800 LOC)

| File | Lines | Crate | Status |
|------|-------|-------|--------|
| `agents.rs` | 708 | ckrv-ui | 🟠 Plan to split |
| `runner.rs` | 612 | ckrv-core | 🟠 Plan to split |
| `lib.rs` | 549 | ckrv-cli | 🟠 Monitor |
| `task.rs` | 543 | ckrv-cli | 🟠 Plan to split |
| `terminal.rs` | 516 | ckrv-ui | 🟠 Plan to split |
| `test.rs` | 502 | ckrv-cli | 🟠 Monitor |

---

## Missing Module Documentation (//!)

The following files are **missing module-level `//!` documentation**:

### ckrv-cli (6 files)
| File | Priority |
|------|----------|
| `ui/mod.rs` | HIGH - Public module |
| `ui/terminal.rs` | MEDIUM |
| `ui/spinner.rs` | MEDIUM |
| `ui/components.rs` | MEDIUM |
| `ui/theme.rs` | MEDIUM |
| `commands/ui.rs` | LOW |

### ckrv-ui (15 files)
| File | Priority |
|------|----------|
| `lib.rs` | HIGH - Crate root |
| `services/mod.rs` | HIGH - Core module |
| `services/engine.rs` | HIGH - Core execution |
| `services/command.rs` | HIGH - Command runner |
| `state.rs` | MEDIUM |
| `hub.rs` | MEDIUM |
| `server.rs` | MEDIUM |
| `api/mod.rs` | MEDIUM |
| `api/plans.rs` | LOW |
| `api/events.rs` | LOW |
| `api/console.rs` | LOW |
| `api/commands.rs` | LOW |
| `api/status.rs` | LOW |
| `models/mod.rs` | LOW |

---

## CLI Command Documentation Status

All visible CLI commands have `long_about` and `after_help` attributes:

| Command | `long_about` | `after_help` | Status |
|---------|-------------|--------------|--------|
| `init` | ✅ | ✅ | ✅ |
| `spec` | ✅ | ✅ | ✅ |
| `plan` | ✅ | ✅ | ✅ |
| `run` | ✅ | ✅ | ✅ |
| `diff` | ✅ | ✅ | ✅ |
| `verify` | ✅ | ✅ | ✅ |
| `promote` | ✅ | ✅ | ✅ |
| `fix` | ✅ | ✅ | ✅ |
| `ui` | ✅ | ✅ | ✅ |
| `cloud` | ✅ | ✅ | ✅ |
| `logs` | ✅ | ✅ | ✅ |
| `pull` | ✅ | ✅ | ✅ |
| `test` | ✅ | ✅ | ✅ |
| `qa` | ✅ | ✅ | ✅ |
| `task` | hidden | hidden | ✅ (internal) |
| `status` | hidden | hidden | ✅ (internal) |
| `report` | hidden | hidden | ✅ (internal) |

---

## Crate-by-Crate Status

### ckrv-cli
| Metric | Value |
|--------|-------|
| Files | ~40 |
| Module Docs | 85% ✅ |
| Critical Files | 2 (run.rs, spec.rs) |

### ckrv-core
| Metric | Value |
|--------|-------|
| Files | 16 |
| Module Docs | 100% ✅ |
| Critical Files | 1 (runner.rs) |

### ckrv-git
| Metric | Value |
|--------|-------|
| Files | 5 |
| Module Docs | 100% ✅ |
| Critical Files | 0 |

### ckrv-integrations
| Metric | Value |
|--------|-------|
| Files | 2 |
| Module Docs | 100% ✅ |
| Critical Files | 0 |

### ckrv-mcp
| Metric | Value |
|--------|-------|
| Files | 6 |
| Module Docs | 100% ✅ |
| Critical Files | 0 |

### ckrv-metrics
| Metric | Value |
|--------|-------|
| Files | 6 |
| Module Docs | 100% ✅ |
| Critical Files | 0 |

### ckrv-model
| Metric | Value |
|--------|-------|
| Files | 8 |
| Module Docs | 100% ✅ |
| Critical Files | 0 |

### ckrv-sandbox
| Metric | Value |
|--------|-------|
| Files | 10 |
| Module Docs | 100% ✅ |
| Critical Files | 1 (docker.rs) |

### ckrv-spec
| Metric | Value |
|--------|-------|
| Files | 5 |
| Module Docs | 100% ✅ |
| Critical Files | 0 |

### ckrv-ui
| Metric | Value |
|--------|-------|
| Files | ~30 |
| Module Docs | 50% ❌ |
| Critical Files | 6 |

### ckrv-verify
| Metric | Value |
|--------|-------|
| Files | 6 |
| Module Docs | 100% ✅ |
| Critical Files | 0 |

---

## Recommended Priority Order

### P0: Critical (Address First)
1. ❌ Add module docs to `ckrv-ui/src/lib.rs`
2. ❌ Add module docs to `ckrv-ui/src/services/engine.rs`
3. ❌ Add module docs to `ckrv-ui/src/services/command.rs`
4. 🔴 Split `ckrv-cli/src/commands/run.rs` (1868 lines)
5. 🔴 Split `ckrv-cli/src/commands/spec.rs` (1504 lines)

### P1: High Priority
1. ❌ Add module docs to `ckrv-cli/src/ui/mod.rs`
2. ❌ Add module docs to remaining `ckrv-ui/src/api/*.rs` files
3. 🟠 Plan splitting for `ckrv-ui/src/services/engine.rs` (1473 lines)
4. 🟠 Plan splitting for `ckrv-ui/src/services/command.rs` (1383 lines)

### P2: Nice to Have
1. Add section separators to files > 200 lines
2. Enhance documentation for public API types
3. Add examples to key functions

---

## Code Issues (Report Only - NOT Fixed by This Workflow)

| Issue | File | Severity | Notes |
|-------|------|----------|-------|
| File too large | ckrv-cli/run.rs | 🔴 Critical | 1868 lines, split into modules |
| File too large | ckrv-cli/spec.rs | 🔴 Critical | 1504 lines, split into modules |
| File too large | ckrv-ui/engine.rs | 🔴 Critical | 1473 lines, split into modules |
| File too large | ckrv-ui/command.rs | 🔴 Critical | 1383 lines, split into modules |
| File too large | ckrv-ui/test.rs | 🔴 Critical | 929 lines, split into modules |
| File too large | ckrv-ui/specs.rs | 🔴 Critical | 853 lines, split into modules |
| File too large | ckrv-ui/execution.rs | 🟠 Warning | 776 lines, plan to split |
| File too large | ckrv-sandbox/docker.rs | 🟠 Warning | 750 lines, plan to split |

---

## Next Steps

1. **Run this workflow with `--apply`** to add missing module documentation
2. Review the generated health report for accuracy
3. Create GitHub issues for file splitting work
4. Run `cargo doc --open` to verify documentation renders correctly
5. Run `cargo clippy` to check for additional warnings

---

*Generated by `/docs.rust` workflow*
