# Quickstart: AI-Native Interface Layer

**Spec**: 017-ai-native-interface  
**Time to complete**: ~20 minutes

## What You'll Build

1. **SKILL.md Generator** - A binary that auto-generates CLI documentation for AI agents
2. **MCP Server** - An MCP-compatible server that exposes CLI commands as tools

## Prerequisites

- Rust toolchain (1.75+)
- Access to the chakravarti-cli repository
- Understanding of clap derive macros

## Part 1: SKILL.md Generator (5 minutes)

### Step 1: Create the binary file

```bash
touch crates/ckrv-cli/src/bin/skill_gen.rs
```

### Step 2: Add basic implementation

```rust
// crates/ckrv-cli/src/bin/skill_gen.rs
use clap::CommandFactory;

// Import Cli from main crate
// Note: We need to make Cli public in lib.rs
fn main() {
    let cmd = ckrv_cli::Cli::command();
    let skill_md = generate_skill_md(&cmd);
    print!("{}", skill_md);
}

fn generate_skill_md(cmd: &clap::Command) -> String {
    let mut output = String::new();
    
    // Frontmatter
    output.push_str(&format!(r#"---
name: chakravarti-cli
description: {}
license: MIT
metadata:
  version: "{}"
  auto-generated: true
---

"#, 
        cmd.get_about().map(|s| s.to_string()).unwrap_or_default(),
        env!("CARGO_PKG_VERSION")
    ));
    
    // Title and commands...
    output.push_str(&format!("# {}\n\n", cmd.get_name()));
    output.push_str("## Commands\n\n");
    
    for subcmd in cmd.get_subcommands() {
        if subcmd.is_hide_set() { continue; }
        output.push_str(&format!("### {} {}\n\n", cmd.get_name(), subcmd.get_name()));
        // ... generate command docs
    }
    
    output
}
```

### Step 3: Add binary to Cargo.toml

```toml
# Add to crates/ckrv-cli/Cargo.toml
[[bin]]
name = "skill_gen"
path = "src/bin/skill_gen.rs"
```

### Step 4: Test generation

```bash
cargo run -p ckrv-cli --bin skill_gen > /tmp/SKILL.md
cat /tmp/SKILL.md
```

### Step 5: Validate

```bash
mkdir -p .agent/skills/chakravarti-cli
cargo run -p ckrv-cli --bin skill_gen > .agent/skills/chakravarti-cli/SKILL.md
uvx --from skills-ref agentskills validate .agent/skills/chakravarti-cli
```

Expected: `✓ chakravarti-cli is valid`

## Part 2: MCP Server (15 minutes)

### Step 1: Create crate structure

```bash
mkdir -p crates/ckrv-mcp/src
cat > crates/ckrv-mcp/Cargo.toml << 'EOF'
[package]
name = "ckrv-mcp"
version.workspace = true
edition.workspace = true

[[bin]]
name = "ckrv-mcp"
path = "src/main.rs"

[dependencies]
ckrv-cli = { workspace = true }
clap = { workspace = true }
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
EOF
```

### Step 2: Add to workspace

```toml
# Add to root Cargo.toml members
members = [
    # ... existing ...
    "crates/ckrv-mcp",
]
```

### Step 3: Create main.rs

```rust
// crates/ckrv-mcp/src/main.rs
use std::io::{self, BufRead, Write};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Deserialize)]
struct Request {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

#[derive(Serialize)]
struct Response {
    jsonrpc: String,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Value>,
}

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        
        if line.is_empty() { continue; }
        
        let response = handle_request(&line);
        if let Some(resp) = response {
            writeln!(stdout, "{}", serde_json::to_string(&resp).unwrap()).ok();
            stdout.flush().ok();
        }
    }
}

fn handle_request(line: &str) -> Option<Response> {
    let req: Request = match serde_json::from_str(line) {
        Ok(r) => r,
        Err(_) => return Some(error_response(Value::Null, -32700, "Parse error")),
    };
    
    let id = req.id.clone().unwrap_or(Value::Null);
    
    match req.method.as_str() {
        "initialize" => Some(Response {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "protocolVersion": "2024-11-05",
                "serverInfo": {"name": "chakravarti-mcp", "version": "0.1.0"},
                "capabilities": {"tools": {}}
            })),
            error: None,
        }),
        "initialized" => None, // Notification, no response
        "tools/list" => Some(Response {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({"tools": get_tools()})),
            error: None,
        }),
        "tools/call" => handle_tool_call(id, &req.params),
        _ => Some(error_response(id, -32601, "Method not found")),
    }
}

fn get_tools() -> Vec<Value> {
    // Generate from clap - simplified for quickstart
    vec![
        json!({
            "name": "ckrv_spec_list",
            "description": "List all specifications",
            "inputSchema": {"type": "object", "properties": {}}
        }),
    ]
}

fn handle_tool_call(id: Value, params: &Value) -> Option<Response> {
    let name = params["name"].as_str().unwrap_or("");
    let args = &params["arguments"];
    
    // Execute via shell
    let result = std::process::Command::new("ckrv")
        .arg("--json")
        .args(parse_tool_name(name))
        .output();
    
    match result {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Some(Response {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(json!({
                    "content": [{"type": "text", "text": stdout}],
                    "isError": !output.status.success()
                })),
                error: None,
            })
        }
        Err(e) => Some(error_response(id, -32001, &format!("Execution failed: {}", e))),
    }
}

fn parse_tool_name(name: &str) -> Vec<&str> {
    name.strip_prefix("ckrv_")
        .unwrap_or(name)
        .split('_')
        .collect()
}

fn error_response(id: Value, code: i32, message: &str) -> Response {
    Response {
        jsonrpc: "2.0".to_string(),
        id,
        result: None,
        error: Some(json!({"code": code, "message": message})),
    }
}
```

### Step 4: Build and test

```bash
cargo build -p ckrv-mcp

echo '{"jsonrpc":"2.0","id":1,"method":"initialize"}' | ./target/debug/ckrv-mcp
```

Expected output:
```json
{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2024-11-05",...}}
```

### Step 5: Add to Claude Desktop

```bash
make install-mcp  # Prints config JSON
```

Add to `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS) or equivalent.

## Verify Everything Works

### Test SKILL.md

```bash
make skill
cat .agent/skills/chakravarti-cli/SKILL.md
```

### Test MCP

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | ./target/release/ckrv-mcp | jq
```

### Test Claude Desktop Integration

1. Restart Claude Desktop
2. Ask: "What Chakravarti tools are available?"
3. Expected: Claude lists the tools from `tools/list`

## Next Steps

1. Implement full command introspection in `get_tools()`
2. Add proper error handling for tool execution
3. Add JSON Schema generation from clap types
4. Run integration tests with MCP Inspector
