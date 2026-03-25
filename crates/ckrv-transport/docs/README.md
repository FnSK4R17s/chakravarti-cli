---
last_commit: b41880d
last_updated: 2026-03-25
related_files:
  - src/lib.rs
  - src/handlers/mod.rs
  - src/axum/mod.rs
  - src/types/agents.rs
  - src/state.rs
---

# ckrv-transport

Transport abstraction layer for Chakravarti CLI.

## Overview

The `ckrv-transport` crate provides a unified handler layer that can be used by multiple transport backends:

- **Axum** (HTTP/WebSocket) - Web UI backend via `ckrv-ui`
- **Tauri** (IPC commands) - Desktop app backend (future)

## Architecture

```text
┌─────────────────────────────────────────────────────────────┐
│                     Frontend (React/Tauri)                  │
└───────────────────────────┬─────────────────────────────────┘
                            │
         ┌──────────────────┴──────────────────┐
         │                                     │
         ▼                                     ▼
┌────────────────────┐              ┌────────────────────┐
│  axum/ (HTTP/WS)   │              │  tauri/ (IPC)      │
│  - route wrappers  │              │  - command funcs   │
└────────┬───────────┘              └────────┬───────────┘
         │                                   │
         └──────────────────┬────────────────┘
                            │
                            ▼
         ┌──────────────────────────────────────┐
         │           handlers/                  │
         │  - Transport-agnostic logic          │
         │  - Returns Result<T, TransportError> │
         └──────────────────────────────────────┘
                            │
                            ▼
         ┌──────────────────────────────────────┐
         │           types/                     │
         │  - Request/Response types            │
         │  - TypeScript generation (ts-rs)     │
         └──────────────────────────────────────┘
```

## Features

- `axum` - Enable Axum HTTP transport (default for web UI)
- `tauri` - Enable Tauri IPC transport (for desktop app)
- `typescript` - Enable TypeScript type generation via ts-rs

## Quick Start

### Using with Axum (Web UI)

```rust
use ckrv_transport::axum::create_router;
use ckrv_transport::AppState;

let state = AppState::new(project_root);
let router = create_router(state);

let listener = TcpListener::bind("0.0.0.0:3000").await?;
axum::serve(listener, router).await?;
```

### Using with Tauri (Desktop App)

```rust
use ckrv_transport::tauri::get_invoke_handlers;

tauri::Builder::default()
    .invoke_handler(get_invoke_handlers())
    .run(generate_context!())?;
```

## Module Structure

```
src/
├── lib.rs              # Crate entry point, re-exports
├── error.rs            # TransportError enum
├── state.rs            # AppState, SystemStatus, RunRegistry
├── hub.rs              # Event broadcasting (OrchestrationEvent)
├── handlers/           # Transport-agnostic handlers
│   ├── mod.rs
│   ├── agents.rs       # Agent CRUD, model listing
│   ├── specs.rs        # Spec listing, detail, creation
│   ├── tasks.rs        # Task listing, detail
│   ├── plans.rs        # Plan generation
│   ├── history.rs      # Execution history
│   ├── status.rs       # System status
│   ├── execution.rs    # Run lifecycle (start/stop/status) with in-process orchestration
│   ├── commands.rs     # CLI command dispatch
│   ├── cloud.rs        # Cloud status
│   ├── console.rs      # Console output
│   ├── diff.rs         # Diff viewing
│   ├── docker.rs       # Docker status
│   ├── events.rs       # Event stream
│   ├── example.rs      # Example handler
│   ├── qa.rs           # QA review
│   ├── session.rs      # Session management
│   ├── terminal.rs     # Terminal access
│   └── test.rs         # Test management
├── types/              # Request/Response types
│   ├── mod.rs
│   ├── agents.rs       # AgentType, AgentConfig, provider configs
│   ├── common.rs       # Common response wrappers
│   ├── execution.rs    # Execution types
│   ├── history.rs      # History types
│   ├── specs.rs        # ListSpecsResponse, BatchModelAssignment
│   └── test_qa.rs      # Test/QA types
├── axum/               # Axum HTTP wrappers
│   ├── mod.rs          # create_router()
│   ├── agents.rs
│   └── ...
└── tauri/              # Tauri IPC wrappers
    ├── mod.rs          # get_invoke_handlers()
    └── ...
```

