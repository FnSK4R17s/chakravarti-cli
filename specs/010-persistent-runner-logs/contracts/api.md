# API Contracts: Persistent Runner Logs

**Feature**: 010-persistent-runner-logs  
**Date**: 2026-01-15

## New REST Endpoints

### GET /api/execution/{execution_id}/logs

Fetch historical logs for an execution with pagination.

**Request**:
```http
GET /api/execution/run-abc123/logs?offset=0&limit=100
```

**Query Parameters**:
| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| offset | integer | No | 0 | Line offset to start from |
| limit | integer | No | 100 | Maximum lines to return (max: 1000) |
| since | string | No | null | ISO 8601 timestamp, only return logs after this time |

**Response (200 OK)**:
```json
{
  "execution_id": "run-abc123",
  "logs": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "execution_id": "run-abc123",
      "timestamp": "2026-01-15T17:00:00Z",
      "level": "start",
      "message": "Starting execution for spec: my-feature",
      "source": null
    },
    {
      "id": "550e8400-e29b-41d4-a716-446655440001",
      "execution_id": "run-abc123",
      "timestamp": "2026-01-15T17:00:01Z",
      "level": "batch_start",
      "message": "Spawning batch: setup",
      "source": "setup"
    }
  ],
  "total_count": 1523,
  "offset": 0,
  "has_more": true
}
```

**Response (404 Not Found)**:
```json
{
  "error": "Execution not found",
  "execution_id": "run-abc123"
}
```

---

### GET /api/execution/{execution_id}/logs/tail

Fetch the most recent N logs for an execution.

**Request**:
```http
GET /api/execution/run-abc123/logs/tail?count=10
```

**Query Parameters**:
| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| count | integer | No | 10 | Number of recent logs to return (max: 100) |

**Response (200 OK)**:
```json
{
  "execution_id": "run-abc123",
  "logs": [
    {
      "id": "550e8400-e29b-41d4-a716-446655441500",
      "execution_id": "run-abc123",
      "timestamp": "2026-01-15T17:05:30Z",
      "level": "log",
      "message": "Compiling crate...",
      "source": "build"
    }
  ],
  "total_count": 1523
}
```

---

### DELETE /api/execution/{execution_id}/logs

Manually delete logs for an execution.

**Request**:
```http
DELETE /api/execution/run-abc123/logs
```

**Response (200 OK)**:
```json
{
  "success": true,
  "execution_id": "run-abc123",
  "deleted_lines": 1523
}
```

**Response (404 Not Found)**:
```json
{
  "error": "Logs not found",
  "execution_id": "run-abc123"
}
```

---

## WebSocket Protocol Extensions

### Endpoint: /api/execution/ws?run_id={run_id}

**Existing behavior**: Streams logs in real-time via WebSocket.

**New behavior**: 
1. On connect, if `last_timestamp` query param provided, send historical logs first
2. Send `history_complete` message to signal end of backfill
3. Continue with normal real-time streaming

### Connect with History Backfill

**Request**:
```
ws://localhost:3002/api/execution/ws?run_id=run-abc123&last_timestamp=2026-01-15T17:00:00Z
```

### Messages from Server

**Historical log batch** (sent during backfill):
```json
{
  "type": "history",
  "logs": [
    {"id": "...", "timestamp": "...", "level": "log", "message": "..."}
  ]
}
```

**History complete signal**:
```json
{
  "type": "history_complete",
  "count": 150
}
```

**Real-time log** (existing format, unchanged):
```json
{
  "type": "log",
  "timestamp": "2026-01-15T17:05:35Z",
  "level": "log",
  "message": "Building module..."
}
```

**Status update** (existing format, unchanged):
```json
{
  "type": "status",
  "status": "running" | "completed" | "failed",
  "message": "Execution completed successfully"
}
```

---

## Error Codes

| Code | Description |
|------|-------------|
| 404 | Execution or logs not found |
| 400 | Invalid parameters (negative offset, etc.) |
| 500 | File I/O error reading/writing logs |

---

## Rate Limiting

No rate limiting for local UI access. All endpoints are localhost-only.

---

## CORS

Same-origin only (frontend and backend on same port).
