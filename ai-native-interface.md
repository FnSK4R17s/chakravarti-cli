# AI-Native Interface Layer for Chakravarti CLI

## Overview

Enable any AI agent to orchestrate Chakravarti through two complementary interfaces:

1. **SKILL.md** - Auto-generated CLI documentation for agents that run shell commands
2. **MCP Server** - Direct API access for agents with MCP support (Claude Desktop, etc.)

**Zero-maintenance goal**: Both interfaces auto-generate from source code. No manual documentation to maintain.

---

## Current State Analysis

### Crate Structure
```
crates/
├── ckrv-cli/       # CLI entry point (clap derive macros)
├── ckrv-core/      # Orchestrator, Spec, Plan, Job, Workflow
├── ckrv-git/       # Git operations
├── ckrv-sandbox/   # Docker execution
├── ckrv-spec/      # Spec parsing
├── ckrv-metrics/   # Cost/timing tracking
├── ckrv-ui/        # Web dashboard (axum)
├── ckrv-verify/    # Test execution
├── ckrv-model/     # LLM routing (unused)
└── ckrv-integrations/ # External services (stub)
```

### CLI Commands (from clap)
| Command | Description | Subcommands |
|---------|-------------|-------------|
| `init` | Initialize repository | - |
| `spec` | Manage specifications | new, clarify, design, init, tasks, validate, list |
| `plan` | Generate execution plan | - |
| `run` | Execute job | - |
| `diff` | View changes | - |
| `verify` | Run tests/lint | - |
| `promote` | Create PR | - |
| `fix` | AI error fixing | - |
| `ui` | Web dashboard | - |
| `cloud` | Cloud execution | - |
| `logs` | Stream logs | - |
| `pull` | Pull results | - |
| `test` | Test runner | run, plan, write |
| `qa` | Code review | review, bugs, report |

### Core Types (ckrv-core)
```rust
pub use agent_task::{AgentTask, AgentTaskStatus, TaskError};
pub use config::Config;
pub use job::{Attempt, AttemptResult, Job, JobConfig, OptimizeMode};
pub use orchestrator::{DefaultOrchestrator, Orchestrator, OrchestratorError};
pub use plan::Plan;
pub use planner::{DefaultPlanner, PlanContext, PlanError, Planner};
pub use spec::{Spec, VerifyConfig};
pub use state::RunState;
pub use workflow::{Workflow, WorkflowStep};
```

---

## Deliverables

| Path | Type | Description |
|------|------|-------------|
| `.agent/skills/chakravarti-cli/SKILL.md` | Generated | CLI skill for AI agents |
| `crates/ckrv-mcp/` | New crate | MCP server exposing core as tools |
| `crates/ckrv-cli/src/bin/skill_gen.rs` | Binary | Generates SKILL.md from clap |

---

## Part 1: SKILL.md Auto-Generation

### Zero-Maintenance Strategy

Use clap's built-in introspection to extract command metadata at build/runtime:

```rust
// skill_gen.rs
use clap::CommandFactory;
use ckrv_cli::Cli;

fn main() {
    let cmd = Cli::command();
    let skill = generate_skill_md(&cmd);
    println!("{}", skill);
}
```

### Output Format

```markdown
---
name: chakravarti-cli
description: Spec-driven agent orchestration. Use for creating specs, planning tasks, running jobs, and reviewing changes.
metadata:
  version: "0.1.0"
  auto-generated: true
---

# Chakravarti CLI

[Auto-generated from clap command definitions]

## Quick Start

\`\`\`bash
# Initialize in a repository
ckrv init

# Create a spec from description
ckrv spec new "Add user authentication with OAuth2"

# Generate execution plan
ckrv plan specs/001-auth

# Run the job
ckrv run specs/001-auth
\`\`\`

## Commands

### ckrv init
Initialize Chakravarti in the current repository.

\`\`\`bash
ckrv init [OPTIONS]
\`\`\`

### ckrv spec
Create or manage feature specifications.

| Subcommand | Description |
|------------|-------------|
| `new <description>` | Create spec from natural language |
| `clarify <spec>` | Resolve clarifications |
| `design <spec>` | Generate technical design |
| `tasks <spec>` | Generate implementation tasks |

[... rest auto-generated ...]
```

### Generation Approach

