# SKILL.md Generation Contract

**Generated**: 2026-01-29  
**Spec**: 017-ai-native-interface

## Overview

The `skill_gen` binary generates a SKILL.md file from clap command definitions that conforms to the Agent Skills specification.

## Usage

```bash
# Generate SKILL.md to stdout
cargo run -p ckrv-cli --bin skill_gen

# Generate and validate
cargo run -p ckrv-cli --bin skill_gen > .agent/skills/chakravarti-cli/SKILL.md
uvx --from skills-ref agentskills validate .agent/skills/chakravarti-cli
```

## Output Format

### Frontmatter (YAML)

```yaml
---
name: chakravarti-cli
description: Spec-driven agent orchestration. Create specs, plan tasks, run jobs, and review changes.
license: MIT
compatibility: Claude Code, Cursor, any CLI-capable agent
metadata:
  version: "0.1.0"
  auto-generated: true
  generated-at: "2026-01-29T12:00:00Z"
---
```

**Required Fields**:
- `name`: Must match `^[a-z][a-z0-9-]{0,63}$`
- `description`: 1-1024 characters

**Recommended Fields**:
- `license`: License identifier
- `compatibility`: Compatible agents/environments
- `metadata.version`: CLI version
- `metadata.auto-generated`: Always `true`
- `metadata.generated-at`: ISO 8601 timestamp

### Markdown Body

```markdown
# Chakravarti CLI

[Auto-generated from clap command definitions]

## Overview

{CLI description from #[command(about)]}

## Installation

\`\`\`bash
cargo install ckrv-cli
# or
git clone https://github.com/FnSK4R17s/chakravarti-cli && cd chakravarti-cli && make install
\`\`\`

## Quick Start

\`\`\`bash
# Initialize in a repository
ckrv init

# Create a spec from description
ckrv spec new "Add user authentication"

# Generate execution plan
ckrv plan

# Run the job
ckrv run
\`\`\`

## Commands

### ckrv init

Initialize Chakravarti in the current repository.

\`\`\`bash
ckrv init [OPTIONS]
\`\`\`

### ckrv spec

Create or manage feature specifications.

#### ckrv spec new

Create a new specification using AI from a natural language description.

\`\`\`bash
ckrv spec new <description> [OPTIONS]
\`\`\`

**Arguments**:
| Name | Required | Description |
|------|----------|-------------|
| `description` | Yes | Natural language description of the feature |

**Options**:
| Flag | Description |
|------|-------------|
| `--name, -n <NAME>` | Optional short name for the spec |

#### ckrv spec list

List all specifications.

\`\`\`bash
ckrv spec list
\`\`\`

{... more commands ...}

## Global Options

| Flag | Description |
|------|-------------|
| `--json` | Output format: JSON instead of human-readable |
| `--quiet, -q` | Suppress non-essential output |
| `--verbose, -v` | Enable verbose logging |
| `--help, -h` | Print help |
| `--version, -V` | Print version |

## Examples

### Create and execute a feature

\`\`\`bash
# 1. Initialize repository
ckrv init

# 2. Create spec
ckrv spec new "Add REST API for user management"

# 3. Generate tasks
ckrv spec tasks

# 4. Generate plan
ckrv plan

# 5. Execute
ckrv run

# 6. Verify
ckrv verify

# 7. Create PR
ckrv promote
\`\`\`

### JSON output for scripting

\`\`\`bash
ckrv --json spec list | jq '.specs[].id'
\`\`\`
```

## Generation Rules

### Command Inclusion

1. **Include**: All commands where `!cmd.is_hide_set()`
2. **Exclude**: Commands marked with `#[command(hide = true)]`

### Command Documentation

For each command, generate:

1. **Heading**: `### ckrv {command_path}`
2. **Description**: From `#[command(about = "...")]` or `///` doc comment
3. **Usage**: `\`\`\`bash\nckrv {command_path} [ARGS] [OPTIONS]\n\`\`\``
4. **Arguments table** (if any positional args)
5. **Options table** (if any flags/options)

### Subcommand Handling

For commands with subcommands:
1. Document parent command with subcommand list
2. Document each subcommand as `#### ckrv {parent} {subcommand}`

### Argument/Option Formatting

**Arguments Table**:
```markdown
| Name | Required | Description |
|------|----------|-------------|
| `{id}` | {Yes/No} | {help_text} |
```

**Options Table**:
```markdown
| Flag | Description |
|------|-------------|
| `--{long}, -{short} <VALUE>` | {help_text} |
```

## Validation

After generation, validate with:

```bash
uvx --from skills-ref agentskills validate .agent/skills/chakravarti-cli
```

### Expected Output

```
✓ chakravarti-cli is valid
```

### Possible Errors

| Error | Cause | Fix |
|-------|-------|-----|
| `name: must be lowercase alphanumeric` | Invalid name format | Use `chakravarti-cli` |
| `description: required field missing` | Missing description | Add description in frontmatter |
| `description: must be 1-1024 characters` | Empty or too long | Adjust description length |

## Determinism

The generator MUST produce identical output given identical input:

1. Commands sorted by display_order, then alphabetically
2. Options sorted alphabetically
3. No timestamps in body content (only in metadata.generated-at)
4. Consistent whitespace (2-space indents, single newline between sections)

## Example Full Output

See `.agent/skills/chakravarti-cli/SKILL.md` after running:

```bash
make skill
```
