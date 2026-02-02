---
description: Update the main README.md based on guiding documents and generated crate docs.
---

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty). The user may specify:
- `--check-only` to only report what's outdated without updating
- `--section <name>` to update only a specific section (e.g., `commands`, `agents`)
- Specific feedback about what needs changing

## Goal

Keep the main `README.md` aligned with:
1. **Guiding documents** (`guiding_docs/vision.md`) - for messaging, positioning, and value proposition
2. **Generated crate docs** (`crates/*/docs/README.md`) - for technical accuracy
3. **CLI command docs** (`crates/docs/cli-commands.md`) - for command reference

## Source of Truth Hierarchy

| Priority | Source | Used For |
|----------|--------|----------|
| 1 | `guiding_docs/vision.md` | Tagline, value prop, target user, product description |
| 2 | `crates/docs/cli-commands.md` | Command table, usage examples |
| 3 | `crates/*/docs/README.md` | Architecture, crate descriptions |
| 4 | `crates/ckrv-cli/src/lib.rs` | Command list (Commands enum) |

---

## Execution Steps

### 1. Read Guiding Documents

// turbo
```bash
cat guiding_docs/vision.md | head -100
```

Extract key messaging elements:
- **Tagline**: From "What Is Chakravarti?" section
- **Value proposition**: The wedge ("use all your AI subscriptions together")
- **Target user**: From "Who It's For" section
- **Product principles**: From "Product Principles" section

### 2. Read Current README

// turbo
```bash
cat README.md
```

Identify sections that need alignment:
- Header/tagline (lines 1-13)
- Introduction (line 17-20)
- Quick Start (lines 32-54)
- Commands table (lines 60-81)
- Architecture (lines 96-112)
- Agents (lines 114-139)

### 3. Read Generated Docs

// turbo
```bash
# CLI commands reference
cat crates/docs/cli-commands.md 2>/dev/null | head -80

# Architecture
cat crates/docs/architecture.md 2>/dev/null | head -50
```

### 4. Check Commands Enum for Accuracy

// turbo
```bash
# Get actual commands from source
grep -E "^\s+[A-Z][a-zA-Z]+\(" crates/ckrv-cli/src/lib.rs | head -30
```

Compare against README command table for missing/outdated entries.

### 4.5 Check GitHub Issues for Agent Integrations

Use the github-issues skill to find any planned CLI/tool integrations:

// turbo
```bash
# Fetch all open issues (uses inline repo extraction)
curl -s "https://api.github.com/repos/$(git remote get-url origin | sed 's|.*github.com[:/]||; s|\.git$||')/issues?state=open&per_page=100" | \
  jq -r '.[] | "#\(.number) \(.title)"'
```

// turbo
```bash
# Extract current Future Integrations from README
grep -A 15 "### Future Integrations" README.md | grep "^\- \*\*" | sed 's/.*\*\*\([^*]*\)\*\*.*/\1/'
```

**Compare the two lists:**

| Check | Action |
|-------|--------|
| Issue exists but NOT in README Future Integrations | ⚠️ Add to README with issue link |
| README lists integration but NO GitHub issue | ⚠️ Prompt user to create issue |

**Output discrepancy table if found:**

```markdown
### Agent Integration Discrepancies

#### Missing from README (have GitHub issues)
| Tool | Issue | Action |
|------|-------|--------|
| Mistral Code | #29 | Add to Future Integrations |

#### Missing GitHub Issues (in README but no tracking)
| Tool | Suggested Issue Title | Suggested Description |
|------|----------------------|----------------------|
| Gemini CLI | Add Gemini CLI integration | Integrate Google's Gemini CLI as an agent provider |
| Cursor CLI | Add Cursor CLI integration | Integrate Cursor's AI coding assistant as an agent |
| ... | ... | ... |
```

**Prompt user:** "Would you like me to create GitHub issues for these untracked integrations?"

### 5. Generate Alignment Report

If `--check-only` was specified, output report and stop:

