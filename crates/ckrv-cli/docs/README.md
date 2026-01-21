---
last_commit: c1bb442
last_updated: 2026-01-21
related_files:
  - src/main.rs
  - src/commands/mod.rs
---

# ckrv-cli

CLI entry point and command handlers for Chakravarti.

## Overview

This crate provides the main `ckrv` binary and all CLI command implementations. It acts as the user-facing interface to the Chakravarti orchestration engine.

## Key Types

- **Commands**: Individual command handlers (`init`, `run`, `spec`, `ui`, etc.)
- **Prompts**: User interaction templates for confirmations and choices
- **UI bindings**: Connection to the web dashboard

## Module Structure

```
src/
├── main.rs           # Entry point, CLI argument parsing
├── commands/         # Command implementations
│   ├── init.rs       # ckrv init
│   ├── run.rs        # ckrv run (orchestration)
│   ├── spec.rs       # ckrv spec (spec management)
│   ├── plan.rs       # ckrv plan
│   ├── verify.rs     # ckrv verify
│   ├── diff.rs       # ckrv diff
│   ├── fix.rs        # ckrv fix
│   ├── promote.rs    # ckrv promote
│   └── ui.rs         # ckrv ui
├── prompts.rs        # User interaction templates
└── ui/               # UI server bindings
```

## Usage

```rust
// The CLI is typically run as a binary:
// $ ckrv --help

// For programmatic use (rare):
use ckrv_cli::commands;
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `ckrv-core` | Orchestration engine |
| `ckrv-git` | Git operations |
| `ckrv-ui` | Web dashboard server |
| `clap` | Argument parsing |
| `console` | Terminal styling |
