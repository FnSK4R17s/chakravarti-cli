---
name: Axum Routing Best Practices
description: Axum web framework routing patterns, path parameters, and common pitfalls. Essential for building HTTP APIs.
---

# Axum Routing Best Practices

This skill covers Axum routing patterns, focusing on common mistakes and breaking changes between versions.

## Critical: Path Parameter Syntax (Axum 0.7+ → 0.8+)

### ⚠️ Breaking Change Alert

**Axum 0.8+ changed path parameter syntax:**

| Version | Syntax | Example |
|---------|--------|---------|
| Axum 0.7 and earlier | `:param` | `/users/:id` |
| **Axum 0.8+** | `{param}` | `/users/{id}` |

### ❌ WRONG (will panic at runtime)
```rust
// This causes a panic in Axum 0.8+!
.route("/users/:id", get(get_user))
.route("/specs/:name/tasks/:task_id", get(get_task))
```

**Error message:**
```
Path segments must not start with `:`. For capture groups, use `{capture}`. 
If you meant to literally match a segment starting with a colon, call `without_v07_checks` on the router.
```

### ✅ CORRECT (Axum 0.8+)
```rust
.route("/users/{id}", get(get_user))
.route("/specs/{name}/tasks/{task_id}", get(get_task))
```

### Why This Matters

- The panic happens **at runtime** when the router is constructed
- This means tests won't catch it unless they actually start the server
- The error message is helpful but the app crashes on startup

---

## Route Definition Patterns

### Basic Routes

```rust
use axum::{Router, routing::{get, post, put, delete, patch}};

pub fn routes() -> Router<AppState> {
    Router::new()
        // Static routes
        .route("/health", get(health_check))
        .route("/users", get(list_users).post(create_user))
        
        // Dynamic routes with path parameters
        .route("/users/{id}", get(get_user).put(update_user).delete(delete_user))
        
        // Nested path parameters
        .route("/users/{user_id}/posts/{post_id}", get(get_user_post))
}
```

### Extracting Path Parameters

```rust
use axum::extract::Path;

// Single parameter
async fn get_user(Path(id): Path<String>) -> impl IntoResponse {
    // id is extracted from /users/{id}
}

// Multiple parameters (tuple)
async fn get_user_post(
    Path((user_id, post_id)): Path<(String, String)>
) -> impl IntoResponse {
    // user_id from {user_id}, post_id from {post_id}
}

// Struct extraction (for many parameters)
#[derive(Deserialize)]
struct PostParams {
    user_id: String,
    post_id: String,
}

async fn get_user_post_v2(Path(params): Path<PostParams>) -> impl IntoResponse {
    // Access params.user_id, params.post_id
}
```

---

## Route Organization

### Module Pattern

```rust
// src/axum/mod.rs
mod users;
mod posts;
mod health;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .merge(users::routes())
        .merge(posts::routes())
        .merge(health::routes())
        .with_state(state)
}

// src/axum/users.rs
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/users", get(list_users).post(create_user))
        .route("/users/{id}", get(get_user).delete(delete_user))
}
```

### Nesting Routes

```rust
// Nest all API routes under /api prefix
let app = Router::new()
    .route("/health", get(health_check))
    .nest("/api", api_router)  // All api routes prefixed with /api
    .with_state(state);
```

---

## Response Patterns

### Standard Response Pattern

```rust
use axum::{Json, response::IntoResponse};
use axum::http::StatusCode;

async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match get_user_handler(&state, id).await {
        Ok(user) => Json(user).into_response(),
        Err(e) => e.into_response(),  // Error type implements IntoResponse
    }
}

// For creation (201 Created)
async fn create_user(
    State(state): State<AppState>,
    Json(request): Json<CreateUserRequest>,
) -> impl IntoResponse {
    match create_user_handler(&state, request).await {
        Ok(user) => (StatusCode::CREATED, Json(user)).into_response(),
        Err(e) => e.into_response(),
    }
}

// For deletion (204 No Content)
async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match delete_user_handler(&state, id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => e.into_response(),
    }
}
```

---

## Common Mistakes

### 1. Route Order Matters

```rust
// ❌ WRONG: Static route after dynamic route
Router::new()
    .route("/users/{id}", get(get_user))
    .route("/users/me", get(get_current_user))  // Never reached!

// ✅ CORRECT: Static routes first
Router::new()
    .route("/users/me", get(get_current_user))  // Matched first
    .route("/users/{id}", get(get_user))
```

### 2. Forgetting State

```rust
// ❌ WRONG: Router without state type
pub fn routes() -> Router {
    Router::new()
        .route("/users", get(list_users))  // Handler uses State<AppState>
}

// ✅ CORRECT: Specify state type
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/users", get(list_users))
}
```

### 3. Conflicting Routes

```rust
// ❌ WRONG: Same path, different methods defined separately
Router::new()
    .route("/users", get(list_users))
    .route("/users", post(create_user))  // Conflicts!

// ✅ CORRECT: Chain methods on same route
Router::new()
    .route("/users", get(list_users).post(create_user))
```

---

## Quick Reference

### HTTP Method Mapping

| Method | Axum Function | Typical Use |
|--------|---------------|-------------|
| GET | `get()` | Fetch resource(s) |
| POST | `post()` | Create resource |
| PUT | `put()` | Replace resource |
| PATCH | `patch()` | Partial update |
| DELETE | `delete()` | Remove resource |

### Path Parameter Syntax (Axum 0.8+)

| Pattern | Example URL | Extracted Value |
|---------|-------------|-----------------|
| `/{id}` | `/123` | `"123"` |
| `/{a}/{b}` | `/foo/bar` | `("foo", "bar")` |
| `/*path` | `/a/b/c` | `"a/b/c"` (wildcard) |

---

## Verification Checklist

Before committing Axum route changes:

- [ ] All path parameters use `{param}` syntax (not `:param`)
- [ ] Static routes come before dynamic routes for same prefix
- [ ] Router has correct state type: `Router<AppState>`
- [ ] Multiple methods on same path are chained, not separate
- [ ] Run `cargo build` - Axum validates routes at compile time for some issues
- [ ] Run the server locally - path parameter syntax is validated at runtime
