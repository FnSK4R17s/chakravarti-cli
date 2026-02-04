# API Contracts: Transport Crate

**Feature**: 019-transport-crate
**Date**: 2026-02-04

## Overview

This document defines the handler function signatures for `ckrv-transport`. Each handler is transport-agnostic and is wrapped by Axum or Tauri-specific adapters.

---

## Handler Signature Pattern

All handlers follow this pattern:

```rust
pub async fn <operation>_handler(
    state: &AppState,
    request: <RequestType>,  // Optional, for POST/PUT
) -> Result<<ResponseType>, TransportError>
```

---

## Status Handlers

### `get_status_handler`

Get current system status.

```rust
/// Returns current system status including git branch and initialization state.
pub async fn get_status_handler(
    state: &AppState,
) -> Result<SystemStatus, TransportError>
```

| Transport | Mapping |
|-----------|---------|
| Axum | `GET /api/status` → 200 OK with JSON body |
| Tauri | `invoke("get_status")` → SystemStatus |

---

## Agent Handlers

### `list_agents_handler`

List all configured agents.

```rust
/// Returns all agent configurations from the agents file.
pub async fn list_agents_handler(
    state: &AppState,
) -> Result<Vec<AgentConfig>, TransportError>
```

| Transport | Mapping |
|-----------|---------|
| Axum | `GET /api/agents` → 200 OK with JSON array |
| Tauri | `invoke("list_agents")` → Vec<AgentConfig> |

### `upsert_agent_handler`

Create or update an agent.

```rust
/// Creates a new agent or updates an existing one.
pub async fn upsert_agent_handler(
    state: &AppState,
    request: UpsertAgentRequest,
) -> Result<AgentConfig, TransportError>
```

| Transport | Mapping |
|-----------|---------|
| Axum | `POST /api/agents` → 200 OK with created agent |
| Tauri | `invoke("upsert_agent", { agent })` → AgentConfig |

### `delete_agent_handler`

Delete an agent.

```rust
/// Deletes an agent by name.
///
/// # Errors
/// - `NotFound` if agent doesn't exist
/// - `BadRequest` if trying to delete default agent
pub async fn delete_agent_handler(
    state: &AppState,
    request: DeleteAgentRequest,
) -> Result<(), TransportError>
```

| Transport | Mapping |
|-----------|---------|
| Axum | `DELETE /api/agents/:name` → 204 No Content |
| Tauri | `invoke("delete_agent", { name })` → () |

### `set_default_agent_handler`

Set the default agent.

```rust
/// Sets an agent as the default.
///
/// # Errors
/// - `NotFound` if agent doesn't exist
pub async fn set_default_agent_handler(
    state: &AppState,
    request: SetDefaultAgentRequest,
) -> Result<AgentConfig, TransportError>
```

| Transport | Mapping |
|-----------|---------|
| Axum | `POST /api/agents/default` → 200 OK with agent |
| Tauri | `invoke("set_default_agent", { name })` → AgentConfig |

### `get_openrouter_models_handler`

Fetch available OpenRouter models.

```rust
/// Fetches available models from OpenRouter API.
/// Falls back to curated list on API failure.
pub async fn get_openrouter_models_handler(
) -> Result<Vec<OpenRouterModel>, TransportError>
```

| Transport | Mapping |
|-----------|---------|
| Axum | `GET /api/agents/openrouter-models` → 200 OK |
| Tauri | `invoke("get_openrouter_models")` → Vec<OpenRouterModel> |

---

## Spec Handlers

### `list_specs_handler`

List all specifications.

```rust
/// Returns all specs in the .specs directory.
pub async fn list_specs_handler(
    state: &AppState,
) -> Result<Vec<SpecSummary>, TransportError>
```

| Transport | Mapping |
|-----------|---------|
| Axum | `GET /api/specs` → 200 OK with JSON array |
| Tauri | `invoke("list_specs")` → Vec<SpecSummary> |

### `get_spec_handler`

Get a single specification.

```rust
/// Returns full spec details including content.
///
/// # Errors
/// - `NotFound` if spec doesn't exist
pub async fn get_spec_handler(
    state: &AppState,
    name: String,
) -> Result<SpecDetail, TransportError>
```

| Transport | Mapping |
|-----------|---------|
| Axum | `GET /api/specs/:name` → 200 OK |
| Tauri | `invoke("get_spec", { name })` → SpecDetail |

### `create_spec_handler`

Create a new specification.

```rust
/// Creates a new spec from a description.
pub async fn create_spec_handler(
    state: &AppState,
    request: CreateSpecRequest,
) -> Result<SpecSummary, TransportError>
```

| Transport | Mapping |
|-----------|---------|
| Axum | `POST /api/specs` → 201 Created |
| Tauri | `invoke("create_spec", { request })` → SpecSummary |

### `update_spec_handler`

Update a specification.

```rust
/// Updates an existing spec.
///
/// # Errors
/// - `NotFound` if spec doesn't exist
pub async fn update_spec_handler(
    state: &AppState,
    name: String,
    request: UpdateSpecRequest,
) -> Result<SpecDetail, TransportError>
```

