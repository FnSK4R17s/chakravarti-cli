# Implementation Plan: Chakravarti CLI MVP

**Branch**: `001-cli-mvp` | **Date**: 2025-12-12 | **Spec**: [spec.md](file:///apps/chakravarti-cli/specs/001-cli-mvp/spec.md)  
**Input**: Feature specification from `/specs/001-cli-mvp/spec.md`

## Summary

Build Chakravarti CLI: a local-first, spec-driven agent orchestration engine that converts specifications into verified, auditable code diffs. The system uses planner/executor separation with isolated git worktrees, containerized execution, and comprehensive cost/time metrics. Technical approach: Rust Cargo workspace with 9 specialized crates following library-first architecture.

## Technical Context

**Language/Version**: Rust (stable, 1.75+)  
**Primary Dependencies**: clap (CLI), tokio (async runtime), serde (serialization), bollard (Docker API), git2 (libgit2 bindings), reqwest (HTTP client)  
**Storage**: Local filesystem only (`.chakravarti/runs/<job_id>/`), JSON for metrics and state  
**Testing**: `cargo test` (unit + integration), `cargo nextest` (parallel execution), `cargo tarpaulin` (coverage)  
**Target Platform**: Linux (primary), macOS, Windows (via WSL2)  
**Project Type**: Cargo workspace with multiple crates  
**Performance Goals**: CLI startup <100ms, plan generation <5s, execution step latency tracked per-step  
**Constraints**: Local-first, no data persistence to cloud, BYOK model keys only, deterministic outputs  
**Scale/Scope**: Single repository operations, MVP targets SWE-bench tasks  

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Requirement | Status |
|-----------|-------------|--------|
| I. Code Quality Excellence | Rust's type system enforces safety; clippy for linting; rustfmt for formatting | ✅ |
| II. Testing Standards | TDD with cargo test; 80% coverage target; contract tests for CLI commands | ✅ |
| III. Reliability First | Result/Option types for error handling; worktree cleanup on failure; state machine for job lifecycle | ✅ |
| IV. Security by Default | Secrets via env vars only; sandboxed execution; tool allow-list; no network unless model API | ✅ |
| V. Deterministic CLI Behavior | JSON + human output; meaningful exit codes; all output reproducible given same inputs | ✅ |

## Project Structure

### Documentation (this feature)

```text
specs/001-cli-mvp/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (internal crate APIs)
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
Cargo.toml               # Workspace manifest
crates/
├── ckrv-cli/            # CLI entrypoint
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs      # Entry point, clap setup
│       └── commands/    # Subcommand handlers
│           ├── mod.rs
│           ├── init.rs
│           ├── spec.rs
│           ├── run.rs
│           ├── status.rs
│           ├── diff.rs
│           ├── report.rs
│           └── promote.rs
│
├── ckrv-core/           # Domain primitives + state machine
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── spec.rs      # Spec type
│       ├── plan.rs      # Plan DAG
│       ├── job.rs       # Job + Attempt types
│       ├── step.rs      # Step execution
│       ├── state.rs     # RunState machine
│       ├── orchestrator.rs  # Lifecycle coordination
│       └── events.rs    # Structured log events
│
├── ckrv-spec/           # Spec-Kit integration + parsing
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── loader.rs    # Load spec files
│       ├── validator.rs # Schema validation
│       └── mapper.rs    # Map to internal types
│
├── ckrv-git/            # Git + worktree engine
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── worktree.rs  # Create/cleanup worktrees
│       ├── branch.rs    # Branch management
│       ├── diff.rs      # Diff generation
│       └── commit.rs    # Commit helpers
│
├── ckrv-sandbox/        # Containerized execution
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── docker.rs    # Docker/Podman wrapper
│       ├── executor.rs  # Run commands in sandbox
│       ├── allowlist.rs # Tool allow-list
│       └── env.rs       # Environment injection
│
├── ckrv-model/          # Model gateway (BYOK)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── provider.rs  # Provider trait
│       ├── openai.rs    # OpenAI implementation
│       ├── anthropic.rs # Anthropic implementation
│       ├── router.rs    # Model selection logic
│       └── accounting.rs # Token/cost tracking
│
├── ckrv-verify/         # Verification pipeline
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── runner.rs    # Test/lint runner
│       ├── parser.rs    # Parse test results
│       ├── acceptance.rs # Spec criteria checks
│       └── verdict.rs   # Structured verdict
│
├── ckrv-metrics/        # Cost/time aggregator
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── collector.rs # Collect metrics events
│       ├── cost.rs      # Cost calculation
│       ├── time.rs      # Timing utilities
│       └── report.rs    # Generate JSON reports
│
└── ckrv-integrations/   # Optional GitLab/GitHub (feature flag)
    ├── Cargo.toml
    └── src/
        ├── lib.rs
        └── gitlab.rs    # MR creation (optional)

tests/                   # Workspace-level integration tests
├── cli_integration.rs   # End-to-end CLI tests
└── fixtures/            # Test repos/specs

docs/
├── decisions/           # Architecture Decision Records
└── demo/                # Demo repo + CI examples
```

**Structure Decision**: Cargo workspace with 9 crates following library-first architecture. `ckrv-cli` is a thin shell over library crates. Dependencies flow inward: CLI → Core → specialized crates. No circular dependencies enforced via Cargo's build system.

## Proposed Changes

### ckrv-cli (CLI Entrypoint)

Thin CLI wrapper using `clap` for argument parsing. Each subcommand delegates to library crates. Handles output formatting (JSON/human), exit codes, and top-level error display.

#### [NEW] [Cargo.toml](file:///apps/chakravarti-cli/crates/ckrv-cli/Cargo.toml)
- Workspace member with dependencies on all internal crates
- Binary target `ckrv`

#### [NEW] [main.rs](file:///apps/chakravarti-cli/crates/ckrv-cli/src/main.rs)
- Clap derive-based CLI definition
- Subcommands: init, spec, run, status, diff, report, promote
- Global flags: `--json`, `--quiet`, `--verbose`

---

### ckrv-core (Domain Primitives + Orchestration)

Core domain types and the orchestration state machine. No I/O—pure business logic.

#### [NEW] [Cargo.toml](file:///apps/chakravarti-cli/crates/ckrv-core/Cargo.toml)
- Minimal dependencies: serde, thiserror, tracing

#### [NEW] [spec.rs](file:///apps/chakravarti-cli/crates/ckrv-core/src/spec.rs)
- `Spec` struct: id, goal, constraints, acceptance criteria
- Validation helpers

#### [NEW] [plan.rs](file:///apps/chakravarti-cli/crates/ckrv-core/src/plan.rs)
- `Plan` struct: DAG of steps with dependencies
- Step ordering and parallelization hints

#### [NEW] [job.rs](file:///apps/chakravarti-cli/crates/ckrv-core/src/job.rs)
- `Job` struct: id, spec_id, attempts, status, created_at
- `Attempt` struct: id, worktree_path, steps, result

#### [NEW] [state.rs](file:///apps/chakravarti-cli/crates/ckrv-core/src/state.rs)
- `RunState` enum: Pending, Planning, Executing, Verifying, Succeeded, Failed
- State transition logic with validation

#### [NEW] [orchestrator.rs](file:///apps/chakravarti-cli/crates/ckrv-core/src/orchestrator.rs)
- `Orchestrator` trait: plan(), execute(), verify(), handle_retry()
- Coordinates all other crates

---

### ckrv-spec (Spec-Kit Integration)

Loads spec files, validates schema, maps to internal `Spec` type.

#### [NEW] [loader.rs](file:///apps/chakravarti-cli/crates/ckrv-spec/src/loader.rs)
- Load YAML/Markdown specs from `.specs/` directory
- Parse frontmatter and content

#### [NEW] [validator.rs](file:///apps/chakravarti-cli/crates/ckrv-spec/src/validator.rs)
- Schema validation for required fields
- Error collection with actionable messages

---

### ckrv-git (Git + Worktree Engine)

Pure git operations library using `git2`.

#### [NEW] [worktree.rs](file:///apps/chakravarti-cli/crates/ckrv-git/src/worktree.rs)
- `create_worktree(job_id, attempt_id)` → path
- `cleanup_worktree(path)` → Result
- Worktree listing and status

#### [NEW] [diff.rs](file:///apps/chakravarti-cli/crates/ckrv-git/src/diff.rs)
- Generate diff between worktree and base commit
- Format as unified diff or structured output

---

### ckrv-sandbox (Containerized Execution)

Docker/Podman wrapper for isolated execution.

#### [NEW] [docker.rs](file:///apps/chakravarti-cli/crates/ckrv-sandbox/src/docker.rs)
- Bollard-based Docker API client
- Container lifecycle: create, start, wait, logs, remove

#### [NEW] [executor.rs](file:///apps/chakravarti-cli/crates/ckrv-sandbox/src/executor.rs)
- Execute commands in container with mounted worktree
- Capture stdout/stderr, exit code

#### [NEW] [allowlist.rs](file:///apps/chakravarti-cli/crates/ckrv-sandbox/src/allowlist.rs)
- Configurable tool allow-list
- Block disallowed commands before execution

---

### ckrv-model (Model Gateway)

BYOK model provider abstraction with token/cost accounting.

#### [NEW] [provider.rs](file:///apps/chakravarti-cli/crates/ckrv-model/src/provider.rs)
- `ModelProvider` trait: `complete(prompt, options) → Response`
- Typed request/response structs

#### [NEW] [router.rs](file:///apps/chakravarti-cli/crates/ckrv-model/src/router.rs)
- Model selection based on step type, retry count, budget
- Optimize modes: cost, time, balanced

#### [NEW] [accounting.rs](file:///apps/chakravarti-cli/crates/ckrv-model/src/accounting.rs)
- Track token usage per request
- Estimate cost based on model pricing

---

### ckrv-verify (Verification Pipeline)

Run tests, parse results, check acceptance criteria.

#### [NEW] [runner.rs](file:///apps/chakravarti-cli/crates/ckrv-verify/src/runner.rs)
- Execute test commands in sandbox
- Collect test output

#### [NEW] [verdict.rs](file:///apps/chakravarti-cli/crates/ckrv-verify/src/verdict.rs)
- `Verdict` struct: passed, failed_tests, logs, artifacts
- Machine-readable format

---

### ckrv-metrics (Cost/Time Aggregator)

Collect and report metrics.

#### [NEW] [collector.rs](file:///apps/chakravarti-cli/crates/ckrv-metrics/src/collector.rs)
- Event-based metrics collection
- Per-step timing, token counts

#### [NEW] [report.rs](file:///apps/chakravarti-cli/crates/ckrv-metrics/src/report.rs)
- Generate `metrics.json` at `.chakravarti/runs/<job_id>/`
- Include wall time, step latencies, token usage, cost estimates

---

### Workspace Configuration

#### [NEW] [Cargo.toml](file:///apps/chakravarti-cli/Cargo.toml)
- Workspace manifest with all crates
- Shared dependency versions
- Workspace-level lints and settings

---

## Verification Plan

### Automated Tests

1. **Unit tests**: Each crate has `#[cfg(test)]` modules
   - Run: `cargo test --workspace`
   - Coverage: `cargo tarpaulin --workspace --out Html`

2. **Integration tests**: `tests/cli_integration.rs`
   - Test full CLI workflows with test fixtures
   - Run: `cargo test --test cli_integration`

3. **Contract tests**: CLI output format verification
   - Verify JSON schema compliance
   - Verify exit codes match spec

4. **Lint/Format gates**:
   - `cargo clippy --workspace -- -D warnings`
   - `cargo fmt --check`

### Manual Verification

1. **Demo workflow**: Run end-to-end on demo repo
   - `ckrv init` → `ckrv spec new test` → `ckrv run` → `ckrv diff` → `ckrv promote`

2. **GitLab CI**: Verify CI pipeline example works
   - Push branch, verify MR artifacts

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| 9 crates in workspace | Clear separation of concerns; each crate is independently testable and reusable | Monolithic crate would violate single responsibility; harder to maintain boundaries |
| Container runtime dependency | Deterministic execution requires isolation; tests must run identically everywhere | Process-based execution is platform-dependent and leaks side effects |
