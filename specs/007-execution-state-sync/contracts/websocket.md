# WebSocket Message Contract

## Connection

**Endpoint**: `ws://localhost:{port}/api/execution/ws?run_id={run_id}`

## Messages: Server → Client

### StatusMessage

Sent when overall execution state changes.

```json
{
  "type": "status",
  "status": "running" | "completed" | "failed" | "aborted",
  "message": "optional description",
  "timestamp": "2026-01-06T16:00:00Z"
}
```

**Trigger Points**:
- `status: "running"` - Immediately after execution starts (before first log)
- `status: "completed"` - After all batches complete and merge
- `status: "failed"` - On any unrecoverable error
- `status: "aborted"` - When user stops execution

### BatchStatusMessage

Sent when a batch state changes.

```json
{
  "type": "batch_status",
  "batch_id": "batch-001",
  "batch_name": "Database Foundation",
  "status": "pending" | "running" | "completed" | "failed",
  "branch": "ckrv-batch-database-foundation-a1b2c3",
  "error": "optional error message",
  "timestamp": "2026-01-06T16:00:00Z"
}
```

**Trigger Points**:
- `status: "running"` - When batch starts execution
- `status: "completed"` - After batch finishes and merges successfully
- `status: "failed"` - On batch error

### LogMessage

General log output for terminal display.

```json
{
  "type": "info" | "error" | "success" | "log" | "start" | "batch_start" | "batch_complete",
  "message": "Log message content",
  "stream": "stdout" | "stderr",
  "timestamp": "2026-01-06T16:00:00Z"
}
```

## Messages: Client → Server

### StopCommand

```json
{
  "command": "stop"
}
```

## Message Sequence

### Successful Execution

```
Server: { type: "status", status: "running" }
Server: { type: "start", message: "Starting execution for spec: X" }
Server: { type: "batch_status", batch_id: "1", status: "running" }
Server: { type: "batch_start", message: "Spawning batch: Auth" }
Server: { type: "log", message: "..." }  // Multiple log messages
Server: { type: "batch_status", batch_id: "1", status: "completed", branch: "..." }
Server: { type: "batch_complete", message: "Batch 1 completed on branch ..." }
// ... more batches ...
Server: { type: "status", status: "completed" }
Server: { type: "success", message: "All batches completed successfully." }
```

### Failed Execution

```
Server: { type: "status", status: "running" }
Server: { type: "start", message: "Starting execution for spec: X" }
Server: { type: "batch_status", batch_id: "1", status: "running" }
Server: { type: "log", message: "..." }
Server: { type: "batch_status", batch_id: "1", status: "failed", error: "Task failed" }
Server: { type: "batch_error", message: "Batch failed: Task failed" }
Server: { type: "status", status: "failed" }
Server: { type: "error", message: "Execution failed" }
```

## Error Handling

- If `run_id` not found: `{ type: "error", message: "Execution not found." }`
- If WebSocket disconnects: Client should attempt reconnection with exponential backoff
- All messages MUST include `timestamp` field
