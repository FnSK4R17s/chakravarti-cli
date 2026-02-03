---
description: Check documentation freshness and update crate docs, top-level docs, and CLI command attributes to match current code.
---

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty). The user may specify:
- A specific crate to update (e.g., `ckrv-core`, `ckrv-cli`)
- A specific doc file to update (e.g., `architecture`, `cli-commands`)
- `--check-only` to only report freshness without updating
- `--all` to check/update all crates and docs

## Goal

Ensure all documentation stays synchronized with the actual code:

1. **Crate documentation** (`crates/<crate>/docs/README.md`)
2. **Top-level docs** (`crates/docs/*.md` - architecture, cli-commands, agent-guide, getting-started)
3. **CLI command attributes** (`long_about`, `after_help` in lib.rs)

## Execution Steps

### 1. Get Current Commit Hash

// turbo
```bash
git rev-parse --short HEAD
```

Store this as `NEW_COMMIT` for later use.

### 2. List All Crate Documentation

// turbo
```bash
find crates -path "*/docs/README.md" -type f | grep -v node_modules
```

If user specified a specific crate, filter to just that crate.

### 3. For Each Crate Doc, Check Freshness

For each doc file found:

// turbo
```bash
# Extract the doc's last commit
DOC_COMMIT=$(grep -oP '^last_commit: \K[a-f0-9]+' <doc_path>)

# Get the crate directory
CRATE_DIR=$(dirname $(dirname <doc_path>))

# Check for changes since doc was generated
git diff --name-only $DOC_COMMIT HEAD -- $CRATE_DIR/src/
```

### 4. Analyze Changes (if any)

For crates with changes, identify:

// turbo
```bash
# Check for public API changes
git diff $DOC_COMMIT HEAD -- $CRATE_DIR/src/ | grep -E '^[\+\-].*(pub fn|pub struct|pub enum|pub trait)'
```

Categorize changes:
| Change Type | Action |
|-------------|--------|
| API signature change | **Immediate update required** |
| New public type | Update within session |
| Internal refactor | Low priority |
| Formatting/comments | Skip |

### 5. Generate Freshness Report

Output a summary table:

```markdown
## Documentation Freshness Report

| Crate | Status | Changes | Priority |
|-------|--------|---------|----------|
| ckrv-core | 🔄 Needs update | +2 pub fn, +1 pub struct | HIGH |
| ckrv-cli | ✅ Up to date | - | - |
| ckrv-git | ✅ Up to date | - | - |
```

If `--check-only` was specified, stop here and report results.

### 6. Update Crate README (if not check-only)

For each crate needing updates:

1. **Read the current README.md** to understand existing structure
2. **Read the changed source files** to understand new/modified APIs
3. **Update the README.md** with:
   - New types/functions in appropriate sections
   - Updated descriptions if behavior changed
   - New usage examples if needed
4. **Update frontmatter**:
   ```yaml
   last_commit: <NEW_COMMIT>
   last_updated: <TODAY_DATE>
   ```

### 7. Verify Updates

// turbo
```bash
# Confirm docs compile (no broken markdown)
cat <updated_doc_path>
```

---

## Step 8-11: Update Top-Level Documentation (`crates/docs/`)

The `crates/docs/` folder contains 4 cross-cutting documentation files that must be kept in sync with the codebase. These aggregate information from **both source code AND crate-level docs**.

| File | Purpose | Primary Sources | Crate Docs to Reference |
|------|---------|-----------------|-------------------------|
| `architecture.md` | System design, crate diagram | `crates/*/Cargo.toml` | All `crates/*/docs/README.md` |
| `cli-commands.md` | Command reference | `ckrv-cli/src/lib.rs` | `crates/ckrv-cli/docs/README.md` |
| `agent-guide.md` | Adding new AI agents | `ckrv-sandbox/src/` | `crates/ckrv-sandbox/docs/README.md` |
| `getting-started.md` | New contributor onboarding | Build scripts, Makefile | All crate docs (for overview) |

