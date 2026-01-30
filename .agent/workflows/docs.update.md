---
description: Check documentation freshness and update crate docs + CLI command attributes to match current code.
---

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty). The user may specify:
- A specific crate to update (e.g., `ckrv-core`, `ckrv-cli`)
- `--check-only` to only report freshness without updating
- `--all` to check/update all crates

## Goal

Ensure crate documentation (`crates/<crate>/docs/README.md`) and CLI command attributes (`long_about`, `after_help`) stay synchronized with the actual code.

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
cat <updated_doc_path> | head -5
```

---

## Step 8: CLI Command Attributes (ckrv-cli specific)

When updating `ckrv-cli`, also update CLI command attributes (`long_about`, `after_help`) in the code.

### 8.1 Identify Commands Needing Documentation

// turbo
```bash
# List all Commands enum variants in lib.rs
grep -E "^\s+/// " crates/ckrv-cli/src/lib.rs | head -30
```

### 8.2 For Each Command

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

### 8.3 Update lib.rs

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

### 8.4 Format Guidelines

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

### 8.5 Commands to Document

Priority order for documentation:

| Priority | Commands | Reason |
|----------|----------|--------|
| HIGH | init, spec new, plan, run | Core workflow |
| MEDIUM | verify, promote, test, qa | Quality checks |
| LOW | cloud, logs, pull, ui | Auxiliary features |

---

## Step 9: Summary Report

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
