---
last_commit: tauri-app-backend
last_updated: 2026-02-09
related_files:
  - ../src/server.rs
  - ../../ckrv-transport/src/axum/mod.rs
  - ../../ckrv-transport/src/axum/*.rs
  - ../../ckrv-transport/src/handlers/*.rs
  - ../../ckrv-transport/src/hub.rs
---

# API Reference: ckrv-ui

## Base URL

```
http://localhost:3000/api
```

## Authentication

Currently no authentication required (local only).

---

## Status & System

### Get System Status

```http
GET /api/status
```

Returns current system mode and status.

**Response:**
```json
{
  "mode": "idle|planning|running|paused",
  "current_spec": "012-feature-name",
  "current_phase": "execution",
  "progress": {
    "completed_tasks": 5,
    "total_tasks": 10
  }
}
```

### Check Docker Status

```http
GET /api/docker
```

**Response:**
```json
{
  "available": true,
  "version": "24.0.5"
}
```

### Get Cloud Status

```http
GET /api/cloud
```

**Response:**
```json
{
  "connected": false,
  "provider": null
}
```

### Get Default Git Branch

```http
GET /api/git/default-branch
```

**Response:**
```json
{
  "branch": "main"
}
```

---

## Specs

### List Specs

```http
GET /api/specs
```

**Response:**
```json
[
  {
    "name": "012-code-documentation",
    "status": "in_progress",
    "created_at": "2026-01-21T14:00:00Z"
  }
]
```

### Get Spec Details

```http
GET /api/specs/detail?name={spec_name}
```

**Query Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| name | string | Yes | Spec name |

**Response:**
```json
{
  "name": "012-code-documentation",
  "content": "# Feature Specification...",
  "status": "in_progress",
  "raw_yaml": "# YAML content..."
}
```

### Create Spec

```http
POST /api/specs/create
```

**Request:**
```json
{
  "name": "013-new-feature",
  "description": "Feature description"
}
```

**Response:** `201 Created`

### Save Spec

```http
POST /api/specs/save
```

**Request:**
```json
{
  "name": "012-code-documentation",
  "raw_yaml": "# Updated YAML content..."
}
```

### Validate Spec

```http
GET /api/specs/{name}/validate
```

**Response:**
```json
{
  "valid": true,
  "spec": "012-feature",
  "errors": []
}
```

### Generate Design

```http
POST /api/specs/{name}/design
```

**Response:**
```json
{
  "status": "ok",
  "spec": "012-feature",
  "message": "Design generation started"
}
```

### Generate Tasks

```http
POST /api/specs/{name}/tasks
```

### Get Clarifications

```http
GET /api/specs/{name}/clarifications
```

### Answer Clarifications

```http
POST /api/specs/{name}/clarify
```

---

## Plans

### Get Plan

```http
GET /api/plans/detail?spec={spec_name}
```

**Query Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| spec | string | Yes | Spec name |

**Response:**
```json
{
  "spec": "012-feature",
  "content": "# Implementation Plan...",
  "phases": ["setup", "core", "integration"]
}
```

### Save Plan

```http
POST /api/plans/save
```

**Request:**
```json
{
  "spec": "012-feature",
  "content": "# Plan content..."
}
```

### Get Available Models

```http
GET /api/plans/models
```

**Response:**
```json
{
  "models": [
    {"id": "claude-sonnet-4", "name": "Claude Sonnet 4"},
    {"id": "claude-opus-4", "name": "Claude Opus 4"},
    {"id": "gemini-2.5-pro", "name": "Gemini 2.5 Pro"}
  ]
}
```

---

## Tasks

### List Tasks

```http
GET /api/tasks?spec={spec_name}
```

**Query Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| spec | string | No | Filter by spec name |

**Response:**
```json
[
  {
    "id": "task-1",
    "title": "Implement feature X",
    "status": "pending|in_progress|completed|failed",
    "spec": "012-feature"
  }
]
```

### Get Task Details

```http
GET /api/tasks/detail?spec={spec}&task={task_id}
```

### Save Task

```http
POST /api/tasks/save
```

**Request:**
```json
{
  "spec": "012-feature",
  "task_id": "task-1",
  "status": "completed"
}
```

### Update Task Status

```http
POST /api/tasks/status
```

**Request:**
```json
{
  "spec": "012-feature",
  "task_id": "task-1",
  "status": "in_progress"
}
```

---

## Execution

### Start Execution

```http
POST /api/execution/start
```

**Request:**
```json
{
  "spec": "012-feature",
  "batch": null,
  "parallel": 3,
  "dry_run": false
}
```

**Response:**
```json
{
  "run_id": "run-abc123",
  "status": "running"
}
```

### Stop Execution

```http
POST /api/execution/stop
```

**Request:**
```json
{
  "run_id": "run-abc123"
}
```

**Response:**
```json
{
  "status": "stopped"
}
```

### Get Execution Status

```http
GET /api/execution/status
```

**Response:**
```json
{
  "run_id": "run-abc123",
  "status": "running",
  "current_batch": 2,
  "total_batches": 5,
  "completed_tasks": 8,
  "total_tasks": 15
}
```

### Get Worktree Branches

```http
GET /api/execution/branches?spec={spec_name}
```

**Response:**
```json
{
  "branches": [
    {
      "name": "task-1-feature",
      "worktree": "/path/to/worktree",
      "status": "ready"
    }
  ]
}
```

### Merge Single Branch

```http
POST /api/execution/merge
```

**Request:**
```json
{
  "branch": "task-1-feature",
  "target": "main"
}
```

### Merge All Branches

```http
POST /api/execution/merge-all
```

**Request:**
```json
{
  "spec": "012-feature"
}
```

### Get Execution Logs

```http
GET /api/execution/{id}/logs?offset=0&limit=100&since=timestamp
```

**Query Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| offset | number | No | Pagination offset |
| limit | number | No | Max results |
| since | string | No | ISO timestamp filter |

### Tail Execution Logs

```http
GET /api/execution/{id}/logs/tail?count=50
```

**Query Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| count | number | No | Number of recent logs |

---

## History

### List Run History

```http
GET /api/history/{spec}
```

**Response:**
```json
[
  {
    "run_id": "run-abc123",
    "spec": "012-feature",
    "status": "completed",
    "started_at": "2026-01-21T14:00:00Z",
    "completed_at": "2026-01-21T15:30:00Z"
  }
]
```

### Create Run

```http
POST /api/history/{spec}
```

**Request:**
```json
{
  "agent": "claude-default",
  "parallel": 3
}
```

### Get Run Details

```http
GET /api/history/{spec}/{run_id}
```

### Update Run

```http
PATCH /api/history/{spec}/{run_id}
```

### Delete Run

```http
DELETE /api/history/{spec}/{run_id}
```

**Response:** `204 No Content`

---

## Agents

### List Agents

```http
GET /api/agents
```

**Response:**
```json
{
  "agents": [
    {
      "id": "claude-default",
      "name": "Claude Code",
      "type": "claude",
      "is_default": true,
      "is_qa": false,
      "is_test_writer": false,
      "enabled": true
    }
  ]
}
```

### Get OpenRouter Models

```http
GET /api/agents/models
```

**Response:**
```json
{
  "models": [
    {"id": "anthropic/claude-3-opus", "name": "Claude 3 Opus"},
    {"id": "openai/gpt-4", "name": "GPT-4"}
  ]
}
```

### Create/Update Agent

```http
POST /api/agents/upsert
```

**Request:**
```json
{
  "name": "my-agent",
  "type": "openrouter",
  "model": "anthropic/claude-3-opus",
  "api_key_env": "OPENROUTER_API_KEY",
  "enabled": true
}
```

### Delete Agent

```http
POST /api/agents/delete
```

**Request:**
```json
{
  "name": "my-agent"
}
```

**Response:** `204 No Content`

### Set Default Agent

```http
POST /api/agents/set-default
```

**Request:**
```json
{
  "name": "claude-default"
}
```

### Set QA Agent

```http
POST /api/agents/set-qa
```

**Request:**
```json
{
  "name": "claude-qa"
}
```

### Set Test Writer Agent

```http
POST /api/agents/set-test-writer
```

**Request:**
```json
{
  "name": "claude-test"
}
```

### Test Agent

```http
POST /api/agents/test
```

**Request:**
```json
{
  "agent_name": "claude-default",
  "prompt": "Hello, can you respond?"
}
```

---

## Commands

All commands execute the corresponding CLI operation.

### Init

```http
POST /api/command/init
```

### Git Init

```http
POST /api/command/git-init
```

### Spec New

```http
POST /api/command/spec-new
```

**Request:**
```json
{
  "description": "Feature description"
}
```

### Spec Tasks

```http
POST /api/command/spec-tasks
```

### Plan

```http
POST /api/command/plan
```

### Execute

```http
POST /api/command/execute
```

### Diff

```http
POST /api/command/diff
```

**Request:**
```json
{
  "branch": "task-1-feature"
}
```

### Verify

```http
POST /api/command/verify
```

**Request:**
```json
{
  "lint": true,
  "tests": true
}
```

### Promote

```http
POST /api/command/promote
```

**Request:**
```json
{
  "push": true,
  "open": true
}
```

### Fix

```http
POST /api/command/fix
```

**Request:**
```json
{
  "auto": true
}
```

---

## Console

### Execute Command

```http
POST /api/console/exec
```

**Request:**
```json
{
  "command": "npm test",
  "cwd": "/path/to/project"
}
```

**Response:**
```json
{
  "stdout": "...",
  "stderr": "",
  "exit_code": 0
}
```

---

## Diff

### Get Diff

```http
GET /api/diff?branch={branch}&base={base}
```

**Query Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| branch | string | No | Branch to diff |
| base | string | No | Base branch (default: main) |

### Get Branches for Diff

```http
GET /api/diff/branches
```

**Response:**
```json
{
  "branches": ["task-1-feature", "task-2-fix"]
}
```

---

## QA

### Get QA Agent

```http
GET /api/qa/agent
```

**Response:**
```json
{
  "agent": {
    "name": "claude-qa",
    "type": "claude"
  }
}
```

### Run QA Review

```http
POST /api/qa/review
```

**Request:**
```json
{
  "files": ["src/main.rs"],
  "focus": "security"
}
```

**Response:**
```json
{
  "success": true,
  "review": "...",
  "error": null
}
```

### Detect Bugs

```http
POST /api/qa/bugs
```

**Request:**
```json
{
  "scope": "changed_files"
}
```

**Response:**
```json
{
  "success": true,
  "issues": [...],
  "error": null
}
```

### Generate QA Report

```http
POST /api/qa/report
```

---

## Test

### Get Test Writer Agent

```http
GET /api/test/agent
```

### Run Tests

```http
POST /api/test/run
```

**Request:**
```json
{
  "pattern": "test_*",
  "parallel": true
}
```

**Response:**
```json
{
  "success": true,
  "result": {
    "total": 50,
    "passed": 48,
    "failed": 2,
    "skipped": 0,
    "duration_ms": 5000
  },
  "error": null
}
```

### Create Test Plan

```http
POST /api/test/plan
```

### Get Test Plan Status

```http
GET /api/test/plan-status
```

### Write Tests

```http
POST /api/test/write
```

### Get Write Status

```http
GET /api/test/write-status
```

### Get Coverage

```http
GET /api/test/coverage
```

### Fix Failing Tests

```http
POST /api/test/fix
```

### Generate Tests

```http
POST /api/test/generate
```

---

## Session

### Start Session

```http
POST /api/session/start
```

**Request:**
```json
{
  "shell": "/bin/bash",
  "cwd": "/path/to/project",
  "env": {"KEY": "value"}
}
```

**Response:**
```json
{
  "session_id": "sess-abc123",
  "status": "running"
}
```

### Execute in Session

```http
POST /api/session/exec
```

**Request:**
```json
{
  "session_id": "sess-abc123",
  "command": "ls -la"
}
```

### Stop Session

```http
POST /api/session/stop
```

**Request:**
```json
{
  "session_id": "sess-abc123"
}
```

**Response:** `204 No Content`

---

## Terminal

### Start Terminal Session

```http
POST /api/terminal/start
```

**Request:**
```json
{
  "shell": "/bin/bash",
  "cwd": "/path/to/project",
  "cols": 80,
  "rows": 24
}
```

**Response:**
```json
{
  "session_id": "term-abc123",
  "status": "running"
}
```

### Stop Terminal Session

```http
POST /api/terminal/stop
```

**Request:**
```json
{
  "session_id": "term-abc123"
}
```

---

## Events (SSE)

### Event Stream

```http
GET /api/events
```

Server-Sent Events stream with heartbeat every 30 seconds.

**Event Format:**
```
event: heartbeat
data: ping
```

---

## Example

### Get Example Info

```http
GET /api/example
```

### Process Example

```http
POST /api/example
```

---

## WebSocket Endpoints

### Execution WebSocket

```
WS /api/execution/ws?run_id={run_id}
```

Real-time orchestration event stream. The connection remains open until the client disconnects or a terminal event (`Success` or `Error`) is received.

**Query Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| run_id | string | Yes | Run ID to subscribe to |

### Terminal WebSocket

```
WS /api/terminal/ws?session_id={session_id}
```

Bidirectional terminal I/O stream for PTY emulation.

**Query Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| session_id | string | Yes | Terminal session ID |

---

## WebSocket Message Types

### Orchestration Events

All events are JSON with a `type` field for discrimination.

| Type | Direction | Description |
|------|-----------|-------------|
| `log` | Server → Client | General log message |
| `stepstart` | Server → Client | Step execution started |
| `stepend` | Server → Client | Step execution completed |
| `error` | Server → Client | Error occurred (terminal) |
| `success` | Server → Client | Operation completed (terminal) |

#### Log Event

```json
{
  "type": "log",
  "message": "Building feature...",
  "timestamp": "2026-01-21T14:05:00Z",
  "metadata": null
}
```

#### StepStart Event

```json
{
  "type": "stepstart",
  "step_name": "execute",
  "timestamp": "2026-01-21T14:05:00Z"
}
```

#### StepEnd Event

```json
{
  "type": "stepend",
  "step_name": "execute",
  "timestamp": "2026-01-21T14:10:00Z",
  "status": "success"
}
```

#### Error Event

```json
{
  "type": "error",
  "message": "Execution failed",
  "timestamp": "2026-01-21T14:10:00Z"
}
```

#### Success Event

```json
{
  "type": "success",
  "message": "All tasks completed",
  "timestamp": "2026-01-21T14:10:00Z"
}
```

---

## Error Responses

All endpoints return errors in this format:

```json
{
  "error": {
    "code": "NOT_FOUND",
    "message": "Spec not found: 012-feature"
  }
}
```

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `NOT_FOUND` | 404 | Resource not found |
| `VALIDATION_ERROR` | 400 | Invalid request |
| `EXECUTION_ERROR` | 500 | Execution failed |
| `DOCKER_UNAVAILABLE` | 503 | Docker not running |

---

## Health Check

### Server Health

```http
GET /health
```

Simple health check endpoint (not under `/api` prefix).

**Response:** `200 OK` with body `OK`
