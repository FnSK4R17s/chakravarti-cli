# API Contracts: Run History

## Endpoints

### GET /api/history/{spec}

List all runs for a specification.

**Path Parameters**:
- `spec` (string, required): Specification name (e.g., "001-build-todo-list")

**Query Parameters**:
- `limit` (integer, optional): Max runs to return (default: 50)
- `offset` (integer, optional): Pagination offset (default: 0)
- `status` (string, optional): Filter by status (running, completed, failed, aborted)

**Response 200 OK**:
```json
{
  "success": true,
  "spec_name": "001-build-todo-list",
  "total_count": 15,
  "runs": [
    {
      "id": "run-2026-01-07-abc123",
      "spec_name": "001-build-todo-list",
      "started_at": "2026-01-07T10:30:00Z",
      "ended_at": "2026-01-07T10:45:00Z",
      "status": "completed",
      "dry_run": false,
      "elapsed_seconds": 900,
      "summary": {
        "total_batches": 2,
        "completed_batches": 2,
        "failed_batches": 0,
        "pending_batches": 0,
        "tasks_completed": 12,
        "branches_merged": 2
      }
    }
  ]
}
```

**Response 404 Not Found** (spec doesn't exist):
```json
{
  "success": false,
  "error": "Specification not found: invalid-spec"
}
```

**Response 200 OK** (no history file, empty runs):
```json
{
  "success": true,
  "spec_name": "001-build-todo-list",
  "total_count": 0,
  "runs": []
}
```

---

### GET /api/history/{spec}/{run_id}

Get details for a single run.

**Path Parameters**:
- `spec` (string, required): Specification name
- `run_id` (string, required): Run ID

**Response 200 OK**:
```json
{
  "success": true,
  "run": {
    "id": "run-2026-01-07-abc123",
    "spec_name": "001-build-todo-list",
    "started_at": "2026-01-07T10:30:00Z",
    "ended_at": "2026-01-07T10:45:00Z",
    "status": "completed",
    "dry_run": false,
    "elapsed_seconds": 900,
    "batches": [
      {
        "id": "batch-001",
        "name": "Database Foundation",
        "status": "completed",
        "started_at": "2026-01-07T10:30:15Z",
        "ended_at": "2026-01-07T10:35:00Z",
        "branch": "ckrv-batch-database-a1b2c3",
        "merged": true,
        "error": null
      }
    ],
    "summary": {
      "total_batches": 2,
      "completed_batches": 2,
      "failed_batches": 0,
      "pending_batches": 0,
      "tasks_completed": 12,
      "branches_merged": 2
    },
    "error": null
  }
}
```

**Response 404 Not Found**:
```json
{
  "success": false,
  "error": "Run not found: run-2026-01-07-abc123"
}
```

---

### POST /api/history/{spec}

Create a new run entry (called when execution starts).

**Path Parameters**:
- `spec` (string, required): Specification name

**Request Body**:
```json
{
  "run_id": "run-2026-01-07-abc123",
  "dry_run": false,
  "batches": [
    { "id": "batch-001", "name": "Database Foundation" },
    { "id": "batch-002", "name": "Authentication" }
  ]
}
```

**Response 201 Created**:
```json
{
  "success": true,
  "run_id": "run-2026-01-07-abc123",
  "started_at": "2026-01-07T10:30:00Z"
}
```

**Response 409 Conflict** (another run is in progress):
```json
{
  "success": false,
  "error": "Another run is already in progress",
  "existing_run_id": "run-2026-01-07-xyz789",
  "existing_started_at": "2026-01-07T10:00:00Z"
}
```

---

### PATCH /api/history/{spec}/{run_id}

Update run or batch status (called during and after execution).

**Path Parameters**:
- `spec` (string, required): Specification name
- `run_id` (string, required): Run ID

**Request Body** (update run status):
```json
{
  "status": "completed",
  "ended_at": "2026-01-07T10:45:00Z",
  "summary": {
    "total_batches": 2,
    "completed_batches": 2,
    "failed_batches": 0,
    "pending_batches": 0,
    "tasks_completed": 12,
    "branches_merged": 2
  }
}
```

**Request Body** (update batch status):
```json
{
  "batch_update": {
    "batch_id": "batch-001",
    "status": "completed",
    "ended_at": "2026-01-07T10:35:00Z",
    "branch": "ckrv-batch-database-a1b2c3",
    "merged": true
  }
}
```

**Response 200 OK**:
```json
{
  "success": true,
  "updated_at": "2026-01-07T10:45:00Z"
}
```

**Response 404 Not Found**:
```json
{
  "success": false,
  "error": "Run not found: run-2026-01-07-abc123"
}
```

---

### DELETE /api/history/{spec}/{run_id}

Delete a run from history.

**Path Parameters**:
- `spec` (string, required): Specification name
- `run_id` (string, required): Run ID

**Response 200 OK**:
```json
{
  "success": true,
  "deleted_run_id": "run-2026-01-07-abc123"
}
```

**Response 400 Bad Request** (cannot delete running run):
```json
{
  "success": false,
  "error": "Cannot delete a running execution. Stop it first."
}
```

**Response 404 Not Found**:
```json
{
  "success": false,
  "error": "Run not found: run-2026-01-07-abc123"
}
```

---

## WebSocket Messages

Existing execution WebSocket (`/api/execution/ws`) will be enhanced to include run_id for history correlation.

### New Message Types

**Run Started** (after run entry created):
```json
{
  "type": "run_started",
  "run_id": "run-2026-01-07-abc123",
  "started_at": "2026-01-07T10:30:00Z"
}
```

**Run Completed**:
```json
{
  "type": "run_completed",
  "run_id": "run-2026-01-07-abc123",
  "status": "completed",
  "summary": {
    "total_batches": 2,
    "completed_batches": 2,
    "failed_batches": 0,
    "tasks_completed": 12,
    "branches_merged": 2
  }
}
```

---

## Error Codes

| HTTP Status | Error Type | Description |
|-------------|------------|-------------|
| 200 | - | Success |
| 201 | - | Created |
| 400 | invalid_request | Malformed request body |
| 404 | not_found | Spec or run doesn't exist |
| 409 | conflict | Run already in progress |
| 500 | internal_error | File I/O or unexpected error |
