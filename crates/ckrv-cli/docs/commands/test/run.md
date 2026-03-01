---
command: test run
generated_from: commands/test.rs
last_commit: f92f604
---

# ckrv test run

Run existing tests in sandbox.

## Description

Run existing tests in a sandboxed environment.

Detects the project's test framework automatically and executes the full test suite. Results are displayed with pass/fail counts and a summary report.

Exits with code 1 if any test fails. Use --json for machine-readable output.

## Options

| Flag | Description |
|------|-------------|
| `--base <BRANCH>` | Branch to compare against (default: main) |

## Examples

```bash
# Run tests comparing against main
ckrv test run

# Run tests comparing against a specific branch
ckrv test run --base develop

# Run tests with JSON output
ckrv test run --json
```
