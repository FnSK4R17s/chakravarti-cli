---
description: Check documentation freshness and update crate docs to match current code.
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

Ensure crate documentation (`crates/<crate>/docs/README.md`) stays synchronized with the actual code. This workflow follows the process defined in `DOCUMENTATION.md`.

## Operating Constraints

- **Follow DOCUMENTATION.md**: All updates must follow the documentation system guidelines
- **Preserve frontmatter**: Always maintain the YAML frontmatter with `last_commit`, `last_updated`, and `related_files`
- **Accurate commits**: Update `last_commit` to the current HEAD commit hash
- **Semantic changes only**: Only update docs when there are meaningful API changes (not formatting/comments)

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

### 6. Update Documentation (if not check-only)

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

### 8. Summary Report

Output final summary:

```markdown
## Documentation Update Summary

### Updated
- `crates/ckrv-core/docs/README.md` - Added RunnerConfig GLM fields
- `crates/ckrv-cli/docs/README.md` - Updated agent_lookup description

### Skipped (no changes)
- `crates/ckrv-git/docs/README.md`
- `crates/ckrv-sandbox/docs/README.md`

### Next Steps
- Review updated docs for accuracy
- Commit with: `git commit -m "docs: update crate docs to <NEW_COMMIT>"`
```

## Root README.md

The root `README.md` must also be checked and updated when:

| Section | Check When |
|---------|------------|
| Commands table | New CLI commands added |
| Architecture section | Crates added/removed |
| Agents table | Agent types or availability changes |
| Requirements | Dependency version changes |

### Checking Root README

// turbo
```bash
# Check for changes that might affect README
git diff $LAST_KNOWN_COMMIT HEAD -- crates/ckrv-cli/src/main.rs crates/ckrv-cli/src/commands/mod.rs
```

### Updating Root README

When updating `README.md`:

1. **Commands table**: Sync with actual commands in `src/commands/mod.rs`
2. **Agents table**: Update availability (CLI only, UI only, CLI + UI)
3. **Architecture tree**: Ensure crate list is current
4. **Quick start**: Verify example commands still work

## Cross-Crate Documentation

If changes affect cross-crate docs (`crates/docs/`), also check:

| Doc | Check When |
|-----|------------|
| `architecture.md` | Crate dependencies change |
| `agent-guide.md` | Agent-related code changes (RunnerConfig, agent types) |
| `cli-commands.md` | Command options/behavior changes |
| `getting-started.md` | Setup requirements change |

### Cross-Crate Freshness Check

// turbo
```bash
# Check cross-crate docs
for doc in crates/docs/*.md; do
  commit=$(grep -oP '^last_commit: \K[a-f0-9]+' "$doc" 2>/dev/null || echo "none")
  echo "$doc: $commit"
done
```

## Operating Principles

### Context Efficiency

- **Incremental checks**: Only analyze crates with actual changes
- **API-focused**: Prioritize public API documentation over internal details
- **Semantic diffs**: Focus on meaningful changes, ignore formatting

### Documentation Quality

- **Code-first**: Documentation must reflect actual code behavior
- **Examples**: Include usage examples for new APIs
- **Tables**: Use tables for quick reference of types/options
- **Mermaid**: Use diagrams for architecture changes

### Commit Tracking

- **Always update `last_commit`**: Every doc update must include the new commit hash
- **Related files**: Update `related_files` if new source files become relevant
- **Date stamps**: Update `last_updated` to current date
