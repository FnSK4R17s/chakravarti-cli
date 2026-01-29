# Chakravarti CLI Development Guidelines

Last updated: 2026-01-21

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
make install

# Build only
cargo build --workspace

# Test
cargo test --workspace

# Lint
cargo clippy --workspace -- -D warnings

# Format
cargo fmt --all

# Generate docs
cargo doc --open --no-deps

# Run CLI
cargo run -p ckrv-cli -- --help
```

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

See [Agent Guide](crates/docs/agent-guide.md) for adding new agents.

## Code Style

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
4. **Build before testing**: Always run `make install` before testing CLI changes

## Troubleshooting

### "command not found: ckrv"
Run `make install` from the repository root.

### Frontend changes not appearing
1. Run `npm run build` in the frontend directory
2. Run `make install` from root
3. Restart `ckrv ui`

### CSS lint warnings about @plugin, @theme, @apply
These are Tailwind v4 directives - the IDE linter doesn't recognize them but they work correctly at build time.


## Active Technologies
- Rust 1.75+ + tokio, bollard (Docker), serde (016-glm-cli-support)
- YAML configuration (`~/.config/chakravarti/agents.yaml`) (016-glm-cli-support)

## Recent Changes
- 016-glm-cli-support: Added Rust 1.75+ + tokio, bollard (Docker), serde
