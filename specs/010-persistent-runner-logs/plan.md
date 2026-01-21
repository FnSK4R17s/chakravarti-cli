# Implementation Plan: Persistent Runner Logs

**Branch**: `010-persistent-runner-logs` | **Date**: 2026-01-15 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/010-persistent-runner-logs/spec.md`

## Summary

Implement persistent log storage for the **Execution Runner page** so users can navigate away during batch execution (Claude Code via subscription or OpenRouter) and return to see all logs generated per batch. Logs are stored as files per execution, displayed with tail-10 in real-time, and auto-cleaned when worktrees are merged.

**Primary Focus**: Execution Runner → Orchestrator Log (per-batch Claude Code output)  
**Secondary**: Dashboard Execution Log (activity summary only)

## Technical Context

**Language/Version**: Rust 1.75 (backend), TypeScript/React (frontend)  
**Primary Dependencies**: Axum (web server), tokio (async), React Query, WebSocket  
**Storage**: File-based (dedicated `.ckrv/logs/` folder with `.gitkeep`)  
**Testing**: cargo test (Rust), vitest (frontend)  
**Target Platform**: Linux/macOS/Windows (CLI + local web UI)
**Project Type**: Web application (backend API + frontend SPA)  
**Performance Goals**: Load 10k log lines in <2 seconds, real-time streaming with minimal latency  
**Constraints**: Zero log loss, chronological ordering, <100MB disk usage per execution  
**Scale/Scope**: Multiple concurrent executions, logs retained until worktrees merged

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Requirement | Status |
|-----------|-------------|--------|
| I. Code Quality Excellence | Full typing, zero lint errors, single responsibility | ✅ Rust strict mode enabled, TypeScript strict |
| II. Testing Standards | TDD approach planned, coverage targets defined | ✅ Unit tests for log writer/reader, integration for WebSocket reconnect |
| III. Reliability First | Error handling strategy, idempotency considered | ✅ Graceful reconnection, atomic log writes, no data loss |
| IV. Security by Default | No hardcoded secrets, input validation planned | ✅ Logs stored locally, no external network calls |
| V. Deterministic CLI Behavior | Machine-readable output, explicit exit codes | ✅ JSON log format, structured responses |

## Project Structure

### Documentation (this feature)

```text
specs/010-persistent-runner-logs/
├── spec.md              # Feature specification
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
└── tasks.md             # Phase 2 output (from /speckit.tasks)
```

### Source Code (repository root)

```text
crates/
├── ckrv-ui/
│   ├── src/
│   │   ├── api/
│   │   │   └── execution.rs     # MODIFY: Add log persistence and history endpoints
│   │   ├── services/
│   │   │   ├── engine.rs        # MODIFY: Write logs to disk as they're generated
│   │   │   └── log_store.rs     # NEW: Log file read/write service
│   │   └── models/
│   │       └── log.rs           # NEW: Log entry models
│   └── frontend/
│       └── src/
│           ├── components/
│           │   ├── ui/
│           │   │   └── carousel.tsx      # NEW: shadcn carousel (install via CLI)
│           │   ├── BatchLogCarousel.tsx  # NEW: Carousel wrapper for batch logs
│           │   ├── BatchLogTerminal.tsx  # NEW: Single batch log terminal
│           │   ├── LogViewer.tsx         # MODIFY: Minimal changes (secondary)
│           │   └── ExecutionRunner.tsx   # MODIFY: Use BatchLogCarousel
│           ├── hooks/
│           │   └── useLogStore.ts        # NEW: Log persistence React hook
│           └── services/
│               └── logService.ts         # NEW: API client for log history

.ckrv/
└── logs/                     # NEW: Execution log storage folder
    ├── .gitkeep              # Keep folder in git
    └── {run_id}/             # Per-execution log files
        ├── orchestrator.jsonl  # Orchestrator-level logs
        ├── batch-1.jsonl       # Batch 1 Claude Code output
        ├── batch-2.jsonl       # Batch 2 Claude Code output
        └── metadata.json       # Run metadata (source: subscription/openrouter)
```

**Structure Decision**: Extends existing ckrv-ui crate with new log persistence layer. Backend writes logs as JSONL files; frontend fetches history on reconnect.

## Architecture Overview

### Data Flow (Execution Runner Focus)

```
┌─────────────────────────────────────────────────────────────────────┐
│                     EXECUTION RUNNER PAGE                           │
└─────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────────┐
│ ExecutionEngine │────▶│   LogStore      │────▶│   .ckrv/logs/       │
│ (Claude Code)   │     │ (writes/reads)  │     │   {run_id}/         │
│ - Subscription  │     └─────────────────┘     │   ├── batch-1.jsonl │
│ - OpenRouter    │            │                │   ├── batch-2.jsonl │
└─────────────────┘            │                │   └── orchestrator. │
                               ▼                └─────────────────────┘
                        ┌─────────────────┐
                        │  WebSocket API  │◀───── User returns to page
                        │ (streams + hist)│
                        └─────────────────┘
                               │
                               ▼
                    ┌─────────────────────────┐
                    │  Execution Runner UI    │
                    │  ┌───────────────────┐  │
                    │  │ Orchestrator Log  │  │ ← Per-batch Claude output
                    │  │ (tail-10 live)    │  │
                    │  │ (scroll for hist) │  │
                    │  └───────────────────┘  │
                    └─────────────────────────┘
