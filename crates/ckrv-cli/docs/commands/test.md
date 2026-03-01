---
command: test
generated_from: lib.rs
last_commit: f92f604
---

# ckrv test

Run tests in sandbox, plan and write new tests.

## Description

Run tests in sandbox, plan and write new tests.

Comprehensive test management with AI assistance. Can run existing tests, analyze coverage gaps, and generate new tests using AI agents.

## Subcommands

| Subcommand | Description |
|------------|-------------|
| `run` | Run existing tests in sandbox |
| `plan` | Analyze changes and generate test plan |
| `write` | Write new tests using test writer agent |
| `coverage` | Check test coverage of changed files |

## Examples

```bash
# Run all tests
ckrv test run

# Plan tests for uncovered code
ckrv test plan

# Write new tests with AI
ckrv test write --agent claude-3.5
```
