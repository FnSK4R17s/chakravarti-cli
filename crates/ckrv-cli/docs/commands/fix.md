---
command: fix
generated_from: lib.rs
last_commit: 2a2da7f
hidden: true
---

# ckrv fix

Fix verification errors with AI.

## Description

Fix verification errors with AI.

Analyzes failed tests, lint errors, or build issues and uses AI to automatically generate fixes. Runs in an isolated Docker sandbox.

Best used after `ckrv verify` identifies issues.

## Options

| Flag | Description |
|------|-------------|
| `--lint` | Fix only lint errors |
| `--type` | Fix only type errors |
| `--test` | Fix only test failures |
| `--check` | Re-run verification after fixing |
| `--error <MSG>` | Specific error message to fix (from UI) |

## Examples

```bash
# Fix all errors
ckrv fix

# Fix with specific agent
ckrv fix --agent claude-3.5

# Fix only test failures
ckrv fix --tests-only
```
