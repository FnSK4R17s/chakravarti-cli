---
command: test
generated_from: crates/ckrv-cli/src/lib.rs
last_commit: 34d5c95
---

# ckrv test

Run tests in sandbox, plan and write new tests.

## Description

Comprehensive test management with AI assistance. Can run existing tests, analyze coverage gaps, and generate new tests using AI agents.

Subcommands: run, plan, write

## Subcommands

| Subcommand | Description |
|------------|-------------|
| `run` | Execute tests in Docker sandbox |
| `plan` | Analyze coverage and plan new tests |
| `write` | Generate new tests with AI |

## Examples

```bash
# Run all tests
ckrv test run

# Plan tests for uncovered code
ckrv test plan

# Write new tests with AI
ckrv test write --agent claude-3.5
```
