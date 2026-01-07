# Data Model: Execution State Synchronization

## WebSocket Message Types

### Base Message Structure

All WebSocket messages share this base structure:

```typescript
interface BaseMessage {
  type: string;           // Message type identifier
  message?: string;       // Human-readable log message
  timestamp: string;      // ISO 8601 timestamp
}
```

### Status Message (NEW)

Explicit execution state transitions:

```typescript
interface StatusMessage extends BaseMessage {
  type: "status";
  status: "running" | "completed" | "failed" | "aborted";
  message?: string;       // Optional description
}
```

**State Transitions:**
```
idle → starting → running → completed
                        ↘→ failed
                        ↘→ aborted
```

### Batch Status Message (NEW)

Explicit batch state updates:

```typescript
interface BatchStatusMessage extends BaseMessage {
  type: "batch_status";
  batch_id: string;       // Batch identifier (e.g., "batch-001")
  batch_name: string;     // Human-readable name
  status: "pending" | "running" | "completed" | "failed";
  branch?: string;        // Git branch name (when completed)
  error?: string;         // Error message (when failed)
}
```

### Log Message (EXISTING - Enhanced)

General log messages for terminal output:

```typescript
interface LogMessage extends BaseMessage {
  type: "info" | "error" | "success" | "log" | "start" | "batch_start" | "batch_complete" | "batch_error";
  message: string;
  stream?: "stdout" | "stderr";
}
```

## Rust Backend Types

### LogMessage Struct (Enhanced)

```rust
#[derive(Debug, Clone, Serialize)]
pub struct LogMessage {
    #[serde(rename = "type")]
    pub type_: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<String>,
    pub timestamp: String,
    // NEW fields for explicit status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
```

### Message Factory Methods

```rust
impl LogMessage {
    /// Create a status change message
    pub fn status(status: &str) -> Self {
        Self {
            type_: "status".to_string(),
            message: String::new(),
            stream: None,
            timestamp: Utc::now().to_rfc3339(),
            status: Some(status.to_string()),
            batch_id: None,
            batch_name: None,
            branch: None,
            error: None,
        }
    }
    
    /// Create a batch status message
    pub fn batch_status(batch_id: &str, batch_name: &str, status: &str) -> Self {
        Self {
            type_: "batch_status".to_string(),
            message: String::new(),
            stream: None,
            timestamp: Utc::now().to_rfc3339(),
            status: Some(status.to_string()),
            batch_id: Some(batch_id.to_string()),
            batch_name: Some(batch_name.to_string()),
            branch: None,
            error: None,
        }
    }
}
```

## Frontend State

### ExecutionStatus

```typescript
type ExecutionStatus = 
  | 'idle'         // No execution in progress
  | 'starting'     // API request sent, waiting for WS
  | 'running'      // Execution in progress
  | 'completed'    // All batches completed
  | 'failed'       // Execution failed
  | 'aborted'      // User aborted
  | 'reconnecting'; // WS disconnected, reconnecting
```

### BatchStatus

```typescript
type BatchStatus = 
  | 'pending'      // Waiting for dependencies
  | 'running'      // In progress
  | 'completed'    // Successfully completed
  | 'failed';      // Failed with error
```

### Batch Entity

```typescript
interface Batch {
  id: string;
  name: string;
  status: BatchStatus;
  branch?: string;
  taskIds: string[];
  dependsOn: string[];
  logs: LogEntry[];
  startTime?: number;
  endTime?: number;
  error?: string;
}
```

## Message Flow Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                        BACKEND (Rust)                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  1. Start Execution                                                  │
│     ├─ send: { type: "status", status: "running" }       ◄── NEW    │
│     └─ send: { type: "start", message: "Starting..." }              │
│                                                                      │
│  2. Spawn Batch                                                      │
│     ├─ send: { type: "batch_status", batch_id, status: "running" }  │
│     └─ send: { type: "batch_start", message: "Spawning..." }        │
│                                                                      │
│  3. Batch Complete                                                   │
│     ├─ send: { type: "batch_status", batch_id, status: "completed" }│
│     └─ send: { type: "batch_complete", message: "Batch..." }        │
│                                                                      │
│  4. Execution Complete                                               │
│     ├─ send: { type: "status", status: "completed" }     ◄── NEW    │
│     └─ send: { type: "success", message: "All batches..." }         │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
                                │
                                │ WebSocket
                                ▼
┌─────────────────────────────────────────────────────────────────────┐
│                       FRONTEND (React)                               │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Message Handler:                                                    │
│                                                                      │
│  if (type === "status") {                                           │
│    setExecutionStatus(status);  // Direct state update              │
│    if (status === "running") startTimer();                          │
│  }                                                                   │
│                                                                      │
│  if (type === "batch_status") {                                     │
│    updateBatch(batch_id, status);                                   │
│    if (status === "completed") incrementCompletedCount();           │
│  }                                                                   │
│                                                                      │
│  // Fallback: Parse log messages for legacy compatibility           │
│  if (message.includes("Spawning batch")) updateBatchFromLog(...);   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Validation Rules

### StatusMessage
- `status` MUST be one of: "running", "completed", "failed", "aborted"
- `timestamp` MUST be valid ISO 8601

### BatchStatusMessage
- `batch_id` MUST be non-empty string
- `batch_name` MUST be non-empty string
- `status` MUST be one of: "pending", "running", "completed", "failed"
- `branch` REQUIRED when status is "completed"
- `error` REQUIRED when status is "failed"
