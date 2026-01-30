# Research: AI-Native Interface Layer

**Generated**: 2026-01-29  
**Spec**: 017-ai-native-interface  
**Status**: Complete

## Existing Pattern Analysis

### Similar Feature: CLI Command Generation

**Search Commands Used**:
```bash
grep -r "CommandFactory" crates/ --include="*.rs" -l
grep -r "clap" crates/ --include="*.rs" | head -30
```

**Findings**:

1. **clap `CommandFactory` trait** is already imported and used in:
   - `crates/ckrv-cli/src/main.rs:5` - `use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};`
   - `crates/ckrv-cli/src/main.rs:160` - `Cli::command()` for help generation

2. **CLI structure uses clap derive macros consistently**:
   - All commands in `crates/ckrv-cli/src/commands/*.rs` use `#[derive(Args)]` or `#[derive(Subcommand)]`
   - Nested subcommands exist (e.g., `spec new`, `test run`, `qa review`)
   - Hidden commands use `#[command(hide = true)]` (Task, Status, Report)

**Implementation Locations**:

| Crate | File | Purpose |
|-------|------|---------|
| ckrv-cli | src/main.rs | Cli struct with all commands |
| ckrv-cli | src/commands/*.rs | Individual command implementations |

### Similar Feature: Binary in Crate

**Search Commands Used**:
```bash
grep -r "\[\[bin\]\]" crates/ --include="*.toml"
```

**Findings**:
- `ckrv-cli/Cargo.toml` has single binary `ckrv` defined as `[[bin]]` section
- Pattern: Binary exists in `src/main.rs`, new binary would be `src/bin/skill_gen.rs`

### Similar Feature: New Crate Structure

**Search Commands Used**:
```bash
ls -la crates/
cat Cargo.toml
```

**Findings**:
- All crates follow `ckrv-<name>` naming convention
- New crate should be `ckrv-mcp` following this pattern
- Workspace configuration in root `Cargo.toml` requires adding new crate to members

**Implementation Locations for New Crate**:

| Path | Purpose |
|------|---------|
| `crates/ckrv-mcp/Cargo.toml` | Crate configuration |
| `crates/ckrv-mcp/src/lib.rs` | Library exports |
| `crates/ckrv-mcp/src/main.rs` | Binary entry point |
| `Cargo.toml` | Add to workspace members |

### CLI/UI Parity Check

- CLI path: `ckrv-cli/src/commands/*.rs` handles CLI commands
- UI path: `ckrv-ui` provides web dashboard for same functionality
- **Conclusion**: SKILL.md and MCP server expose CLI commands only, no UI routes. This is correct per spec FR-023.

## Technical Decisions

### Decision 1: SKILL.md Generation Approach

**Options Considered**:
1. **Build-time generation via build.rs** - Would require re-running on every build
2. **Runtime binary (`skill_gen`)** - Explicit generation via `make skill`
3. **Library function called by main CLI** - Would clutter main binary

**Decision**: Option 2 - Separate binary `skill_gen`

**Rationale**:
- Explicit control over when documentation is regenerated
- Separates concerns - documentation is a build artifact, not runtime behavior
- Matches pattern in original design doc
- Easy CI integration via Makefile target

### Decision 2: clap → JSON Schema Conversion

**Options Considered**:
1. **Manual mapping** - Map each clap type to JSON Schema type manually
2. **Use schemars crate** - Auto-derive JSON Schema from Rust types
3. **Simple type mapping** - Map common clap types (String, PathBuf, bool, i32)

**Decision**: Option 3 - Simple type mapping with fallback to string

**Rationale**:
- clap arguments are primarily String, PathBuf, bool, optional variants
- Complex enum types can fallback to string with documented values
- Keeps implementation simple (~100 lines as estimated)
- MCP clients are forgiving about schema precision

### Decision 3: MCP Tool Execution Strategy

**Options Considered**:
1. **Shell out to `ckrv --json`** - Call CLI as subprocess
2. **Direct core integration** - Call `ckrv-core` functions directly
3. **Hybrid** - Shell out with fallback to direct for performance-critical operations

**Decision**: Option 1 - Shell out to CLI

**Rationale**:
- Zero maintenance - new CLI commands automatically available as MCP tools
- CLI already handles all error formatting, validation, and JSON output
- Latency is acceptable for AI agent use cases (not real-time)
- Matches original design doc recommendation

### Decision 4: MCP Transport

**Options Considered**:
1. **stdio JSON-RPC** - Standard MCP transport
2. **HTTP** - REST-like API
3. **WebSocket** - Persistent connection

**Decision**: Option 1 - stdio JSON-RPC

**Rationale**:
- Required by MCP specification for Claude Desktop compatibility
- Simplest implementation
- Already proven pattern for MCP servers

### Decision 5: SKILL.md Output Location

**Options Considered**:
1. `.agent/skills/chakravarti-cli/SKILL.md` - Standard agent skills location
2. `docs/SKILL.md` - With documentation
3. `crates/ckrv-cli/SKILL.md` - With CLI crate

**Decision**: Option 1 - `.agent/skills/chakravarti-cli/SKILL.md`

**Rationale**:
- Follows agent skills specification
- Existing `.agent/skills/` directory structure in project
- Discoverable by AI agents automatically

## Dependencies

### Existing (in workspace)

| Dependency | Usage |
|------------|-------|
| `clap` | Command introspection via `CommandFactory` |
| `serde` | JSON serialization |
| `serde_json` | JSON output |
| `tokio` | Async runtime for MCP server |

### New Dependencies Needed

| Dependency | Version | Usage |
|------------|---------|-------|
| None | - | All required deps already in workspace |

**Note**: MCP stdio transport can be implemented with standard Rust (stdin/stdout + serde_json). No additional dependencies required.

## Risks & Mitigations

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| clap schema conversion incomplete | Medium | Medium | Start with common types, iterate based on actual command usage |
| MCP protocol changes | Low | High | Pin to stable MCP version, document version compatibility |
| Shell execution too slow | Low | Low | Profile after implementation, optimize if >5s for simple commands |
| Hidden commands leak | Low | Low | Unit test to verify `is_hide_set()` filtering |
| Subcommand naming conflicts | Low | Low | Use underscore-separated names consistently |

## Implementation Notes

### clap Command Introspection API

```rust
use clap::{CommandFactory, Arg, ArgAction};
use ckrv_cli::Cli;

let cmd = Cli::command();

// Get subcommands
for subcmd in cmd.get_subcommands() {
    println!("Command: {}", subcmd.get_name());
    println!("About: {:?}", subcmd.get_about());
    println!("Hidden: {}", subcmd.is_hide_set());
    
    // Positional arguments
    for arg in subcmd.get_positionals() {
        println!("  Arg: {}", arg.get_id());
        println!("  Help: {:?}", arg.get_help());
        println!("  Required: {}", arg.is_required_set());
    }
    
    // Options (--flag, --option=value)
    for arg in subcmd.get_opts() {
        println!("  Option: --{}", arg.get_id());
        println!("  Help: {:?}", arg.get_help());
    }
}
```

### MCP JSON-RPC Message Format

```json
// Initialize request
{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05"}}

// Tools list request
{"jsonrpc": "2.0", "id": 2, "method": "tools/list"}

// Tool call request
{"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {"name": "ckrv_spec_list", "arguments": {}}}
```

### Agent Skills Frontmatter Format

```yaml
---
name: chakravarti-cli
description: Spec-driven agent orchestration. Use for creating specs, planning tasks, running jobs, and reviewing changes.
license: MIT
compatibility: Claude Code, Cursor, any CLI agent
metadata:
  version: "0.1.0"
  auto-generated: true
  generated-at: "2026-01-29T12:00:00Z"
---
```

## Open Questions (Resolved)

1. **Should MCP expose all commands or a curated subset?**
   - **Resolution**: All non-hidden commands (matches SKILL.md)

2. **Should MCP support streaming for long operations?**
   - **Resolution**: V1 without streaming, add later if needed

3. **Where should generated SKILL.md live?**
   - **Resolution**: `.agent/skills/chakravarti-cli/SKILL.md`