```markdown
## README Alignment Report

### Messaging Alignment (vs vision.md)
| Section | Current | Should Be | Status |
|---------|---------|-----------|--------|
| Tagline | "Spec-driven Agent Orchestration" | "Cross-platform orchestration engine for AI coding agents" | ⚠️ Needs update |
| Value prop | Generic productivity | Multi-provider wedge | ⚠️ Needs update |

### Technical Accuracy (vs crate docs)
| Section | Status | Notes |
|---------|--------|-------|
| Commands table | ✅ Up to date | All 15 commands listed |
| Architecture | ⚠️ Outdated | Missing ckrv-mcp crate |
| Agents | ⚠️ Outdated | Add new integrations |

### Recommended Actions
1. Update tagline to match vision.md
2. Add ckrv-mcp to architecture diagram
3. Update agents section with latest supported models
```

### 6. Update README Sections

For each section needing updates:

#### 6.1 Header/Tagline (align with vision.md)

Update lines 1-7 to reflect:
- Main tagline from vision "What Is Chakravarti?" 
- Sub-tagline emphasizing multi-provider orchestration
- The "Finally someone built this" positioning

**Format:**
```html
<h1 align="center">Chakravarti</h1>

<p align="center">
  <strong>Cross-Platform Orchestration Engine for AI Coding Agents</strong><br>
  <em>Use all your AI coding subscriptions together.</em><br>
  <sub>You write specs. Your agents implement them. Together.</sub>
</p>
```

#### 6.2 Introduction (align with vision.md)

Update intro paragraph to include:
- The multi-provider pain point (from vision "What Is Chakravarti?")
- Fire-and-forget paradigm (from vision "Product Principles")
- Git-native workflow mention

#### 6.3 Commands Table (align with cli-commands.md)

Sync the commands table with `crates/docs/cli-commands.md`:
- Add any missing commands
- Update descriptions to match
- Remove deprecated commands

#### 6.4 Architecture (align with crate docs)

Update architecture section:
- Ensure all crates in `crates/` are listed
- Update descriptions from crate READMEs
- Remove ⚠️ unused markers if crates are now active

#### 6.5 Agents (align with supported agents + GitHub issues)

**Step 1:** Check `crates/ckrv-sandbox/src/` for current agent support:
- Update "Currently Supported" table

**Step 2:** Sync "Future Integrations" with GitHub issues (from step 4.5):
- Add any integrations that have GitHub issues but aren't listed
- Include issue links: `([#29](https://github.com/.../issues/29))`
- For integrations listed without issues, output a table for user to create issues

### 7. Preserve README Structure

**DO NOT change:**
- Badge links (lines 10-12)
- Screenshot paths
- License section
- External links (DeepWiki, etc.)

**DO update:**
- Textual content to match sources
- Command examples to current syntax
- Architecture diagram if crates changed

### 8. Validate Updates

// turbo
```bash
# Check markdown renders correctly
head -30 README.md

# Verify no broken links
grep -oE '\[.*\]\(.*\)' README.md | head -20
```

---

## Section Mapping

| README Section | Primary Source | Fallback Source |
|----------------|----------------|-----------------|
| Header/Tagline | `guiding_docs/vision.md` → "What Is Chakravarti?" | - |
| Introduction | `guiding_docs/vision.md` → "How It Works" | - |
| Quick Start | `crates/docs/cli-commands.md` | `ckrv --help` output |
| Commands | `crates/docs/cli-commands.md` | `lib.rs` Commands enum |
| Architecture | `crates/docs/architecture.md` | Crate READMEs |
| Agents | `crates/ckrv-sandbox/docs/README.md` | Agent provider files |
| Development | `crates/docs/getting-started.md` | - |

---

## Summary Report

After updates, output:

```markdown
## README Update Summary

### Sections Updated
- ✅ Header/Tagline - Aligned with vision.md messaging
- ✅ Introduction - Added multi-provider positioning  
- ✅ Commands - Added `mcp` command (was missing)
- ⏭️ Agents - No changes needed

### Sources Used
- guiding_docs/vision.md (messaging)
- crates/docs/cli-commands.md (commands)
- crates/ckrv-cli/src/lib.rs (command verification)

### Next Steps
1. Review changes: `git diff README.md`
2. Commit with: `git commit -m "docs: align README with vision and generated docs"`
```
