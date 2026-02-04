# Adding New Endpoints

This guide explains how to add new API endpoints to the ckrv-transport crate.

## Overview

Adding a new endpoint requires modifications in **only one crate** (`ckrv-transport`), ensuring both Axum (web) and Tauri (desktop) backends automatically get the new functionality.

## Steps

### 1. Define Types (if needed)

If your endpoint needs new request/response types, add them to the appropriate file in `src/types/`:

```rust
// src/types/my_feature.rs
use serde::{Deserialize, Serialize};

#[cfg(feature = "typescript")]
use ts_rs::TS;

/// Request for my feature.
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct MyFeatureRequest {
    pub name: String,
    pub value: u32,
}

/// Response from my feature.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct MyFeatureResponse {
    pub success: bool,
    pub result: String,
}
```

Then add it to `src/types/mod.rs`:

```rust
pub mod my_feature;
pub use my_feature::*;
```

### 2. Create Handler

Create the transport-agnostic handler in `src/handlers/`:

```rust
// src/handlers/my_feature.rs
use crate::error::TransportError;
use crate::state::AppState;
use crate::types::{MyFeatureRequest, MyFeatureResponse};

/// Handle my feature request.
pub async fn my_feature_handler(
    state: &AppState,
    request: MyFeatureRequest,
) -> Result<MyFeatureResponse, TransportError> {
    // Your business logic here
    // Use state.project_root for file operations
    
    Ok(MyFeatureResponse {
        success: true,
        result: format!("Processed: {}", request.name),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_my_feature_handler() {
        let state = AppState::new(PathBuf::from("/tmp/test"));
        let request = MyFeatureRequest {
            name: "test".to_string(),
            value: 42,
        };
        let result = my_feature_handler(&state, request).await;
        assert!(result.is_ok());
    }
}
```

Add it to `src/handlers/mod.rs`:

```rust
pub mod my_feature;
pub use my_feature::*;
```

### 3. Create Axum Wrapper

Create the Axum route wrapper in `src/axum/`:

```rust
// src/axum/my_feature.rs
use crate::handlers::my_feature::my_feature_handler;
use crate::state::AppState;
use crate::types::MyFeatureRequest;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Json, Router};

/// Handle POST request for my feature.
async fn my_feature(
    State(state): State<AppState>,
    Json(request): Json<MyFeatureRequest>,
) -> impl IntoResponse {
    match my_feature_handler(&state, request).await {
        Ok(result) => Json(result).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Create my feature routes.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/my-feature", post(my_feature))
}
```

Add it to `src/axum/mod.rs`:

```rust
pub mod my_feature;

// In create_router():
.merge(my_feature::routes())
```

### 4. (Optional) Create Tauri Command

If you want the feature available in the desktop app:

```rust
// src/tauri/my_feature.rs
use crate::handlers::my_feature::my_feature_handler;
use crate::state::AppState;
use crate::types::{MyFeatureRequest, MyFeatureResponse};

#[tauri::command]
pub async fn my_feature(
    state: tauri::State<'_, AppState>,
    request: MyFeatureRequest,
) -> Result<MyFeatureResponse, String> {
    my_feature_handler(&state, request)
        .await
        .map_err(|e| e.to_string())
}
```

### 5. Verify

```bash
# Build with axum feature
cargo build -p ckrv-transport --features axum

# Run tests
cargo test -p ckrv-transport --features axum

# Test endpoint manually
curl -X POST http://localhost:3000/api/my-feature \
  -H "Content-Type: application/json" \
  -d '{"name": "test", "value": 42}'
```

## Handler Pattern

All handlers follow this signature:

```rust
pub async fn handler_name(
    state: &AppState,           // Required: Shared state
    request: RequestType,       // Optional: Input data
) -> Result<ResponseType, TransportError>
```

### Error Handling

Use `TransportError` variants:

```rust
// Bad request (400)
Err(TransportError::BadRequest("Missing required field".to_string()))

// Not found (404)
Err(TransportError::NotFound("Spec not found".to_string()))

// Internal error (500)
Err(TransportError::Internal("Database connection failed".to_string()))
```

### State Access

The `AppState` provides:

```rust
state.project_root     // PathBuf: Project root directory
state.hub              // Arc<Hub>: Event broadcasting
```

## Best Practices

1. **Keep handlers pure**: No transport-specific code in handlers
2. **Test handlers directly**: Unit test without HTTP layer
3. **Use typed errors**: Return `TransportError`, not strings
4. **Document types**: Add rustdoc comments for TypeScript generation
5. **Add tests**: Every handler should have at least one test

## TypeScript Types

When you add `#[derive(TS)]` to types, they'll be exported to TypeScript:

```bash
cargo test -p ckrv-transport --features typescript

# Types exported to: crates/ckrv-ui/frontend/src/types/api.generated.ts
```