```rust
fn generate_skill_md(cmd: &clap::Command) -> String {
    let mut output = String::new();
    
    // Frontmatter
    output.push_str(&format!(r#"---
name: {}
description: {}
metadata:
  version: "{}"
  auto-generated: true
---

"#, cmd.get_name(), cmd.get_about().unwrap_or_default(), env!("CARGO_PKG_VERSION")));

    // Title
    output.push_str(&format!("# {}\n\n", cmd.get_name()));
    
    // Commands
    output.push_str("## Commands\n\n");
    for subcmd in cmd.get_subcommands() {
        if subcmd.is_hide_set() { continue; }
        output.push_str(&format!("### {} {}\n", cmd.get_name(), subcmd.get_name()));
        output.push_str(&format!("{}\n\n", subcmd.get_about().unwrap_or_default()));
        output.push_str(&format!("```bash\n{} {} [OPTIONS]", cmd.get_name(), subcmd.get_name()));
        
        // Arguments
        for arg in subcmd.get_positionals() {
            output.push_str(&format!(" <{}>", arg.get_id()));
        }
        output.push_str("\n```\n\n");
        
        // Options table
        let opts: Vec<_> = subcmd.get_opts().collect();
        if !opts.is_empty() {
            output.push_str("| Option | Description |\n|--------|-------------|\n");
            for opt in opts {
                output.push_str(&format!("| `--{}` | {} |\n", 
                    opt.get_id(), 
                    opt.get_help().unwrap_or_default()));
            }
            output.push_str("\n");
        }
    }
    
    output
}
```

### Build Integration

```makefile
# Makefile
.PHONY: skill
skill:
	cargo run -p ckrv-cli --bin skill_gen > .agent/skills/chakravarti-cli/SKILL.md
	uvx --from skills-ref agentskills validate .agent/skills/chakravarti-cli
```

### Feasibility: ✅ HIGH

| Factor | Assessment |
|--------|------------|
| clap introspection API | Stable, well-documented |
| Zero maintenance | ✅ Regenerate on release |
| Validation | Use `agentskills validate` |
| Complexity | Low (~100 lines Rust) |

---

## Part 2: MCP Server

### Zero-Maintenance Strategy

The MCP server should:
1. **Wrap CLI commands** - Don't duplicate logic, shell out to `ckrv`
2. **Parse JSON output** - CLI supports `--json` flag
3. **Auto-discover commands** - Use same clap introspection

### Architecture

```
┌─────────────────────────────────────────────────┐
│                   MCP Client                    │
│            (Claude Desktop, etc.)               │
└─────────────────────┬───────────────────────────┘
                      │ JSON-RPC / stdio
┌─────────────────────▼───────────────────────────┐
│                  ckrv-mcp                       │
│  ┌─────────────────────────────────────────┐   │
│  │  Transport Layer (stdio JSON-RPC)       │   │
│  └────────────────────┬────────────────────┘   │
│  ┌────────────────────▼────────────────────┐   │
│  │  Tool Registry (from clap Commands)     │   │
│  └────────────────────┬────────────────────┘   │
│  ┌────────────────────▼────────────────────┐   │
│  │  Executor (shells out to `ckrv --json`) │   │
│  └─────────────────────────────────────────┘   │
└─────────────────────────────────────────────────┘
```

### Dynamic Tool Discovery

Instead of hardcoding 5 tools, generate them from clap:

```rust
fn discover_tools() -> Vec<Tool> {
    let cmd = Cli::command();
    let mut tools = Vec::new();
    
    for subcmd in cmd.get_subcommands() {
        if subcmd.is_hide_set() { continue; }
        
        // Build JSON schema from clap arguments
        let schema = build_schema(subcmd);
        
        tools.push(Tool {
            name: format!("ckrv_{}", subcmd.get_name()),
            description: subcmd.get_about().unwrap_or_default().to_string(),
            input_schema: schema,
        });
    }
    
    tools
}
```

### Tool Execution

```rust
async fn execute_tool(name: &str, args: Value) -> Result<Value> {
    // Parse tool name: "ckrv_spec_new" -> ["spec", "new"]
    let parts: Vec<&str> = name.strip_prefix("ckrv_").unwrap().split('_').collect();
    
    // Build command
    let mut cmd = Command::new("ckrv");
    cmd.arg("--json");
    cmd.args(&parts);
    
    // Add arguments from JSON
    if let Some(obj) = args.as_object() {
        for (key, value) in obj {
            cmd.arg(format!("--{}", key));
            cmd.arg(value.as_str().unwrap_or(&value.to_string()));
        }
    }
    
    // Execute and parse
    let output = cmd.output().await?;
    let result: Value = serde_json::from_slice(&output.stdout)?;
    
    Ok(result)
}
```

### Crate Structure

```
crates/ckrv-mcp/
├── Cargo.toml
├── src/
│   ├── main.rs          # stdio transport entry
│   ├── lib.rs           # Server struct
│   ├── transport.rs     # JSON-RPC stdio handler
│   ├── tools.rs         # Tool discovery + execution
│   └── schema.rs        # clap -> JSON Schema converter
```

### Cargo.toml

```toml
[package]
name = "ckrv-mcp"
version.workspace = true
edition.workspace = true

