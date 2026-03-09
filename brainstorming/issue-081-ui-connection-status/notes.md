# UI: Connection Badge Flips to Disconnected on Message Submit

**Issue**: [#81](https://github.com/FnSK4R17s/chakravarti-cli/issues/81)
**Created**: 2026-03-09
**Status**: Tasks Generated

## Problem Statement

When a user submits a message (spec creation) in the dashboard, the "Connected" badge in the header immediately flips to "Disconnected" then recovers after a few seconds. This undermines trust in the UI and makes users think the server crashed.

## Current State

### Backend: Blocking I/O in async handlers

Every command handler in `ckrv-transport` uses synchronous `std::process::Command::output()` directly inside `async fn` axum handlers — no `spawn_blocking`. This blocks the Tokio executor threads.

**Affected handlers:**
- `handlers/commands.rs` — `run_ckrv_command()` spawns `ckrv` CLI subprocess (can take seconds)
- `handlers/status.rs` — `detect_git_branch()` runs 3 synchronous git commands
- `handlers/specs.rs` — multiple handlers with synchronous CLI/filesystem calls

When `POST /api/command/spec-new` blocks an executor thread for seconds, concurrent `GET /api/status` requests either queue behind it or time out.

### Frontend: Aggressive polling + duplication

Two independent systems poll `/api/status` every 5 seconds:
1. `useConnection(5000)` hook in `Dashboard.tsx:58` — custom fetch with 3s abort timeout
2. React Query `['status']` in `Dashboard.tsx:65-72` — separate `useQuery` with `refetchInterval: 5000`

Additionally, `ChatDashboard.tsx:121` invalidates `['status']` on mutation success, triggering yet another immediate fetch during server load.

The `useConnection` hook has a strict 3s `AbortController` timeout — any delay from blocked executor threads = instant "disconnected".

## Proposed Solution

Two-pronged fix: unblock the backend, harden the frontend.

### Backend: `spawn_blocking` for all CLI/git subprocess calls

Wrap every handler that calls `std::process::Command` with `tokio::task::spawn_blocking()` so they run on Tokio's blocking thread pool instead of starving the async executor.

### Frontend: Single source of truth + resilience

- Remove the duplicate polling (either `useConnection` or the React Query `['status']`, not both)
- Add a grace period (e.g., 2 consecutive failures) before flipping to "disconnected"
- Increase abort timeout from 3s to 5s

## Technical Approach

### Options Considered

| Option | Pros | Cons |
|--------|------|------|
| A: `spawn_blocking` on all handlers | Simple, targeted fix, correct Tokio pattern | Doesn't reduce actual request latency |
| B: Convert to `tokio::process::Command` | Truly async, no thread pool needed | Larger refactor, changes handler signatures, needs `async` throughout |
| C: Add `/api/health` lightweight endpoint | Decouples health check from heavy `/api/status` | Doesn't fix the root blocking issue, just masks it |
| D: Frontend-only fix (longer timeout, retries) | No backend changes needed | Papering over the real problem |

### Decision

**Option A (spawn_blocking) + frontend dedup** — Best effort-to-impact ratio. Option B is the ideal long-term path but is a larger refactor. Option A fixes the immediate bug with minimal change.

## Implementation Notes

### Backend changes

**`crates/ckrv-transport/src/axum/commands.rs`** — All handlers need wrapping:
```rust
async fn run_spec_new(
    State(state): State<AppState>,
    Json(request): Json<SpecNewRequest>,
) -> impl IntoResponse {
    let result = tokio::task::spawn_blocking(move || {
        run_spec_new_handler(&state, request)
    }).await;
    match result {
        Ok(Ok(resp)) => Json(resp).into_response(),
        Ok(Err(e)) => e.into_response(),
        Err(e) => TransportError::Internal(format!("Task panicked: {e}")).into_response(),
    }
}
```

**`crates/ckrv-transport/src/axum/status.rs`** — Same pattern for status handler.

**Note:** `AppState` needs to be `Clone` + `Send + 'static` for `spawn_blocking` move closure. It already uses `Arc<RwLock<_>>` internally so this should work, but verify.

### Frontend changes

**`crates/ckrv-ui/frontend/src/hooks/useConnection.ts`**:
- Increase timeout from 3s to 5s
- Add consecutive failure count — only flip to "disconnected" after 2+ failures
- Use a lighter endpoint if available (`/health` returns `"OK"` and never blocks)

**`crates/ckrv-ui/frontend/src/layouts/Dashboard.tsx`**:
- Remove the separate `useQuery(['status'], ...)` that duplicates `useConnection`
- OR remove `useConnection` and derive connection status from the React Query status

## Open Questions

- [ ] Should we add a dedicated `/api/ping` or use existing `/health` for connection checks instead of `/api/status`?
- [ ] Do we want to convert to `tokio::process::Command` (Option B) as a follow-up?
- [ ] Should `useConnection` use `/health` (instant response) while React Query uses `/api/status` (richer data)?

## Success Criteria

| Metric | Target |
|--------|--------|
| Connection badge stays "connected" during spec creation | Always |
| No duplicate `/api/status` requests in network tab | Verified |
| Status endpoint responds within 1s even during command execution | p99 < 1s |

## Next Steps

- [ ] Implement `spawn_blocking` wrappers in `axum/commands.rs`
- [ ] Implement `spawn_blocking` wrapper in `axum/status.rs`
- [ ] Deduplicate frontend status polling
- [ ] Add connection resilience (consecutive failure threshold)
- [ ] Test with `ckrv ui` and verify badge stays green during spec submission

## References

- [Tokio: spawn_blocking](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html)
- [Axum: don't block the executor](https://docs.rs/axum/latest/axum/index.html#handlers)
- `crates/ckrv-transport/src/axum/commands.rs` — route handlers
- `crates/ckrv-transport/src/handlers/status.rs` — blocking git calls
- `crates/ckrv-ui/frontend/src/hooks/useConnection.ts` — connection polling
- `crates/ckrv-ui/frontend/src/layouts/Dashboard.tsx` — duplicate polling
