---
last_commit: e74f093
last_updated: 2026-01-29
related_files:
  - src/main.rs
  - src/commands/mod.rs
  - src/commands/test.rs
  - src/commands/qa.rs
  - src/services/mod.rs
---

# ckrv-cli

CLI entry point and command handlers for Chakravarti.

## Overview

This crate provides the main `ckrv` binary and all CLI command implementations. It acts as the user-facing interface to the Chakravarti orchestration engine.

## Key Types

- **Commands**: Individual command handlers (`init`, `run`, `spec`, `plan`, `test`, `qa`, `cloud`, etc.)
- **Services**: Shared functionality for commands (agent lookup, diff analysis, test framework detection)
- **Prompts**: Embedded prompts for AI agents (QA reviewer, test writer)
- **Cloud**: Cloud execution client for remote job management
- **UI utilities**: Terminal styling, spinners, and theme support

## Module Structure

```
src/
├── main.rs              # Entry point, CLI argument parsing
├── prompts.rs           # User confirmation prompts
├── commands/            # Command implementations
│   ├── init.rs          # ckrv init
│   ├── run.rs           # ckrv run (orchestration)
│   ├── spec.rs          # ckrv spec (spec management)
│   ├── spec_structs.rs  # Spec data structures
│   ├── plan.rs          # ckrv plan
│   ├── verify.rs        # ckrv verify
│   ├── diff.rs          # ckrv diff
│   ├── fix.rs           # ckrv fix
│   ├── promote.rs       # ckrv promote
│   ├── test.rs          # ckrv test (test management)
│   ├── qa.rs            # ckrv qa (QA review)
│   ├── task.rs          # ckrv task
│   ├── status.rs        # ckrv status
│   ├── logs.rs          # ckrv logs
│   ├── report.rs        # ckrv report
│   ├── pull.rs          # ckrv pull
│   ├── ui.rs            # ckrv ui
│   └── cloud/           # ckrv cloud subcommands
├── services/            # Shared services
│   ├── agent_lookup.rs  # Agent configuration loading
│   ├── diff_analyzer.rs # Git diff analysis
│   ├── report_generator.rs  # Test/QA report generation
│   └── test_framework.rs    # Test framework detection
├── cloud/               # Cloud execution client
│   ├── auth.rs          # Authentication
│   ├── client.rs        # API client
│   ├── config.rs        # Cloud configuration
│   ├── credentials.rs   # Credential storage
│   ├── jobs.rs          # Job management
│   └── logs.rs          # Log streaming
├── ui/                  # Terminal UI utilities
│   ├── components.rs    # Reusable UI components
│   ├── spinner.rs       # Progress spinners
│   ├── terminal.rs      # Terminal helpers
│   └── theme.rs         # Color themes
├── prompts/             # Embedded AI prompts
│   ├── qa_reviewer.md   # QA review agent prompt
│   └── test_writer.md   # Test writer agent prompt
└── templates/           # Spec templates
    ├── design-template.md   # Design doc template
    ├── spec-template.yaml   # Spec file template
    └── tasks-template.yaml  # Tasks file template
```

## Commands

| Command | File | Description |
|---------|------|-------------|
| `ckrv init` | `init.rs` | Initialize Chakravarti in repository |
| `ckrv spec` | `spec.rs` | Manage specifications |
| `ckrv plan` | `plan.rs` | Generate execution plans |
| `ckrv run` | `run.rs` | Execute orchestration |
| `ckrv diff` | `diff.rs` | View branch changes |
| `ckrv verify` | `verify.rs` | Run code quality checks |
| `ckrv test` | `test.rs` | Test management with AI |
| `ckrv qa` | `qa.rs` | QA review with AI |
| `ckrv fix` | `fix.rs` | AI-powered error fixing |
| `ckrv promote` | `promote.rs` | Push and create PRs |
| `ckrv status` | `status.rs` | Show workflow status |
| `ckrv logs` | `logs.rs` | View execution logs |
| `ckrv report` | `report.rs` | Generate reports |
| `ckrv task` | `task.rs` | Manage individual tasks |
| `ckrv pull` | `pull.rs` | Pull remote changes |
| `ckrv ui` | `ui.rs` | Launch web dashboard |
| `ckrv cloud` | `cloud/` | Cloud execution commands |

## Services Module

| Service | Purpose |
|---------|---------|
| `agent_lookup` | Load agent configs from `~/.config/chakravarti/agents.yaml` |
| `diff_analyzer` | Analyze git diffs for changed files and types |
| `report_generator` | Generate markdown reports for test/QA results |
| `test_framework` | Detect project test framework (Rust, Node, Python, etc.) |

## Cloud Module

Client for remote job execution via Chakravarti Cloud:

| Module | Purpose |
|--------|---------|
| `auth` | OAuth/API key authentication |
| `client` | REST API client |
| `config` | Cloud endpoint configuration |
| `credentials` | Secure credential storage |
| `jobs` | Remote job submission and monitoring |
| `logs` | Real-time log streaming |

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
| `ckrv-sandbox` | Docker execution, agent providers |
| `ckrv-ui` | Web dashboard server |
| `clap` | Argument parsing |
| `console` | Terminal styling |
| `indicatif` | Progress bars and spinners |
