---
last_commit: 508766e
last_updated: 2026-02-15
related_files:
  - src/lib.rs
  - src/handlers/mod.rs
  - src/axum/mod.rs
  - src/types/agents.rs
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
├── lib.rs              # Crate entry point
├── error.rs            # TransportError enum
├── state.rs            # AppState shared state
├── hub.rs              # Event broadcasting
├── handlers/           # Transport-agnostic handlers
│   ├── mod.rs
│   ├── agents.rs
│   ├── specs.rs
│   ├── tasks.rs
│   ├── plans.rs
│   ├── history.rs
│   ├── status.rs
│   ├── execution.rs
│   └── ...
├── types/              # Request/Response types
│   ├── mod.rs
│   ├── agents.rs
│   ├── specs.rs
│   └── ...
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
}
```

## Documentation

- [Adding Endpoints](./adding-endpoints.md) - How to add new API endpoints
- [Tauri Integration](./tauri-integration.md) - Desktop app integration guide

## Related Crates

- `ckrv-ui` - Web UI server (uses ckrv-transport with axum feature)
- `ckrv-core` - Core orchestration logic
- `ckrv-sandbox` - Docker sandbox execution
