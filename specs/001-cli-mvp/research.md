# Research: Chakravarti CLI MVP

**Feature**: 001-cli-mvp  
**Date**: 2025-12-12  
**Status**: Complete

## Overview

This document captures research decisions for the Chakravarti CLI implementation. Since the user provided a comprehensive tech stack specification, most decisions are pre-resolved.

---

## 1. Rust Async Runtime

**Decision**: Tokio

**Rationale**:
- Industry standard for Rust async applications
- Excellent ecosystem support (reqwest, bollard, tonic)
- Battle-tested in production systems
- Supports both multi-threaded and current-thread runtimes

**Alternatives Considered**:
- **async-std**: Viable but smaller ecosystem; fewer integrations with Docker clients
- **smol**: Minimal runtime; lacks ecosystem depth for complex applications

---

## 2. CLI Framework

**Decision**: Clap v4 with derive macros

**Rationale**:
- De facto standard for Rust CLIs
- Derive macros reduce boilerplate
- Built-in support for subcommands, flags, and help generation
- Supports both human and machine-readable output patterns

**Alternatives Considered**:
- **argh**: Google's parser; simpler but less feature-rich
- **structopt**: Deprecated in favor of clap derive

---

## 3. Git Library

**Decision**: git2 (libgit2 bindings)

**Rationale**:
- C library with mature Rust bindings
- Full git functionality including worktree support
- Widely used in Rust ecosystem (cargo, gitoxide)
- Stable API

**Alternatives Considered**:
- **gitoxide (gix)**: Pure Rust; rapidly maturing but worktree API less stable
- **Command-line git**: Shell out to `git`; less reliable, harder to test

---

## 4. Container Runtime Interface

**Decision**: Bollard (Docker API client)

**Rationale**:
- Pure Rust, async-first Docker client
- Supports both Docker and Podman (via socket)
- Full container lifecycle management
- Active maintenance

**Alternatives Considered**:
- **shiplift**: Older library, less active
- **Shell out to docker CLI**: Works but harder to handle errors, parse output

---

## 5. HTTP Client for Model APIs

**Decision**: reqwest

**Rationale**:
- Standard Rust HTTP client
- Async with tokio support
- Handles JSON, TLS, retries
- Wide adoption

**Alternatives Considered**:
- **hyper**: Lower-level; more work for same result
- **ureq**: Blocking only; doesn't fit async architecture

---

## 6. Serialization

**Decision**: Serde with serde_json and serde_yaml

**Rationale**:
- Universal Rust serialization framework
- Supports all needed formats (JSON, YAML, TOML)
- Derive macros for ergonomic usage
- Zero-copy parsing where needed

**Alternatives Considered**:
- **Manual parsing**: Impractical for complex structures
- **simd-json**: Faster but less flexible; overkill for this use case

---

## 7. Error Handling

**Decision**: thiserror for library crates, anyhow for CLI

**Rationale**:
- `thiserror`: Define domain errors with context in libraries
- `anyhow`: Easy error chaining and display in application layer
- Clear separation between recoverable/unrecoverable errors
- Rust standard practice

**Alternatives Considered**:
- **eyre**: Nice reports but heavier
- **custom Result types**: More work, same outcome

---

## 8. Logging/Tracing

**Decision**: tracing + tracing-subscriber

**Rationale**:
- Structured logging with spans
- Async-aware
- Supports multiple outputs (console, JSON, files)
- Events can be used for metrics collection

**Alternatives Considered**:
- **log + env_logger**: Simpler but no structured events or spans
- **slog**: Structured but less ecosystem integration

---

## 9. Testing Framework

**Decision**: Built-in cargo test + nextest + tarpaulin

**Rationale**:
- `cargo test`: Standard, integrates with IDE
- `nextest`: Faster parallel execution, better output
- `tarpaulin`: Coverage reporting for Rust

**Alternatives Considered**:
- **llvm-cov**: More accurate but harder to set up
- **cargo-llvm-cov**: Modern alternative to tarpaulin; viable backup

---

## 10. Model Provider Abstraction

**Decision**: Trait-based abstraction with implementations for OpenAI, Anthropic, and generic HTTP

**Rationale**:
- BYOK requirement means multiple providers
- Trait allows easy testing with mocks
- Shared token/cost accounting across providers

**Pattern**:
```rust
#[async_trait]
pub trait ModelProvider: Send + Sync {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse>;
    fn name(&self) -> &str;
    fn pricing(&self) -> &ModelPricing;
}
```

---

## 11. Spec File Format

**Decision**: YAML with optional Markdown content blocks

**Rationale**:
- YAML provides structured data (id, goal, constraints)
- Markdown allows rich descriptions where needed
- Matches DESIGN.md examples
- Easy to parse with serde_yaml

**Schema**:
```yaml
id: string          # Required, unique identifier
goal: string        # Required, human-readable goal
constraints:        # Optional, list of constraints
  - string
acceptance:         # Required, list of acceptance criteria
  - string
```

---

## 12. State Persistence

**Decision**: JSON files in `.chakravarti/` directory

**Rationale**:
- Local-first requirement
- Human-readable for debugging
- Easy to version control if needed
- No database dependency

**Structure**:
```text
.chakravarti/
├── config.json          # User configuration
└── runs/
    └── <job_id>/
        ├── job.json     # Job state
        ├── plan.json    # Generated plan
        ├── metrics.json # Cost/time metrics
        └── logs/        # Execution logs
```

---

## 13. Worktree Strategy

**Decision**: Flat structure under `.worktrees/`

**Rationale**:
- Simple path management
- Easy cleanup
- Avoid nesting complexity

**Pattern**:
```text
.worktrees/<job_id>/<attempt_id>/
```

**Cleanup**: On success, worktree is deleted after branch promotion. On failure, preserved for debugging until explicit cleanup.

---

## 14. Security Model Implementation

**Decision**: Environment variable injection + tool allow-list

**Rationale**:
- Secrets via `CKRV_*` prefixed env vars or `.env` file
- Allow-list defines permitted commands in sandbox
- Network isolation via container configuration
- No code/prompt storage by default

**Allow-list Format**:
```yaml
allowed_commands:
  - cargo
  - npm
  - make
  - git  # read-only operations only
blocked_patterns:
  - curl
  - wget
  - ssh
```

---

## 15. Output Formatting

**Decision**: Dual output mode (human + JSON)

**Rationale**:
- Constitution requires machine-readable output
- Human output for interactive use
- `--json` flag switches modes
- Errors always go to stderr

**Pattern**:
- Normal: Pretty-printed human output to stdout
- `--json`: Structured JSON to stdout, progress to stderr
- Exit codes: 0=success, 1=user error, 2=system error, 3=verification failed

---

## Resolved: No NEEDS CLARIFICATION Items

All technical decisions were provided in user input or derived from DESIGN.md. No clarification needed.