---

### Step 8: Update `architecture.md`

**Sources of truth:**
1. The actual crates in `crates/` directory
2. Each crate's `docs/README.md` for descriptions

// turbo
```bash
# List all crates
ls -d crates/ckrv-* | xargs -I{} basename {}
```

// turbo
```bash
# Extract crates mentioned in architecture.md
grep -oP '`ckrv-[a-z]+`' crates/docs/architecture.md | sort -u
```

// turbo
```bash
# List available crate docs
find crates -path "*/docs/README.md" -type f | grep -v node_modules
```

**Compare the lists.** For any missing crates:

1. **Read the crate's Cargo.toml** to understand its purpose
2. **Read the crate's `docs/README.md`** (if exists) for:
   - Description and purpose
   - Key types and traits
   - Dependencies on other crates
3. **Check if it's a dependency** of other crates using:
   ```bash
   grep -l "ckrv-<new>" crates/*/Cargo.toml
   ```
4. **Add to the Crate Responsibilities table** with appropriate status:
   - `✅ Used` - actively used in the codebase
   - `⚠️ Unused` - exists but not integrated
   - `⚠️ Stub` - placeholder for future work

5. **Update the Mermaid dependency graph** based on Cargo.toml dependencies

6. **Sync descriptions** - ensure the Purpose column matches the crate's README overview

7. **Update frontmatter**:
   ```yaml
   last_commit: <NEW_COMMIT>
   last_updated: <TODAY_DATE>
   ```

---

### Step 9: Update `cli-commands.md`

**Sources of truth:**
1. `crates/ckrv-cli/src/lib.rs` Commands enum
2. `crates/ckrv-cli/docs/README.md` for command documentation

// turbo
```bash
# Get all commands from source
grep -E "^\s+[A-Z][a-zA-Z]+\(" crates/ckrv-cli/src/lib.rs | sed 's/.*\s\+\([A-Z][a-zA-Z]*\)(.*/\1/' | sort
```

