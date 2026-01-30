# Data Model: AI-Native Interface Layer

**Generated**: 2026-01-29  
**Spec**: 017-ai-native-interface  
**Status**: Complete

## Entities

### 1. CommandMetadata

Extracted from clap `Command` for documentation generation.

```rust
/// Metadata extracted from a clap Command for documentation/MCP generation
#[derive(Debug, Clone, Serialize)]
pub struct CommandMetadata {
    /// Full command path (e.g., "ckrv spec new")
    pub path: Vec<String>,
    
    /// Command name (e.g., "new")
    pub name: String,
    
    /// Description from #[command(about = "...")] or doc comment
    pub description: String,
    
    /// Positional arguments
    pub arguments: Vec<ArgumentMetadata>,
    
    /// Optional flags and options (--flag, --option=value)
    pub options: Vec<OptionMetadata>,
    
    /// Whether this command is hidden from external interfaces
    pub hidden: bool,
    
    /// Nested subcommands (empty for leaf commands)
    pub subcommands: Vec<CommandMetadata>,
}
```

### 2. ArgumentMetadata

Positional argument information.

```rust
/// Metadata for a positional argument
#[derive(Debug, Clone, Serialize)]
pub struct ArgumentMetadata {
    /// Argument identifier (e.g., "description")
    pub id: String,
    
    /// Help text describing the argument
    pub help: String,
    
    /// Whether the argument is required
    pub required: bool,
    
    /// Type hint for documentation (e.g., "STRING", "PATH")
    pub type_hint: String,
}
```

### 3. OptionMetadata

Flag and option information.

```rust
/// Metadata for a flag or option
#[derive(Debug, Clone, Serialize)]
pub struct OptionMetadata {
    /// Option identifier (e.g., "force")
    pub id: String,
    
    /// Long flag name (e.g., "force" -> --force)
    pub long: Option<String>,
    
    /// Short flag character (e.g., 'f' -> -f)
    pub short: Option<char>,
    
    /// Help text describing the option
    pub help: String,
    
    /// Whether this is a flag (no value) or takes a value
    pub takes_value: bool,
    
    /// Type hint for value (e.g., "STRING", "NUMBER")
    pub value_type: String,
    
    /// Default value if any
    pub default: Option<String>,
}
```

### 4. MCPTool

MCP tool representation for tools/list response.

```rust
/// MCP Tool definition (JSON Schema format)
#[derive(Debug, Clone, Serialize)]
pub struct MCPTool {
    /// Tool name (e.g., "ckrv_spec_new")
    pub name: String,
    
    /// Human-readable description
    pub description: String,
    
    /// JSON Schema for input parameters
    pub input_schema: serde_json::Value,
    
    /// Optional annotations for Claude Desktop
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<MCPToolAnnotations>,
}

/// Tool annotations for client hints
#[derive(Debug, Clone, Serialize)]
pub struct MCPToolAnnotations {
    /// Hint that this tool only reads data
    #[serde(rename = "readOnlyHint")]
    pub read_only_hint: Option<bool>,
    
    /// Hint that this tool may modify state
    #[serde(rename = "destructiveHint")]
    pub destructive_hint: Option<bool>,
}
```

### 5. MCPRequest

Incoming JSON-RPC request structure.

```rust
/// JSON-RPC 2.0 request
#[derive(Debug, Deserialize)]
pub struct MCPRequest {
    /// JSON-RPC version (must be "2.0")
    pub jsonrpc: String,
    
    /// Request ID (can be number or string)
    pub id: serde_json::Value,
    
    /// Method name (e.g., "initialize", "tools/list", "tools/call")
    pub method: String,
    
    /// Method parameters
    #[serde(default)]
    pub params: serde_json::Value,
}
```

### 6. MCPResponse

Outgoing JSON-RPC response structure.

```rust
/// JSON-RPC 2.0 response
#[derive(Debug, Serialize)]
pub struct MCPResponse {
    /// JSON-RPC version (always "2.0")
    pub jsonrpc: String,
    
    /// Request ID (echo back from request)
    pub id: serde_json::Value,
    
    /// Result on success
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    
    /// Error on failure
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<MCPError>,
}

/// JSON-RPC error object
#[derive(Debug, Serialize)]
pub struct MCPError {
    /// Error code (standard JSON-RPC codes or custom)
    pub code: i32,
    
    /// Human-readable error message
    pub message: String,
    
    /// Additional error data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}
```

