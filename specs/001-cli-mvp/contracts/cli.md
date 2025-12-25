# CLI Contracts: Chakravarti CLI

**Feature**: 001-cli-mvp  
**Date**: 2025-12-12  
**Binary**: `ckrv`

---

## Global Flags

All commands support these flags:

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--json` | bool | false | Output structured JSON instead of human-readable text |
| `--quiet` / `-q` | bool | false | Suppress non-essential output |
| `--verbose` / `-v` | bool | false | Enable verbose logging to stderr |
| `--help` / `-h` | bool | - | Show help message |
| `--version` / `-V` | bool | - | Show version information |

---

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | User error (invalid input, missing file, etc.) |
| 2 | System error (git failure, container error, etc.) |
| 3 | Verification failed (tests failed, acceptance criteria not met) |
| 130 | Interrupted (Ctrl+C) |

---

## Commands

### `ckrv init`

Initialize a repository for Chakravarti.

**Usage**:
```bash
ckrv init [OPTIONS]
```

**Options**:
| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--force` | bool | false | Overwrite existing configuration |

**Output (JSON)**:
```json
{
  "success": true,
  "created": [
    ".specs/",
    ".chakravarti/",
    ".chakravarti/config.json"
  ]
}
```

**Output (Human)**:
```
✓ Initialized Chakravarti in /path/to/repo
  Created .specs/
  Created .chakravarti/
```

**Exit Codes**:
- 0: Initialized successfully
- 1: Already initialized (use --force to overwrite)
- 1: Not a git repository
- 2: Permission denied

---

### `ckrv spec new <NAME>`

Create a new specification file.

**Usage**:
```bash
ckrv spec new <NAME> [OPTIONS]
```

**Arguments**:
| Argument | Required | Description |
|----------|----------|-------------|
| `NAME` | Yes | Spec identifier (alphanumeric + underscore) |

**Options**:
| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--goal` / `-g` | string | "" | Set the goal inline |

**Output (JSON)**:
```json
{
  "success": true,
  "spec_path": ".specs/add_rate_limiter.yaml",
  "id": "add_rate_limiter"
}
```

**Exit Codes**:
- 0: Spec created
- 1: Invalid name format
- 1: Spec already exists

---

### `ckrv spec validate <PATH>`

Validate a specification file.

**Usage**:
```bash
ckrv spec validate <PATH>
```

**Output (JSON)**:
```json
{
  "valid": true,
  "spec_id": "add_rate_limiter",
  "warnings": []
}
```

```json
{
  "valid": false,
  "errors": [
    {
      "field": "acceptance",
      "message": "At least one acceptance criterion required"
    }
  ]
}
```

**Exit Codes**:
- 0: Valid
- 1: Invalid spec

---

### `ckrv run <SPEC_PATH>`

Execute a specification.

**Usage**:
```bash
ckrv run <SPEC_PATH> [OPTIONS]
```

**Arguments**:
| Argument | Required | Description |
|----------|----------|-------------|
| `SPEC_PATH` | Yes | Path to spec file |

**Options**:
| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--optimize` | enum | balanced | Optimization mode: cost, time, balanced |
| `--max-attempts` | int | 3 | Maximum retry attempts |
| `--planner-model` | string | - | Override planner model |
| `--executor-model` | string | - | Override executor model |
| `--dry-run` | bool | false | Generate plan without executing |

**Output (JSON, streaming)**:
```json
{"event": "job_started", "job_id": "abc-123", "spec_id": "add_rate_limiter"}
{"event": "planning_started", "job_id": "abc-123"}
{"event": "planning_completed", "job_id": "abc-123", "steps": 5}
{"event": "attempt_started", "job_id": "abc-123", "attempt": 1}
{"event": "step_started", "step": "analyze_login_flow"}
{"event": "step_completed", "step": "analyze_login_flow", "duration_ms": 1200}
...
{"event": "verification_started", "attempt": 1}
{"event": "verification_completed", "passed": true}
{"event": "job_succeeded", "job_id": "abc-123", "diff_path": ".worktrees/abc-123/1/"}
```

