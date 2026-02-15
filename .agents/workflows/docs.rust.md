---
description: Analyze Rust crate documentation and apply conventions without changing code logic.
---

# Rust Documentation & Convention Application

## User Input

```
$ARGUMENTS
```

**Optional:** Use the `--crate` flag with a name (e.g., `ckrv-core`) to focus on a specific crate.
By default, runs on **all crates**.

## Goal

Detect missing documentation and apply it in a single pass. No report files are generated — issues are fixed inline and a summary is printed at the end.

**This workflow:**
- ✅ Adds documentation comments (`///`, `//!`, `//`)
- ✅ Adds section separators (`// ===...`)
- ✅ Adds CLI `long_about`/`after_help` attributes (ckrv-cli)
- ✅ Fixes issues as they're found (detect-and-fix)
- ⚠️ Warns about unfixable issues (file too large, needs splitting)
- ❌ Does NOT modify code logic
- ❌ Does NOT generate report files

## Conventions Reference

The conventions are defined in:

```
/apps/chakravarti-cli/crates/RUST_CONVENTIONS.md
```

Read this file fully before applying any documentation. Key sections:
- **Crate-Specific Requirements** - CLI `long_about`/`after_help` patterns
- **Module Structure** - File organization template
- **Documentation Requirements** - What needs `///` vs `//!`
- **Import Organization** - std → external → workspace → crate

---

## Phase 1: Crate Discovery

### Step 1.1: Get Target Crates

By default, analyze **all** crates. Use the `--crate` flag with a crate name to focus on one.

<!-- turbo -->
```bash
ls -d /apps/chakravarti-cli/crates/ckrv-* | xargs -I{} basename {}
```

If `--crate <name>` was specified, filter to just that crate.

### Step 1.2: Loop Structure

For each crate in the list:
1. Run Phase 2 (Detect & Fix)
2. Run Phase 3 if crate is `ckrv-cli` (CLI-specific)

Then output the combined summary (Phase 4).

---

## Phase 2: Detect & Fix (Per Crate)

**For each crate, detect issues and fix them immediately.**

Set the current crate:
```bash
CRATE="<current_crate_from_loop>"  # e.g., ckrv-core
```

### Step 2.1: File Inventory

<!-- turbo -->
```bash
find /apps/chakravarti-cli/crates/$CRATE/src -name "*.rs" 2>/dev/null | head -50
```

### Step 2.2: Add Missing Module Docs

For each `.rs` file, check if `//!` module docs exist in the first 10 lines.

<!-- turbo -->
```bash
for f in $(find /apps/chakravarti-cli/crates/$CRATE/src -name "*.rs" 2>/dev/null); do
  name=$(basename "$f")
  if ! head -10 "$f" | grep -q "^//!"; then
    echo "FIX $name - Missing module docs (//!)"
  fi
done
```

**For each file that says FIX:**
1. Read the file to understand its purpose
2. **Edit the file** — add a module documentation header at line 1:

```rust
//! # <ModuleName>
//!
//! <Brief description inferred from file contents>
//!
//! ## Overview
//!
//! <What this module does and how it fits in>
```

### Step 2.3: Add Missing Public API Docs

For each file, find undocumented `pub` items.

<!-- turbo -->
```bash
for f in $(find /apps/chakravarti-cli/crates/$CRATE/src -name "*.rs" 2>/dev/null); do
  name=$(basename "$f")
  pub_count=$(grep -c "^pub " "$f" 2>/dev/null || echo 0)
  doc_count=$(grep -B1 "^pub " "$f" | grep -c "^///" 2>/dev/null || echo 0)
  
  if [ "$pub_count" -gt 0 ]; then
    undoc=$((pub_count - doc_count))
    if [ "$undoc" -gt 0 ]; then
      echo "FIX $name: $undoc undocumented pub items"
    fi
  fi
done
```

**For each file that says FIX:**
1. Read the file to find the undocumented `pub` items
2. **Edit the file** — add `///` documentation above each:

```rust
/// <Description inferred from function name, params, and return type>
///
/// # Arguments
///
/// * `param` - <Description>
///
/// # Returns
///
/// <Description>
pub fn undocumented_fn(param: Type) -> Result<Output> {
```

### Step 2.4: Add Missing Section Separators

For files > 100 lines, check for `// ===` section separators.

<!-- turbo -->
```bash
for f in $(find /apps/chakravarti-cli/crates/$CRATE/src -name "*.rs" 2>/dev/null); do
  name=$(basename "$f")
  lines=$(wc -l < "$f")
  
  if [ "$lines" -gt 100 ]; then
    sections=$(grep -c "// ===" "$f" 2>/dev/null || echo 0)
    if [ "$sections" -lt 2 ]; then
      echo "FIX $name ($lines lines): needs section separators (found $sections)"
    fi
  fi
done
```

**For each file that says FIX:**
1. Read the file to identify logical sections
2. **Edit the file** — add section separators:

```rust
// ============================================================
// IMPORTS
// ============================================================

// ============================================================
// TYPES
// ============================================================

// ============================================================
// IMPLEMENTATION
// ============================================================
```