```
```

### Key Design Decisions

1. **JSONL Format**: Each log line is a separate JSON object for append-only streaming
2. **Tail-10 Display**: Real-time view shows only most recent 10 logs (per clarification)
3. **Lazy History Load**: Full history loaded only when user scrolls up
4. **File-per-Execution**: Isolated logs prevent cross-contamination
5. **Auto-Cleanup Trigger**: Logs deleted when all worktrees merged (existing merge flow)

## Complexity Tracking

> **No Constitution violations requiring justification**

| Area | Complexity | Notes |
|------|------------|-------|
| Log File I/O | Low | Simple append-only JSONL |
| WebSocket Reconnection | Medium | Must handle state sync on reconnect |
| Scroll-based History Loading | Medium | Virtualized list for performance |
| Auto-Cleanup on Merge | Low | Hook into existing merge flow |

## Phase 0: Research

*Completed inline - no external research required*

| Topic | Decision | Rationale |
|-------|----------|-----------|
| Log Format | JSONL (JSON Lines) | Append-only, easy parsing, line-by-line streaming |
| Storage Location | `.ckrv/logs/{execution_id}/log.jsonl` | Gitignored, per-execution isolation |
| History Fetch | REST endpoint with pagination | Simpler than WebSocket for history queries |
| Scroll Loading | Load 100 lines per scroll page | Balance between UX and memory |
| Cleanup Trigger | Post-merge hook | Existing infrastructure in merge_all_branches |

## Phase 1: Data Model

See [data-model.md](./data-model.md) for complete entity definitions.

### Key Entities

```
LogEntry {
  id: UUID
  execution_id: String
  timestamp: DateTime<Utc>
  level: LogLevel (info, warning, error, log, batch_start, batch_complete, etc.)
  message: String
  source: Option<String>  // batch name or component
}

ExecutionLogFile {
  execution_id: String
  path: PathBuf
  line_count: usize
  created_at: DateTime<Utc>
  last_modified: DateTime<Utc>
}
```

## Phase 1: Contracts

See [contracts/](./contracts/) for API specifications.

### New Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/execution/{id}/logs` | Fetch historical logs (paginated) |
| GET | `/api/execution/{id}/logs/tail` | Fetch last N logs |
| DELETE | `/api/execution/{id}/logs` | Manual log cleanup |

### WebSocket Extensions

- On connect: Send all historical logs first, then stream live
- Message type: `{ type: "history_complete" }` to signal end of backfill
- Frontend reconnection: Request logs since last received timestamp

## UI Changes (Browser Inspection)

Based on examining the live UI at `localhost:3002`:

### PRIMARY: Execution Runner Page - Orchestrator Log

**File**: `crates/ckrv-ui/frontend/src/components/ExecutionRunner.tsx`

This is the **main focus** - the Orchestrator Log in the Execution Runner where Claude Code batch output is displayed.

**Current State** (observed via browser):
- Empty black terminal box after execution completes
- No indication that logs ever existed from previous runs
- During execution: streams all Claude Code output (subscription or OpenRouter)
- Logs lost when navigating away or on page refresh

**Per-Batch Log Structure**:
Each batch (e.g., "setup", "build", "test") runs Claude Code and streams output. We need to:
1. Capture each batch's Claude Code output separately
2. Store with batch identifier for filtering
3. Allow viewing specific batch logs

**Changes Required**:

| Element | Current | Change To |
|---------|---------|-----------|
| After completion | Empty terminal | Load full history from disk |
| During execution | All logs mixed | **Carousel** with one slide per batch |
| Batch selector | None | **shadcn Carousel** with swipe/arrows |
| Log source | Not indicated | Show "Claude (Subscription)" or "Claude (OpenRouter)" |
| Scroll behavior | N/A | Virtual scroll within each carousel slide |

### Carousel-Based Batch Log Viewer