[[bin]]
name = "ckrv-mcp"
path = "src/main.rs"

[dependencies]
ckrv-cli = { workspace = true }  # For Cli::command()
clap = { workspace = true }
tokio = { workspace = true, features = ["process", "io-std"] }
serde = { workspace = true }
serde_json = { workspace = true }
```

### Feasibility Assessment

| Factor | Assessment | Risk |
|--------|------------|------|
| clap → JSON Schema | Need custom converter | Medium |
| stdio transport | Well-documented pattern | Low |
| Shell execution | Simple, but adds latency | Low |
| Zero maintenance | ✅ Tools derive from clap | None |

### Alternative: Direct Core Integration

For lower latency, MCP could call `ckrv-core` directly:

```rust
// Higher complexity, but faster
async fn execute_tool(name: &str, args: Value) -> Result<Value> {
    match name {
        "ckrv_spec_new" => {
            let desc = args["description"].as_str().unwrap();
            let spec = ckrv_core::Spec::create_from_description(desc).await?;
            Ok(serde_json::to_value(spec)?)
        }
        // ... more cases
    }
}
```

**Trade-off**: Direct integration is faster but requires maintenance when core APIs change. Shell approach is slower but zero-maintenance.

**Recommendation**: Start with shell approach. Optimize later if latency is a problem.

---

## Part 3: Build Integration

### Makefile Additions

```makefile
.PHONY: skill mcp install-mcp

# Generate SKILL.md from clap
skill:
	@echo "Generating SKILL.md..."
	cargo run -p ckrv-cli --bin skill_gen > .agent/skills/chakravarti-cli/SKILL.md
	uvx --from skills-ref agentskills validate .agent/skills/chakravarti-cli

# Build MCP server
mcp:
	cargo build --release -p ckrv-mcp

# Install MCP + print Claude Desktop config
install-mcp: mcp
	@echo "MCP server built. Add to Claude Desktop config:"
	@echo ""
	@echo '{'
	@echo '  "mcpServers": {'
	@echo '    "chakravarti": {'
	@echo '      "command": "$(shell pwd)/target/release/ckrv-mcp"'
	@echo '    }'
	@echo '  }'
	@echo '}'
```

### CI Integration

```yaml
# .github/workflows/release.yml
- name: Generate SKILL.md
  run: make skill

- name: Validate skill
  run: uvx --from skills-ref agentskills validate .agent/skills/chakravarti-cli

- name: Build MCP
  run: make mcp
```

---

## Acceptance Criteria

### SKILL.md Generation
- [ ] `cargo run --bin skill_gen` produces valid SKILL.md
- [ ] All non-hidden commands are documented
- [ ] Subcommands include arguments and options
- [ ] Output passes `agentskills validate`
- [ ] Regeneration produces identical output (deterministic)

### MCP Server
- [ ] `cargo build -p ckrv-mcp` succeeds
- [ ] `echo '{"jsonrpc":"2.0","id":1,"method":"initialize"}' | ckrv-mcp` returns valid response
- [ ] `tools/list` returns all non-hidden CLI commands as tools
- [ ] `tools/call` with `ckrv_spec_list` returns spec list
- [ ] Each tool has valid JSON Schema for inputs
- [ ] MCP Inspector can connect and execute tools

### Zero Maintenance
- [ ] Adding new CLI command automatically appears in SKILL.md
- [ ] Adding new CLI command automatically appears as MCP tool
- [ ] No manual documentation files to update

---

## Implementation Order

| Phase | Task | Effort |
|-------|------|--------|
| 1 | `skill_gen.rs` binary | 2-4 hours |
| 2 | Validate + integrate in Makefile | 1 hour |
| 3 | `ckrv-mcp` crate scaffold | 2 hours |
| 4 | clap → JSON Schema converter | 4-6 hours |
| 5 | stdio transport | 2-3 hours |
| 6 | Tool execution (shell) | 2-3 hours |
| 7 | Integration tests | 2-3 hours |

**Total Estimate**: 15-22 hours

---

## Risks and Mitigations

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| clap schema conversion incomplete | Medium | Medium | Start with common types, iterate |
| MCP protocol changes | Low | High | Pin to specific MCP version |
| Shell execution too slow | Low | Low | Profile first, optimize if needed |
| Hidden commands leak | Low | Low | Filter `is_hide_set()` consistently |

---

## Open Questions

1. **Should MCP expose all commands or a curated subset?**
   - Recommendation: All non-hidden commands (matches SKILL.md)

2. **Should MCP support streaming for long operations?**
   - Recommendation: V1 without streaming, add later if needed

3. **Where should generated SKILL.md live?**
   - Recommendation: `.agent/skills/chakravarti-cli/SKILL.md`
