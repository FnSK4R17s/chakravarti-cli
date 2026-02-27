# Chakravarti CLI Development Guidelines

Last updated: 2026-01-29

> [!CAUTION]
> **AI AGENTS: NEVER COMMIT OR PUSH**
> 
> Never run `git commit`, `git push`, or any git commands that modify repository state. Only the user commits and pushes code.

> [!IMPORTANT]
> **SPECS FOLDER RULES**
> 
> The `specs/` folder is **strictly for automation workflows only**. Do NOT create or modify files in `specs/` unless:
> 1. The user explicitly asks to create a spec
> 2. Using `/speckit.specify`, `/speckit.plan`, `/speckit.tasks` or similar workflows
> 
> **For casual brainstorming/discussion about features:**
> - Create files in `brainstorming/` folder instead
> - Use the template at `.templates/brainstorm.notes.md`
> - Name folders as `issue-{NNN}-{slug}/` to link to GitHub issues
> - Example: `brainstorming/issue-012-npm-package/notes.md`
> 
> **When discussing GitHub issues:**
> 1. First check `brainstorming/` for existing plans: `ls brainstorming/ | grep "issue-0{NUM}"`
> 2. Reference existing brainstorms before creating new ones
> 3. See the **brainstorming** skill for workflow details
> 
> This separation ensures `specs/` contains only validated, ready-to-execute specifications.

## Overview

Chakravarti is a spec-driven autonomous agent orchestration engine. It transforms high-level specifications into shipping code by orchestrating AI agents across isolated Git worktrees and Docker sandboxes.

## Documentation

**Before making code changes, consult these docs:**

| Document | Purpose |
|----------|---------|
| [Architecture](crates/docs/architecture.md) | Crate dependencies, execution flow, key abstractions |
| [Getting Started](crates/docs/getting-started.md) | Setup, build commands, first contribution |
| [CLI Commands](crates/docs/cli-commands.md) | All commands with options and exit codes |
| [Agent Guide](crates/docs/agent-guide.md) | Adding new AI agent integrations |
| [Rust Conventions](crates/RUST_CONVENTIONS.md) | Rust patterns, CLI documentation (`long_about`, `after_help`) |
| [Frontend Conventions](crates/ckrv-ui/FRONTEND_CONVENTIONS.md) | React/TypeScript patterns, JSDoc requirements |

**Per-crate documentation** is in `crates/<crate>/docs/README.md`.

## Technologies

- **Rust 1.75+** - Core language
- **clap** - CLI argument parsing
- **tokio** - Async runtime
- **axum** - Web server (for UI)
- **bollard** - Docker API client
- **git2** - Git operations

## Project Structure

```text
chakravarti-cli/
├── crates/
│   ├── ckrv-cli/           # CLI entry point, commands
│   ├── ckrv-core/          # Orchestration engine, domain types
│   ├── ckrv-git/           # Git worktrees, branches, diffs
│   ├── ckrv-sandbox/       # Docker execution, agent providers
│   ├── ckrv-spec/          # Spec parsing/validation
│   ├── ckrv-model/         # LLM provider routing (⚠️ unused)
│   ├── ckrv-metrics/       # Cost/timing tracking, file storage
│   ├── ckrv-verify/        # Test execution/parsing (⚠️ unused)
│   ├── ckrv-integrations/  # External services stub (⚠️ stub)
│   └── ckrv-ui/            # Web dashboard server + frontend
├── crates/docs/            # Cross-crate documentation
├── specs/                  # Feature specifications
└── npm/                    # npm package for distribution
```

## Commands

```bash
# Build and install
just install

# Build only
just build

# Test
just test

# Lint
just lint

# Format
just fmt

# Generate docs
just docs

# Run CLI
cargo run -p ckrv-cli -- --help

# Quick install (skip Docker)
just install-quick
```

> **Note**: The Makefile is a thin compatibility shim that forwards to just. If you don't have just installed, `make install` will prompt you to install it. See [just installation](https://github.com/casey/just#installation).

## CLI Usage

```bash
ckrv init                    # Initialize repository
ckrv spec new "description"  # Create spec
ckrv spec tasks              # Generate tasks
ckrv plan                    # Generate execution plan
ckrv run                     # Execute orchestration
ckrv diff                    # View changes
ckrv verify                  # Run tests/lint
ckrv fix                     # AI-powered fixes
ckrv promote --push --open   # Create PR
ckrv ui                      # Launch Web UI
```

## Agents

Chakravarti uses Claude Code CLI as the execution interface:

- **Claude Code (Native)** - Default agent
- **Claude Code + OpenRouter** - 12+ models via Claude Code CLI
- **Claude Code + GLM Coding Plan** - Z.AI's GLM models (UI only for now)
- **OpenAI Codex** - Native CLI integration
- **Qwen Code** - Alibaba's Qwen coding agent CLI

