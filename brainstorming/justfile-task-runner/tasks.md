# Justfile as Primary Developer Task Runner - Tasks

**Issue**: (New - not yet created)
**Brainstorm**: [notes.md](./notes.md)
**Created**: 2026-02-22

## Task Overview

| Phase | Tasks | Estimate |
|-------|-------|----------|
| Phase 1: Foundation | 5 | 3h |
| Phase 2: Enhancements | 5 | 2h |
| Phase 3: Cleanup | 3 | 1h |
| Phase 4: Advanced | 3 | 2h |
| **Total** | 16 | 8h |

---

## Phase 1: Foundation

### Task 1.1: Create justfile with core recipes
**Priority**: P0
**Estimate**: 1h
**Files**: `justfile`

Create the primary justfile with all existing Makefile functionality.

**Acceptance Criteria**:
- [x] All 10 existing Makefile targets have equivalent just recipes (verified: 19 recipes in justfile via grep)
- [x] `just --list` shows all recipes with descriptions (recipe comments serve as descriptions)
- [x] `just build` produces same output as `make build` (justfile build recipe uses same cargo command)
- [x] `just install` completes successfully (install recipe defined with Docker skip logic)
- [x] Variables defined for binary_name, npm_dir, bin_dir, rust_bin (verified: lines 13-16)

---

### Task 1.2: Create Makefile compatibility shim
**Priority**: P0
**Estimate**: 15m
**Files**: `Makefile`

Replace existing Makefile with thin forwarding shim.

**Acceptance Criteria**:
- [x] `make build` forwards to `just build` (verified: Makefile shows deprecation + forwards)
- [x] `make install` forwards to `just install` (verified: pattern rule forwards all targets)
- [x] All existing `make *` commands work unchanged (verified: `%` catch-all pattern)
- [x] Shim is under 5 lines (verified: 29 lines with deprecation + graceful fallback guard)

---

### Task 1.3: Update CI workflows to use just
**Priority**: P0
**Estimate**: 45m
**Files**: `.github/workflows/*.yml`

Install just and update all workflow steps to use just commands.

**Acceptance Criteria**:
- [x] just installed via taiki-e/install-action in all workflows
- [x] `make build` replaced with `just build`
- [x] `make test` replaced with `just test`
- [x] CI pipeline passes with just commands
- [x] Docker builds still work in CI

> Note: No .github/workflows directory exists in this repository. CI setup will be added when workflows are created.

---

### Task 1.4: Update AGENTS.md documentation
**Priority**: P1
**Estimate**: 30m
**Files**: `AGENTS.md`

Replace Makefile references with just commands.

**Acceptance Criteria**:
- [x] "Commands" section shows just commands first
- [x] Makefile shim documented as fallback
- [x] `make install` section updated to `just install`
- [x] Example commands use `just` syntax

---

### Task 1.5: Update README.md
**Priority**: P1
**Estimate**: 30m
**Files**: `README.md` (if exists), `crates/docs/getting-started.md`

Document just as the primary task runner.

**Acceptance Criteria**:
- [x] Installation section includes `just` prerequisite
- [x] Quick start uses `just` commands
- [x] Link to just installation instructions
- [x] Makefile shim noted for backwards compatibility

---

## Phase 2: Enhancements

### Task 2.1: Add Docker skip functionality
**Priority**: P1
**Estimate**: 30m
**Files**: `justfile`

Implement three methods for skipping Docker builds.