## Type Mappings

### clap Type → JSON Schema Type

| clap Type | JSON Schema Type | Notes |
|-----------|------------------|-------|
| `String` | `{"type": "string"}` | |
| `PathBuf` | `{"type": "string"}` | File path |
| `bool` | `{"type": "boolean"}` | |
| `i32`, `u32`, etc. | `{"type": "integer"}` | |
| `f32`, `f64` | `{"type": "number"}` | |
| `Option<T>` | Schema of T, not in `required` | |
| `Vec<T>` | `{"type": "array", "items": {...}}` | |
| Enum | `{"type": "string", "enum": [...]}` | From `value_parser` if available |

### clap Type Detection

```rust
fn infer_value_type(arg: &clap::Arg) -> String {
    // Check for explicit value hint
    if let Some(hint) = arg.get_value_hint() {
        match hint {
            ValueHint::FilePath | ValueHint::DirPath => return "path".to_string(),
            ValueHint::Url => return "url".to_string(),
            ValueHint::CommandName => return "command".to_string(),
            _ => {}
        }
    }
    
    // Default to string
    "string".to_string()
}
```

## Validation Rules

### SKILL.md

1. Frontmatter MUST include `name` matching regex `^[a-z][a-z0-9-]{0,63}$`
2. Frontmatter MUST include `description` (1-1024 characters)
3. Generated markdown MUST pass `agentskills validate`
4. All non-hidden commands MUST be documented
5. Generated output MUST be deterministic (same input → same output)

### MCP Server

1. All responses MUST be valid JSON-RPC 2.0
2. Error codes MUST follow JSON-RPC standard:
   - `-32700`: Parse error
   - `-32600`: Invalid request
   - `-32601`: Method not found
   - `-32602`: Invalid params
   - `-32603`: Internal error
3. Tool names MUST match regex `^ckrv_[a-z_]+$`
4. Input schemas MUST be valid JSON Schema

## State Transitions

### SKILL.md Generation

```
[Source Code] → [Compile CLI] → [Run skill_gen] → [SKILL.md written]
                                        ↓
                              [agentskills validate]
                                        ↓
                              [Pass: Done] / [Fail: Error]
```

### MCP Tool Execution

```
[JSON-RPC Request] → [Parse Request] → [Validate Method]
                                              ↓
                     [Method: tools/call] → [Build CLI Command]
                                              ↓
                     [Execute: ckrv --json ...] → [Parse JSON Output]
                                              ↓
                     [JSON-RPC Response] ← [Wrap Result]
```

## Relationships

```
┌──────────────────────────────────────────────────────────┐
│                    ckrv-cli (main.rs)                    │
│                         Cli struct                       │
│                           │                              │
│                  Cli::command() → clap::Command          │
└──────────────────────────────────────────────────────────┘
                           │
           ┌───────────────┴───────────────┐
           │                               │
           ▼                               ▼
┌──────────────────────┐      ┌──────────────────────┐
│   skill_gen binary   │      │   ckrv-mcp crate     │
│                      │      │                      │
│ CommandMetadata[]    │      │ CommandMetadata[]    │
│         │            │      │         │            │
│         ▼            │      │         ▼            │
│    SKILL.md          │      │    MCPTool[]         │
│    (markdown)        │      │    (JSON Schema)     │
└──────────────────────┘      └──────────────────────┘
```

## Implementation Checklist

- [ ] Define `CommandMetadata` struct in `ckrv-cli/src/skill_gen.rs`
- [ ] Implement clap → `CommandMetadata` extractor
- [ ] Implement `CommandMetadata` → SKILL.md Markdown generator
- [ ] Create `skill_gen` binary entry point
- [ ] Define MCP types in `ckrv-mcp/src/types.rs`
- [ ] Implement clap → JSON Schema converter
- [ ] Implement MCP stdio transport
- [ ] Implement MCP method handlers
