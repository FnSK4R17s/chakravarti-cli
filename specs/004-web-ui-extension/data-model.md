# Data Model

## Entities

### `SystemStatus`
Represents the current state of the Chakravarti CLI environment.
- **active_branch** (string): Context of the current git branch.
- **feature_number** (string): Current extracted feature number (e.g., "004").
- **is_ready** (boolean): True if `ckrv` identifies a valid workspace.
- **mode** (enum): `idle`, `planning`, `running`, `promoting`.

### `Batch`
A logical grouping of tasks that can be executed in parallel.
- **id** (string): Unique identifier (e.g., "batch-01").
- **status** (enum): `pending`, `running`, `completed`, `failed`.
- **tasks** (string[]): List of Task IDs in this batch.

### `Task`
A single unit of work defined in the implementation plan.
- **id** (string): Unique identifier (e.g., "TSK-001").
- **description** (string): What needs to be done.
- **file_path** (string): Target file for modification.
- **dependencies** (string[]): IDs of tasks that must complete first.
- **status** (enum): `pending`, `in_progress`, `verified`, `failed`.
- **logs** (string[]): Recent execution logs.

### `OrchestrationEvent`
A single event in the SSE stream during an active command execution.
- **type** (enum): `log`, `step_start`, `step_end`, `error`, `success`.
- **timestamp** (string): ISO-8601.
- **message** (string): Human readable log content.
- **metadata** (object, optional): Structured data (e.g., job_id, step_name).

### `SpecForm`
Data required to initiate a new spec feature.
- **description** (string): The natural language feature request.
- **short_name** (string, optional): Override for the branch name.

## API Contracts Strategy

We will use REST for mutable actions and queries, and SSE for long-running observation.

### Endpoints Map

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/api/status` | Get `SystemStatus` |
| POST | `/api/command/init` | Trigger `ckrv init` |
| POST | `/api/command/spec` | Trigger `ckrv spec new` (Body: `SpecForm`) |
| POST | `/api/command/tasks` | Trigger `ckrv spec tasks` |
| POST | `/api/command/run` | Trigger `ckrv run` |
| POST | `/api/command/promote` | Trigger `ckrv promote` |
| GET | `/api/events` | SSE connection for live logs |
