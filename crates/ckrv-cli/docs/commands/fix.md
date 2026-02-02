---
command: ckrv fix
generated_from: crates/ckrv-cli/src/lib.rs
last_commit: 0ad833d
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
| `--agent <agent>` | AI agent to use for fixing |
| `--tests-only` | Fix only test failures |
| `--check` | Re-run verification after fix |

## Examples

```bash
# Fix all errors
ckrv fix

# Fix with specific agent
ckrv fix --agent claude-3.5

# Fix only test failures
ckrv fix --tests-only
```
