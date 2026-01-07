# Execution WebSocket API Contract

**Endpoint**: `/api/execution/ws`  
**Protocol**: WebSocket  
**Query Parameters**: `run_id` (required)

## Message Types

### Client → Server

No client-to-server messages required (execution controlled via HTTP endpoints).

### Server → Client

#### Status Update

```json
{
    "type": "status",
    "status": "running" | "completed" | "failed" | "aborted",
    "message": string | null,
    "timestamp": string (ISO 8601)
}
```

#### Log Message

```json
{
    "type": "info" | "success" | "error" | "start" | "batch_start" | "batch_complete" | "batch_error",
    "message": string,
    "stream": "stdout" | "stderr" | null,
    "timestamp": string (ISO 8601)
}
```

#### Error

```json
{
    "type": "error",
    "message": string
}
```

## Connection Lifecycle

1. Client connects with `run_id` query parameter
2. Server sends historical messages (replay from execution start)
3. Server streams new messages in real-time
4. Connection closes when:
   - Execution completes (success/failed/aborted)
   - Client disconnects
   - Server error

## Error Handling

### Connection Refused
- HTTP 404 if `run_id` not found
- Client should not retry for unknown run_id

### Connection Lost
- Client SHOULD implement auto-retry with exponential backoff
- Maximum 3 retries at 5s, 10s, 20s intervals
- Display countdown indicator during reconnection
- After 3 failures, require manual reconnection

### Message Format Error
- Log malformed message to console
- Continue processing subsequent messages
- Do not crash or disconnect
