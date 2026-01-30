---
last_commit: 039d181
last_updated: 2026-01-30
related_files:
  - src/lib.rs
  - src/main.rs
  - src/types.rs
  - src/tools.rs
  - src/schema.rs
  - src/transport.rs
---

# ckrv-mcp

MCP (Model Context Protocol) server for Chakravarti CLI. Exposes CLI commands as tools for AI agents.

## Overview

This crate provides an MCP server that automatically exposes all non-hidden Chakravarti CLI commands as tools. It implements the MCP protocol version 2024-11-05 over stdio transport, enabling seamless integration with Claude Desktop and other MCP-compatible clients.

**Key Design Principle**: Zero maintenance. When new CLI commands are added to `ckrv`, they automatically become available as MCP tools without any code changes to this crate.

## Key Types

| Type | Module | Purpose |
|------|--------|---------|
| `MCPRequest` | types.rs | JSON-RPC 2.0 request from client |
| `MCPResponse` | types.rs | JSON-RPC 2.0 response to client |
| `MCPError` | types.rs | Standard JSON-RPC error codes |
| `MCPTool` | types.rs | Tool definition with schema |
| `MCPToolAnnotations` | types.rs | Client hints (read-only, destructive) |

## Architecture

```
Client (Claude Desktop/MCP Inspector)
        ↓ stdin (JSON-RPC 2.0)
    transport.rs
        ↓ handle_request()
    ┌───────────────────┐
    │  initialize       │ → server info & capabilities
    │  tools/list       │ → discover_tools() → ckrv-cli introspection
    │  tools/call       │ → execute_tool() → shells out to `ckrv --json`
    └───────────────────┘
        ↓ stdout (JSON-RPC 2.0)
Client
```

## Module Structure

```
src/
├── main.rs        # Binary entry point
├── lib.rs         # Public exports
├── types.rs       # JSON-RPC 2.0 types (173 lines)
├── tools.rs       # Tool discovery and execution (289 lines)
├── schema.rs      # JSON Schema generation (160 lines)
└── transport.rs   # stdio transport handler (191 lines)
```

## Public API

All exports from `lib.rs`:

```rust
// Transport
pub use transport::run_stdio_transport;

// Tool discovery
pub use tools::discover_tools;

// Types
pub use types::{MCPError, MCPRequest, MCPResponse, MCPTool, MCPToolAnnotations};
```

## Usage

### As a Binary

```bash
# Build
cargo build -p ckrv-mcp --release

# Run (reads from stdin, writes to stdout)
./target/release/ckrv-mcp
```

### Testing with JSON-RPC

```bash
# Initialize
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | ckrv-mcp

# List tools
echo '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' | ckrv-mcp

# Call a tool
echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"ckrv_spec_list","arguments":{}}}' | ckrv-mcp
```

### Using the Library

```rust
use ckrv_mcp::{discover_tools, run_stdio_transport, MCPTool};

// Get all available tools
let tools: Vec<MCPTool> = discover_tools();
for tool in &tools {
    println!("{}: {}", tool.name, tool.description);
}

// Run the server (blocking)
run_stdio_transport();
```

## MCP Protocol

### Supported Methods

| Method | Description |
|--------|-------------|
| `initialize` | Initialize session, returns server info |
| `initialized` | Notification (no response) |
| `tools/list` | Returns all available tools |
| `tools/call` | Execute a tool with arguments |
| `ping` | Health check |

### Error Codes

| Code | Constant | Meaning |
|------|----------|---------|
| -32700 | `parse_error()` | Invalid JSON |
| -32600 | `invalid_request()` | Invalid JSON-RPC structure |
| -32601 | `method_not_found()` | Unknown method |
| -32602 | `invalid_params()` | Invalid method parameters |
| -32603 | `internal_error()` | Internal server error |
| -32001 | `tool_execution_failed()` | Tool execution failed |

## Tool Discovery

Tools are discovered via clap introspection:

```rust
// From ckrv-cli
pub fn extract_command_metadata() -> CommandMetadata

// Converted to MCP tools
fn discover_tools() -> Vec<MCPTool>
```

### Naming Convention

CLI commands are converted to tool names:

| CLI Command | Tool Name |
|-------------|-----------|
| `ckrv init` | `ckrv_init` |
| `ckrv spec new` | `ckrv_spec_new` |
| `ckrv cloud login` | `ckrv_cloud_login` |

### Tool Annotations

Annotations hint at tool behavior:

| Annotation | Applied To |
|------------|------------|
| `readOnlyHint: true` | `*_list`, `*_validate`, `*_diff`, `*_status`, `*_report` |
| `destructiveHint: true` | `*_init`, `*_new`, `*_plan`, `*_run`, `*_fix`, `*_promote`, `*_submit` |

## JSON Schema Generation

Arguments and options are converted to JSON Schema:

```rust
fn build_json_schema(cmd: &CommandMetadata) -> Value
```

### Type Mapping

| clap Type | JSON Schema Type |
|-----------|-----------------|
| `FLAG` | `boolean` |
| `INTEGER`, `NUMBER` | `integer` |
| `FLOAT` | `number` |
| `STRING`, `PATH`, `URL` | `string` |

## Claude Desktop Integration

Add to `claude_desktop_config.json`:

**macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
**Linux**: `~/.config/claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "chakravarti": {
      "command": "/path/to/chakravarti-cli/target/release/ckrv-mcp"
    }
  }
}
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `ckrv-cli` | CLI command introspection |
| `serde` | JSON serialization |
| `serde_json` | JSON-RPC parsing |

## Performance

| Metric | Value | Requirement |
|--------|-------|-------------|
| Tool discovery | ~5ms | Not specified |
| Initialize response | <10ms | <100ms ✅ |
| tools/list response | ~10ms | Not specified |

## Tests

14 unit tests covering:

- JSON-RPC request parsing
- Method dispatch
- Error code generation
- Tool discovery
- Schema generation
- Annotation inference
