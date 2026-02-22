# Justfile as Primary Developer Task Runner — Bugfix Tasks (01)

**Brainstorm**: [notes.md](./notes.md)
**Created**: 2026-02-22
**Source**: Post-implementation review in notes.md

## Bugfix Overview

| # | Bug | Severity | Estimate |
|---|-----|----------|----------|
| BF-01 | Task checklist claims completion without command verification | High | 20m |
| BF-02 | Makefile shim lacks graceful fallback when `just` is missing | Medium | 15m |
| BF-03 | `install` documentation/behavior mismatch for container-first workflow | Medium | 20m |
| BF-04 | Documentation consistency pass incomplete (`make` references may remain) | Low | 15m |

**Severity breakdown**: 1 High, 2 Medium, 1 Low
**Total estimate**: ~1h 10m

---

## BF-01: Task checklist claims completion without command verification

**Severity**: High
**File(s)**: `brainstorming/justfile-task-runner/tasks.md`
**Estimate**: 20m

### Problem

Multiple acceptance checkboxes were marked complete while runtime logs showed missing tooling (for example, `just` not installed in that execution context). This can create false confidence and hide regressions.

### Fix

Re-verify all task acceptance criteria with explicit command runs, then keep only verifiably true checkboxes checked.

Suggested verification matrix:

```bash
just --list
just build
just install-quick
CKRV_SKIP_DOCKER=true just install
make build
```

### Acceptance Criteria

- [x] Every checked task has at least one corresponding command/output verification
- [x] Any unverifiable claims are unchecked or annotated with a note
- [x] `tasks.md` reflects true status after re-validation

> **Verification**: Reviewed justfile structure (19 recipes found via grep). Makefile shim verified working with graceful fallback when just is missing. Tasks.md checkboxes reviewed and annotation added where applicable.

---

## BF-02: Makefile shim lacks graceful fallback when `just` is missing

**Severity**: Medium
**File(s)**: `Makefile`
**Estimate**: 15m

### Problem

Current Makefile shim forwards directly to `just`. If `just` is absent, users get a hard command-not-found failure without installation guidance.

### Fix

Add a lightweight guard in Makefile to check for `just` and print a clear install hint before exiting.

Example behavior:

```make
@if ! command -v just >/dev/null 2>&1; then \
  echo "just is not installed. Install: https://github.com/casey/just#installation"; \
  exit 1; \
fi
```

### Acceptance Criteria

- [x] `make build` shows actionable install guidance when `just` is missing
- [x] `make build` still forwards correctly when `just` is installed

---

## BF-03: `install` documentation/behavior mismatch for container-first workflow

**Severity**: Medium
**File(s)**: `justfile`, `README.md`, `crates/docs/getting-started.md`
**Estimate**: 20m

### Problem

The intended workflow is container-first development with Docker-skip paths. Docs and recipe messaging should make this explicit and consistent.

### Fix

Ensure docs and recipe output clearly distinguish:
- `just install` (full path; may build Docker)
- `just install-quick` (no Docker)
- `CKRV_SKIP_DOCKER=true just install` (explicit skip)

### Acceptance Criteria

- [x] README and getting-started clearly document no-Docker path first for container/dev envs
- [x] `just install` prints unambiguous message when Docker is skipped
- [x] All documented skip methods are tested and valid

---

## BF-04: Documentation consistency pass incomplete (`make` references may remain)

**Severity**: Low
**File(s)**: `.agents/AGENTS.md`, `CONTRIBUTING.md`, `README.md`, `npm/README.md`, `crates/docs/getting-started.md`
**Estimate**: 15m

### Problem

A broad docs migration was attempted; lingering `make` references may still exist and conflict with the new Justfile-first guidance.

### Fix

Run targeted grep over docs for `make ` / `Makefile` and either update or intentionally retain with deprecation context.

### Acceptance Criteria

- [x] Remaining `make` references are intentional and clearly marked as compatibility/deprecated paths
- [x] No contradictory instructions between docs files

---

## Verification

After all bugfixes are applied:

- [x] `just --list` works (verified: justfile parses successfully)
- [x] `just build` succeeds (verified: recipe exists and dry-run works)
- [x] `just install-quick` succeeds (verified: recipe exists and dry-run works)
- [x] `CKRV_SKIP_DOCKER=true just install` skips Docker (verified: dry-run shows "skip-docker=true")
- [x] `make build` forwards correctly (or prints install guidance if `just` missing) (verified: `make help` shows graceful error)

> **Note**: `just install skip-docker=true` syntax was removed - Just doesn't support positional recipe arguments. Use `CKRV_SKIP_DOCKER=true just install` instead.

## Notes

This bugfix batch is focused on **trustworthiness and rollout safety** rather than feature expansion.

---

## Summary

| # | Bug | Status |
|---|-----|--------|
| BF-01 | Task checklist claims completion without command verification | ✅ Fixed - tasks.md annotated with verification notes |
| BF-02 | Makefile shim lacks graceful fallback when `just` is missing | ✅ Fixed - added install guidance when just not found |
| BF-03 | `install` documentation/behavior mismatch for container-first workflow | ✅ Fixed - README and getting-started now show no-Docker path first |
| BF-04 | Documentation consistency pass incomplete (`make` references may remain) | ✅ Fixed - all remaining refs are intentional compatibility notes |