See [Agent Guide](crates/docs/agent-guide.md) for adding new agents.

## Code Style

> [!IMPORTANT]
> **When writing Rust code, follow the conventions in [`crates/RUST_CONVENTIONS.md`](crates/RUST_CONVENTIONS.md).**
> 
> Key requirements for `ckrv-cli`:
> - Every command needs `long_about` and `after_help` attributes
> - These power the SKILL.md generation and MCP server tools

- Follow Rust standard conventions
- Use `rustfmt` for formatting
- Pass `clippy` with no warnings
- Document public APIs with doc comments (`///`)
- Add crate-level docs (`//!`) to each `lib.rs`
- Add tests for new functionality

## Key Files

| File | Purpose |
|------|---------|
| `crates/ckrv-core/src/orchestrator.rs` | Execution orchestration |
| `crates/ckrv-core/src/job.rs` | Job lifecycle management |
| `crates/ckrv-sandbox/src/agent/mod.rs` | Agent provider trait |
| `crates/ckrv-git/src/worktree.rs` | Git worktree management |
| `crates/ckrv-cli/src/commands/run.rs` | Main run command |
| `crates/ckrv-ui/src/api/` | Web UI API endpoints |

## Testing

- Unit tests in each crate's source files
- Integration tests in `crates/ckrv-cli/tests/`
- Tests marked `#[ignore]` require API keys or Docker
- Run `cargo test --workspace` before committing

## Frontend Development

> [!IMPORTANT]
> **When writing frontend code, always follow the conventions in [`crates/ckrv-ui/FRONTEND_CONVENTIONS.md`](crates/ckrv-ui/FRONTEND_CONVENTIONS.md).**
> 
> Key requirements:
> - Every file needs a `@module` header with `@description`, `@context`, `@dependencies`
> - All Props interfaces must have JSDoc documentation
> - State variables need comments explaining their purpose
> - Components over 500 lines must be split

### Tech Stack

- **React 18** with TypeScript
- **Tailwind CSS v4** with `@theme inline` for custom utilities
- **Vite** for bundling
- **shadcn/ui** components (Radix-based)
- **TanStack Query** for data fetching

### Frontend Commands

```bash
# Navigate to frontend directory
cd crates/ckrv-ui/frontend

# Development mode (hot reload)
npm run dev

# Production build
npm run build

# Type checking
npx tsc --noEmit

# Add shadcn component
npx shadcn@latest add [component-name]
```

### CSS Theme System

All colors are centralized in `crates/ckrv-ui/frontend/src/index.css` using OKLCH format:

```css
:root {
  /* === THEME COLORS START === */
  --accent-cyan: oklch(0.82 0.19 195);
  --accent-cyan-dim: oklch(0.82 0.19 195 / 15%);
  /* ... more colors ... */
  /* === THEME COLORS END === */
}
```

**When styling components:**
- ✅ Use semantic Tailwind utilities: `text-accent-cyan`, `bg-accent-green-dim`
- ✅ Use shadcn semantic colors: `text-foreground`, `bg-muted`, `border-border`
- ❌ Avoid inline `style={}` with hardcoded colors
- ❌ Avoid arbitrary values like `text-[#00ff00]`

### Component Guidelines

1. **Use shadcn/ui components** from `@/components/ui/`
2. **Import paths** use `@/` alias for `src/`
3. **State management** via TanStack Query for server state
4. **Icons** from `lucide-react`

## Important Notes

1. **Tailwind v4**: Uses `@theme inline` for custom utilities, not `tailwind.config.js` extend
2. **OKLCH colors**: All theme colors use OKLCH format for better color manipulation
3. **Dark mode only**: The UI is dark-mode only (no light theme toggle)
4. **Build before testing**: Always run `just install` before testing CLI changes

## Troubleshooting

### "command not found: ckrv"
Run `just install` from the repository root.

### Frontend changes not appearing
1. Run `just ui-build` from the repository root
2. Run `just install`
3. Restart `ckrv ui`

### CSS lint warnings about @plugin, @theme, @apply
These are Tailwind v4 directives - the IDE linter doesn't recognize them but they work correctly at build time.


## Active Technologies
- Rust 1.75+ + tokio, bollard (Docker), serde (016-glm-cli-support)
- YAML configuration (`~/.config/chakravarti/agents.yaml`) (016-glm-cli-support)
- Rust 1.75 + clap 4.4 (already in workspace), serde, serde_json, tokio (017-ai-native-interface)
- File system (SKILL.md), no database required (017-ai-native-interface)
- Rust 1.75+ + axum 0.8, tauri 2.0, ts-rs 7.x, thiserror 1.x (019-transport-crate)
- File-based (YAML configs, spec files) - unchanged (019-transport-crate)

## Recent Changes
- 016-glm-cli-support: Added Rust 1.75+ + tokio, bollard (Docker), serde
