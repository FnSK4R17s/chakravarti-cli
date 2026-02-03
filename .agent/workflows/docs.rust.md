---
description: Analyze Rust crate documentation and apply conventions without changing code logic.
---

# Rust Documentation & Convention Analysis

## User Input

```
$ARGUMENTS
```

**Optional:** `--crate <name>` to focus on a specific crate (e.g., `ckrv-core`).
By default, runs on **all crates**.

## Goal

Analyze all Rust crates for documentation convention compliance and apply missing documentation.

**This workflow:**
- ✅ Adds documentation comments (`///`, `//!`, `//`)
- ✅ Adds section separators (`// ===...`)
- ✅ Reports code issues (but does NOT fix them)
- ❌ Does NOT modify code logic
- ❌ Does NOT refactor

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

By default, analyze **all** crates. Use `--crate <name>` to focus on one.

// turbo
```bash
# List all crates (or filter if --crate specified)
ls -d /apps/chakravarti-cli/crates/ckrv-* | xargs -I{} basename {}
```

Store the list:
```
CRATES=(
  ckrv-cli
  ckrv-core
  ckrv-git
  ckrv-integrations
  ckrv-mcp
  ckrv-metrics
  ckrv-model
  ckrv-sandbox
  ckrv-spec
  ckrv-ui
  ckrv-verify
)
```

If `--crate <name>` was specified, filter to just that crate.

### Step 1.2: Loop Structure

For each crate in the list:
1. Run Phase 2 (Crate Analysis)
2. Run Phase 3 if crate is `ckrv-cli` (CLI-specific)
3. Generate per-crate health report (Phase 4)
4. Apply documentation if `--apply` (Phase 5)

Then generate a combined summary (Phase 6).

---

## Phase 2: Crate Analysis (Per Crate)

**Run this phase for each crate in the list.**

Set the current crate:
```bash
CRATE="<current_crate_from_loop>"  # e.g., ckrv-core
```

### Step 2.1: File Inventory

// turbo
```bash
find /apps/chakravarti-cli/crates/$CRATE/src -name "*.rs" 2>/dev/null | head -50
```

### Step 2.2: File Size Analysis

// turbo
```bash
for f in $(find /apps/chakravarti-cli/crates/$CRATE/src -name "*.rs" 2>/dev/null); do
  lines=$(wc -l < "$f")
  echo "$lines $(basename $f)"
done | sort -rn | head -20
```

**Thresholds:**
| Lines | Rating | Action |
|-------|--------|--------|
| < 300 | ✅ Good | Maintain |
| 300-500 | ⚠️ Monitor | Consider splitting |
| 500-800 | 🟠 Warning | Plan to split |
| > 800 | 🔴 Critical | Report as issue |

### Step 2.3: Module Documentation Check

// turbo
```bash
for f in $(find /apps/chakravarti-cli/crates/$CRATE/src -name "*.rs" 2>/dev/null); do
  name=$(basename "$f")
  # Check for //! module docs in first 10 lines
  if head -10 "$f" | grep -q "^//!"; then
    echo "✅ $name"
  else
    echo "❌ $name - Missing module docs (//!)"
  fi
done
```

### Step 2.4: Public API Documentation Check

// turbo
```bash
for f in $(find /apps/chakravarti-cli/crates/$CRATE/src -name "*.rs" 2>/dev/null); do
  name=$(basename "$f")
  
  # Count pub items
  pub_count=$(grep -c "^pub " "$f" 2>/dev/null || echo 0)
  
  # Count documented pub items (/// before pub)
  doc_count=$(grep -B1 "^pub " "$f" | grep -c "^///" 2>/dev/null || echo 0)
  
  if [ "$pub_count" -gt 0 ]; then
    pct=$((doc_count * 100 / pub_count))
    if [ "$pct" -lt 50 ]; then
      echo "❌ $name: $doc_count/$pub_count pub items documented ($pct%)"
    elif [ "$pct" -lt 100 ]; then
      echo "⚠️  $name: $doc_count/$pub_count pub items documented ($pct%)"
    else
      echo "✅ $name: All $pub_count pub items documented"
    fi
  fi
done
```

### Step 2.5: Section Separator Check

