---
description: Generate command documentation files and SKILL.md from CLI code attributes.
---

## User Input

```text
$ARGUMENTS
```

The user may specify:
- `--check-only` to only report what would be generated
- A specific command to generate (e.g., `init`, `spec`)

## Goal

Generate command documentation files (`docs/commands/*.md`) and regenerate `SKILL.md` from the CLI code's `long_about` and `after_help` attributes.

## Prerequisites

Run `/docs.update ckrv-cli` first to ensure `long_about` and `after_help` are up-to-date in the code.

## Execution Steps

### 1. Get Current Commit Hash

<!-- turbo -->
```bash
git rev-parse --short HEAD
```

Store this as `NEW_COMMIT`.

### 2. Find Commands with Documentation

<!-- turbo -->
```bash
# List top-level commands that have long_about in lib.rs
echo "=== Top-level commands (lib.rs) ==="
grep -B2 "long_about" crates/ckrv-cli/src/lib.rs | grep "///"

# List subcommands with long_about in command files
echo "=== Subcommands (commands/*.rs) ==="
for f in $(grep -rl "#\[derive(Subcommand)\]" crates/ckrv-cli/src/commands/ 2>/dev/null); do
  name=$(basename "$f" .rs)
  if grep -q "long_about" "$f"; then
    echo "$name: $(grep -B2 'long_about' "$f" | grep '///' | head -5)"
  else
    echo "$name: ⚠️ No subcommands have long_about"
  fi
done
```

### 3. Create Output Directory

<!-- turbo -->
```bash
mkdir -p crates/ckrv-cli/docs/commands
```

### 4. For Each Documented Command

For each command that has `long_about` in the code (both top-level in `lib.rs` AND subcommands in `commands/*.rs`):

1. **Extract from source file** (`lib.rs` for top-level, `commands/<parent>.rs` for subcommands):
   - Doc comment (`/// ...`) = short description
   - `long_about = "..."` = detailed description
   - `after_help = "..."` = examples

2. **Extract from commands/<cmd>.rs**:
   - Arguments from the Args struct
   - Options from `#[arg(...)]` attributes

3. **Determine output path**:
   - Top-level command (init, plan): `commands/<cmd>.md`
   - Subcommand (spec new): `commands/<parent>/<sub>.md`

4. **Create subdirectory if needed**:
   ```bash
   mkdir -p crates/ckrv-cli/docs/commands/<parent>
   ```

5. **Generate the markdown file**

### 5. Command Doc Format

Each generated file follows this format:

```markdown
---
command: <command-path>
generated_from: <source-file>  # lib.rs for top-level, commands/<parent>.rs for subcommands
last_commit: <NEW_COMMIT>
---

# ckrv <command>

<Short description from doc comment>

## Description

<long_about content, formatted as paragraphs>

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<arg>` | Yes/No | <help text> |

(Omit if no arguments)

## Options

| Flag | Description |
|------|-------------|
| `--<flag>`, `-<short>` | <help text> |

(Omit if no options)

## Examples

<after_help content, formatted nicely>

(Omit if no after_help)
```

### 6. Regenerate SKILL.md

<!-- turbo -->
```bash
make skill
```

### 7. Summary Report

```markdown
## Command Documentation Generated

### Generated Files
| Command | File | Has Examples |
|---------|------|--------------|
| init | docs/commands/init.md | ✅ |
| spec new | docs/commands/spec/new.md | ✅ |

### Not Generated (no long_about)
| Command | Action |
|---------|--------|
| plan | Run `/docs.update ckrv-cli` to add long_about |
| run | Run `/docs.update ckrv-cli` to add long_about |

### SKILL.md
- Regenerated: ✅
- Validation: ✅ Passed

### Next Steps
1. Review generated files in `crates/ckrv-cli/docs/commands/`
2. Commit: `git commit -m "docs: generate command docs from code"`
```

## Source of Truth

```
┌─────────────────────────────┐
│  lib.rs / commands/*.rs     │  ← Source of truth
│  (long_about, after_help)   │
└─────────────┬───────────────┘
              │
              ▼
┌─────────────────────────────┐
│  /docs.update workflow      │  ← Updates code from README + implementation
└─────────────┬───────────────┘
              │
              ▼
┌─────────────────────────────┐
│  /docs.skills workflow      │  ← Generates files from code
│  (this workflow)            │
└─────────────┬───────────────┘
              │
              ├──▶ docs/commands/*.md (human reference)
              │
              └──▶ SKILL.md (AI agent reference)
```

## Notes

- Only generates docs for commands that have `long_about` in code
- Scans both `lib.rs` (top-level commands) and `commands/*.rs` (subcommands with `#[derive(Subcommand)]`)
- Subcommands go in subdirectories matching command hierarchy (e.g., `commands/spec/clarify.md`)
- `generated_from` frontmatter reflects the actual source file
- Run `make skill` after to regenerate SKILL.md

---

## Next Workflow

Read the **docs-order** skill to determine what workflow to run next based on what was changed.
