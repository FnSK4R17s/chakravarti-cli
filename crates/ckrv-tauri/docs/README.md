---
last_commit: 1b27ca2
last_updated: 2026-02-10
related_files:
  - src/main.rs
  - src/commands/mod.rs
  - tauri.conf.json
  - capabilities/default.json
---

# ckrv-tauri

Tauri v2 desktop application for Chakravarti.

## Overview

This crate wraps the `ckrv-transport` handlers and `ckrv-ui` frontend into a native desktop application using Tauri v2. It shares the same React frontend as `ckrv ui` but replaces HTTP/WebSocket communication with Tauri IPC, and adds native capabilities like PTY terminals, file dialogs, and project persistence.

## Architecture

```text
┌──────────────────────────────────────────────────────┐
│                  React Frontend                       │
│              (ckrv-ui/frontend/dist)                  │
└────────────────────────┬─────────────────────────────┘
                         │ Tauri IPC (invoke)
┌────────────────────────▼─────────────────────────────┐
│                  ckrv-tauri                            │
│  ┌────────────────────────────────────────────────┐   │
│  │              commands/ (12 modules)              │   │
│  │  agents · cli · diff · execution · history      │   │
│  │  plans · project · qa · specs · status          │   │
│  │  terminal · test                                │   │
│  └────────────────────────┬───────────────────────┘   │
│                           │                            │
│  ┌────────────────────────▼───────────────────────┐   │
│  │            ckrv-transport                        │   │
│  │      handlers · types · AppState                │   │
│  └────────────────────────────────────────────────┘   │
│                                                        │
│  Plugins: shell · dialog · fs · pty · process          │
└────────────────────────────────────────────────────────┘
```

## Module Structure

```
src/
├── main.rs              # Tauri app builder, plugin registration, 50+ IPC handlers
└── commands/
    ├── mod.rs           # Module declarations
    ├── agents.rs        # Agent CRUD, OpenRouter models, default/QA/test agent (112 lines)
    ├── cli.rs           # CLI command wrappers: init, spec, plan, run, verify, etc. (153 lines)
    ├── diff.rs          # Branch listing, default branch, diff generation (51 lines)
    ├── execution.rs     # Start/stop execution, status, logs, branch listing (164 lines)
    ├── history.rs       # Run history CRUD (95 lines)
    ├── plans.rs         # Plan CRUD (56 lines)
    ├── project.rs       # Project root persistence, recent projects, folder dialog (150 lines)
    ├── qa.rs            # QA agent, review/bugs/report commands (124 lines)
    ├── specs.rs         # Spec CRUD, validation, design/tasks generation (98 lines)
    ├── status.rs        # System status, Docker check, cloud status (38 lines)
    ├── terminal.rs      # Docker-based terminal sessions with PTY (330 lines)
    └── test.rs          # Test runner, planner, writer, coverage, fix (379 lines)
```

## IPC Commands (50+)

### Status
| Command | Handler | Purpose |
|---------|---------|---------|
| `get_status` | `status::get_status` | System orchestration status |
| `check_docker` | `status::check_docker` | Docker availability check |
| `get_cloud_status` | `status::get_cloud_status` | Cloud connection status |

### Agents
| Command | Handler | Purpose |
|---------|---------|---------|
| `list_agents` | `agents::list_agents` | List configured agents |
| `get_openrouter_models` | `agents::get_openrouter_models` | Fetch OpenRouter model catalog |
| `upsert_agent` | `agents::upsert_agent` | Create or update agent config |
| `delete_agent` | `agents::delete_agent` | Remove agent |
| `set_default_agent` | `agents::set_default_agent` | Set default execution agent |
| `set_qa_agent` | `agents::set_qa_agent` | Set QA review agent |
| `set_test_writer_agent` | `agents::set_test_writer_agent` | Set test writer agent |
| `test_agent` | `agents::test_agent` | Validate agent connectivity |

### Specs
| Command | Handler | Purpose |
|---------|---------|---------|
| `list_specs` | `specs::list_specs` | List all specifications |
| `get_spec` | `specs::get_spec` | Get spec details |
| `create_spec` | `specs::create_spec` | Create new spec |
| `update_spec` | `specs::update_spec` | Update existing spec |
| `delete_spec` | `specs::delete_spec` | Delete spec |
| `validate_spec` | `specs::validate_spec` | Validate spec file |
| `generate_design` | `specs::generate_design` | Generate design doc |
| `generate_tasks` | `specs::generate_tasks` | Generate implementation tasks |

