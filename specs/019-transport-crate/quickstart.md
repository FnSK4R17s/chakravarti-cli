# Quickstart: ckrv-transport Development

**Feature**: 019-transport-crate
**Date**: 2026-02-04

## Overview

This guide helps developers understand how to work with the `ckrv-transport` crate, add new endpoints, and generate TypeScript types.

---

## Crate Structure

```
crates/ckrv-transport/
├── Cargo.toml           # Feature flags: axum, tauri
├── src/
│   ├── lib.rs           # Public exports, feature gates
│   ├── error.rs         # TransportError enum
│   ├── state.rs         # AppState struct
│   ├── types/           # Request/Response types
│   │   ├── mod.rs       # Re-exports all types
│   │   └── *.rs         # Domain-specific types
│   ├── handlers/        # Transport-agnostic handlers
│   │   ├── mod.rs       # Re-exports all handlers
│   │   └── *.rs         # Domain-specific handlers
│   ├── axum/            # Axum wrappers (feature = "axum")
│   │   ├── mod.rs       # create_router() function
│   │   └── *.rs         # Route handlers
│   └── tauri/           # Tauri wrappers (feature = "tauri")
│       ├── mod.rs       # get_invoke_handlers() macro
│       └── *.rs         # Tauri commands
└── tests/
    └── handler_tests.rs # Unit tests for handlers
```

---

## Adding a New Endpoint

### Step 1: Define Types

Add request/response types to `src/types/<domain>.rs`:

```rust
// src/types/widgets.rs
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Request to create a widget.
#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct CreateWidgetRequest {
    pub name: String,
    pub color: String,
}

/// Widget response.
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct Widget {
    pub id: String,
    pub name: String,
    pub color: String,
    pub created_at: String,
}
```

Re-export in `src/types/mod.rs`:

```rust
mod widgets;
pub use widgets::*;
```

### Step 2: Implement Handler

Add the handler to `src/handlers/<domain>.rs`:

```rust
// src/handlers/widgets.rs
use crate::error::TransportError;
use crate::state::AppState;
use crate::types::{CreateWidgetRequest, Widget};

/// Creates a new widget.
///
/// # Arguments
/// * `state` - Application state
/// * `request` - Widget creation request
///
/// # Returns
/// The created widget with generated ID.
///
/// # Errors
/// * `BadRequest` - Invalid widget parameters
pub async fn create_widget_handler(
    state: &AppState,
    request: CreateWidgetRequest,
) -> Result<Widget, TransportError> {
    // Validate input
    if request.name.is_empty() {
        return Err(TransportError::BadRequest("Name cannot be empty".into()));
    }
    
    // Business logic here
    let widget = Widget {
        id: uuid::Uuid::new_v4().to_string(),
        name: request.name,
        color: request.color,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    
    Ok(widget)
}

/// Lists all widgets.
pub async fn list_widgets_handler(
    state: &AppState,
) -> Result<Vec<Widget>, TransportError> {
    // Implementation
    Ok(vec![])
}
```

Re-export in `src/handlers/mod.rs`:

```rust
mod widgets;
pub use widgets::*;
```

### Step 3: Add Axum Wrapper

Add the route wrapper to `src/axum/widgets.rs`:

```rust
// src/axum/widgets.rs
use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use crate::handlers;
use crate::state::AppState;
use crate::types::CreateWidgetRequest;

pub async fn create_widget(
    State(state): State<AppState>,
    Json(request): Json<CreateWidgetRequest>,
) -> impl IntoResponse {
    match handlers::create_widget_handler(&state, request).await {
        Ok(widget) => Json(widget).into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn list_widgets(
    State(state): State<AppState>,
) -> impl IntoResponse {
    match handlers::list_widgets_handler(&state).await {
        Ok(widgets) => Json(widgets).into_response(),
        Err(e) => e.into_response(),
    }
}
```

Add routes to `src/axum/mod.rs`:

```rust
use axum::{routing::{get, post}, Router};

mod widgets;

pub fn create_router() -> Router<AppState> {
    Router::new()
        // ... existing routes ...
        .route("/api/widgets", get(widgets::list_widgets))
        .route("/api/widgets", post(widgets::create_widget))
}
```

### Step 4: Add Tauri Wrapper (Optional)

Add Tauri commands to `src/tauri/widgets.rs`:

```rust
// src/tauri/widgets.rs
use tauri::State;
use crate::handlers;
use crate::state::AppState;
use crate::types::CreateWidgetRequest;

#[tauri::command]
pub async fn create_widget(
    state: State<'_, AppState>,
    request: CreateWidgetRequest,
) -> Result<Widget, String> {
    handlers::create_widget_handler(&state, request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_widgets(
    state: State<'_, AppState>,
) -> Result<Vec<Widget>, String> {
    handlers::list_widgets_handler(&state)
        .await
        .map_err(|e| e.to_string())
}
```

