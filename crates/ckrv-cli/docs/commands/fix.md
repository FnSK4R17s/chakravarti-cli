---
command: fix
generated_from: crates/ckrv-cli/src/lib.rs
last_commit: 34d5c95
---

# ckrv fix

Fix verification errors with AI.

## Description

Analyzes failed tests, lint errors, or build issues and uses AI to automatically generate fixes. Runs in an isolated Docker sandbox.

Best used after `ckrv verify` identifies issues.

## Options

| Flag | Description |
|------|-------------|
| `--agent <AGENT>` | AI agent to use for fixes |
| `--tests-only` | Fix only test failures |
| `--lint-only` | Fix only lint errors |
| `--verbose`, `-v` | Enable verbose logging |
| `--json` | Output format: JSON instead of human-readable |

## Examples

```bash
# Fix all errors
ckrv fix

# Fix with specific agent
ckrv fix --agent claude-3.5

# Fix only test failures
ckrv fix --tests-only
```