// turbo
```bash
# Get commands documented in cli-commands.md
grep -E "^### \`" crates/docs/cli-commands.md | sed 's/### `\([^`]*\)`.*/\1/' | sort
```

// turbo
```bash
# Check the ckrv-cli crate README for command details
cat crates/ckrv-cli/docs/README.md 2>/dev/null
```

**Compare the lists.** For any missing commands:

1. **Read the command implementation** in `commands/<cmd>.rs`
2. **Check ckrv-cli's docs/README.md** for existing command documentation
3. **Extract Clap attributes** (`about`, `long_about`, subcommands)
4. **Add a new section** following the existing format:
   ```markdown
   ### `<command>`
   
   <description from crate README or about attribute>
   
   ```bash
   ckrv <command> [OPTIONS]
   ```
   
   **Options:**
   - `--<flag>`: <description>
   
   **Examples:**
   ```bash
   ckrv <command> <example>
   ```
   ```

5. **Update frontmatter**

---

### Step 10: Update `agent-guide.md`

**Sources of truth:**
1. `crates/ckrv-sandbox/src/` agent provider implementations
2. `crates/ckrv-sandbox/docs/README.md` for agent documentation

// turbo
```bash
# List agent provider files
ls crates/ckrv-sandbox/src/*.rs | xargs -I{} basename {} .rs
```

// turbo
```bash
# Get agents documented in agent-guide.md
grep -E "^## |^### " crates/docs/agent-guide.md
```

// turbo
```bash
# Check the ckrv-sandbox crate README for agent details
cat crates/ckrv-sandbox/docs/README.md 2>/dev/null
```

**For each agent provider file:**

1. **Check if documented** in agent-guide.md
2. **Read ckrv-sandbox's docs/README.md** for:
   - Agent configuration patterns
   - Authentication methods
   - Docker execution details
3. **For new agents**, add:
   - Configuration section
   - Authentication requirements
   - Docker image details
   - Example usage (from crate README if available)

4. **Update frontmatter**

---

### Step 11: Update `getting-started.md`

**Sources of truth:**
1. `Makefile`, `package.json`, build scripts
2. All crate `docs/README.md` files for development setup

// turbo
```bash
# Check if Makefile targets match getting-started.md
grep -E "^[a-z-]+:" Makefile | cut -d: -f1
```

// turbo
```bash
# Check documented commands
grep -E "^make |^cargo |^pnpm " crates/docs/getting-started.md
```

// turbo
```bash
# List all crate docs for development reference
find crates -path "*/docs/README.md" -exec grep -l "Development\|Setup\|Building" {} \;
```

**Verify by cross-referencing crate docs:**

1. **Prerequisites** still match actual requirements (check root Cargo.toml)
2. **Installation steps** work with current build system
3. **Development workflow** commands are accurate
4. **Per-crate development notes** - if crate READMEs have dev-specific sections, summarize key points

5. **Update frontmatter**

---

## Step 12: CLI Command Attributes (ckrv-cli specific)

When updating `ckrv-cli`, also update CLI command attributes (`long_about`, `after_help`) in the code.

### 12.1 Identify Commands Needing Documentation

// turbo
```bash
# List all Commands enum variants in lib.rs
grep -E "^\s+/// " crates/ckrv-cli/src/lib.rs
```

### 12.2 For Each Command

For each command in the `Commands` enum:

1. **Read the crate README** (`crates/ckrv-cli/docs/README.md`) to understand:
   - What the command does
   - Its options and arguments
   - Usage patterns

2. **Read the command implementation** (`commands/<cmd>.rs`) to understand:
   - Actual behavior
   - Edge cases
   - Error conditions

3. **Generate documentation content**:
   - `long_about`: Detailed multi-paragraph description
   - `after_help`: Practical examples

### 12.3 Update lib.rs

Add `long_about` and `after_help` to the `#[command(...)]` attribute:

```rust
/// Short description (shown in command list)
#[command(
    display_order = 1,
    long_about = "Detailed description.\n\n\
                  This explains what the command does in depth.\n\n\
                  Include important notes about behavior.",
    after_help = "Examples:\n\
                  # Basic usage\n\
                  ckrv <cmd>\n\n\
                  # With options\n\
                  ckrv <cmd> --option"
)]
CommandName(commands::cmd::CmdArgs),
```

### 12.4 Format Guidelines

**For `long_about`:**
- First paragraph = expanded version of the doc comment
- Second paragraph = what gets created/modified
- Third paragraph = important notes or requirements
- Use `\n\n` between paragraphs
- Use `\n` for line breaks within paragraphs

**For `after_help`:**
- Start with "Examples:\n"
- Use `# comment` format for descriptions
- Show 2-3 practical examples
- Include common option combinations

### 12.5 Commands to Document

Priority order for documentation:

| Priority | Commands | Reason |
|----------|----------|--------|
| HIGH | init, spec new, plan, run | Core workflow |
| MEDIUM | verify, promote, test, qa | Quality checks |
| LOW | cloud, logs, pull, ui | Auxiliary features |

---

## Step 13: Summary Report

Output final summary:

```markdown
## Documentation Update Summary

### Crate READMEs Updated
- `crates/ckrv-core/docs/README.md` - Added RunnerConfig GLM fields
- `crates/ckrv-cli/docs/README.md` - Updated agent_lookup description

### CLI Commands Updated (in lib.rs)
- `Init` - Added long_about and after_help
- `Spec` - Added long_about and after_help

### Skipped (no changes)
- `crates/ckrv-git/docs/README.md`

### Next Steps
1. Run `/docs.skills` to generate command documentation files
2. Run `make skill` to regenerate SKILL.md
3. Commit with: `git commit -m "docs: update crate docs to <NEW_COMMIT>"`
```

---

## Next Workflow

Read the **docs-order** skill to determine what workflow to run next based on what was changed.
