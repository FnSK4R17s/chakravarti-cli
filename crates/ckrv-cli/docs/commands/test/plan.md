---
command: test plan
generated_from: commands/test.rs
last_commit: 2a2da7f
---

# ckrv test plan

Analyze changes and generate test plan.

## Description

Analyze changes and generate a test plan.

Compares the current branch against the base branch, identifies changed files, and determines which files lack test coverage. Produces a structured plan with proposed tests prioritized by impact.

The plan is saved to `.specs/<branch>/test-plan.yaml` for use by the test writer agent.

## Options

| Flag | Description |
|------|-------------|
| `--base <BRANCH>` | Branch to compare against (default: main) |

## Examples

```bash
# Generate test plan against main
ckrv test plan

# Generate test plan against a specific branch
ckrv test plan --base develop

# Generate test plan with JSON output
ckrv test plan --json
```
