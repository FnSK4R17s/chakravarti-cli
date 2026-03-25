# Test Strategy: Execution Pipeline & System-Wide Gaps

**Created**: 2026-03-22
**Status**: Brainstorming
**Context**: Issue #88 (execution pipeline) is now fully implemented but undertested. This doc covers testing the execution pipeline specifically and the systemic gaps that surfaced during review.

## Problem Statement

The execution pipeline is the riskiest code in the codebase and the least tested. The `execution.rs` handler — which manages run lifecycle, event bridging, JSONL persistence, Docker cleanup, and cancellation — has 5 tests. Three check trivial things. The core `run_orchestration` function has zero test coverage.

More broadly:
- **175 Rust tests** sounds healthy until you look at distribution: ckrv-sandbox has 53 (agent parsing), ckrv-transport has 70 (mostly type serialization). The execution _logic_ is a desert.
- **`#[ignore]` tests never run in CI** — Docker and API key tests are permanently skipped. For a tool whose core value is orchestrating Docker containers, this means every PR merges without validating the actual product path.
- **Frontend-backend contract is unvalidated** — TypeScript codegen (`export_types.rs`) is `#[ignore]`d. Rust struct changes silently break the frontend until E2E catches it (or a user does).
- **26% frontend line coverage** — below the point where it catches anything. The threshold exists but never fails, which is worse than no threshold (false confidence).

### Why this matters now

Issue #88 added JSONL log persistence, stderr forwarding, and Docker cleanup. None of that has tests. The next person to touch `execution.rs` — including an agent — has no safety net. Given the vision of "fire and forget" agent execution, untested orchestration code is the one place where a regression means the product literally stops working.

## Current State

### Test distribution by area (Rust)

| Area | Tests | Coverage quality |
|------|-------|-----------------|
| Agent type parsing (ckrv-sandbox) | 53 | Good — comprehensive |
| CLI commands (ckrv-cli integration) | 52 | Good — real binary, temp repos |
| Type serialization (ckrv-transport) | ~30 | Adequate — boilerplate but catches serde regressions |
| Hub broadcast (ckrv-transport) | 3 | Adequate — simple pub-sub |
| State/registry (ckrv-transport) | 3 | Thin — only checks defaults |
| **Execution pipeline** (ckrv-transport) | **5** | **Critical gap** |
| Orchestrator (ckrv-core) | ~10 | Thin — simulated steps only |
| Git operations (ckrv-git) | 10 | Adequate |
| Verification (ckrv-verify) | ~8 | Adequate |
| Metrics/cost (ckrv-metrics, ckrv-model) | ~10 | Adequate |

### What the 5 execution tests actually cover

```
test_get_execution_status_handler_no_active_run  — empty registry returns running: false
test_start_blocks_concurrent_execution           — concurrent guard rejects second run
test_stop_nonexistent_run                        — stop on empty registry returns error
test_hub_event_handler_maps_step_started         — StepStarted → StepStart mapping
test_hub_event_handler_maps_step_failed          — StepFailed → StepEnd + Log mapping
```

### What's NOT tested