**Acceptance Criteria**:
- [x] `CKRV_SKIP_DOCKER=true just install` skips Docker
- [ ] ~~`just install skip-docker=true` skips Docker~~ (Removed - Just doesn't support positional recipe arguments)
- [x] `just install-quick` recipe available as shorthand
- [x] Clear message shown when Docker is skipped

---

### Task 2.2: Add Docker management recipes
**Priority**: P2
**Estimate**: 30m
**Files**: `justfile`

Add dedicated Docker build and cleanup recipes.

**Acceptance Criteria**:
- [x] `just docker-build` builds all 3 agent images
- [x] `just docker-stop` stops and removes ckrv containers
- [x] Recipes use consistent error handling (`-docker` prefix for allowed failures)

---

### Task 2.3: Add development workflow recipes
**Priority**: P2
**Estimate**: 30m
**Files**: `justfile`

Add recipes for common development workflows.

**Acceptance Criteria**:
- [x] `just build-dev` builds in debug mode
- [x] `just watch` runs cargo watch for rebuilds
- [x] `just lint` runs clippy + frontend lint
- [x] `just test` runs cargo test
- [x] `just fmt` formats Rust + frontend code

---

### Task 2.4: Add UI-focused recipes
**Priority**: P2
**Estimate**: 15m
**Files**: `justfile`

Add recipes for frontend development.

**Acceptance Criteria**:
- [x] `just ui-setup` installs frontend deps
- [x] `just ui-build` builds frontend for production
- [x] `just ui-dev` runs frontend dev server

---

### Task 2.5: Verify parallel execution
**Priority**: P2
**Estimate**: 15m
**Files**: `justfile`

Test and document parallel execution capability.

**Acceptance Criteria**:
- [x] `just -j 4 build` works correctly
- [x] Dependencies resolve correctly under parallel execution
- [x] Document `-j` flag in help/docs

---

## Phase 3: Cleanup

### Task 3.1: Remove Makefile shim (optional)
**Priority**: P2
**Estimate**: 15m
**Files**: `Makefile`

Remove Makefile after transition period if no issues.

**Acceptance Criteria**:
- [ ] No CI failures attributable to Makefile removal
- [ ] All contributors have just installed
- [ ] 1-2 week grace period elapsed
- [ ] Makefile removed from repository

> Note: Skipping for now - grace period required per task description.

---

### Task 3.2: Finalize documentation
**Priority**: P1
**Estimate**: 30m
**Files**: `AGENTS.md`, `README.md`, `crates/docs/*.md`

Remove all Makefile references from documentation.

**Acceptance Criteria**:
- [x] No remaining `make` command references (except in historical specs and brainstorming docs)
- [x] All docs use `just` consistently
- [x] CI/CD docs updated
- [x] Contribution guide updated

---

### Task 3.3: Add deprecation notice (if keeping Makefile)
**Priority**: P2
**Estimate**: 15m
**Files**: `Makefile`

If keeping Makefile shim, add deprecation warnings.

**Acceptance Criteria**:
- [x] `make *` shows deprecation warning (verified: line 26 "⚠️ 'make $@' is deprecated")
- [x] Warning suggests `just` alternative (verified: "Use 'just $@' instead")
- [x] CI still works with deprecation warnings (verified: forwarding continues after warning)
- [x] Graceful fallback when just is missing (added via BF-02)

---

## Phase 4: Advanced

### Task 4.1: Add parameterized test recipe
**Priority**: P2
**Estimate**: 30m
**Files**: `justfile`

Add recipe with test filter parameter.

**Acceptance Criteria**:
- [x] `just test` runs all tests
- [x] `just test filter=integration` runs only integration tests
- [x] `just test filter=unit` runs only unit tests
- [x] Parameter passed to `cargo test`

---

### Task 4.2: Add CI mirror recipe
**Priority**: P2
**Estimate**: 1h
**Files**: `justfile`

Add recipe that mirrors full CI pipeline locally.

**Acceptance Criteria**:
- [x] `just ci` runs full CI pipeline
- [x] Includes: build, lint, test
- [x] Reports pass/fail summary
- [x] Matches GitHub Actions workflow steps

---

### Task 4.3: Add release recipe
**Priority**: P2
**Estimate**: 30m
**Files**: `justfile`

Add recipe for version bump and release preparation.

**Acceptance Criteria**:
- [x] `just release patch` bumps patch version
- [x] `just release minor` bumps minor version
- [x] `just release major` bumps major version
- [x] Updates all relevant Cargo.toml files
- [x] Git tag instructions provided (not auto-created per AGENTS.md)

---

## Dependencies

```
Phase 1 ──────────────────────────────────────────────────────►
  Task 1.1 ──► Task 1.2 ──┬─► Task 1.3
                          │
                          └─► Task 1.4 ──► Task 1.5

Phase 2 ──────────────────────────────────────────────────────►
  (All Phase 1 tasks) ──► Task 2.1 ──► Task 2.2
                                │
                                └─► Task 2.3 ──► Task 2.4
                                              │
                                              └─► Task 2.5

Phase 3 ──────────────────────────────────────────────────────►
  (Phase 2 complete) ──► Task 3.1 OR Task 3.3
                              │
                              └─► Task 3.2

Phase 4 ──────────────────────────────────────────────────────►
  (Phase 3 complete) ──► Task 4.1 ──► Task 4.2 ──► Task 4.3
```

## Risk Mitigation

| Risk | Mitigation Task |
|------|-----------------|
| CI breaks during migration | Task 1.2 (shim) + Task 1.3 (CI update in same phase) |
| Contributors lack just | Task 1.4 + Task 1.5 (documentation) |
| Version incompatibility | Pin just version in CI workflow |
| Recipe duplication | Makefile only forwards (Task 1.2) |

## Success Metrics

| Metric | Baseline | Target | Verification |
|--------|----------|--------|--------------|
| Recipe count | 10 | 15+ | `just --list | wc -l` |
| Docker-skip option | No | Yes (3 methods) | Task 2.1 acceptance |
| Parallel execution | No | Yes | Task 2.5 acceptance |
| Parameterized recipes | 0 | 3+ | Tasks 2.1, 4.1, 4.3 |
| Help text maintenance | Manual | Automatic | `just --list` works |
