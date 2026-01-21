# Implementation Plan: OpenAI Codex CLI Agent

**Branch**: `011-openai-codex-agent` | **Date**: 2026-01-21 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/011-openai-codex-agent/spec.md`

## Summary

Add OpenAI Codex CLI as an alternative agent to Claude Code, enabling users to select their preferred AI coding assistant for batch executions. This requires abstracting the current Claude-specific implementation into a pluggable agent interface.

## Technical Context

**Language/Version**: Rust 1.75 (workspace already configured)
**Primary Dependencies**: bollard (Docker), tokio (async), serde (config)
**Storage**: YAML configuration files in `.chakravarti/` and environment variables
**Testing**: cargo test with tempfile for integration tests
**Target Platform**: Linux/macOS CLI with Docker sandbox execution
**Project Type**: Monorepo with multiple crates (ckrv-*)
**Performance Goals**: Same execution latency as current Claude implementation (±15%)
**Constraints**: Must maintain backward compatibility with existing Claude Code configurations
**Scale/Scope**: ~500 lines of new code, primarily in ckrv-sandbox and ckrv-ui crates

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Requirement | Status |
|-----------|-------------|--------|
| I. Code Quality Excellence | Full typing via Rust type system, zero clippy warnings, trait-based abstraction | ✅ PASS |
| II. Testing Standards | Unit tests for agent provider, integration tests for execution | ✅ PLANNED |
| III. Reliability First | Explicit error handling with Result types, fallback to Claude if Codex unconfigured | ✅ PASS |
| IV. Security by Default | API keys via environment variables only (OPENAI_API_KEY), no hardcoded secrets | ✅ PASS |
| V. Deterministic CLI Behavior | --agent flag for explicit selection, JSON output support, clear exit codes | ✅ PASS |

## Project Structure

### Documentation (this feature)

```text
specs/011-openai-codex-agent/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
└── contracts/           # Phase 1 output
```

### Source Code (repository root)

```text
crates/
├── ckrv-sandbox/src/
│   ├── agent/           # NEW: Agent abstraction module
│   │   ├── mod.rs       # Agent trait + factory
│   │   ├── claude.rs    # Claude Code provider
│   │   └── codex.rs     # OpenAI Codex provider
│   ├── docker.rs        # MODIFY: Use agent providers
│   ├── env.rs           # MODIFY: Support agent-specific env vars
│   └── executor.rs      # MODIFY: Accept agent configuration
├── ckrv-ui/src/
│   ├── services/
│   │   └── engine.rs    # MODIFY: Pass agent config to sandbox
│   └── api/
│       └── command.rs   # MODIFY: Accept --agent flag
└── ckrv-cli/src/
    └── commands/
        └── run.rs       # MODIFY: Add --agent CLI flag

docker/
└── Dockerfile           # MODIFY: Install Codex CLI alongside Claude
```

**Structure Decision**: Single crate extension pattern - add `agent/` module to ckrv-sandbox with provider abstraction, then propagate agent selection through execution stack.

## Complexity Tracking

> No Constitution violations. Design follows existing patterns.

| Aspect | Decision | Rationale |
|--------|----------|-----------|
| Agent abstraction | Trait-based provider | Rust idiomatic, testable, extensible |
| Config location | Environment variables | Matches existing Claude pattern |
| Default agent | Claude Code | Backward compatibility |
