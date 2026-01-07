# Data Model: Bug-Free and Polished Chakravarti CLI UI

**Feature**: 006-bug-free-polished-ui  
**Date**: 2026-01-05

## State Machines

### ExecutionStatus

The execution lifecycle state machine with new 'reconnecting' status for WebSocket handling:

```
┌──────────┐
│   idle   │◄──────────────────────────────────────────┐
└────┬─────┘                                           │
     │ user clicks "Run"                               │
     ▼                                                 │
┌───────────┐                                          │
│ starting  │                                          │
└────┬──────┘                                          │
     │ WebSocket connected                             │
     ▼                                                 │
┌───────────┐    WS error    ┌──────────────┐         │
│  running  │───────────────►│ reconnecting │         │
└─────┬─────┘                └───────┬──────┘         │
      │                              │                 │
      │                    ┌─────────┴─────────┐      │
      │                    │                   │      │
      │               retry success       3 retries   │
      │                    │               failed     │
      │                    ▼                   │      │
      │              ┌───────────┐             │      │
      ├──────────────│  running  │             │      │
      │              └───────────┘             │      │
      │                                        │      │
      ├──── execution success ─────────────────┼──────┤
      │                                        │      │
      ▼                                        ▼      │
┌───────────┐                           ┌──────────┐  │
│ completed │                           │  failed  │──┘
└───────────┘                           └──────────┘
      │                                        │
      │ user clicks "Reset"                    │ user clicks "Reset"  
      └────────────────────────────────────────┘

┌───────────┐
│  aborted  │◄── user clicks "Stop" from 'running' state
└─────┬─────┘
      │ user clicks "Reset"
      └──────────────────────────► idle
```

**New Status**: `reconnecting` - Shows countdown timer for auto-retry attempts

### BatchStatus

Per-batch execution status:

```
┌─────────┐
│ pending │ ◄── Initial state, dependencies not met
└────┬────┘
     │ all dependencies completed
     ▼
┌─────────┐
│ waiting │ ◄── Ready to run, waiting for agent slot
└────┬────┘
     │ agent spawned
     ▼
┌─────────┐
│ running │ ◄── Active execution in progress
└────┬────┘
     │
     ├──── success ────►┌───────────┐
     │                  │ completed │
     │                  └───────────┘
     │
     └──── error ──────►┌────────┐
                        │ failed │
                        └────────┘
```

## Entity Definitions

### LogEntry

```typescript
interface LogEntry {
    time: string;           // Formatted timestamp "HH:mm:ss"
    message: string;        // Log message content
    type: LogEntryType;     // Classification for styling
    stream?: 'stdout' | 'stderr';  // Optional stream source
    batchId?: string;       // Associated batch if applicable
}

type LogEntryType = 
    | 'info'           // General information
    | 'success'        // Success message (green)
    | 'error'          // Error message (red)
    | 'start'          // Execution start (yellow)
    | 'batch_start'    // Batch spawned (cyan)
    | 'batch_complete' // Batch finished (green)
    | 'batch_error';   // Batch failed (red)
```

### WebSocketState (New)

```typescript
interface WebSocketState {
    status: 'disconnected' | 'connecting' | 'connected' | 'reconnecting';
    retryCount: number;        // Current retry attempt (0-3)
    retryCountdown: number;    // Seconds until next retry
    lastError?: string;        // Last error message
}
```

### Batch

```typescript
interface Batch {
    id: string;
    name: string;
    task_ids: string[];
    depends_on: string[];
    model_assignment: {
        default: string;
        overrides: Record<string, string>;
    };
    execution_strategy: string;
    estimated_cost: number;
    estimated_time: string;
    reasoning: string;
    status: BatchStatus;
}

type BatchStatus = 'pending' | 'waiting' | 'running' | 'completed' | 'failed';
```

### UnmergedBranch

```typescript
interface UnmergedBranch {
    name: string;           // Full branch name
    batch_name: string;     // Human-readable batch name
    ahead_commits: number;  // Commits ahead of base
    is_clean: boolean;      // Working directory status
}
```

## UI State Models

### ExecutionRunnerState

```typescript
interface ExecutionRunnerState {
    // Selection
    selectedSpecName: string | null;
    
    // Batches
    batches: Batch[];
    batchLogs: Record<string, LogEntry[]>;
    completedBatches: Set<string>;
    batchCompletedAt: Record<string, number>; // Timestamps for auto-collapse
    expandedBatchId: string | null;
    
    // Execution
    executionStatus: ExecutionStatus;
    elapsedTime: number;
    
    // WebSocket (New)
    wsState: WebSocketState;
    
    // UI
    orchestratorMinimized: boolean;
    
    // Post-execution
    unmergedBranches: UnmergedBranch[];
    isMerging: boolean;
    mergeResult: { success: boolean; message: string } | null;
}

type ExecutionStatus = 
    | 'idle' 
    | 'starting' 
    | 'running' 
    | 'reconnecting'  // NEW
    | 'completed' 
    | 'failed' 
    | 'aborted';
```

### ErrorState (New)

```typescript
interface ErrorState {
    visible: boolean;
    title: string;
    message: string;
    retryAction?: () => void;
    dismissable: boolean;
}
```

## Validation Rules

### Log Message Throttling

- Maximum 60 log messages rendered per second (aligned with requestAnimationFrame)
- Messages in excess of 60/s are batched and rendered in next frame
- Terminal output uses virtual scrolling for >1000 lines

### WebSocket Reconnection

- Maximum 3 retry attempts
- Exponential backoff: 5s → 10s → 20s
- After 3 failures, require manual reconnection
- Cancel reconnection if user navigates away

### Timer Cleanup

- All setTimeout/setInterval calls must be tracked in refs
- Cleanup on component unmount
- Cancel on state changes that invalidate the timer purpose

## Animation Tokens

```css
/* Timing */
--duration-fast: 150ms;      /* Micro-interactions */
--duration-normal: 200ms;    /* Standard transitions */
--duration-slow: 300ms;      /* Modal/panel transitions */
--duration-focus: 5000ms;    /* Auto-collapse delay */

/* Easing */
--ease-default: cubic-bezier(0.4, 0, 0.2, 1);
--ease-in: cubic-bezier(0.4, 0, 1, 1);
--ease-out: cubic-bezier(0, 0, 0.2, 1);
```
