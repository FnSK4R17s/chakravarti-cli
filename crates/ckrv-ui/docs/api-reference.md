---
last_commit: c1bb442
last_updated: 2026-01-21
related_files:
  - src/api/mod.rs
  - src/api/specs.rs
  - src/api/execution.rs
  - src/api/agents.rs
---

# API Reference: ckrv-ui

## Base URL

```
http://localhost:3000/api
```

## Authentication

Currently no authentication required (local only).

---

## Specs

### List Specs

```http
GET /api/specs
```

**Response:**
```json
{
  "specs": [
    {
      "id": "012-code-documentation",
      "name": "Comprehensive Code Documentation",
      "status": "in_progress",
      "created_at": "2026-01-21T14:00:00Z"
    }
  ]
}
```

### Get Spec

```http
GET /api/specs/:id
```

**Response:**
```json
{
  "id": "012-code-documentation",
  "name": "Comprehensive Code Documentation",
  "content": "# Feature Specification...",
  "status": "in_progress"
}
```

---

## Plans

### Get Plan

```http
GET /api/plans/:spec_id
```

**Response:**
```json
{
  "spec_id": "012-code-documentation",
  "content": "# Implementation Plan...",
  "phases": ["setup", "core", "integration"],
  "tasks_count": 15
}
```

### Generate Plan

```http
POST /api/plans/:spec_id/generate
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
  "spec_id": "012-code-documentation",
  "batch": null,
  "parallel": 3
}
```

**Response:**
```json
{
  "job_id": "job-abc123",
  "status": "running"
}
```

### Stop Execution

```http
POST /api/execution/stop
```

### Get Execution Status

```http
GET /api/execution/status
```

**Response:**
```json
{
  "job_id": "job-abc123",
  "status": "running",
  "current_batch": 2,
  "total_batches": 5,
  "completed_tasks": 8,
  "total_tasks": 15
}
```

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
      "enabled": true
    }
  ]
}
```

### Update Agent

```http
PUT /api/agents/:id
```

**Request:**
```json
{
  "enabled": true,
  "is_default": false
}
```

---

## Tasks

### List Tasks

```http
GET /api/tasks/:spec_id
```

### Retry Task

```http
POST /api/tasks/:task_id/retry
```

---

## History

### List Runs

```http
GET /api/history
```

### Get Run Details

```http
GET /api/history/:run_id
```

---

## Terminal / Logs

### Stream Logs (WebSocket)

```
WS /api/terminal/logs
```

**Message format:**
```json
{
  "type": "log",
  "batch_id": "batch-1",
  "content": "Building feature...",
  "timestamp": "2026-01-21T14:05:00Z"
}
```

### Get Session Logs

```http
GET /api/console/:session_id
```

---

## Status

### Health Check

```http
GET /api/status/health
```

**Response:**
```json
{
  "status": "healthy",
  "version": "0.1.0"
}
```

### Docker Status

```http
GET /api/docker/status
```

**Response:**
```json
{
  "available": true,
  "version": "24.0.5"
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
