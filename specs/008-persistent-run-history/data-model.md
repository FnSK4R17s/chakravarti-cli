# Data Model: Persistent Run History

## Entities

### Run

Represents a single execution attempt for a specification.

```yaml
# Rust struct (backend)
Run:
  id: String              # Unique run ID (e.g., "run-2026-01-07-abc123")
  spec_name: String       # Name of the specification
  started_at: DateTime    # ISO 8601 timestamp
  ended_at: Option<DateTime>  # Null if still running
  status: RunStatus       # Enum: pending, running, completed, failed, aborted
  dry_run: bool           # Whether this was a dry run
  elapsed_seconds: Option<u64>  # Total elapsed time
  batches: Vec<BatchResult>     # List of batch outcomes
  summary: RunSummary     # Aggregate statistics
  error: Option<String>   # Top-level error message if failed
```

```typescript
// TypeScript interface (frontend)
interface Run {
  id: string;
  spec_name: string;
  started_at: string;      // ISO 8601
  ended_at: string | null;
  status: RunStatus;
  dry_run: boolean;
  elapsed_seconds: number | null;
  batches: BatchResult[];
  summary: RunSummary;
  error: string | null;
}

type RunStatus = 'pending' | 'running' | 'completed' | 'failed' | 'aborted';
```

### BatchResult

Represents the outcome of a single batch within a run.

```yaml
# Rust struct
BatchResult:
  id: String              # Batch ID from plan
  name: String            # Human-readable batch name
  status: BatchStatus     # Enum: pending, running, completed, failed
  started_at: Option<DateTime>
  ended_at: Option<DateTime>
  branch: Option<String>  # Git branch name (when created)
  merged: bool            # Whether branch was merged
  error: Option<String>   # Error message if failed
```

```typescript
// TypeScript interface
interface BatchResult {
  id: string;
  name: string;
  status: BatchStatus;
  started_at: string | null;
  ended_at: string | null;
  branch: string | null;
  merged: boolean;
  error: string | null;
}

type BatchStatus = 'pending' | 'running' | 'completed' | 'failed';
```

### RunSummary

Aggregate statistics for quick display.

```yaml
# Rust struct
RunSummary:
  total_batches: u32
  completed_batches: u32
  failed_batches: u32
  pending_batches: u32
  tasks_completed: u32
  branches_merged: u32
```

```typescript
// TypeScript interface
interface RunSummary {
  total_batches: number;
  completed_batches: number;
  failed_batches: number;
  pending_batches: number;
  tasks_completed: number;
  branches_merged: number;
}
```

### RunHistory

Collection of runs for a specification (the root of runs.yaml).

```yaml
# File: .specs/<spec-name>/runs.yaml
RunHistory:
  version: "1.0"          # Schema version for future migrations
  spec_name: String
  runs: Vec<Run>          # List of runs, newest first
```

## State Transitions

### Run Status Transitions

```
┌─────────┐
│ pending │ ─── start execution ──→ ┌─────────┐
└─────────┘                          │ running │
                                     └────┬────┘
                                          │
                    ┌─────────────────────┼─────────────────────┐
                    │                     │                     │
                    ▼                     ▼                     ▼
             ┌───────────┐         ┌──────────┐         ┌─────────┐
             │ completed │         │  failed  │         │ aborted │
             └───────────┘         └──────────┘         └─────────┘
             (all batches ok)     (any batch failed)   (user stopped)
```

### Batch Status Transitions

```
┌─────────┐
│ pending │ ─── spawn ──→ ┌─────────┐
└─────────┘               │ running │
                          └────┬────┘
                               │
                  ┌────────────┴────────────┐
                  │                         │
                  ▼                         ▼
           ┌───────────┐             ┌──────────┐
           │ completed │             │  failed  │
           └───────────┘             └──────────┘
           (exit code 0)            (exit code != 0)
```

## Validation Rules

### Run

- `id` MUST be unique within the runs.yaml file
- `id` MUST match pattern: `run-YYYY-MM-DD-[a-z0-9]{6}`
- `started_at` MUST be valid ISO 8601 timestamp
- `ended_at` MUST be null if status is "pending" or "running"
- `ended_at` MUST be non-null if status is "completed", "failed", or "aborted"
- `elapsed_seconds` MUST equal `ended_at - started_at` when both present
- `batches` MUST not be empty

### BatchResult

- `id` MUST match a batch ID from the plan.yaml
- `started_at` MUST be null if status is "pending"
- `branch` MUST be non-null if status is "completed" or "failed" (branch was created)
- `error` SHOULD be non-null if status is "failed"

### RunHistory

- `version` MUST be semver format (currently "1.0")
- `runs` MUST be sorted by `started_at` descending (newest first)
- File MUST be valid YAML syntax

## Relationships

```
RunHistory (1) ────────∈ Run (N)
    │
    └── runs.yaml file

Run (1) ────────∈ BatchResult (N)
    │
    └── Embedded in Run.batches

Run ──── references ──── Spec (via spec_name)
    │
    └── Located in .specs/<spec_name>/

BatchResult ──── references ──── Batch (via id)
    │
    └── Matches batch.id in plan.yaml
```

## Sample YAML File

```yaml
# .specs/001-build-todo-list/runs.yaml
version: "1.0"
spec_name: "001-build-todo-list"
runs:
  - id: "run-2026-01-07-abc123"
    spec_name: "001-build-todo-list"
    started_at: "2026-01-07T10:30:00Z"
    ended_at: "2026-01-07T10:45:00Z"
    status: "completed"
    dry_run: false
    elapsed_seconds: 900
    batches:
      - id: "batch-001"
        name: "Database Foundation"
        status: "completed"
        started_at: "2026-01-07T10:30:15Z"
        ended_at: "2026-01-07T10:35:00Z"
        branch: "ckrv-batch-database-a1b2c3"
        merged: true
        error: null
      - id: "batch-002"
        name: "Authentication"
        status: "completed"
        started_at: "2026-01-07T10:35:30Z"
        ended_at: "2026-01-07T10:45:00Z"
        branch: "ckrv-batch-auth-d4e5f6"
        merged: true
        error: null
    summary:
      total_batches: 2
      completed_batches: 2
      failed_batches: 0
      pending_batches: 0
      tasks_completed: 12
      branches_merged: 2
    error: null
  
  - id: "run-2026-01-06-xyz789"
    spec_name: "001-build-todo-list"
    started_at: "2026-01-06T14:00:00Z"
    ended_at: "2026-01-06T14:10:00Z"
    status: "failed"
    dry_run: false
    elapsed_seconds: 600
    batches:
      - id: "batch-001"
        name: "Database Foundation"
        status: "completed"
        started_at: "2026-01-06T14:00:15Z"
        ended_at: "2026-01-06T14:05:00Z"
        branch: "ckrv-batch-database-g7h8i9"
        merged: true
        error: null
      - id: "batch-002"
        name: "Authentication"
        status: "failed"
        started_at: "2026-01-06T14:05:30Z"
        ended_at: "2026-01-06T14:10:00Z"
        branch: "ckrv-batch-auth-j0k1l2"
        merged: false
        error: "Task failed with exit code 1: Authentication service not responding"
    summary:
      total_batches: 2
      completed_batches: 1
      failed_batches: 1
      pending_batches: 0
      tasks_completed: 6
      branches_merged: 1
    error: "Execution failed: 1 batch failed"
```
