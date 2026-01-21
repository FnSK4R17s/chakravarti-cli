---
last_commit: c1bb442
last_updated: 2026-01-21
related_files:
  - src/lib.rs
  - src/api/mod.rs
  - frontend/src/App.tsx
---

# ckrv-ui

Web UI dashboard server for Chakravarti.

## Overview

This crate provides the web dashboard backend including REST API endpoints, WebSocket event streaming, and the React frontend.

## Key Components

| Component | Purpose |
|-----------|---------|
| REST API | CRUD operations for specs, plans, tasks |
| WebSocket | Real-time execution streaming |
| Frontend | React-based dashboard |

## Architecture

```
crates/ckrv-ui/
├── src/
│   ├── lib.rs         # Server entry point
│   └── api/           # REST endpoints
│       ├── agents.rs
│       ├── execution.rs
│       ├── specs.rs
│       ├── plans.rs
│       └── ...
└── frontend/          # React application
    ├── src/
    │   ├── App.tsx
    │   └── components/
    └── package.json
```

## Usage

```rust
use ckrv_ui::start_server;

// Start the UI server
start_server(UiConfig {
    port: 3000,
    open_browser: true,
})?;
```

## API Overview

See [api-reference.md](api-reference.md) for complete endpoint documentation.

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/specs` | GET | List specs |
| `/api/specs/:id` | GET | Get spec details |
| `/api/execution/start` | POST | Start execution |
| `/api/execution/logs` | WS | Stream logs |

## Frontend Development

```bash
cd crates/ckrv-ui/frontend
npm install
npm run dev  # Development server
npm run build  # Production build
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `axum` | Web framework |
| `tokio` | Async runtime |
| `tower` | Middleware |
| `rust-embed` | Static file embedding |
