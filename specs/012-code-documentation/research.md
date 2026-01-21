---
last_commit: c1bb442
last_updated: 2026-01-21
---

# Research: Comprehensive Code Documentation

## Executive Summary

This research documents the current state of documentation across the Chakravarti CLI codebase and identifies patterns for the documentation implementation.

## Current State Analysis

### Existing Documentation

| Location | Status | Notes |
|----------|--------|-------|
| `/README.md` | ✅ Good | Comprehensive overview, quick start, command reference |
| `/DESIGN.md` | ✅ Good | Architecture philosophy and principles |
| `/CONTRIBUTING.md` | ✅ Good | Contribution guidelines |
| `/AGENTS.md` | ✅ Good | AI agent guidance |
| `/docs/` | ⚠️ Minimal | Only `coverage.md`, `optimization.md`, `decisions/` |
| Crate `lib.rs` | ⚠️ Partial | All crates have crate-level `//!` comments but minimal depth |
| Per-crate README | ❌ Missing | No crate has a dedicated README.md |
| Per-crate docs/ | ❌ Missing | No crate has a docs/ subfolder |

### Crate Inventory

| Crate | Purpose | Key Public Items | Module Count |
|-------|---------|------------------|--------------|
| `ckrv-cli` | CLI interface | Commands, main entry | 21 commands |
| `ckrv-core` | Orchestration logic | Spec, Plan, Job, Orchestrator | 16 modules |
| `ckrv-git` | Git/worktree management | GitRepo, Worktree | 6 modules |
| `ckrv-sandbox` | Docker execution | Sandbox, Agent, AllowList | 6 modules |
| `ckrv-spec` | Spec loading | SpecLoader, Validation | 6 modules |
| `ckrv-metrics` | Telemetry | Metrics, Events | 7 modules |
| `ckrv-model` | Model routing | ModelRouter, Providers | 7 modules |
| `ckrv-integrations` | External integrations | GitHub, etc. | 3 modules |
| `ckrv-verify` | Code verification | Linting, Tests, Typecheck | 7 modules |
| `ckrv-ui` | Web UI server | API, WebSocket, Frontend | 16 API files |

### API Endpoints (ckrv-ui)

Current API modules requiring documentation:
- `agents.rs` - Agent management endpoints
- `execution.rs` - Execution control endpoints  
- `specs.rs` - Spec CRUD endpoints
- `plans.rs` - Plan management endpoints
- `tasks.rs` - Task operations
- `history.rs` - Run history endpoints
- `terminal.rs` - Terminal WebSocket
- `session.rs` - Session management
- `status.rs` - Status endpoints
- `console.rs` - Console output
- `cloud.rs` - Cloud integration
- `commands.rs` - Command execution
- `diff.rs` - Diff viewing
- `docker.rs` - Docker status
- `events.rs` - Event streaming

## Decisions

### D1: Documentation Format

**Decision**: Use YAML frontmatter with Markdown body

**Rationale**: 
- Enables machine parsing for staleness detection
- Compatible with static site generators
- Familiar format for developers

**Alternatives Considered**:
- AsciiDoc: More feature-rich but less tooling support
- reStructuredText: Good for Python, less common in Rust

### D2: Folder Structure

**Decision**: `crates/docs/` for cross-crate docs, `crates/<crate>/docs/` for per-crate

**Rationale**:
- Keeps docs close to code they document
- Clear hierarchy: shared → specific
- Easy navigation for AI agents

**Alternatives Considered**:
- Single `/docs/` folder: Harder to maintain per-crate docs
- Inline only: No narrative documentation possible

### D3: Git Hash Frontmatter

**Decision**: Include `last_commit`, `last_updated`, and optional `related_files`

**Rationale**:
- `last_commit` enables automated staleness detection
- `related_files` helps identify which code changes should trigger doc updates
- `last_updated` provides human-readable timestamp

**Alternatives Considered**:
- Full commit hash: Too verbose, 7-char short hash sufficient
- No tracking: Docs become stale without detection

### D4: Rustdoc vs External Docs

**Decision**: Both - rustdoc for API reference, markdown for guides/concepts

**Rationale**:
- Rustdoc auto-generates from code, ensures accuracy
- Markdown guides explain "why" and "how to extend"
- Separate concerns: API vs conceptual

## Best Practices Research

### Rust Documentation Patterns

1. **Crate-level docs** (`//!`): Purpose, quick example, feature flags
2. **Module-level docs** (`//!`): What this module provides
3. **Function docs** (`///`): Purpose, params, returns, examples with `# Examples`
4. **Struct/Enum docs**: Field-by-field documentation

### Documentation Testing

```bash
# Verify all docs compile
cargo doc --deny warnings

# Run doc tests
cargo test --doc
```

### Mermaid Diagrams

Use fenced code blocks with `mermaid` language identifier:
- Crate dependency graph
- Execution flow diagram
- Agent integration sequence

## Open Questions (Resolved)

All NEEDS CLARIFICATION items from Technical Context have been resolved through analysis:

1. **Documentation tooling**: `cargo doc` + markdown files
2. **Diagram format**: Mermaid (GitHub compatible)
3. **API documentation format**: Markdown with request/response examples
