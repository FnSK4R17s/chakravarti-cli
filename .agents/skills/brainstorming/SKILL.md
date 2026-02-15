---
name: brainstorming
description: Create and manage brainstorming documents for GitHub issues. Use when exploring ideas, planning features, or documenting decisions before creating formal specs.
license: MIT
compatibility: Requires curl and basic shell
metadata:
  author: FnSK4R17s
  version: "1.1"
---

# Brainstorming

Create and manage brainstorming documents linked to GitHub issues. Brainstorming docs are **ephemeral** - they become formal specs when ready for implementation, then can be discarded.

## Guiding Documents

Unlike brainstorming, **guiding docs** are permanent reference material that shape all project decisions:

| Location | Purpose |
|----------|---------|
| `guiding_docs/vision.md` | Product vision, principles, non-goals - the north star |

**Always read `guiding_docs/vision.md` first** before starting any brainstorm. New features should align with the vision's:
- Target user (solo founders, senior ICs with 2+ AI subscriptions)
- Product principles (spec-first, fire-and-forget, isolation, git-native)
- Non-goals (not another coding agent - orchestration layer only)

When brainstorming produces lasting insights, promote them to guiding docs rather than keeping them in `brainstorming/`.

## Overview

```
GitHub Issue #12 ←→ brainstorming/issue-012-npm-package/
                         ├── notes.md      (brainstorm doc from .templates/brainstorm.notes.md)
                         ├── tasks.md      (optional, from .templates/brainstorm.tasks.md)
                         └── research.md   (optional extras)
```

## Quick Start

### Create New Brainstorm from Issue

```bash
# 1. Get issue details
ISSUE_NUM=12
REPO=$(git remote get-url origin | sed 's/.*github.com[:/]\(.*\)\.git/\1/')
ISSUE=$(curl -s "https://api.github.com/repos/$REPO/issues/$ISSUE_NUM")
TITLE=$(echo "$ISSUE" | jq -r '.title' | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9]/-/g' | sed 's/--*/-/g')

# 2. Create from template
FOLDER="brainstorming/issue-$(printf '%03d' $ISSUE_NUM)-${TITLE:0:30}"
mkdir -p "$FOLDER"
cp .templates/brainstorm.notes.md "$FOLDER/notes.md"

# 3. Fill in placeholders
sed -i "s/{{ISSUE_NUMBER}}/$ISSUE_NUM/g" "$FOLDER/notes.md"
sed -i "s/{{TITLE}}/$(echo "$ISSUE" | jq -r '.title')/g" "$FOLDER/notes.md"
sed -i "s/{{DATE}}/$(date +%Y-%m-%d)/g" "$FOLDER/notes.md"

echo "Created: $FOLDER/notes.md"
```

### One-Liner

```bash
ISSUE_NUM=12 && mkdir -p "brainstorming/issue-$(printf '%03d' $ISSUE_NUM)" && cp .templates/brainstorm.notes.md "brainstorming/issue-$(printf '%03d' $ISSUE_NUM)/notes.md" && sed -i "s/{{ISSUE_NUMBER}}/$ISSUE_NUM/g; s/{{DATE}}/$(date +%Y-%m-%d)/g" "brainstorming/issue-$(printf '%03d' $ISSUE_NUM)/notes.md"
```

### Add Tasks (Optional)

When you need to break down implementation:

```bash
ISSUE_NUM=12
FOLDER=$(ls -d brainstorming/issue-$(printf '%03d' $ISSUE_NUM)-* 2>/dev/null | head -1)
cp .templates/brainstorm.tasks.md "$FOLDER/tasks.md"
sed -i "s/{{ISSUE_NUMBER}}/$ISSUE_NUM/g; s/{{DATE}}/$(date +%Y-%m-%d)/g" "$FOLDER/tasks.md"
echo "Created: $FOLDER/tasks.md"
```

## Commands

### List All Brainstorms

```bash
ls -la brainstorming/
```

### Find Brainstorm for Issue

```bash
# By issue number
ls brainstorming/ | grep "issue-012"

# Or search content
grep -r "#12" brainstorming/ --include="*.md"
```

### Check Brainstorm Status

```bash
# Find all brainstorms and their status
grep -h "^\\*\\*Status\\*\\*:" brainstorming/*/notes.md 2>/dev/null
```

### List Issues with Brainstorms

```bash
# Show which issues have brainstorming docs
for dir in brainstorming/issue-*/; do
  num=$(echo "$dir" | grep -oE '[0-9]+' | head -1)
  title=$(head -1 "$dir/notes.md" | sed 's/# //')
  echo "#$num: $title"
done
```

## Folder Naming Convention

```
brainstorming/
├── issue-004-headless-operation/      # Linked to #4
├── issue-012-npm-package/             # Linked to #12
├── issue-023-mcp-server/              # Linked to #23
└── npm-package/                       # Legacy (no issue link)
```

**Format**: `issue-{NNN}-{slug}/`
- `NNN`: Zero-padded issue number (for sorting)
- `slug`: Lowercase, hyphenated title excerpt

## Template Placeholders

| Placeholder | Replaced With |
|-------------|---------------|
| `{{ISSUE_NUMBER}}` | GitHub issue number |
| `{{TITLE}}` | Issue title |
| `{{DATE}}` | Creation date (YYYY-MM-DD) |

## Workflow

```
1. Issue Created (#12)
        ↓
2. Brainstorm Created (brainstorming/issue-012-*)
        ↓
3. Ideas Explored, Decisions Made
        ↓
4. Status → "Ready for Spec"
        ↓
5. /speckit.specify (creates specs/018-*)
        ↓
6. Brainstorm Status → "Archived"
```

## Integration with GitHub Issues Skill

After fetching issues, check for existing brainstorms:

```bash
# Fetch open issues
curl -s "https://api.github.com/repos/$REPO/issues?state=open&per_page=20" | \
  jq -r '.[] | "#\(.number) \(.title)"' | \
  while read line; do
    num=$(echo "$line" | grep -oE '^#[0-9]+' | tr -d '#')
    if ls brainstorming/issue-$(printf '%03d' $num)-* 2>/dev/null | grep -q .; then
      echo "$line 📝"
    else
      echo "$line"
    fi
  done
```

Output:
```
#29 Add Mistral code
#12 Create NPM package for quick global install 📝
#11 Build cloud integration
```

## Best Practices

1. **One brainstorm per issue** - Keep them linked
2. **Update status** - Draft → In Progress → Ready for Spec → Archived
3. **Link to issue** - Always include the issue link in frontmatter
4. **Archive don't delete** - Mark as archived when spec is created

## Templates

| Template | Purpose |
|----------|--------|
| `.templates/brainstorm.notes.md` | Main brainstorm document - ideas, decisions, approach |
| `.templates/brainstorm.tasks.md` | Task breakdown - phases, estimates, acceptance criteria |

Customize them for your project's needs.