Ensure at least: IMPORTS, TYPES/CONSTANTS, IMPLEMENTATION sections are marked.

### Step 2.5: Warn About Large Files (Cannot Auto-Fix)

<!-- turbo -->
```bash
for f in $(find /apps/chakravarti-cli/crates/$CRATE/src -name "*.rs" 2>/dev/null); do
  lines=$(wc -l < "$f")
  name=$(basename "$f")
  if [ "$lines" -gt 500 ]; then
    echo "⚠️ WARN $name ($lines lines): consider splitting"
  fi
done
```

These are logged in the summary but **not fixed** — splitting files requires code changes.

---

## Phase 3: CLI Command Docs (ckrv-cli only)

**Only run this phase when processing the `ckrv-cli` crate.**

### Step 3.1: Find Top-Level Commands Missing long_about (lib.rs)

<!-- turbo -->
```bash
# Extract command names and check for long_about in lib.rs
grep -B 10 "^\s*[A-Z][a-zA-Z]*(" /apps/chakravarti-cli/crates/ckrv-cli/src/lib.rs | \
awk '
  /long_about/ { has_long=1 }
  /after_help/ { has_after=1 }
  /^\s+[A-Z][a-zA-Z]+\(/ {
    cmd=$1
    gsub(/\(.*/, "", cmd)
    gsub(/^\s+/, "", cmd)
    if (!has_long) print "FIX lib.rs : " cmd ": missing long_about"
    if (!has_after) print "FIX lib.rs : " cmd ": missing after_help"
    has_long=0; has_after=0
  }
'
```

### Step 3.2: Find Subcommands Missing long_about (command files)

<!-- turbo -->
```bash
# Find all files with #[derive(Subcommand)] and check each variant
for f in $(grep -rl "#\[derive(Subcommand)\]" /apps/chakravarti-cli/crates/ckrv-cli/src/commands/ 2>/dev/null); do
  name=$(basename "$f")
  # Extract enum variants and check for long_about/after_help
  awk '
    /derive\(Subcommand\)/ { in_enum=1; next }
    in_enum && /^pub enum/ { next }
    in_enum && /^\}/ { in_enum=0; next }
    in_enum && /long_about/ { has_long=1 }
    in_enum && /after_help/ { has_after=1 }
    in_enum && /^\s+[A-Z][a-zA-Z]+\s*[\{(]/ {
      cmd=$1
      gsub(/\s*[\{(].*/, "", cmd)
      gsub(/^\s+/, "", cmd)
      if (!has_long) print "FIX " FILENAME " : " cmd ": missing long_about"
      if (!has_after) print "FIX " FILENAME " : " cmd ": missing after_help"
      has_long=0; has_after=0
    }
  ' "$f"
done
```

### Step 3.3: Add Missing CLI Attributes

**For each command/subcommand that says FIX:**

1. Read the command implementation to understand what it does
2. **Edit the file** — add the missing attributes to the variant:

**For top-level commands in `lib.rs`:**
```rust
/// <Short description>
#[command(
    display_order = X,
    long_about = "<Detailed description.>\n\n\
                  <Explain what this command does in depth.>",
    after_help = "Examples:\n\
                  # Basic usage\n\
                  ckrv <cmd>\n\n\
                  # With options\n\
                  ckrv <cmd> --option"
)]
CommandName(commands::cmd::CmdArgs),
```

**For subcommands in command files (e.g., `spec.rs`, `qa.rs`, `test.rs`):**
```rust
/// <Short description>
#[command(
    long_about = "<Detailed description.>\n\n\
                  <What this subcommand does and when to use it.>",
    after_help = "Examples:\n\
                  # Basic usage\n\
                  ckrv <parent> <sub>\n\n\
                  # With options\n\
                  ckrv <parent> <sub> --option value"
)]
SubcommandName {
    /// <field docs>
    field: Type,
},
```

**Format guidelines:**
- `long_about`: First paragraph = expanded doc comment. Second = what gets created/modified. Third = important notes.
- `after_help`: Start with `"Examples:\n"`. Show 2-3 practical examples. Use `# comment` format.
- For subcommands, always include the parent command in examples (e.g., `ckrv spec clarify`, not just `clarify`).

---

## Phase 4: Post-Edit Verification

**After all edits are complete, re-check every modified file to confirm the applied documentation follows conventions.** This catches mistakes — especially from smaller models that may generate wrong formats.

### Step 4.1: Verify Module Docs Format

For each file that was modified, confirm `//!` docs follow the convention:

<!-- turbo -->
```bash
for f in $(find /apps/chakravarti-cli/crates/$CRATE/src -name "*.rs" 2>/dev/null); do
  name=$(basename "$f")
  if head -10 "$f" | grep -q "^//!"; then
    # Check it has the required "# ModuleName" header
    if ! head -5 "$f" | grep -q "^//! #"; then
      echo "REFIX $name - Module doc missing '# ModuleName' header"
    fi
  fi
done
```

**Convention check:** Module docs must start with `//! # <ModuleName>` followed by a blank `//!` line and a description. If any say REFIX, edit the file to correct the format.

