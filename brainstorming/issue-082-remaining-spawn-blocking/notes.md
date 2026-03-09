# Wrap Remaining Blocking Handlers with spawn_blocking

**Issue**: [#82](https://github.com/FnSK4R17s/chakravarti-cli/issues/82)
**Created**: 2026-03-09
**Status**: Draft
**Related**: [#81](https://github.com/FnSK4R17s/chakravarti-cli/issues/81) — initial fix for 5 files

## Problem Statement

Issue #81 fixed the connection badge bug by wrapping 5 axum route files with `spawn_blocking`. A full audit revealed 12 more files with blocking calls in async handlers. While these don't currently trigger the connection badge bug (since `/health` is now used for heartbeat), they still starve the Tokio executor and can cause latency spikes across all concurrent requests.

## Audit Results

### Already Fixed in #81

| Axum Route File | Blocking Pattern |
|-----------------|-----------------|
| `axum/commands.rs` | `std::process::Command` (ckrv CLI) |
| `axum/status.rs` | `std::process::Command` (3 git calls) |
| `axum/specs.rs` | `std::fs` + `std::process::Command` |
| `axum/docker.rs` | `std::process::Command` (docker info/version) |
| `axum/cloud.rs` | `std::process::Command` (cloud auth) |

### Already Async (No Fix Needed)

| Axum Route File | Pattern |
|-----------------|---------|
| `axum/test.rs` | Uses `tokio::process::Command` |
| `axum/qa.rs` | Uses `tokio::process::Command` |

### Still Blocking — `std::process::Command` in async handlers

| Axum Route File | Handler Function | What It Spawns | Severity |
|-----------------|-----------------|----------------|----------|
| `axum/agents.rs` | `test_agent_handler` | 11+ CLI version checks (claude, codex, kilo, etc.) | **High** — tests all agents sequentially |
| `axum/console.rs` | `execute_command_handler` | `sh -c <command>` — arbitrary shell exec | **Critical** — unbounded runtime |
| `axum/diff.rs` | `get_branches_handler` | 2 git commands (rev-parse, branch list) | **High** |
| `axum/diff.rs` | `get_diff_handler` | 4+ git commands (diff, apply, checkout) | **High** |
| `axum/execution.rs` | `start_execution_handler` | `ckrv execute` subprocess spawn | **Critical** |
| `axum/execution.rs` | `stop_execution_handler` | `ckrv abort` | **High** |
| `axum/execution.rs` | `list_branches_handler` | 4+ git commands (rev-parse, worktree, merge-base, rev-list) | **High** |
| `axum/execution.rs` | `merge_all_branches_handler` | Multiple git merge/worktree commands | **Critical** — merge can take seconds |
| `axum/execution.rs` | `merge_branch_handler` | `git merge` | **High** |
| `axum/tasks.rs` | `list_tasks_handler` | `git rev-parse` for branch detection | **Medium** |

### Still Blocking — `std::fs` in async handlers

| Axum Route File | Handler Function | Operations | Severity |
|-----------------|-----------------|------------|----------|
| `axum/agents.rs` | `load_agents`, `save_agents` | `fs::read_to_string`, `fs::write`, `fs::create_dir_all` | **Medium** |
| `axum/history.rs` | `load_history`, `save_history` | `fs::read_to_string`, `fs::write` | **Medium** |
| `axum/tasks.rs` | list/get/update handlers | `fs::read_to_string`, `fs::write` | **Medium** |
| `axum/plans.rs` | list/get/update/delete | `fs::read_dir`, `fs::read_to_string`, `fs::write`, `fs::remove_file` | **Medium** |

### Still Blocking — `std::sync::Mutex::lock()` in async handlers

| Axum Route File | Handler Function | What's Locked | Severity |
|-----------------|-----------------|---------------|----------|
| `axum/session.rs` | All handlers (list, create, get, delete) | `SESSIONS` static Mutex | **Medium** — short-held but blocks executor |
| `axum/terminal.rs` | All handlers | Terminal process Mutex | **High** — held while managing PTY processes |

## Proposed Solution

Apply the same `spawn_blocking` pattern from #81 to all 12 remaining files. The pattern is mechanical and identical:

```rust
// Before (blocking)
async fn some_handler(State(state): State<AppState>) -> impl IntoResponse {
    match some_sync_handler(&state) {
        Ok(result) => Json(result).into_response(),
        Err(e) => e.into_response(),
    }
}

// After (non-blocking)
async fn some_handler(State(state): State<AppState>) -> impl IntoResponse {
    match tokio::task::spawn_blocking(move || some_sync_handler(&state)).await {
        Ok(Ok(result)) => Json(result).into_response(),
        Ok(Err(e)) => e.into_response(),
        Err(e) => TransportError::Internal(format!("Task panicked: {e}")).into_response(),
    }
}
```

### Mutex handlers (session.rs, terminal.rs)

For `std::sync::Mutex` in async context, two options:
1. **Wrap in `spawn_blocking`** — same as above, simplest
2. **Switch to `tokio::sync::Mutex`** — more idiomatic but requires signature changes

Option 1 is recommended for consistency with the rest of the codebase.

## Implementation Priority

| Priority | Files | Rationale |
|----------|-------|-----------|
| P0 | `console.rs`, `execution.rs` | Unbounded/long-running subprocess calls |
| P1 | `diff.rs`, `agents.rs`, `terminal.rs` | Multi-step git operations, PTY management |
| P2 | `tasks.rs`, `history.rs`, `plans.rs`, `session.rs` | Short filesystem/mutex operations |

## Open Questions

- [ ] Should `terminal.rs` Mutex be migrated to `tokio::sync::Mutex` instead of `spawn_blocking`? PTY management may benefit from async-native locking.
- [ ] Should `console.rs` have a timeout on the subprocess to prevent indefinite blocking?
- [ ] Long-term: convert all `std::process::Command` to `tokio::process::Command` across the codebase?

## References

- [#81 brainstorm](../issue-081-ui-connection-status/notes.md) — original fix and analysis
- [Tokio: spawn_blocking](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html)
- [Tokio: bridging sync and async](https://tokio.rs/tokio/topics/bridging)
- `crates/ckrv-transport/src/axum/` — all route handler files
- `crates/ckrv-transport/src/handlers/` — all handler implementation files
