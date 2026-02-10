---
last_commit: 1b27ca2
last_updated: 2026-02-10
related_files:
  - src/lib.rs
  - src/server.rs
  - src/services/engine.rs
  - frontend/src/App.tsx
---

# ckrv-ui

Web UI dashboard server for Chakravarti.

## Overview

This crate provides the full-stack web dashboard including:
- **Backend**: Axum server with API routes from `ckrv-transport`, execution engine, static file serving
- **Frontend**: React/TypeScript dashboard with 27+ components

**Since 0.1.0**: The backend API layer has been migrated to `ckrv-transport`. This crate now acts as a thin server layer that composes transport routes with static file serving and application services.

## Backend Architecture

```
src/
├── lib.rs              # Public exports, ckrv-transport re-exports
├── server.rs           # Axum server, static file handler, route composition
├── services/           # Core services
│   ├── engine.rs       # Execution engine (53KB)
│   ├── command.rs      # Command runner (50KB)
│   ├── log_store.rs    # Log persistence (11KB)
│   └── history.rs      # Run history service
└── models/             # Data models
    ├── history.rs      # History types
    └── log.rs          # Log types
```

### Re-exports from ckrv-transport

The following types are re-exported for backward compatibility:

| Type | Source | Purpose |
|------|--------|---------|
| `AppState` | `ckrv-transport` | Shared application state |
| `Hub` | `ckrv-transport` | WebSocket event broadcast hub |
| `OrchestrationEvent` | `ckrv-transport` | Event types for real-time streaming |
| `SharedHub` | `ckrv-transport` | Thread-safe hub reference |
| `SystemMode` | `ckrv-transport` | Active mode (idle, planning, running) |
| `SystemStatus` | `ckrv-transport` | Current orchestration status |

## API Endpoints

All API routes are provided by the `ckrv-transport` crate and mounted by `server.rs`:

### Core CRUD

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/specs` | GET/POST | List and create specs |
| `/api/specs/:id` | GET/PUT/DELETE | Spec CRUD |
| `/api/plans` | GET/POST | Plan management |
| `/api/tasks` | GET/POST | Task operations |
| `/api/agents` | GET/PUT | Agent configuration |

### Execution

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/execution/start` | POST | Start execution run |
| `/api/execution/stop` | POST | Stop execution |
| `/api/execution/status` | GET | Get execution status |
| `/api/execution/logs` | WS | Stream execution logs |
| `/api/terminal/ws` | WS | PTY terminal streaming |

### Test & QA

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/test/run` | POST | Run tests in sandbox |
| `/api/test/plan` | POST | Generate test plan |
| `/api/test/write` | POST | Write tests via agent |
| `/api/test/results` | GET | Get test results |
| `/api/qa/review` | POST | Run QA review |
| `/api/qa/bugs` | POST | Analyze for bugs |
| `/api/qa/report` | POST | Generate QA report |

### Utilities

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/diff` | GET | Get code diff |
| `/api/history` | GET | Run history |
| `/api/status` | GET | System status |
| `/api/docker/status` | GET | Docker availability |

## Frontend Architecture

```
frontend/src/
├── App.tsx             # Main app with routing
├── main.tsx            # Entry point
├── index.css           # Global styles (17KB)
├── types.ts            # Shared types
├── components/         # 27 feature components
│   ├── ExecutionRunner.tsx    # Main executor (70KB)
│   ├── AgentManager.tsx       # Agent config (47KB)
│   ├── TestRunner.tsx         # Test UI (41KB)
│   ├── PlanEditor.tsx         # Plan editing (38KB)
│   ├── TaskEditor.tsx         # Task management (33KB)
│   ├── SpecEditor.tsx         # Spec editing (32KB)
│   ├── QAReviewer.tsx         # QA review UI (26KB)
│   ├── TaskDetailModal.tsx    # Task details (24KB)
│   ├── BarebonesExecutor.tsx  # Simple executor
│   ├── ChatDashboard.tsx      # Chat interface
│   ├── CommandPalette.tsx     # Cmd+K palette
│   ├── LogViewer.tsx          # Log display
│   ├── DiffViewer.tsx         # Diff display
│   ├── WorkflowPanel.tsx      # Workflow status
│   └── ui/                    # 26 shadcn components
├── hooks/              # 12 custom hooks
│   ├── useSpec.ts             # Spec state (9KB)
│   ├── useLogStore.ts         # Log management (9KB)
│   ├── useWebSocketReconnect.ts
│   ├── useRunHistory.ts
│   ├── useWorkflowProgress.ts
│   └── ...
├── layouts/            # Layout components
├── lib/                # Utility libraries
├── services/           # API service layer
└── types/              # TypeScript types
```

## Key Components

| Component | Purpose | Size |
|-----------|---------|------|
| `ExecutionRunner` | Main orchestration UI | 70KB |
| `AgentManager` | Agent configuration | 47KB |
| `TestRunner` | Test execution UI | 41KB |
| `PlanEditor` | Plan editing/viewing | 38KB |
| `TaskEditor` | Task management | 33KB |
| `SpecEditor` | Spec editing | 32KB |
| `QAReviewer` | QA review interface | 26KB |
| `CommandPalette` | Keyboard-driven commands | 12KB |

## Services

### Engine (53KB)

The execution engine handles:
- Batch execution orchestration
- Agent invocation
- Log streaming
- State management

### LogStore (11KB)

Centralized log management:
- Log persistence
- Batch-specific logs
- Real-time streaming

## Usage

```rust
use ckrv_ui::start_server;

// Start the UI server on port 3000
start_server(3000).await?;
```

## Frontend Development

```bash
cd crates/ckrv-ui/frontend
npm install
npm run dev    # Development server (Vite)
npm run build  # Production build
```

## Dependencies

### Backend

| Crate | Purpose |
|-------|---------|
| `ckrv-transport` | API handlers, types, and Axum router |
| `axum` | Web framework (route composition) |
| `tokio` | Async runtime |
| `tower` | Middleware |
| `rust-embed` | Static file embedding |
| `serde` | Serialization |

### Frontend

| Package | Purpose |
|---------|---------|
| `react` | UI framework |
| `vite` | Build tool |
| `tailwindcss` | Styling |
| `shadcn/ui` | Component library |
| `lucide-react` | Icons |
