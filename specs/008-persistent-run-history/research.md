# Research: Persistent Run History

## Storage Format Decision

### Decision: YAML Files per Specification

**Rationale**:
- User explicitly requested YAML-based storage instead of a database
- Aligns with existing `.specs/<name>/` directory structure
- Human-readable and editable if needed
- No additional dependencies (serde_yaml already in use)
- Simple deployment (no database setup)

**Alternatives Considered**:

| Option | Pros | Cons | Decision |
|--------|------|------|----------|
| SQLite | ACID, queries, indexing | Adds dependency, overkill for simple history | Rejected |
| JSON files | Simpler parsing | Less readable, no anchors/aliases | Rejected |
| YAML files | Readable, existing tooling, user request | Larger file size, slower for huge datasets | **Selected** |
| IndexedDB (browser) | Fast, structured | Not persistent on server, browser-only | Rejected |

### File Location

**Decision**: `.specs/<spec-name>/runs.yaml`

**Rationale**:
- Co-located with spec data (spec.yaml, plan.yaml, tasks.yaml)
- Easy to backup/restore along with spec
- Natural scoping per specification
- Follows existing pattern

## YAML Schema Design

### Decision: Flat Run List with Embedded Batches

```yaml
runs:
  - id: "run-2026-01-07-abc123"
    spec_name: "build-todo-list"
    started_at: "2026-01-07T10:30:00Z"
    ended_at: "2026-01-07T10:45:00Z"
    status: "completed"  # pending, running, completed, failed, aborted
    dry_run: false
    elapsed_seconds: 900
    batches:
      - id: "batch-001"
        name: "Database Foundation"
        status: "completed"
        started_at: "2026-01-07T10:30:15Z"
        ended_at: "2026-01-07T10:35:00Z"
        branch: "ckrv-batch-database-a1b2c3"
        error: null
      - id: "batch-002"
        name: "Authentication"
        status: "completed"
        started_at: "2026-01-07T10:35:30Z"
        ended_at: "2026-01-07T10:45:00Z"
        branch: "ckrv-batch-auth-d4e5f6"
        error: null
    summary:
      total_batches: 2
      completed_batches: 2
      failed_batches: 0
      tasks_completed: 12
      branches_merged: 2
```

**Rationale**:
- Single file per spec keeps reads simple
- Embedded batches avoid joins/lookups
- Summary section enables quick display without parsing all batch data
- ISO 8601 timestamps for consistency

## Real-Time Update Strategy

### Decision: Write-Through on Status Change

**Rationale**:
- Every batch status change triggers a YAML write
- Ensures persistence even on crash
- Acceptable performance (few updates per run)

**Implementation**:
```rust
// In engine.rs, after updating batch status
self.persist_run_status(&run).await?;
```

**Considerations**:
- Atomic writes using temp file + rename pattern
- Async file I/O to avoid blocking execution
- Error handling: log warning but don't fail execution

## API Design

### Decision: RESTful Endpoints

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/history/{spec}` | GET | List all runs for spec |
| `/api/history/{spec}/{run_id}` | GET | Get single run details |
| `/api/history/{spec}` | POST | Create new run entry |
| `/api/history/{spec}/{run_id}` | PATCH | Update run status |
| `/api/history/{spec}/{run_id}` | DELETE | Delete run entry |

**Rationale**:
- Standard REST patterns
- Aligns with existing API style
- Spec-based routing matches file structure

## UI Consistency Analysis

### Decision: Adopt Existing Design System

**Components to Match**:
1. Page header (same height, typography, navigation icons)
2. Panel layout (sidebar + main content split)
3. Card styling (same shadows, borders, padding)
4. Empty states (same icon style, messaging)
5. Button styles (primary, secondary, danger variants)

**Reference Pages**:
- Planner (`/planner`) - Panel layout, card grid
- Tasks (`/tasks`) - List display, status badges
- Settings (`/settings`) - Form styling (if applicable)

**Implementation Approach**:
- Extract common styles to shared CSS classes
- Use existing component patterns from other pages
- Ensure same responsive breakpoints

## Concurrency Handling

### Decision: Run Lock with Warning

**Scenario**: User opens two browser tabs and tries to run same spec

**Approach**:
1. When run starts, check for existing running run in history
2. If found and status is "running", show warning modal
3. User can choose to:
   - Cancel (don't start new run)
   - Override (abort previous, start new)
   - View existing (navigate to existing run)

**Implementation**:
- Backend checks `runs.yaml` for `status: running`
- Returns 409 Conflict with existing run_id if found
- Frontend displays warning modal with options

## History Size Management

### Decision: Soft Limit with Pagination

**Limit**: Display 50 most recent runs, paginate older

**Implementation**:
- Load all runs from YAML (typically small file)
- Sort by started_at descending
- Display first 50, "Load More" button for older
- No automatic pruning (user controls their history)

**Future Enhancement**: Add "Clear History" button if needed