// turbo
```bash
for f in $(find /apps/chakravarti-cli/crates/$CRATE/src -name "*.rs" 2>/dev/null); do
  name=$(basename "$f")
  lines=$(wc -l < "$f")
  
  # Only check files > 100 lines
  if [ "$lines" -gt 100 ]; then
    sections=$(grep -c "// ===" "$f" 2>/dev/null || echo 0)
    if [ "$sections" -lt 2 ]; then
      echo "⚠️  $name ($lines lines): Missing section separators (found $sections)"
    else
      echo "✅ $name: $sections sections"
    fi
  fi
done
```

### Step 2.6: Import Organization Check

// turbo
```bash
for f in $(find /apps/chakravarti-cli/crates/$CRATE/src -name "*.rs" 2>/dev/null); do
  name=$(basename "$f")
  
  # Check if std imports come first
  first_import=$(grep "^use " "$f" | head -1)
  if echo "$first_import" | grep -q "^use std::"; then
    echo "✅ $name: std imports first"
  elif [ -n "$first_import" ]; then
    echo "⚠️  $name: std imports should come first"
  fi
done
```

---

## Phase 3: CLI Command Analysis (when $CRATE is ckrv-cli)

### Step 3.1: Find Commands Enum

// turbo
```bash
grep -n "pub enum Commands" /apps/chakravarti-cli/crates/ckrv-cli/src/lib.rs
```

### Step 3.2: Check long_about Coverage

// turbo
```bash
# Count commands in enum
commands=$(grep -A 100 "pub enum Commands" /apps/chakravarti-cli/crates/ckrv-cli/src/lib.rs | \
           grep -E "^\s+[A-Z][a-zA-Z]+\(" | wc -l)

# Count long_about attributes
long_about=$(grep -c "long_about" /apps/chakravarti-cli/crates/ckrv-cli/src/lib.rs)

echo "Commands: $commands"
echo "long_about: $long_about"

if [ "$long_about" -lt "$commands" ]; then
  echo "❌ Missing long_about for $((commands - long_about)) commands"
else
  echo "✅ All commands have long_about"
fi
```

### Step 3.3: Check after_help Coverage

// turbo
```bash
after_help=$(grep -c "after_help" /apps/chakravarti-cli/crates/ckrv-cli/src/lib.rs)
commands=$(grep -A 100 "pub enum Commands" /apps/chakravarti-cli/crates/ckrv-cli/src/lib.rs | \
           grep -E "^\s+[A-Z][a-zA-Z]+\(" | wc -l)

echo "after_help: $after_help"

if [ "$after_help" -lt "$commands" ]; then
  echo "❌ Missing after_help for $((commands - after_help)) commands"
else
  echo "✅ All commands have after_help"
fi
```

### Step 3.4: List Commands Without Documentation

// turbo
```bash
# Extract command names and check for long_about
grep -B 10 "^\s*[A-Z][a-zA-Z]*(" /apps/chakravarti-cli/crates/ckrv-cli/src/lib.rs | \
awk '
  /long_about/ { has_long=1 }
  /after_help/ { has_after=1 }
  /^\s+[A-Z][a-zA-Z]+\(/ {
    cmd=$1
    gsub(/\(.*/, "", cmd)
    gsub(/^\s+/, "", cmd)
    if (!has_long) print "❌ " cmd ": missing long_about"
    if (!has_after) print "❌ " cmd ": missing after_help"
    has_long=0; has_after=0
  }
'
```

---

## Phase 4: Generate Health Report

Create a comprehensive report at `crates/<crate>/docs/health-report.md`:

```markdown
# <Crate> Documentation Health Report

Generated: <CURRENT_DATE>
Commit: <CURRENT_COMMIT>

## Summary

| Metric | Value | Status |
|--------|-------|--------|
| Total Files | X | - |
| Total Lines | X | - |
| Files > 500 LOC | X | 🔴 if > 0 |
| Module Docs Coverage | X% | ⚠️ if < 100% |
| Public API Docs Coverage | X% | ⚠️ if < 80% |
| Section Separators | X files need | ⚠️ |

## File Analysis

| File | Lines | Module Doc | Pub Docs | Sections | Status |
|------|-------|------------|----------|----------|--------|
| lib.rs | 450 | ✅ | 80% | 5 | ⚠️ |
| orchestrator.rs | 320 | ❌ | 60% | 2 | 🔴 |
| ... | ... | ... | ... | ... | ... |

## CLI Commands (if ckrv-cli)

| Command | long_about | after_help | Status |
|---------|------------|------------|--------|
| init | ✅ | ✅ | ✅ |
| spec | ✅ | ❌ | ⚠️ |
| plan | ❌ | ❌ | 🔴 |
| ... | ... | ... | ... |

## Code Issues (Report Only)

These are issues found but NOT fixed by this workflow:

| File | Line | Issue | Severity |
|------|------|-------|----------|
| executor.rs | - | 850 lines, needs splitting | 🔴 |
| agent.rs | 45 | Unused import | ⚠️ |
| ... | ... | ... | ... |

## Recommended Priority

### P0: Critical (Fix First)
1. Add module docs to `orchestrator.rs`
2. Add `long_about` to `plan` command
3. Split `executor.rs` (850 lines)

### P1: Important
1. Add section separators to files > 200 lines
2. Document remaining public APIs in `lib.rs`

### P2: Nice to Have
1. Organize imports in std-first order
2. Add examples to key functions
```

---

## Phase 5: Apply Documentation

**Rule:** Only add documentation. Never change code logic.

### Step 5.1: Add Module Docs

For each file missing `//!` module docs:

1. Read the file to understand its purpose
2. Add module documentation header:

```rust
//! # <ModuleName>
//!
//! <Brief description inferred from file contents>
//!
//! ## Overview
//!
//! <What this module does and how it fits in>
//!
//! ## Key Types
//!
//! - [`TypeName`] - Description
//!
//! ## Example
//!
//! ```rust
//! // Example usage if applicable
//! ```
```

### Step 5.2: Add Section Separators

For files > 100 lines without sections:

```rust
// ============================================================
// IMPORTS
// ============================================================

// ... imports ...

// ============================================================
// TYPES
// ============================================================

// ... types ...

// ============================================================
// IMPLEMENTATION
// ============================================================

// ... impl blocks ...
```

### Step 5.3: Add Public API Docs

For undocumented `pub` items, add stub documentation:

```rust
/// TODO: Document this function.
///
/// # Arguments
///
/// * `param` - Description needed
///
/// # Returns
///
/// Description needed
pub fn undocumented_fn(param: Type) -> Result<Output> {
```

### Step 5.4: Add CLI Command Docs (ckrv-cli only)

For commands missing `long_about` or `after_help`:

```rust
/// <Short description>
#[command(
    display_order = X,
    long_about = "TODO: Add detailed description.\n\n\
                  Explain what this command does.",
    after_help = "Examples:\n\
                  # TODO: Add examples\n\
                  ckrv <cmd>"
)]
CommandName(args::CmdArgs),
```

---

## Phase 6: Output Summary

After processing all crates, provide a combined summary:

```markdown
## Rust Documentation Analysis Complete

### Crates Processed
- ckrv-cli ✅
- ckrv-core ✅
- ckrv-git ✅
- ...

### Overall Statistics

| Crate | Files | Module Docs | Pub API Docs | CLI Cmds |
|-------|-------|-------------|--------------|----------|
| ckrv-cli | 12 | 100% | 85% | 15/15 |
| ckrv-core | 8 | 100% | 80% | - |
| ... | ... | ... | ... | ... |

### Files Modified
- `ckrv-cli/lib.rs` - Added module docs
- `ckrv-core/orchestrator.rs` - Added section separators
- ...

### Code Issues (Reported, Not Fixed)
1. 🔴 `ckrv-core/executor.rs` (850 lines) - Needs splitting
2. ⚠️ `ckrv-cli/agent.rs:45` - Unused import

### Health Reports Generated
📋 `crates/ckrv-cli/docs/health-report.md`
📋 `crates/ckrv-core/docs/health-report.md`
...

### Next Steps
1. Review added documentation for accuracy
2. Fix reported code issues manually
3. Run `cargo doc --open` to verify
4. Run `cargo clippy` to check for warnings
```

---

## Notes

- **Documentation only**: Adds comments and docs, never changes code logic
- **Reports issues**: Code problems are reported but not fixed (manual review required)
- **Idempotent**: Safe to run multiple times
- **Goal**: LLM-friendly code where each file is self-contained context

---

## Next Workflow

Read the **docs-order** skill to determine what workflow to run next based on what was changed.