**Output (Human)**:
```
▶ Running spec: add_rate_limiter
  Job ID: abc-123

📋 Planning...
  Generated 5 steps

🔄 Attempt 1/3
  [1/5] analyze_login_flow ✓ (1.2s)
  [2/5] add_rate_limit_middleware ✓ (3.4s)
  [3/5] update_config ✓ (0.8s)
  [4/5] add_tests ✓ (2.1s)
  [5/5] run_tests ✓ (5.2s)

✅ Verification passed

📊 Summary
  Time: 12.7s
  Tokens: 4,521 (input) + 2,103 (output)
  Cost: ~$0.08

→ Diff available: ckrv diff abc-123
→ Promote: ckrv promote abc-123 --branch <name>
```

**Exit Codes**:
- 0: Job succeeded
- 1: Invalid spec or config
- 2: System error (container, git, model API)
- 3: Verification failed after all attempts
- 130: Interrupted

---

### `ckrv status <JOB_ID>`

Get status of a job.

**Usage**:
```bash
ckrv status <JOB_ID>
```

**Output (JSON)**:
```json
{
  "job_id": "abc-123",
  "spec_id": "add_rate_limiter",
  "state": "succeeded",
  "attempts": 1,
  "created_at": "2025-12-12T11:30:00Z",
  "completed_at": "2025-12-12T11:30:12Z"
}
```

**Exit Codes**:
- 0: Status retrieved
- 1: Job not found

---

### `ckrv diff <JOB_ID>`

Show the diff produced by a job.

**Usage**:
```bash
ckrv diff <JOB_ID> [OPTIONS]
```

**Options**:
| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--stat` | bool | false | Show diffstat only |
| `--color` | enum | auto | Color mode: auto, always, never |

**Output (JSON)**:
```json
{
  "job_id": "abc-123",
  "diff": "diff --git a/src/login.rs b/src/login.rs\n...",
  "files_changed": 3,
  "insertions": 45,
  "deletions": 12
}
```

**Output (Human)**:
Standard unified diff format (git diff output).

**Exit Codes**:
- 0: Diff retrieved
- 1: Job not found or failed

---

### `ckrv report <JOB_ID>`

Show cost and time report for a job.

**Usage**:
```bash
ckrv report <JOB_ID>
```

**Output (JSON)**:
```json
{
  "job_id": "abc-123",
  "total_time_ms": 12700,
  "attempts": 1,
  "token_usage": {
    "gpt-4.1": {"input": 3200, "output": 1800},
    "gpt-4o-mini": {"input": 1321, "output": 303}
  },
  "cost_usd": 0.082,
  "step_metrics": [
    {"step": "analyze_login_flow", "duration_ms": 1200, "model": "gpt-4.1"},
    ...
  ]
}
```

**Output (Human)**:
```
📊 Report: abc-123

Time
  Total: 12.7s
  Planning: 2.1s
  Execution: 8.3s
  Verification: 2.3s

Tokens
  gpt-4.1: 3,200 in / 1,800 out
  gpt-4o-mini: 1,321 in / 303 out

Cost
  Estimated: $0.08

Attempts: 1/3
```

**Exit Codes**:
- 0: Report retrieved
- 1: Job not found

---

### `ckrv promote <JOB_ID>`

Promote job changes to a branch.

**Usage**:
```bash
ckrv promote <JOB_ID> [OPTIONS]
```

**Options**:
| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--branch` / `-b` | string | Required | Target branch name |
| `--force` | bool | false | Overwrite existing branch |
| `--push` | bool | false | Push to remote after creating |

**Output (JSON)**:
```json
{
  "success": true,
  "job_id": "abc-123",
  "branch": "add-rate-limiter",
  "pushed": false
}
```

**Exit Codes**:
- 0: Branch created
- 1: Job not found or failed
- 1: Branch already exists (use --force)
- 2: Git error

---

## Error Response Format

All error responses follow this schema:

```json
{
  "error": {
    "code": "SPEC_NOT_FOUND",
    "message": "Specification file not found: .specs/missing.yaml",
    "details": {
      "path": ".specs/missing.yaml"
    }
  }
}
```

**Error Codes**:
| Code | Exit | Description |
|------|------|-------------|
| `NOT_INITIALIZED` | 1 | Repository not initialized |
| `SPEC_NOT_FOUND` | 1 | Spec file not found |
| `SPEC_INVALID` | 1 | Spec validation failed |
| `JOB_NOT_FOUND` | 1 | Job ID not found |
| `JOB_FAILED` | 3 | Job failed verification |
| `GIT_ERROR` | 2 | Git operation failed |
| `CONTAINER_ERROR` | 2 | Container runtime error |
| `MODEL_ERROR` | 2 | Model API error |
| `INTERRUPTED` | 130 | User interrupted |
