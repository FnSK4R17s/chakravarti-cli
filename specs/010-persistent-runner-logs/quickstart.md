# Quickstart: Persistent Runner Logs

**Feature**: 010-persistent-runner-logs  
**Date**: 2026-01-15

## Overview

This feature adds persistent log storage so users can navigate away from an execution and return to see all logs generated during their absence.

## Quick Test

1. **Start the UI**:
   ```bash
   ckrv ui --port=3002
   ```

2. **Run an execution** via the UI (start a spec run)

3. **Navigate away** (switch browser tabs or go to another UI page)

4. **Return** to the execution view

5. **Verify**: All logs generated during your absence should be visible and scrollable

## Development Setup

### Backend Changes

Files to modify/create in `crates/ckrv-ui/`:

```
src/
├── api/
│   └── execution.rs    # Add log history endpoints
├── services/
│   ├── engine.rs       # Modify to write logs to disk
│   └── log_store.rs    # NEW: Log persistence service
└── models/
    └── log.rs          # NEW: Log entry types
```

### Frontend Changes

Files to modify/create in `crates/ckrv-ui/frontend/`:

```
src/
├── components/
│   ├── LogViewer.tsx        # Add history loading on reconnect
│   └── ExecutionRunner.tsx  # Handle navigation state
├── hooks/
│   └── useLogStore.ts       # NEW: Log persistence hook
└── services/
    └── logService.ts        # NEW: API client
```

## Key Implementation Steps

### 1. Create Log Storage Service (Backend)

```rust
// crates/ckrv-ui/src/services/log_store.rs

pub struct LogStore {
    base_path: PathBuf,
}

impl LogStore {
    pub fn new(project_root: &Path) -> Self {
        Self {
            base_path: project_root.join(".ckrv/logs"),
        }
    }
    
    pub async fn append(&self, execution_id: &str, entry: &LogEntry) -> Result<()>;
    pub async fn read_all(&self, execution_id: &str) -> Result<Vec<LogEntry>>;
    pub async fn read_range(&self, execution_id: &str, offset: usize, limit: usize) -> Result<Vec<LogEntry>>;
    pub async fn read_since(&self, execution_id: &str, since: DateTime<Utc>) -> Result<Vec<LogEntry>>;
    pub async fn delete(&self, execution_id: &str) -> Result<usize>;
}
```

### 2. Integrate with ExecutionEngine (Backend)

```rust
// Modify crates/ckrv-ui/src/services/engine.rs

impl ExecutionEngine {
    async fn log(&self, type_: &str, message: &str) {
        let entry = LogEntry::new(self.run_id.clone(), type_, message);
        
        // Write to disk
        if let Err(e) = self.log_store.append(&self.run_id, &entry).await {
            eprintln!("Failed to persist log: {}", e);
        }
        
        // Broadcast to WebSocket (existing behavior)
        let _ = self.sender.send(LogMessage::from(&entry)).await;
    }
}
```

### 3. Add History Loading (Frontend)

```typescript
// frontend/src/hooks/useLogStore.ts

export function useLogHistory(executionId: string) {
  return useQuery({
    queryKey: ['logs', executionId],
    queryFn: () => fetch(`/api/execution/${executionId}/logs`).then(r => r.json()),
    enabled: !!executionId,
  });
}
```

### 4. Handle Reconnection (Frontend)

```typescript
// frontend/src/components/LogViewer.tsx

// On component mount, check if we have a last known timestamp
// If so, request logs since that timestamp
// Merge with existing logs in state
```

## Testing

### Unit Tests

```bash
# Backend
cd crates/ckrv-ui
cargo test log_store

# Frontend
cd frontend
npm test -- --grep "LogViewer"
```

### Integration Tests

```bash
# Full E2E test
ckrv ui --port=3002 &
# ... manually test navigation scenario
```

## Storage Location

Logs are stored in:
```
.ckrv/
└── logs/
    ├── .gitkeep
    └── {execution_id}/
        └── log.jsonl
```

This folder is gitignored and local to the project.

## Cleanup

Logs are automatically cleaned when:
- All worktrees are merged (via merge flow)
- User manually deletes via API or UI

No time-based retention.
