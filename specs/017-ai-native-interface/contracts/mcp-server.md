# MCP Server API Contract

**Generated**: 2026-01-29  
**Spec**: 017-ai-native-interface  
**Protocol Version**: 2024-11-05

## Overview

The MCP server exposes Chakravarti CLI commands as MCP tools over stdio transport using JSON-RPC 2.0.

## Transport

- **Protocol**: JSON-RPC 2.0
- **Transport**: Standard input/output (stdio)
- **Encoding**: UTF-8, newline-delimited JSON
- **Binary**: `ckrv-mcp` (from `crates/ckrv-mcp`)

## Methods

### initialize

Initialize the MCP session.

**Request**:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "protocolVersion": "2024-11-05",
    "clientInfo": {
      "name": "claude-desktop",
      "version": "1.0.0"
    }
  }
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "protocolVersion": "2024-11-05",
    "serverInfo": {
      "name": "chakravarti-mcp",
      "version": "0.1.0"
    },
    "capabilities": {
      "tools": {}
    }
  }
}
```

### initialized

Notification sent by client after initialize response received.

**Request** (notification - no response):
```json
{
  "jsonrpc": "2.0",
  "method": "initialized"
}
```

### tools/list

List available tools.

**Request**:
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/list"
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "tools": [
      {
        "name": "ckrv_init",
        "description": "Initialize Chakravarti in the current repository",
        "inputSchema": {
          "type": "object",
          "properties": {},
          "required": []
        }
      },
      {
        "name": "ckrv_spec_new",
        "description": "Create a new specification using AI from a natural language description",
        "inputSchema": {
          "type": "object",
          "properties": {
            "description": {
              "type": "string",
              "description": "Natural language description of the feature"
            },
            "name": {
              "type": "string",
              "description": "Optional short name for the spec"
            }
          },
          "required": ["description"]
        },
        "annotations": {
          "destructiveHint": true
        }
      },
      {
        "name": "ckrv_spec_list",
        "description": "List all specifications",
        "inputSchema": {
          "type": "object",
          "properties": {},
          "required": []
        },
        "annotations": {
          "readOnlyHint": true
        }
      }
    ]
  }
}
```

### tools/call

Execute a tool.

**Request**:
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "ckrv_spec_list",
    "arguments": {}
  }
}
```

**Response (success)**:
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\"specs\": [{\"id\": \"001-auth\", \"status\": \"ready\"}]}"
      }
    ],
    "isError": false
  }
}
```

**Response (tool error)**:
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "Error: Not initialized. Run 'ckrv init' first."
      }
    ],
    "isError": true
  }
}
```

## Tool Naming Convention

| CLI Command | MCP Tool Name |
|-------------|---------------|
| `ckrv init` | `ckrv_init` |
| `ckrv spec new` | `ckrv_spec_new` |
| `ckrv spec list` | `ckrv_spec_list` |
| `ckrv plan` | `ckrv_plan` |
| `ckrv run` | `ckrv_run` |
| `ckrv diff` | `ckrv_diff` |
| `ckrv verify` | `ckrv_verify` |
| `ckrv test run` | `ckrv_test_run` |

**Rule**: Replace spaces with underscores, lowercase.

## Tool Annotations

| Command | readOnlyHint | destructiveHint | Rationale |
|---------|--------------|-----------------|-----------|
| `ckrv_init` | false | true | Creates files |
| `ckrv_spec_new` | false | true | Creates spec files |
| `ckrv_spec_list` | true | false | Read-only listing |
| `ckrv_spec_validate` | true | false | Validation only |
| `ckrv_plan` | false | true | Creates plan files, may run agent |
| `ckrv_run` | false | true | Executes agent, modifies code |
| `ckrv_diff` | true | false | Read-only diff |
| `ckrv_verify` | true | false | Runs tests, no modifications |

## Error Codes

| Code | Message | Cause |
|------|---------|-------|
| -32700 | Parse error | Invalid JSON |
| -32600 | Invalid Request | Missing required fields |
| -32601 | Method not found | Unknown method |
| -32602 | Invalid params | Invalid tool name or arguments |
| -32603 | Internal error | Server-side error |
| -32001 | Tool execution failed | `ckrv` command failed |

## Schema Definitions

### Tool Input Schema Format

```json
{
  "type": "object",
  "properties": {
    "<arg_name>": {
      "type": "<json_type>",
      "description": "<help_text>"
    }
  },
  "required": ["<required_args>"]
}
```

### Tool Result Format

All tool results are wrapped in content array per MCP spec:

```json
{
  "content": [
    {
      "type": "text",
      "text": "<json_output_from_ckrv>"
    }
  ],
  "isError": <boolean>
}
```

## Example Session

```
< {"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05"}}
> {"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2024-11-05","serverInfo":{"name":"chakravarti-mcp","version":"0.1.0"},"capabilities":{"tools":{}}}}
< {"jsonrpc":"2.0","method":"initialized"}
< {"jsonrpc":"2.0","id":2,"method":"tools/list"}
> {"jsonrpc":"2.0","id":2,"result":{"tools":[...]}}
< {"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"ckrv_spec_list","arguments":{}}}
> {"jsonrpc":"2.0","id":3,"result":{"content":[{"type":"text","text":"{...}"}],"isError":false}}
```

## Testing

### Smoke Test

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | ckrv-mcp
```

Expected: Valid JSON-RPC response with server info.

### MCP Inspector

```bash
npx @anthropic-ai/mcp-inspector ckrv-mcp
```

Opens browser UI for interactive testing.
