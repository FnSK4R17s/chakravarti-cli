# Implementation Plan: Comprehensive Code Documentation

**Branch**: `012-code-documentation` | **Date**: 2026-01-21 | **Spec**: [spec.md](file:///apps/chakravarti-cli/specs/012-code-documentation/spec.md)  
**Input**: Feature specification from `/specs/012-code-documentation/spec.md`

## Summary

Create comprehensive documentation for all 10 crates in the Chakravarti CLI workspace. Documentation includes:
- Top-level cross-crate docs in `crates/docs/`
- Per-crate documentation in `crates/<crate>/docs/`
- Git commit hash frontmatter for staleness detection
- Enhanced rustdoc comments in source code

## Technical Context

**Language/Version**: Rust 1.75+  
**Primary Dependencies**: Markdown, Mermaid diagrams, cargo doc  
**Storage**: N/A (documentation files only)  
**Testing**: `cargo doc --deny warnings`, `cargo test --doc`  
**Target Platform**: Documentation for developers and AI agents  
**Project Type**: Rust workspace with 10 crates  
**Performance Goals**: N/A  
**Constraints**: Docs must be AI-agent-friendly with git hash tracking  
**Scale/Scope**: 10 crates, ~100 source files, ~20 CLI commands

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Requirement | Status |
|-----------|-------------|--------|
| I. Code Quality Excellence | Documentation promotes self-documenting code | ✅ |
| II. Testing Standards | Doc tests via `cargo test --doc` | ✅ |
| III. Reliability First | Staleness detection via git hash | ✅ |
| IV. Security by Default | No secrets in docs | ✅ |
| V. Deterministic CLI Behavior | CLI command reference with exit codes | ✅ |

## Proposed Changes

### Top-Level Documentation (`crates/docs/`)

---

#### [NEW] [architecture.md](file:///apps/chakravarti-cli/crates/docs/architecture.md)

System architecture overview with:
- Crate dependency diagram (Mermaid)
- Data flow: spec → plan → job → execution
- Key abstractions: Orchestrator, Sandbox, Agent

---

#### [NEW] [getting-started.md](file:///apps/chakravarti-cli/crates/docs/getting-started.md)

New contributor onboarding:
- Prerequisites
- Building from source
- Running tests
- Making first contribution

---

#### [NEW] [cli-commands.md](file:///apps/chakravarti-cli/crates/docs/cli-commands.md)

Complete CLI reference:
- All 21 commands with options
- Exit codes
- Examples

---

#### [NEW] [agent-guide.md](file:///apps/chakravarti-cli/crates/docs/agent-guide.md)

Agent extensibility guide:
- Agent trait interface
- Adding new providers (Claude, Codex, OpenRouter)
- Configuration format

---

### Per-Crate Documentation

---

#### [NEW] [README.md](file:///apps/chakravarti-cli/crates/ckrv-cli/docs/README.md)

CLI crate overview: entry point, command structure, argument handling

---

#### [NEW] [README.md](file:///apps/chakravarti-cli/crates/ckrv-core/docs/README.md)

Core crate: Spec, Plan, Job, Orchestrator, Workflow concepts

---

#### [NEW] [README.md](file:///apps/chakravarti-cli/crates/ckrv-git/docs/README.md)

Git operations: GitRepo, Worktree, branch management

---

#### [NEW] [README.md](file:///apps/chakravarti-cli/crates/ckrv-sandbox/docs/README.md)

Sandbox execution: Docker, AllowList, Agent providers

---

#### [NEW] [README.md](file:///apps/chakravarti-cli/crates/ckrv-spec/docs/README.md)

Spec loading and validation

---

#### [NEW] [README.md](file:///apps/chakravarti-cli/crates/ckrv-metrics/docs/README.md)

Telemetry and metrics collection

---

#### [NEW] [README.md](file:///apps/chakravarti-cli/crates/ckrv-model/docs/README.md)

Model routing: providers, pricing, budget tracking

---

#### [NEW] [README.md](file:///apps/chakravarti-cli/crates/ckrv-integrations/docs/README.md)

External integrations (GitHub, etc.)

---

#### [NEW] [README.md](file:///apps/chakravarti-cli/crates/ckrv-verify/docs/README.md)

Verification: linting, testing, type checking

---

#### [NEW] [README.md](file:///apps/chakravarti-cli/crates/ckrv-ui/docs/README.md)

Web UI: API endpoints, WebSocket events, frontend

---

#### [NEW] [api-reference.md](file:///apps/chakravarti-cli/crates/ckrv-ui/docs/api-reference.md)

Complete API documentation for all 16 endpoint modules

---

### Rustdoc Enhancements

#### [MODIFY] All `lib.rs` files

Add comprehensive crate-level `//!` documentation with examples

#### [MODIFY] Public modules

Add module-level `//!` documentation

#### [MODIFY] Public functions/structs

Add `///` documentation with parameters, returns, examples

---

## Verification Plan

### Automated Tests

```bash
# Verify rustdoc compiles without warnings
cargo doc --deny warnings --no-deps

# Run doc tests
cargo test --doc

# Verify frontmatter in all doc files
find crates/docs crates/*/docs -name "*.md" -exec grep -L "^---" {} \;
# Should output nothing (all files have frontmatter)
```

### Manual Verification

1. **Check folder structure exists**:
   ```bash
   ls -la crates/docs/
   ls -la crates/*/docs/
   ```

2. **View generated rustdoc**:
   ```bash
   cargo doc --open --no-deps
   ```
   Verify navigation works for all crates.

3. **Validate Mermaid diagrams**: Open `architecture.md` in GitHub or VS Code preview to confirm diagrams render.

## Complexity Tracking

No constitution violations - this feature aligns with all principles.

## Phase 1 Artifacts

- [research.md](file:///apps/chakravarti-cli/specs/012-code-documentation/research.md) - Current state analysis
- [data-model.md](file:///apps/chakravarti-cli/specs/012-code-documentation/data-model.md) - Documentation entities
- [quickstart.md](file:///apps/chakravarti-cli/specs/012-code-documentation/quickstart.md) - Developer guide
- [contracts/frontmatter-schema.md](file:///apps/chakravarti-cli/specs/012-code-documentation/contracts/frontmatter-schema.md) - Frontmatter API
