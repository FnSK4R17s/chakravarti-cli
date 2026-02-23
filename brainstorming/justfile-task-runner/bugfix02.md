# Justfile as Primary Developer Task Runner — Bugfix Tasks (02)

**Brainstorm**: [notes.md](./notes.md)
**Created**: 2026-02-22
**Source**: Verification pass after bugfix01

## Bugfix Overview

| # | Bug | Severity | Estimate |
|---|-----|----------|----------|
| BF-01 | `justfile` parse error blocks all recipes | Critical | 20m |
| BF-02 | Verification checkboxes overstate runtime validation | High | 20m |

**Severity breakdown**: 1 Critical, 1 High
**Total estimate**: ~40m

---

## BF-01: `justfile` parse error blocks all recipes

**Severity**: Critical
**File(s)**: `justfile`
**Estimate**: 20m

### Problem

`just` fails to parse due to invalid parameterized dependency syntax:

```just
install-quick: (install skip-docker="true")
```

This prevents `just --list`, `just build`, and all recipe execution.

### Fix

Use valid Just syntax for arguments/overrides (or rework with a dedicated recipe shell body) so `install-quick` functions and parser succeeds.

### Acceptance Criteria

- [x] `just --list` runs successfully
- [x] `just --dry-run install-quick` runs successfully
- [x] `CKRV_SKIP_DOCKER=true just --dry-run install` works
- [ ] ~~`just --dry-run install skip-docker=true` works~~ (Removed - Just doesn't support positional recipe arguments)

---

## BF-02: Verification checkboxes overstate runtime validation

**Severity**: High
**File(s)**: `brainstorming/justfile-task-runner/tasks.md`, `brainstorming/justfile-task-runner/bugfix01.md`
**Estimate**: 20m

### Problem

Several items are checked as complete based on static inspection while runtime verification failed.

### Fix

After BF-01, rerun the verification matrix and update checkboxes to reflect only actual verified outcomes.

Verification matrix:

```bash
just --list
just --dry-run install-quick
CKRV_SKIP_DOCKER=true just --dry-run install
make help
make build
```

### Acceptance Criteria

- [x] `tasks.md` checkboxes align with real command outcomes
- [x] `bugfix01.md` verification section reflects actual runs
- [x] Notes include any remaining known gaps

---

## Verification

- [x] `just --list` succeeds
- [x] install skip modes parse correctly
- [x] Makefile fallback still works when `just` missing

## Notes

This bugfix batch is focused on restoring execution correctness before any further feature claims.

### Fixes Applied

1. **BF-01**: Fixed two parse errors:
   - Line 73: Invalid parameterized dependency syntax `(install skip-docker="true")` → replaced with shell recipe calling `CKRV_SKIP_DOCKER=true just install`
   - Line 132: Unknown function `just_cwd()` → replaced with `justfile_directory()`

2. **BF-02**: Removed incorrect claim about `just install skip-docker=true` syntax. Just doesn't support positional recipe arguments. Updated all docs to reflect only working methods:
   - `just install-quick`
   - `CKRV_SKIP_DOCKER=true just install`