Using [shadcn/ui Carousel](https://ui.shadcn.com/docs/components/carousel) (Embla-based) for swipe navigation between batches:

**Installation**:
```bash
pnpm dlx shadcn@latest add carousel
```

**Component Structure**:
```tsx
import {
  Carousel,
  CarouselContent,
  CarouselItem,
  CarouselNext,
  CarouselPrevious,
  type CarouselApi,
} from "@/components/ui/carousel"

function BatchLogCarousel({ batches, runId }: Props) {
  const [api, setApi] = React.useState<CarouselApi>()
  const [current, setCurrent] = React.useState(0)
  
  React.useEffect(() => {
    if (!api) return
    api.on("select", () => setCurrent(api.selectedScrollSnap()))
  }, [api])

  return (
    <div className="w-full">
      <Carousel setApi={setApi} opts={{ align: "start" }}>
        <CarouselContent>
          {batches.map((batch, index) => (
            <CarouselItem key={batch.id} className="h-[400px]">
              <BatchLogTerminal 
                batchId={batch.id}
                batchName={batch.name}
                logs={batch.logs}
                isLive={batch.status === 'running'}
              />
            </CarouselItem>
          ))}
        </CarouselContent>
        <CarouselPrevious />
        <CarouselNext />
      </Carousel>
      
      {/* Batch indicator */}
      <div className="text-center text-sm text-muted-foreground py-2">
        Batch {current + 1} of {batches.length}: <strong>{batches[current]?.name}</strong>
      </div>
    </div>
  )
}
```

**New UI Layout for Execution Runner**:

```
┌──────────────────────────────────────────────────────────────────┐
│  Orchestrator          ● Live    Batch 2 of 5: setup            │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ◀  ┌──────────────────────────────────────────────────────┐  ▶  │
│     │  ↑ Scroll for 234 more logs                          │     │
│     ├──────────────────────────────────────────────────────┤     │
│     │  17:05:32 Installing dependencies...                 │     │
│     │  17:05:33 ✓ npm install completed                    │     │
│     │  17:05:35 Running Claude Code (OpenRouter)...        │     │
│     │  17:05:36 Creating component file...                 │     │
│     │  17:05:40 ✓ HelloWorld.tsx created                   │     │
│     │  ... (tail-10 shown)                                 │     │
│     ├──────────────────────────────────────────────────────┤     │
│     │  456 logs │ via Claude (Subscription)                │     │
│     └──────────────────────────────────────────────────────┘     │
│                                                                  │
│                    ○ ○ ● ○ ○  (batch indicators)                 │
├──────────────────────────────────────────────────────────────────┤
│  Swipe or use arrows to switch batches                           │
└──────────────────────────────────────────────────────────────────┘
```

**Carousel Features**:
- **Swipe gestures**: Touch-friendly navigation between batches
- **Arrow buttons**: `<CarouselPrevious />` and `<CarouselNext />`
- **Dot indicators**: Visual progress showing current batch
- **Per-batch terminal**: Each slide is a full log terminal for that batch
- **API access**: Track current batch via `CarouselApi`


### SECONDARY: Dashboard - Execution Log Panel

**File**: `crates/ckrv-ui/frontend/src/components/LogViewer.tsx`

The Dashboard panel is **secondary** - just shows activity summary, not full log viewer.

**Minimal Changes**:
- Show "X logs available" count if execution has persisted logs
- "View in Runner →" link to navigate to Execution Runner
- Basic activity feed (last 5 events only)

**No need for**:
- Full log history loading
- Scroll-based pagination
- Per-batch filtering

### State Management (Execution Runner Focus)

```typescript
// Store in localStorage per execution
interface ExecutionLogState {
  executionId: string;
  runId: string;
  lastSeenTimestamp: string;  // ISO 8601
  selectedBatch: string | 'all';  // Filter by batch
  scrollPosition: number;
}

// Log entry with batch info
interface BatchLogEntry {
  id: string;
  timestamp: string;
  batch: string;        // "batch-1", "batch-2", etc.
  level: string;
  message: string;
  source: 'subscription' | 'openrouter';  // Which Claude mode
}
```

### Reconnection Flow (Execution Runner)

1. User views Execution Runner → starts execution → Claude Code streams logs
2. User navigates to Dashboard or another browser tab
3. Claude Code continues, logs written to disk per batch
4. User returns to Execution Runner
5. Frontend detects `lastSeenTimestamp`, fetches missed logs
6. Logs merged, "Loaded 150 missed logs" toast shown
7. User can scroll up to see full history or filter by batch

### Visual Design (Execution Runner)

**Badges**:
- **● Live** (green): During active batch execution
- **✓ Completed** (green check): Execution finished successfully
- **✗ Failed** (red): Execution failed
- **📁 History** (folder): Viewing past execution logs

**Batch Colors** (for visual grouping):
- batch-1: Blue accent
- batch-2: Purple accent
- batch-3: Teal accent
- (cycling colors for more batches)

**Toast Notifications**:
- "Loaded 150 missed logs from batch-2" - On reconnection
- "Execution completed while you were away" - Status update


## Next Steps

1. Run `/speckit.tasks` to generate implementation tasks
2. Implement in priority order: P1 (navigate away) → P2 (scroll) → P3 (refresh)
3. Test with existing execution infrastructure

---

*Plan generated by /speckit.plan on 2026-01-15*
*UI analysis via browser inspection on 2026-01-15*