## Key Types

### Agent Types (`types/agents.rs`)

| Type | Purpose |
|------|---------|
| `AgentType` | Enum of agent backends (Claude, ClaudeOpenRouter, ClaudeGlm, Codex, KiloCode) |
| `AgentConfig` | Full agent configuration with id, name, level, model, flags |
| `OpenRouterConfig` | OpenRouter-specific settings (api_key, model, base_url) |
| `GlmConfig` | GLM Coding Plan settings (api_key, model, timeout_ms) |
| `KiloCodeConfig` | Kilo Code settings (model in provider/model format) |
| `OpenRouterModel` | Model from OpenRouter catalog |
| `KiloCodeModel` | Model from Kilo Code CLI (id, provider, name, free flag) |
| `GlmModel` | Model from Z.AI GLM (id, name, context_length) |

### AgentType Enum

```rust
pub enum AgentType {
    Claude,             // Default Claude Code CLI
    ClaudeOpenRouter,   // Claude Code with OpenRouter API
    ClaudeGlm,          // Claude Code with Z.AI GLM Coding Plan
    Codex,              // OpenAI Codex CLI
    KiloCode,           // Kilo Code multi-provider CLI
    Gemini,             // Google Gemini CLI
    Cursor,             // Cursor CLI
    Amp,                // Amp CLI
    Qwen,               // Qwen Code CLI
    Opencode,           // Opencode CLI
    FactoryDroid,       // Factory Droid CLI
    GithubCopilot,      // GitHub Copilot CLI
    MistralVibe,        // Mistral Vibe CLI
}
```

### AgentConfig Structure

```rust
pub struct AgentConfig {
    pub id: String,
    pub name: String,
    pub agent_type: AgentType,
    pub level: u8,              // Capability level 1-5
    pub model: Option<String>,
    pub is_default: bool,
    pub is_qa_agent: bool,
    pub is_test_writer: bool,
    pub enabled: bool,
    pub description: Option<String>,
    pub openrouter: Option<OpenRouterConfig>,
    pub glm: Option<GlmConfig>,
    pub kilo: Option<KiloCodeConfig>,
    pub gemini: Option<GeminiConfig>,
}
```

### GeminiConfig Structure

```rust
pub struct GeminiConfig {
    pub api_key: Option<String>,
    pub model: Option<String>,
}
```

### Run Registry (`state.rs`)

Tracks active and recent execution runs. Added to `AppState` as `run_registry: SharedRunRegistry`.

| Type | Purpose |
|------|---------|
| `RunStatus` | Enum: `Pending`, `Running`, `Done`, `Error` |
| `RunEntry` | Single run with `run_id`, `spec_name`, `started_at`, `status`, `cancel_token`, `error_message` |
| `RunRegistry` | HashMap-backed registry with lookup by ID or spec name |
| `SharedRunRegistry` | `Arc<RwLock<RunRegistry>>` type alias |

```rust
pub struct AppState {
    pub status: Arc<RwLock<SystemStatus>>,
    pub hub: SharedHub,
    pub run_registry: SharedRunRegistry,   // NEW
    pub project_root: PathBuf,
}
```

### Execution Handlers

The execution handlers (`handlers/execution.rs`) now run orchestration **in-process** via `tokio::spawn` rather than shelling out. A `HubEventHandler` bridges `ckrv_core::JobEvent` into `OrchestrationEvent` for real-time WebSocket updates, with optional JSONL log persistence.

### ListSpecsResponse

Changed from a type alias (`Vec<SpecSummary>`) to a struct:

```rust
pub struct ListSpecsResponse {
    pub specs: Vec<SpecSummary>,
}
```

### BatchModelAssignment

New type for assigning models to spec batches:

```rust
pub struct BatchModelAssignment {
    pub batch_name: String,
    pub agent_id: String,
}
```

## Documentation

- [Adding Endpoints](./adding-endpoints.md) - How to add new API endpoints
- [Tauri Integration](./tauri-integration.md) - Desktop app integration guide

## Related Crates

- `ckrv-ui` - Web UI server (uses ckrv-transport with axum feature)
- `ckrv-core` - Core orchestration logic (now used in-process by execution handlers)
- `ckrv-sandbox` - Docker sandbox execution
