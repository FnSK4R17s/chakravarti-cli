# Tauri Integration Guide

This guide covers how to use `ckrv-transport` with a Tauri desktop application.

## Overview

The `tauri` feature enables Tauri-specific command wrappers that use the same handlers as the Axum backend, ensuring feature parity between web and desktop.

## Prerequisites

- Rust toolchain
- Tauri v2 CLI (`cargo install tauri-cli --version ^2.0`)
- System dependencies for Tauri (varies by OS)

## Setup

### 1. Enable Tauri Feature

In your Tauri app's `Cargo.toml`:

```toml
[dependencies]
ckrv-transport = { path = "../crates/ckrv-transport", features = ["tauri"] }
```

### 2. Register Commands

In `src-tauri/src/lib.rs`:

```rust
use ckrv_transport::state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = AppState::new(std::env::current_dir().unwrap());

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            // Status commands
            ckrv_transport::tauri::status::get_status,
            // Agent commands
            ckrv_transport::tauri::agents::list_agents,
            // Add more commands as needed...
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 3. Create TypeScript Bindings

The frontend can import types directly from the generated bindings:

```typescript
// src-tauri/src/types.ts (copied from ckrv-transport typescript output)
import type { SystemStatus, AgentConfig } from './types';

const status = await invoke<SystemStatus>('get_status');
const agents = await invoke<AgentConfig[]>('list_agents');
```

## Available Commands

The following commands are available when the `tauri` feature is enabled:

| Command | Description |
|---------|-------------|
| `get_status` | Get system status (project, git, docker) |
| `check_docker` | Check Docker availability |
| `get_cloud_status` | Get cloud connection status |
| `list_agents` | List configured agents |
| `upsert_agent` | Create or update an agent |
| `delete_agent` | Delete an agent |
| `list_specs` | List specifications |
| `get_spec` | Get spec details |
| `get_plan` | Get execution plan |
| `list_tasks` | List tasks |
| `start_execution` | Start batch execution |
| `stop_execution` | Stop execution |
| `qa_review` | Run QA review |
| `test_run` | Run tests |

## Command Pattern

Each Tauri command wraps a transport-agnostic handler:

```rust
#[tauri::command]
pub async fn list_agents(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<AgentConfig>, String> {
    crate::handlers::agents::list_agents_handler(&state)
        .await
        .map_err(|e| e.to_string())
}
```

## State Management

Use Tauri's state management to pass `AppState`:

```rust
// Create state once
let state = AppState::new(project_root);

// Register with Tauri
tauri::Builder::default()
    .manage(state)  // Available as State<'_, AppState>
```

## Event Broadcasting

For real-time updates, use Tauri's event system:

```rust
use tauri::Manager;

// In handler with Tauri window access
window.emit("execution-progress", progress)?;
```

```typescript
// In frontend
listen('execution-progress', (event) => {
  console.log('Progress:', event.payload);
});
```

## Development Workflow

1. Build transport crate with tauri feature:
   ```bash
   cargo build -p ckrv-transport --features tauri
   ```

2. Create Tauri app:
   ```bash
   cargo tauri init
   ```

3. Add ckrv-transport dependency
4. Register commands and state
5. Build and run:
   ```bash
   cargo tauri dev
   ```

## Note

The Tauri integration is currently at stub level. Full implementation will be completed when the desktop application is developed. The architecture ensures parity with the web UI by sharing the same handler logic.
