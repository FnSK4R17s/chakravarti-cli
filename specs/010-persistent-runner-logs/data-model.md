# Data Model: Persistent Runner Logs

**Feature**: 010-persistent-runner-logs  
**Date**: 2026-01-15

## Entities

### LogEntry

A single log entry generated during execution.

```rust
/// A single log entry persisted to disk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Unique identifier for this log entry
    pub id: Uuid,
    
    /// ID of the execution run this log belongs to
    pub execution_id: String,
    
    /// When this log was generated (UTC)
    pub timestamp: DateTime<Utc>,
    
    /// Log level/type
    pub level: LogLevel,
    
    /// The log message content
    pub message: String,
    
    /// Optional source identifier (batch name, component, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// Log level/type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LogLevel {
    /// Informational messages
    Info,
    /// Warning messages
    Warning,
    /// Error messages
    Error,
    /// Generic log output
    Log,
    /// Execution started
    Start,
    /// Batch started
    BatchStart,
    /// Batch completed successfully
    BatchComplete,
    /// Batch failed
    BatchError,
    /// Execution completed successfully
    Success,
    /// Status update
    Status,
}
```

### ExecutionLogFile

Metadata about a log file on disk.

```rust
/// Metadata about a log file for an execution
#[derive(Debug, Clone)]
pub struct ExecutionLogFile {
    /// The execution ID this log file belongs to
    pub execution_id: String,
    
    /// Path to the log file on disk
    pub path: PathBuf,
    
    /// Number of log lines in the file
    pub line_count: usize,
    
    /// When the log file was created
    pub created_at: DateTime<Utc>,
    
    /// When the log file was last modified
    pub last_modified: DateTime<Utc>,
    
    /// Total size in bytes
    pub size_bytes: u64,
}
```

### LogHistoryRequest

API request for fetching historical logs.

```rust
/// Request to fetch historical logs
#[derive(Debug, Deserialize)]
pub struct LogHistoryRequest {
    /// Start from this line offset (0-indexed)
    #[serde(default)]
    pub offset: usize,
    
    /// Maximum number of lines to return
    #[serde(default = "default_limit")]
    pub limit: usize,
    
    /// If provided, only return logs after this timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<DateTime<Utc>>,
}

fn default_limit() -> usize { 100 }
```

### LogHistoryResponse

API response with historical logs.

```rust
/// Response containing historical logs
#[derive(Debug, Serialize)]
pub struct LogHistoryResponse {
    /// The execution ID
    pub execution_id: String,
    
    /// The log entries
    pub logs: Vec<LogEntry>,
    
    /// Total number of logs in the file
    pub total_count: usize,
    
    /// Offset used for this request
    pub offset: usize,
    
    /// Whether there are more logs after this batch
    pub has_more: bool,
}
```

## File Format

### JSONL Structure

Each line in the log file is a complete JSON object:

```jsonl
{"id":"550e8400-e29b-41d4-a716-446655440000","execution_id":"run-123","timestamp":"2026-01-15T17:00:00Z","level":"start","message":"Starting execution for spec: my-feature","source":null}
{"id":"550e8400-e29b-41d4-a716-446655440001","execution_id":"run-123","timestamp":"2026-01-15T17:00:01Z","level":"batch_start","message":"Spawning batch: setup","source":"setup"}
{"id":"550e8400-e29b-41d4-a716-446655440002","execution_id":"run-123","timestamp":"2026-01-15T17:00:05Z","level":"log","message":"Installing dependencies...","source":"setup"}
```

### Storage Layout

```
.ckrv/
├── logs/
│   ├── .gitkeep
│   ├── run-abc123/
│   │   ├── log.jsonl          # Main log file
│   │   └── metadata.json      # Optional: execution metadata
│   └── run-def456/
│       └── log.jsonl
```

## Relationships

```
┌────────────────┐       ┌──────────────────┐
│   Execution    │──────▶│ ExecutionLogFile │
│  (existing)    │  1:1  │   (new)          │
└────────────────┘       └──────────────────┘
                                  │
                                  │ 1:N
                                  ▼
                         ┌──────────────────┐
                         │    LogEntry      │
                         │   (new)          │
                         └──────────────────┘
```

## Validation Rules

1. **LogEntry.id**: Must be a valid UUID v4
2. **LogEntry.execution_id**: Must be non-empty, alphanumeric with hyphens
3. **LogEntry.timestamp**: Must be valid UTC datetime
4. **LogEntry.message**: May be empty, no max length enforced
5. **LogEntry.level**: Must be one of the defined variants

## State Transitions

Log files follow a simple lifecycle:

```
[Created] → [Appending] → [Closed] → [Deleted]
     │           │            │
     │           │            └── When worktrees merged
     │           └── Execution completes/fails
     └── Execution starts
```

## Frontend Types (TypeScript)

```typescript
interface LogEntry {
  id: string;
  execution_id: string;
  timestamp: string; // ISO 8601
  level: LogLevel;
  message: string;
  source?: string;
}

type LogLevel = 
  | 'info' 
  | 'warning' 
  | 'error' 
  | 'log' 
  | 'start' 
  | 'batch_start' 
  | 'batch_complete' 
  | 'batch_error' 
  | 'success' 
  | 'status';

interface LogHistoryResponse {
  execution_id: string;
  logs: LogEntry[];
  total_count: number;
  offset: number;
  has_more: boolean;
}
```

## Migration Notes

- No database migration needed (file-based storage)
- Existing `LogMessage` in engine.rs should be converted to `LogEntry`
- Backwards compatible: old executions without logs simply have no history