Register in `src/tauri/mod.rs`:

```rust
mod widgets;

pub fn get_invoke_handlers() -> impl Fn(tauri::Invoke) {
    tauri::generate_handler![
        // ... existing handlers ...
        widgets::create_widget,
        widgets::list_widgets,
    ]
}
```

### Step 5: Add Tests

Add unit tests to `tests/handler_tests.rs`:

```rust
use ckrv_transport::{handlers, types::CreateWidgetRequest};

#[tokio::test]
async fn test_create_widget_success() {
    let state = test_app_state();
    let request = CreateWidgetRequest {
        name: "Test Widget".into(),
        color: "blue".into(),
    };
    
    let result = handlers::create_widget_handler(&state, request).await;
    
    assert!(result.is_ok());
    let widget = result.unwrap();
    assert_eq!(widget.name, "Test Widget");
    assert_eq!(widget.color, "blue");
}

#[tokio::test]
async fn test_create_widget_empty_name_fails() {
    let state = test_app_state();
    let request = CreateWidgetRequest {
        name: "".into(),
        color: "blue".into(),
    };
    
    let result = handlers::create_widget_handler(&state, request).await;
    
    assert!(matches!(result, Err(TransportError::BadRequest(_))));
}
```

### Step 6: Generate TypeScript Types

Run type generation:

```bash
# From repository root
cd crates/ckrv-transport
cargo test  # This triggers ts-rs generation

# Or specifically
cargo test --features typescript export_bindings
```

Update frontend types:

```bash
# Copy generated types
cp crates/ckrv-transport/bindings/*.ts crates/ckrv-ui/frontend/src/types/
```

---

## Building and Testing

### Build with Axum Feature

```bash
cd crates/ckrv-transport
cargo build --features axum
```

### Build with Tauri Feature

```bash
cd crates/ckrv-transport
cargo build --features tauri
```

### Run Tests (No Features)

```bash
cd crates/ckrv-transport
cargo test
```

### Run Tests with Axum

```bash
cd crates/ckrv-transport
cargo test --features axum
```

---

## Using in Consumer Crates

### In ckrv-ui (Web)

```toml
# crates/ckrv-ui/Cargo.toml
[dependencies]
ckrv-transport = { workspace = true, features = ["axum"] }
```

```rust
// crates/ckrv-ui/src/lib.rs
use ckrv_transport::{axum::create_router, AppState};

pub async fn run_server(state: AppState) {
    let app = create_router().with_state(state);
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

### In ckrv-tauri (Desktop)

```toml
# crates/ckrv-tauri/Cargo.toml
[dependencies]
ckrv-transport = { workspace = true, features = ["tauri"] }
```

```rust
// crates/ckrv-tauri/src/main.rs
use ckrv_transport::{tauri::get_invoke_handlers, AppState};

fn main() {
    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(get_invoke_handlers())
        .run(tauri::generate_context!())
        .unwrap();
}
```

---

## Checklist for New Endpoints

- [ ] Types defined in `src/types/<domain>.rs`
- [ ] Types exported in `src/types/mod.rs`
- [ ] Handler implemented in `src/handlers/<domain>.rs`
- [ ] Handler exported in `src/handlers/mod.rs`
- [ ] Axum wrapper in `src/axum/<domain>.rs` (if using axum feature)
- [ ] Route added to `src/axum/mod.rs`
- [ ] Tauri command in `src/tauri/<domain>.rs` (if using tauri feature)
- [ ] Command registered in `src/tauri/mod.rs`
- [ ] Unit tests added
- [ ] TypeScript types regenerated
- [ ] Documentation updated

---

## Common Patterns

### Error Handling

```rust
// In handlers - return TransportError
pub async fn my_handler(...) -> Result<T, TransportError> {
    let item = find_item(id)
        .ok_or_else(|| TransportError::NotFound(format!("Item {} not found", id)))?;
    
    validate(&item)
        .map_err(|e| TransportError::BadRequest(e.to_string()))?;
    
    Ok(item)
}
```

### Async Operations

```rust
// Use tokio for async operations
pub async fn my_handler(state: &AppState) -> Result<T, TransportError> {
    let data = state.status.read().await;
    // ... use data
    Ok(result)
}
```

### File Operations

```rust
// Use state.project_root for file paths
pub async fn read_config_handler(state: &AppState) -> Result<Config, TransportError> {
    let config_path = state.project_root.join(".ckrv/config.yaml");
    
    let contents = tokio::fs::read_to_string(&config_path)
        .await
        .map_err(|e| TransportError::Internal(e.to_string()))?;
    
    let config: Config = serde_yaml::from_str(&contents)
        .map_err(|e| TransportError::Internal(e.to_string()))?;
    
    Ok(config)
}
```