### Step 4.2: Verify Public API Docs Format

For each modified file, confirm `///` docs have proper structure:

<!-- turbo -->
```bash
for f in $(find /apps/chakravarti-cli/crates/$CRATE/src -name "*.rs" 2>/dev/null); do
  name=$(basename "$f")
  
  # Find /// docs that are just a bare description with no structure
  bare_docs=$(grep -B1 "^pub " "$f" | grep "^///" | grep -vc "# \|Arguments\|Returns\|Errors\|Example\|Panics" 2>/dev/null || echo 0)
  total_docs=$(grep -B1 "^pub " "$f" | grep -c "^///" 2>/dev/null || echo 0)
  
  if [ "$total_docs" -gt 0 ] && [ "$bare_docs" -eq "$total_docs" ]; then
    echo "⚠️ CHECK $name - All pub docs are bare descriptions (may need # Arguments, # Returns sections)"
  fi
done
```

**Convention check:** Public functions with parameters should have `# Arguments` section. Functions returning `Result` should have `# Errors` section. If any show CHECK, review and add missing sections.

### Step 4.3: Verify Section Separators Format

<!-- turbo -->
```bash
for f in $(find /apps/chakravarti-cli/crates/$CRATE/src -name "*.rs" 2>/dev/null); do
  name=$(basename "$f")
  lines=$(wc -l < "$f")
  
  if [ "$lines" -gt 100 ]; then
    # Check separator format: must be exactly "// ============..." (60 =)
    bad_separators=$(grep "// ===" "$f" | grep -vc "// ============================================================" 2>/dev/null || echo 0)
    if [ "$bad_separators" -gt 0 ]; then
      echo "REFIX $name - $bad_separators section separators have wrong format (need 60 = characters)"
    fi
  fi
done
```

**Convention check:** Separators must use exactly 60 `=` characters. If any say REFIX, edit the file to fix the separator width.

### Step 4.4: Verify CLI Attributes (ckrv-cli only)

If `ckrv-cli` was modified, verify `long_about` and `after_help` follow the format:

<!-- turbo -->
```bash
# Check long_about has multi-paragraph structure (contains \n\n)
grep -A 3 "long_about" /apps/chakravarti-cli/crates/ckrv-cli/src/lib.rs | \
  grep -c "\\\\n\\\\n" || echo "⚠️ CHECK - Some long_about may be single-paragraph (should be multi-paragraph)"

# Check after_help starts with "Examples:\n"  
grep -A 1 "after_help" /apps/chakravarti-cli/crates/ckrv-cli/src/lib.rs | \
  grep -v "Examples:" && echo "⚠️ CHECK - Some after_help don't start with 'Examples:'" || true
```

**Convention check:**
- `long_about`: Must be multi-paragraph (separated by `\n\n`). First paragraph = expanded description. Second = what gets created/modified.
- `after_help`: Must start with `"Examples:\n"`. Must show 2-3 practical examples with `# comment` format.

If any fail, edit `lib.rs` to correct the format.

### Step 4.5: Build Check

<!-- turbo -->
```bash
cargo check --workspace 2>&1 | tail -5
```

Confirm no compilation errors were introduced by documentation changes. If errors appear, fix them before proceeding.

---

## Phase 5: Output Summary

After processing all crates, provide a combined summary **in the conversation** (no file generated):

```markdown
## Rust Documentation Applied

### Crates Processed
- ckrv-cli ✅ (5 files modified)
- ckrv-core ✅ (3 files modified)
- ckrv-git ✅ (no changes needed)
- ...

### Changes Made

| Crate | Module Docs Added | Pub API Docs Added | Sections Added | CLI Attrs Added |
|-------|:-:|:-:|:-:|:-:|
| ckrv-cli | 2 | 5 | 3 | 4 long_about, 2 after_help |
| ckrv-core | 1 | 3 | 2 | — |
| ... | ... | ... | ... | ... |

### Files Modified
- `ckrv-cli/src/lib.rs` - Added long_about for Plan, Run commands
- `ckrv-core/src/orchestrator.rs` - Added module docs, 3 pub fn docs
- `ckrv-core/src/runner.rs` - Added section separators
- ...

### Post-Edit Verification
- ✅ Module doc format: all correct
- ✅ Pub API doc structure: all correct
- ✅ Section separator format: all correct
- ✅ CLI attribute format: all correct
- ✅ `cargo check`: no errors

### ⚠️ Warnings (Manual Action Needed)
- `ckrv-core/src/executor.rs` (850 lines) - Consider splitting
- `ckrv-sandbox/src/agent.rs` (520 lines) - Consider splitting
```

---

## Notes

- **Detect-and-fix in one pass**: No separate analysis phase — issues are fixed as they're found
- **No report files**: Summary is printed in conversation, not saved to `health-report.md`
- **Documentation only**: Adds comments and docs, never changes code logic
- **Warnings for unfixable issues**: Large files that need splitting are warned about but not touched
- **Idempotent**: Safe to run multiple times — already-documented items are skipped

---

## Next Workflow

Read the **docs-order** skill to determine what workflow to run next based on what was changed.