| Transport | Mapping |
|-----------|---------|
| Axum | `PUT /api/specs/:name` → 200 OK |
| Tauri | `invoke("update_spec", { name, request })` → SpecDetail |

### `delete_spec_handler`

Delete a specification.

```rust
/// Deletes a spec and its artifacts.
///
/// # Errors
/// - `NotFound` if spec doesn't exist
/// - `Conflict` if spec has running execution
pub async fn delete_spec_handler(
    state: &AppState,
    name: String,
) -> Result<(), TransportError>
```

| Transport | Mapping |
|-----------|---------|
| Axum | `DELETE /api/specs/:name` → 204 No Content |
| Tauri | `invoke("delete_spec", { name })` → () |

---

## Execution Handlers

### `start_execution_handler`

Start executing a spec.

```rust
/// Starts execution of a spec.
///
/// # Errors
/// - `NotFound` if spec doesn't exist
/// - `Conflict` if spec is already running
pub async fn start_execution_handler(
    state: &AppState,
    request: StartExecutionRequest,
) -> Result<StartExecutionResponse, TransportError>
```

| Transport | Mapping |
|-----------|---------|
| Axum | `POST /api/execution/start` → 200 OK |
| Tauri | `invoke("start_execution", { request })` → StartExecutionResponse |

### `stop_execution_handler`

Stop a running execution.

```rust
/// Stops execution of a spec.
///
/// # Errors
/// - `NotFound` if no running execution
pub async fn stop_execution_handler(
    state: &AppState,
    request: StopExecutionRequest,
) -> Result<(), TransportError>
```

| Transport | Mapping |
|-----------|---------|
| Axum | `POST /api/execution/stop` → 200 OK |
| Tauri | `invoke("stop_execution", { request })` → () |

### `get_execution_status_handler`

Get execution status.

```rust
/// Returns current execution status for a spec.
pub async fn get_execution_status_handler(
    state: &AppState,
    spec: String,
) -> Result<Option<ExecutionRun>, TransportError>
```

| Transport | Mapping |
|-----------|---------|
| Axum | `GET /api/execution/:spec` → 200 OK |
| Tauri | `invoke("get_execution_status", { spec })` → Option<ExecutionRun> |

---

## History Handlers

### `list_runs_handler`

List execution history.

```rust
/// Returns execution history, optionally filtered by spec.
pub async fn list_runs_handler(
    state: &AppState,
    spec: Option<String>,
) -> Result<Vec<RunSummary>, TransportError>
```

| Transport | Mapping |
|-----------|---------|
| Axum | `GET /api/history?spec=<name>` → 200 OK |
| Tauri | `invoke("list_runs", { spec })` → Vec<RunSummary> |

### `get_run_handler`

Get run details.

```rust
/// Returns detailed run information including task outputs.
pub async fn get_run_handler(
    state: &AppState,
    run_id: String,
) -> Result<RunDetail, TransportError>
```

| Transport | Mapping |
|-----------|---------|
| Axum | `GET /api/history/:run_id` → 200 OK |
| Tauri | `invoke("get_run", { run_id })` → RunDetail |

---

## Docker Handlers

### `check_docker_handler`

Check Docker status.

```rust
/// Checks if Docker is available and running.
pub async fn check_docker_handler(
) -> Result<DockerStatus, TransportError>
```

| Transport | Mapping |
|-----------|---------|
| Axum | `GET /api/docker/status` → 200 OK |
| Tauri | `invoke("check_docker")` → DockerStatus |

---

## Special Handlers (Transport-Specific)

### Events (SSE vs Tauri Events)

```rust
// Axum: Server-Sent Events
pub async fn events_stream(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = ...>>

// Tauri: Event emission (handled differently)
pub fn emit_event(app_handle: &AppHandle, event: TransportEvent)
```

### Terminal (WebSocket vs PTY)

```rust
// Axum: WebSocket upgrade
pub async fn terminal_ws(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse

// Tauri: Direct PTY integration (handled in tauri module)
pub fn create_pty_session(/* ... */) -> Result<PtySession, TransportError>
```

---

## Error Mapping

### Axum Error Response

```rust
impl IntoResponse for TransportError {
    fn into_response(self) -> Response {
        let status = match &self {
            TransportError::NotFound(_) => StatusCode::NOT_FOUND,
            TransportError::BadRequest(_) => StatusCode::BAD_REQUEST,
            TransportError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            TransportError::Forbidden(_) => StatusCode::FORBIDDEN,
            TransportError::Conflict(_) => StatusCode::CONFLICT,
            TransportError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            TransportError::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
        };
        
        let body = Json(serde_json::json!({
            "error": self.to_string()
        }));
        
        (status, body).into_response()
    }
}
```

### Tauri Error Response

```rust
impl From<TransportError> for String {
    fn from(err: TransportError) -> String {
        err.to_string()
    }
}
```
