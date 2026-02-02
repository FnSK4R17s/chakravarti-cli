---
command: ckrv test
generated_from: crates/ckrv-cli/src/lib.rs
last_commit: 0ad833d
---

# ckrv test

Run tests in sandbox, plan and write new tests.

## Description

Run tests in sandbox, plan and write new tests.

Comprehensive test management with AI assistance. Can run existing tests, analyze coverage gaps, and generate new tests using AI agents.

Subcommands: run, plan, write

## Subcommands

| Subcommand | Description |
|------------|-------------|
| `run` | Run existing tests in sandbox |
| `plan` | Analyze changes and generate test plan |
| `write` | Write new tests using test writer agent |

## Options

| Flag | Description |
|------|-------------|
| `--agent <agent>` | AI agent to use for test writing |
| `--base <branch>` | Branch to compare against (default: main) |
| `--json` | Output in JSON format |

## Examples

```bash
# Run all tests
ckrv test run

# Plan tests for uncovered code
ckrv test plan

# Write new tests with AI
ckrv test write --agent claude-3.5
```
