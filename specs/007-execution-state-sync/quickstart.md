# Quickstart: Testing Execution State Synchronization Fix

## Prerequisites

- Chakravarti CLI built and running (`ckrv ui --port 3002`)
- A spec with plan.yaml (batches defined)
- Browser dev tools open for WebSocket inspection

## Testing Steps

### 1. Verify Timer Starts

1. Navigate to **Runner** page
2. Select a spec with batches
3. Click **Run Execution**
4. **Expected**: Timer should start counting from 0s upward
5. **Verify**: In dev tools Network → WS, look for `{ type: "status", status: "running" }`

### 2. Verify Batch Counter Updates

1. Start an execution with multiple batches
2. Watch the "X/Y BATCHES COMPLETED" counter
3. **Expected**: Counter increments as each batch completes
4. **Verify**: Look for `{ type: "batch_status", status: "completed" }` messages

### 3. Verify Execution Completes

1. Let execution run to completion
2. **Expected**: 
   - Timer stops
   - Status shows "Completed"
   - All batch cards show ✓
3. **Verify**: Look for `{ type: "status", status: "completed" }` message

### 4. Verify Error Handling

1. Cause a batch to fail (e.g., invalid spec)
2. **Expected**:
   - Affected batch shows error state
   - Execution status shows "Failed"
   - Timer stops
3. **Verify**: Look for `{ type: "status", status: "failed" }` message

## Manual WebSocket Testing

Open browser console and run:

```javascript
// Connect to execution WebSocket
const ws = new WebSocket('ws://localhost:3002/api/execution/ws?run_id=YOUR_RUN_ID');

ws.onmessage = (e) => {
  const data = JSON.parse(e.data);
  console.log('Message type:', data.type, 'Status:', data.status || data.message?.substring(0, 50));
};

// Should see:
// Message type: status Status: running
// Message type: batch_status Status: running
// Message type: log Status: ...
// Message type: batch_status Status: completed
// Message type: status Status: completed
```

## Verification Checklist

- [ ] Timer starts when Run Execution clicked
- [ ] Timer shows increasing seconds
- [ ] Batch counter starts at 0/N
- [ ] Batch counter increments on completion
- [ ] Batch cards transition: pending → running → completed
- [ ] Final status shows "Completed" when all batches done
- [ ] Final status shows "Failed" on error
- [ ] Stop button works and shows "Aborted" state

## Debugging

### Timer Not Starting
1. Check WebSocket connection in Network tab
2. Look for `{ type: "status", status: "running" }` message
3. If missing, backend not sending status message

### Counter Not Updating
1. Check for `{ type: "batch_status" }` messages
2. Check for `type: "batch_complete"` log messages
3. Verify batch name matches between plan and messages

### Messages Not Appearing
1. Check if execution started (`run_id` in URL)
2. Check backend logs for errors
3. Verify WebSocket handshake completed

## Expected Message Sequence

```
1. [HTTP POST] /api/execution/run → { run_id: "abc123" }
2. [WS Connect] /api/execution/ws?run_id=abc123
3. [WS Message] { type: "status", status: "running" }
4. [WS Message] { type: "start", message: "Starting execution..." }
5. [WS Message] { type: "batch_status", batch_id: "1", status: "running" }
6. [WS Message] { type: "batch_start", message: "Spawning batch: X" }
7. [WS Message] { type: "log", message: "..." }  // Multiple
8. [WS Message] { type: "batch_status", batch_id: "1", status: "completed" }
9. [WS Message] { type: "batch_complete", message: "Batch 1 completed..." }
10. // ... repeat for each batch ...
11. [WS Message] { type: "status", status: "completed" }
12. [WS Message] { type: "success", message: "All batches completed" }
```
