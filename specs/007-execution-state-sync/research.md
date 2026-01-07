# Research: Execution State Synchronization

## Problem Analysis

### Issue 1: Timer Not Starting

**Symptom**: Elapsed time shows "0s" even though execution is in progress.

**Root Cause**:
- Frontend `ExecutionRunner.tsx` line 496-511: Timer only starts when `executionStatus === 'running'`
- `executionStatus` is set to `'running'` only at line 626 when receiving `{ type: "status", status: "running" }`
- Backend `engine.rs` line 130: Sends `{ type_: "start", message: "Starting execution for spec: ..." }`
- **Mismatch**: Backend sends `type_: "start"`, frontend expects `type: "status", status: "running"`

**Decision**: Modify backend to send explicit status message
**Rationale**: Backend should be authoritative about execution state
**Alternatives Considered**: 
- Frontend-only fix (infer running from "start" message) - Rejected: Less reliable, state should be explicit

### Issue 2: Batch Completion Counter Stuck at 0

**Symptom**: Shows "0/14 BATCHES COMPLETED" despite individual batches showing complete.

**Root Cause**:
- Frontend `ExecutionRunner.tsx` lines 543-564: Uses regex patterns to detect batch completion
- Expected patterns:
  - `/Spawning batch:\s*(.+)/i` - for starting
  - `/Mission completed:\s*(.+)/i` - for completion
  - `/Successfully merged batch\s*'?([^']+)'?/i` - for completion
- Backend `engine.rs` line 235: Sends `"Batch {} completed on branch {}"`
- **Mismatch**: Backend format doesn't match any frontend regex

**Decision**: Two-pronged approach:
1. Add frontend regex to match actual backend format
2. Backend sends explicit `{ type: "batch_complete", batch_id, status }` messages

**Rationale**: Defense in depth - both log parsing and explicit status
**Alternatives Considered**:
- Log format change only - Rejected: Breaks existing log readability
- Regex-only fix - Rejected: Fragile, depends on message wording

### Issue 3: Execution Never Completes

**Symptom**: Execution status never transitions to "completed" even when all batches finish.

**Root Cause**:
- Frontend line 629-631: Expects `{ type: "status", status: "completed" }`
- Backend line 267: Sends `{ type_: "success", message: "All batches completed successfully." }`
- **Mismatch**: Frontend expects `status: "completed"`, backend sends `type_: "success"`

**Decision**: Backend sends proper status message before success log
**Rationale**: Explicit state transitions are more reliable than parsing log messages

## Message Format Standardization

### Current Backend Messages (engine.rs)

| Event | Current Format | Line |
|-------|---------------|------|
| Execution start | `{ type_: "start", message: "Starting execution..." }` | 130 |
| Batch spawn | `{ type_: "batch_start", message: "Spawning batch: X" }` | 197 |
| Batch complete | `{ type_: "batch_complete", message: "Batch X completed on branch Y" }` | 235 |
| Execution success | `{ type_: "success", message: "All batches completed..." }` | 267 |
| Errors | `{ type_: "error", message: "..." }` | various |

### Required Status Messages (NEW)

| Event | Required Format | Purpose |
|-------|-----------------|---------|
| Execution started | `{ type: "status", status: "running" }` | Start timer, update UI state |
| Execution completed | `{ type: "status", status: "completed" }` | Stop timer, update UI state |
| Execution failed | `{ type: "status", status: "failed" }` | Stop timer, show error state |

### Enhanced Log Messages (ENHANCED)

| Event | Enhanced Format | Purpose |
|-------|-----------------|---------|
| Batch start | `{ type: "batch_status", batch_id: "X", status: "running" }` | Update batch card |
| Batch complete | `{ type: "batch_status", batch_id: "X", status: "completed" }` | Update batch card + counter |
| Batch failed | `{ type: "batch_status", batch_id: "X", status: "failed", error: "..." }` | Show batch error |

## Frontend Parsing Enhancement

### New Regex Patterns to Add

```typescript
// Match backend's actual batch completion format
const batchCompleteMatch = message.match(/Batch\s+(\S+)\s+completed on branch\s+(\S+)/i);

// Match backend's batch start format
const batchStartMatch = message.match(/Spawning batch:\s*(.+)/i);
```

### Message Type Handling Enhancement

```typescript
// Handle explicit status messages (NEW)
if (data.type === 'status') {
  setExecutionStatus(data.status);
}

// Handle explicit batch status messages (NEW)
if (data.type === 'batch_status') {
  updateBatchStatus(data.batch_id, data.status);
}

// ALSO handle legacy log messages (EXISTING - enhanced)
if (data.type === 'start' || data.message?.includes('Starting execution')) {
  setExecutionStatus('running'); // Fallback for old backend
}
```

## Implementation Approach

### Backend Changes (engine.rs)

1. After `self.log("start", ...)` on line 130, add:
   ```rust
   self.sender.send(LogMessage { 
       type_: "status".to_string(),
       status: Some("running".to_string()),
       ...
   }).await;
   ```

2. After `self.log("success", ...)` on line 267, add:
   ```rust
   self.sender.send(LogMessage { 
       type_: "status".to_string(),
       status: Some("completed".to_string()),
       ...
   }).await;
   ```

3. Enhance `LogMessage` struct to include optional `status` and `batch_id` fields.

### Frontend Changes (ExecutionRunner.tsx)

1. Add handling for `type: "status"` messages (already exists at line 624-641, just need backend to send)

2. Add regex pattern for backend's batch completion format:
   ```typescript
   const batchCompleteMatch = message.match(/Batch\s+(\S+)\s+completed on branch\s+(\S+)/i);
   ```

3. Add handling for `type: "batch_status"` explicit messages as fallback.

4. Add fallback: If `type: "start"` received, also set status to running.

## Testing Strategy

1. **Unit Tests**: Test message parsing functions with all format variations
2. **Integration Tests**: Verify WebSocket message flow end-to-end
3. **Manual Testing**: Run actual execution and verify timer/counter work

## Backward Compatibility

- All existing log message formats preserved
- New status messages added alongside, not replacing
- Frontend handles both old (log parsing) and new (explicit status) approaches
