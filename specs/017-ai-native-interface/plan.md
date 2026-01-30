# Implementation Plan: AI-Native Interface Layer

**Branch**: `017-ai-native-interface` | **Date**: 2026-01-29 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/017-ai-native-interface/spec.md`

## Summary

Implement auto-generated AI agent interfaces for Chakravarti CLI with zero-maintenance guarantee. The feature consists of two complementary components:

1. **SKILL.md Generator** (`skill_gen` binary) - Generates Agent Skills documentation from clap command definitions
2. **MCP Server** (`ckrv-mcp` crate) - Exposes CLI commands as MCP tools over stdio transport

Both components derive their command metadata from clap introspection, ensuring any new CLI command is automatically documented and exposed.

## Technical Context

**Language/Version**: Rust 1.75  
**Primary Dependencies**: clap 4.4 (already in workspace), serde, serde_json, tokio  
**Storage**: File system (SKILL.md), no database required  
**Testing**: Vitest/cargo test  
**Target Platform**: Linux server (primary), macOS (development)  
**Project Type**: Monorepo with multiple crates  
**Performance Goals**: SKILL.md generation <2s, MCP initialize <100ms, tool execution <5s  
**Constraints**: Zero manual documentation maintenance  
**Scale/Scope**: 15 CLI commands, 40+ subcommands

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-checked after Phase 1 design.*

| Principle | Requirement | Status |
|-----------|-------------|--------|
| I. Code Quality Excellence | Full typing, zero lint errors, single responsibility | ✅ All Rust types defined, clippy-clean |
| II. Testing Standards | TDD approach planned, coverage targets defined | ✅ Unit tests for generation, contract tests for MCP |
| III. Reliability First | Error handling strategy, idempotency considered | ✅ Deterministic generation, proper JSON-RPC errors |
| IV. Security by Default | No hardcoded secrets, input validation planned | ✅ No secrets, MCP validates JSON schema |
| V. Deterministic CLI Behavior | Machine-readable output, explicit exit codes | ✅ JSON output for both, proper error handling |

## Project Structure

### Documentation (this feature)

```text
specs/017-ai-native-interface/
├── plan.md              # This file
├── research.md          # Phase 0 output ✅
├── data-model.md        # Phase 1 output ✅
├── quickstart.md        # Phase 1 output ✅
├── contracts/           # Phase 1 output ✅
│   ├── mcp-server.md    # MCP JSON-RPC contract
│   └── skill-gen.md     # SKILL.md generation contract
└── tasks.md             # Phase 2 output (via /speckit.tasks)
```

### Source Code (repository root)

```text
crates/
├── ckrv-cli/
│   ├── src/
│   │   ├── lib.rs            # [MODIFY] Export Cli struct
│   │   ├── main.rs           # [EXISTING] CLI entry
│   │   └── bin/
│   │       └── skill_gen.rs  # [CREATE] SKILL.md generator binary
│   └── Cargo.toml            # [MODIFY] Add [[bin]] entry
│
└── ckrv-mcp/                 # [CREATE] New crate
    ├── Cargo.toml
    └── src/
        ├── main.rs           # stdio transport entry
        ├── lib.rs            # Server struct, exports
        ├── transport.rs      # JSON-RPC stdio handler
        ├── tools.rs          # Tool discovery + execution
        └── schema.rs         # clap → JSON Schema

.agent/skills/
└── chakravarti-cli/
    └── SKILL.md              # [CREATE] Generated output

Makefile                      # [MODIFY] Add skill, mcp, install-mcp targets
Cargo.toml                    # [MODIFY] Add ckrv-mcp to workspace members
```

**Structure Decision**: Single crate (`ckrv-mcp`) with internal modules for transport, tools, and schema. Binary generator (`skill_gen`) lives in existing `ckrv-cli` crate to access the `Cli` struct directly.

## Affected Files Analysis

Based on Pattern Analysis from research.md:

| File | Change | Justification |
|------|--------|---------------|
| `crates/ckrv-cli/src/lib.rs` | [MODIFY] | Export `Cli` struct for skill_gen and ckrv-mcp |
| `crates/ckrv-cli/src/bin/skill_gen.rs` | [CREATE] | SKILL.md generation binary |
| `crates/ckrv-cli/Cargo.toml` | [MODIFY] | Add `[[bin]]` entry for skill_gen |
| `crates/ckrv-mcp/` | [CREATE] | New MCP server crate |
| `Cargo.toml` | [MODIFY] | Add ckrv-mcp to workspace members |
| `Makefile` | [MODIFY] | Add skill, mcp, install-mcp targets |
| `.agent/skills/chakravarti-cli/SKILL.md` | [CREATE] | Generated skill documentation |

## Complexity Tracking

> No constitution violations requiring justification.

| Aspect | Complexity | Notes |
|--------|------------|-------|
| clap introspection | Low | Stable API, well-documented |
| JSON Schema generation | Medium | Need to map clap types to JSON Schema |
| MCP stdio transport | Low | Standard pattern |
| Shell execution | Low | Simple subprocess call |
| Testing | Medium | Need contract tests for MCP protocol |

## Phase Summary

| Phase | Title | Deliverables |
|-------|-------|--------------|
| 0 | Research | research.md ✅ |
| 1.1 | Design Artifacts | data-model.md, contracts/, quickstart.md ✅ |
| 1.2 | Agent Context | CLAUDE.md updated (via script) |
| 2 | Tasks | tasks.md (via /speckit.tasks) |

## Generated Artifacts

- ✅ `research.md` - Pattern analysis, technical decisions, dependencies, risks
- ✅ `data-model.md` - Entity definitions, type mappings, validation rules
- ✅ `contracts/mcp-server.md` - MCP JSON-RPC API contract
- ✅ `contracts/skill-gen.md` - SKILL.md generation contract
- ✅ `quickstart.md` - Implementation quickstart guide

## Next Steps

Run `/speckit.tasks` to generate actionable implementation tasks from this plan.
