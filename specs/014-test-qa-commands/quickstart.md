# Quickstart: Test and QA Commands

**Feature**: 014-test-qa-commands  
**Date**: 2026-01-21

## Prerequisites

1. **Chakravarti CLI**: Must have `ckrv` installed
2. **Agent Configuration**: At least one agent configured with test writer or QA role
3. **Docker**: Running for sandbox execution
4. **Git Repository**: Changes committed to a branch (not main)

## Configuration

### Set Test Writer Agent

In Agent Manager UI or directly in `~/.config/chakravarti/agents.yaml`:

```yaml
agents:
  - id: claude-test
    name: Claude Test Writer
    agent_type: claude
    is_test_writer: true  # <-- Enable test writer role
    enabled: true
```

### Set QA Agent

```yaml
agents:
  - id: claude-qa
    name: Claude QA
    agent_type: claude
    is_qa_agent: true  # <-- Enable QA role
    enabled: true
```

## Commands

### Run Existing Tests

```bash
# Run tests in sandbox (auto-detect framework)
ckrv test run

# Specify custom base branch
ckrv test run --base develop
```

### Plan New Tests

```bash
# Analyze changes and generate test plan
ckrv test plan

# View the generated plan
cat .chakravarti/<branch>/test-plan.md
```

### Write Tests

```bash
# Write tests from plan using test writer agent
ckrv test write

# Write and immediately run
ckrv test write --run
```

### QA Review

```bash
# Review code quality of changes
ckrv qa review

# Output to file instead of stdout
ckrv qa review --output qa-report.md
```

### Full QA Report

```bash
# Generate comprehensive QA report
ckrv qa report

# Include both quality and bug analysis
ckrv qa report --full
```

## Example Workflow

```bash
# 1. Make changes on feature branch
git checkout -b feature/my-changes
# ... make code changes ...
git commit -am "Add new feature"

# 2. Run existing tests
ckrv test run

# 3. Plan new tests for changes
ckrv test plan

# 4. Have agent write the tests
ckrv test write

# 5. Run all tests including new ones
ckrv test run

# 6. Get QA review
ckrv qa review

# 7. Fix any issues and repeat
```

## Output Examples

### Markdown Test Report

```markdown
# Test Results

**Branch**: feature/my-changes  
**Base**: main  
**Date**: 2026-01-21

## Summary

| Metric | Value |
|--------|-------|
| Total | 42 |
| Passed | 40 |
| Failed | 2 |
| Duration | 3.2s |

## Failures

### test_user_validation

**File**: tests/user_test.rs:45
**Error**: assertion failed: expected 200, got 400
```

### Markdown QA Report

```markdown
# QA Review

**Branch**: feature/my-changes  
**Compared to**: main

## Issues Found

### 🔴 Critical (1)

1. **Missing null check** - `src/handler.rs:89`
   - User input not validated before database query
   - **Fix**: Add input validation before query

### 🟡 Major (2)

1. **Duplicate logic** - `src/utils.rs:45-60`
   - Same validation repeated in 3 places
   - **Fix**: Extract to shared validate() function
```