### Plans
| Command | Handler | Purpose |
|---------|---------|---------|
| `list_plans` | `plans::list_plans` | List execution plans |
| `get_plan` | `plans::get_plan` | Get plan details |
| `save_plan` | `plans::save_plan` | Save plan |
| `delete_plan` | `plans::delete_plan` | Delete plan |

### Execution
| Command | Handler | Purpose |
|---------|---------|---------|
| `start_execution` | `execution::start_execution` | Start orchestration run |
| `stop_execution` | `execution::stop_execution` | Stop execution |
| `get_execution_status` | `execution::get_execution_status` | Execution status |
| `get_execution_logs` | `execution::get_execution_logs` | Execution logs |
| `list_execution_branches` | `execution::list_execution_branches` | List worktree branches |

### Terminal
| Command | Handler | Purpose |
|---------|---------|---------|
| `terminal_start` | `terminal::terminal_start` | Start Docker terminal session |
| `terminal_stop` | `terminal::terminal_stop` | Stop terminal session |
| `terminal_write` | `terminal::terminal_write` | Send input to terminal |
| `terminal_read` | `terminal::terminal_read` | Read terminal output |
| `terminal_is_running` | `terminal::terminal_is_running` | Check session status |
| `terminal_list` | `terminal::terminal_list` | List active sessions |

### Project
| Command | Handler | Purpose |
|---------|---------|---------|
| `get_project_root` | `project::get_project_root` | Get current project path |
| `set_project_root` | `project::set_project_root` | Set project (persists to disk) |
| `get_recent_projects` | `project::get_recent_projects` | Recent project list (max 10) |
| `open_project_dialog` | `project::open_project_dialog` | Native folder picker |

### QA, Test, CLI, Diff, History

Additional commands wrap the corresponding `ckrv-transport` handlers for QA reviews, test execution, CLI operations, diff viewing, and run history.

## Tauri Plugins

| Plugin | Version | Purpose |
|--------|---------|---------|
| `tauri-plugin-shell` | 2 | Open URLs in browser |
| `tauri-plugin-dialog` | 2 | Native file/folder dialogs |
| `tauri-plugin-fs` | 2 | File system access |
| `tauri-plugin-pty` | 0.2 | PTY for interactive terminals |
| `tauri-plugin-process` | 2 | App restart/exit |

## Capabilities

The default capability grants (`capabilities/default.json`):

```json
{
    "permissions": [
        "core:default",
        "shell:allow-open",
        "dialog:allow-open",
        "dialog:allow-save",
        "fs:allow-read-text-file",
        "fs:allow-write-text-file",
        "pty:default",
        "process:allow-restart",
        "process:allow-exit"
    ]
}
```

## State Management

| Type | Storage | Purpose |
|------|---------|---------|
| `SharedState` | `Arc<RwLock<AppState>>` | Transport app state (project root, orchestration) |
| `TerminalSessions` | `Arc<Mutex<HashMap>>` | Active Docker terminal sessions |
| `TauriConfig` | `~/.ckrv/tauri-config.json` | Persisted project root and recent projects |

## Build & Development

```bash
# Development (frontend + Tauri in watch mode)
cd crates/ckrv-tauri
cargo tauri dev

# Production build
cargo tauri build

# Frontend is shared with ckrv-ui
cd crates/ckrv-ui/frontend && npm run build
```

### Build Chain

```text
tauri.conf.json → beforeBuildCommand → cd ckrv-ui/frontend && npm run build
                → frontendDist → ../ckrv-ui/frontend/dist
                → beforeDevCommand → cd ckrv-ui/frontend && npm run dev
                → devUrl → http://localhost:5173
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `ckrv-transport` (with `tauri` feature) | Shared API types, handlers, AppState |
| `ckrv-core` | Domain types |
| `ckrv-sandbox` | DockerClient for terminal containers |
| `tauri` v2 | Desktop framework |
| `tokio` | Async runtime |
| `serde` / `serde_json` | Serialization |
| `parking_lot` | Sync mutex for terminal sessions |
| `dirs` | Home directory discovery |
| `tracing` | Logging |

## Related Documentation

- [Terminal Integration](tauri-terminal-integration.md) — PTY architecture details