- JSONL write + read roundtrip (just implemented, zero coverage)
- Cancellation flow (token fires → registry updated → cleanup runs)
- `cleanup_docker_containers` (doesn't panic when Docker is missing)
- stderr forwarding metadata (level: "error" in metadata)
- `run_orchestration` plan loading and batch iteration
- `with_log_file` constructor (directory creation, file open)
- `persist_event` (JSON serialization, flush behavior)
- Status handler with an active run (only tests empty case)
- Stop handler with a running entry (only tests nonexistent case)

### Frontend gaps

| Area | Status |
|------|--------|
| Vitest unit tests | 17 files, 26% line coverage (threshold never fails) |
| E2E (Playwright) | 6 suites, well-structured, in CI |
| Contract validation | `export_types.rs` is `#[ignore]` — codegen divergence is silent |
| MSW mocks | Exist but may drift from real API shapes |

### CI pipeline gaps

| Check | Status | Risk |
|-------|--------|------|
| `cargo test --workspace` | Runs (excludes `#[ignore]`) | Misses Docker/API tests |
| TypeScript codegen check | Not in CI | Frontend breaks silently |
| Docker smoke test | Not in CI | Core product path untested |
| `#[ignore]` tests (agents) | Never run | Only manual validation |

## Proposed Test Architecture

### Tier 1: Unit tests (every save, < 10s)

Fast, isolated, no I/O. Test pure logic and data transformations.

**Execution pipeline — what to add:**

```rust
// In execution.rs #[cfg(test)]

#[test]
fn test_jsonl_roundtrip() {
    // Create temp dir, HubEventHandler::with_log_file
    // Fire events through handler
    // Read back via read_jsonl_logs
    // Assert events roundtrip correctly
}

#[test]
fn test_with_log_file_creates_directory() {
    // Verify nested directory creation
    // Verify file exists and is appendable
}

#[test]
fn test_persist_event_flushes() {
    // Write event, read file immediately
    // Assert content is present (not buffered)
}

#[test]
fn test_stderr_events_have_error_level() {
    // Construct Log event with metadata
    // Assert metadata.level == "error"
}

#[test]
fn test_cleanup_docker_containers_noop_when_docker_missing() {
    // Call cleanup_docker_containers("nonexistent-spec")
    // Should not panic
}

#[tokio::test]
async fn test_status_handler_with_active_run() {
    // Insert a Running entry into registry
    // Call get_execution_status_handler
    // Assert running: true, correct spec_name
}

#[tokio::test]
async fn test_stop_handler_cancels_running_entry() {
    // Insert a Running entry with cancel_token
    // Call stop_execution_handler
    // Assert cancel_token.is_cancelled()
}

#[tokio::test]
async fn test_stop_handler_rejects_completed_run() {
    // Insert a Done entry
    // Call stop_execution_handler
    // Assert error "Run is not active"
}
```

**What NOT to unit test:**
- `run_orchestration` — it calls `Command::new("ckrv")` and reads the filesystem. Don't mock all that. Test the components it calls (event handler, plan parsing) and cover the integration via Tier 2.
- WebSocket forwarding — already covered by E2E. Unit testing WebSocket upgrade logic is pain for no gain.

### Tier 2: Integration tests (pre-commit, < 2 min)

Test handler flows with real `AppState`, temp directories, and fixture files. No Docker, no network.

**What to add:**

```rust
// crates/ckrv-transport/tests/execution_integration.rs

#[tokio::test]
async fn test_start_creates_registry_entry_and_log_dir() {
    // Create AppState with temp dir
    // Write minimal spec + plan.yaml fixture
    // Call start_execution_handler
    // Assert: run_id returned, registry has Running entry
    // Assert: .specs/{spec}/runs/{run_id}/ directory created
    // Wait for spawned task to finish (will fail since ckrv binary not available)
    // Assert: registry status is Error (expected — no ckrv binary)
    // Assert: logs.jsonl was created and has entries
}

#[tokio::test]
async fn test_full_cancellation_flow() {
    // Create AppState with temp dir, write spec + plan fixture
    // Call start_execution_handler
    // Immediately call stop_execution_handler
    // Assert: cancel token fired
    // Wait briefly for task to observe cancellation
    // Assert: registry status is Error with "cancelled" message
}

#[tokio::test]
async fn test_hub_receives_events_during_execution() {
    // Create AppState, subscribe to hub
    // Start execution (will fail quickly without ckrv binary)
    // Collect events from hub receiver
    // Assert: at least a Log event and an Error event were broadcast
}

#[tokio::test]
async fn test_logs_handler_reads_persisted_events() {
    // Create AppState with temp dir
    // Manually write a logs.jsonl fixture
    // Call get_logs_handler with offset/limit
    // Assert: pagination works correctly
    // Call tail_logs_handler
    // Assert: returns last N entries
}
```

**Fixture strategy:**
- Minimal `plan.yaml` with 1-2 batches (no real agent config needed)
- Tests don't execute real agents — they test the harness around execution
- `run_orchestration` will fail at `Command::new("ckrv")` — that's fine, we're testing the setup/teardown/event flow, not the agent

### Tier 3: Contract tests (CI, < 1 min)

Prevent frontend-backend drift.

**TypeScript codegen check:**
```yaml
# In .github/workflows/ci.yml — new job
contract-check:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - uses: Swatinem/rust-cache@v2
    - name: Regenerate TypeScript types
      run: cargo test -p ckrv-transport --features typescript export_typescript_types -- --ignored
    - name: Check for drift
      run: git diff --exit-code crates/ckrv-ui/frontend/src/types/api.generated.ts
```

If someone changes a Rust struct and doesn't regenerate, CI fails with a clear diff showing what changed.

### Tier 4: Docker smoke tests (CI, < 2 min)

Validate that the Docker integration path _exists_ without needing API keys.

```rust
// crates/ckrv-transport/tests/docker_smoke.rs

#[test]
#[ignore] // Run with: cargo test -- --ignored
fn test_docker_available() {
    let output = Command::new("docker").args(["info"]).output();
    assert!(output.is_ok() && output.unwrap().status.success(),
        "Docker must be available for smoke tests");
}

#[test]
#[ignore]
fn test_cleanup_kills_labeled_container() {
    // Start a sleep container with ckrv.spec label
    Command::new("docker").args([
        "run", "-d", "--rm",
        "--label", "ckrv.spec=test-smoke",
        "alpine", "sleep", "300"
    ]).output().unwrap();

    // Run cleanup
    cleanup_docker_containers("test-smoke");

    // Verify container is gone
    let output = Command::new("docker").args([
        "ps", "-q", "--filter", "label=ckrv.spec=test-smoke"
    ]).output().unwrap();
    assert!(String::from_utf8_lossy(&output.stdout).trim().is_empty());
}
```

**CI change:** Add a job that runs `cargo test -- --ignored` for non-API-key tests. Separate from the agent tests that need secrets.

```yaml
# In .github/workflows/ci.yml
docker-smoke:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - uses: Swatinem/rust-cache@v2
    - name: Run Docker smoke tests
      run: cargo test -p ckrv-transport --test docker_smoke -- --ignored
```

### Tier 5: Agent integration tests (nightly/manual)

The `#[ignore]` tests that need API keys. Run on a schedule or manual trigger with repository secrets.

```yaml
# .github/workflows/nightly.yml
name: Nightly Agent Tests
on:
  schedule:
    - cron: '0 6 * * *'  # 6 AM UTC daily
  workflow_dispatch:

jobs:
  agent-tests:
    runs-on: ubuntu-latest
    env:
      ANTHROPIC_API_KEY: ${{ secrets.ANTHROPIC_API_KEY }}
      OPENAI_API_KEY: ${{ secrets.OPENAI_API_KEY }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Pull agent images
        run: just pull-images
      - name: Run agent integration tests
        run: cargo test --workspace -- --ignored --test-threads=1
```

This stops the `#[ignore]` tests from being dead code. They run daily, failures notify via GitHub Actions, and you can trigger manually before releases.

## Frontend Test Strategy

### Raise the bar or drop the pretense

Current state: 26% line coverage with thresholds at 26/24/21. This never fails. Two options:

**Option A: Meaningful thresholds** — Raise to 40% lines / 35% functions as a ratchet. Every PR must maintain or improve. Exclude `components/ui/` (shadcn primitives).

**Option B: Drop thresholds, test what matters** — Remove coverage thresholds entirely. Instead, require tests for:
- Every custom hook (useExecutionStream, useWorkflowProgress, etc.)
- Every data transformation (log parsing, event mapping, state machines)
- Skip testing pure render components — E2E covers those

I lean toward **Option B**. Coverage thresholds encourage writing tests for the _easy_ stuff to hit numbers. Testing hooks and data flows catches the bugs that actually ship.

### MSW mock drift

The MSW handlers in `src/test/mocks/` should derive from the generated TypeScript types (`api.generated.ts`). If the contract check (Tier 3) keeps types in sync, MSW handlers that use those types will get type errors when the API changes. This is free contract validation on the frontend side.

## Open Questions

- [ ] Should `#[ignore]` tests be split into categories (`#[ignore = "docker"]`, `#[ignore = "api_key"]`) so CI can run Docker-only tests without API keys?
- [ ] Is a nightly schedule enough for agent tests, or should they also run on PRs that touch `ckrv-core` or `ckrv-sandbox`?
- [ ] Should we add `cargo-mutants` or property testing for the event mapping? The mapping is simple now but the translation layer is a correctness-critical boundary.
- [ ] Should E2E tests run against a mock backend (faster, more reliable) or always the real binary (slower, catches real bugs)?

## Anti-Patterns to Avoid

1. **Don't test the framework.** We don't need to test that `tokio::select!` works or that `broadcast::channel` delivers messages. Test _our_ logic.
2. **Don't mock what you own.** `HubEventHandler` is ours — test it with a real `Hub`, not a mock. Mocks hide bugs in the boundary.
3. **Don't write tests that pass when the code is deleted.** Every test should fail if you remove the line it's supposed to protect. If you can't point to what breaks without it, delete the test.
4. **Don't test internals.** Test behavior, not implementation. "Events are broadcast" not "broadcast() was called N times."

## Priority Order

If we only do 3 things:

1. **Execution pipeline unit tests** (Tier 1) — 8 tests, < 1 hour. Biggest ROI. Covers the code we just wrote with zero coverage.
2. **TypeScript contract check in CI** (Tier 3) — 10 min CI job change. Prevents a whole class of silent breakage.
3. **Docker smoke test in CI** (Tier 4) — validates the product's core integration path.

Everything else is important but not urgent.

## Files Involved

| File | Role | Changes |
|------|------|---------|
| `crates/ckrv-transport/src/handlers/execution.rs` | Execution handlers | Add ~8 unit tests in `#[cfg(test)]` |
| `crates/ckrv-transport/tests/execution_integration.rs` | Integration tests | New file — handler flow tests |
| `crates/ckrv-transport/tests/docker_smoke.rs` | Docker smoke tests | New file — container lifecycle |
| `.github/workflows/ci.yml` | CI pipeline | Add contract-check + docker-smoke jobs |
| `.github/workflows/nightly.yml` | Nightly tests | New file — agent integration tests |
| `crates/ckrv-ui/frontend/vitest.config.ts` | Frontend test config | Update or remove coverage thresholds |
