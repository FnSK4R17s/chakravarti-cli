# Quickstart: AI-Powered Spec Workflow

**Feature**: 001-spec-setup
**Date**: 2026-01-13

## Overview

The enhanced `ckrv spec` command uses Claude Code to generate comprehensive specifications following the spec-kit workflow: **specify → clarify → design → tasks**.

**File Naming**:
- `spec.yaml` - Main specification (YAML format, renders on UI)
- `design.md` - Technical design document
- `plan.yaml` - Execution plan (existing, for task orchestration)

## Quick Start (2 minutes)

### 1. Create a New Spec

```bash
# From your project root
ckrv spec new "Add user authentication with OAuth2"
```

**Output**:
- Creates `.specs/001-user-auth/spec.md` with detailed specification
- Creates and switches to git branch `001-user-auth`
- Generates user stories, requirements, success criteria

### 2. Review and Clarify

```bash
# If spec has [NEEDS CLARIFICATION] markers
ckrv spec clarify
```

**Interactive prompts**:
```
Q1: Authentication Method
  A) OAuth2 only
  B) OAuth2 + email/password
  C) Custom

Your choice: B

✓ Updated spec.md with your selection
```

### 3. Generate Tasks

```bash
# Generate implementation tasks
ckrv spec tasks
```

**Output**:
- Creates `.specs/001-user-auth/tasks.yaml`
- Each task has complexity, model tier, and dependencies

### 4. Start Implementing

```bash
# Run a specific task
ckrv task T001
```

## Command Reference

| Command | Purpose | When to Use |
|---------|---------|-------------|
| `ckrv spec new "desc"` | Generate spec from description | Starting a new feature |
| `ckrv spec clarify` | Resolve ambiguities | After `spec new` if markers present |
| `ckrv spec design` | Generate technical design | Before task generation (optional) |
| `ckrv spec tasks` | Generate implementation tasks | After spec is ready |
| `ckrv spec validate` | Check spec quality | Before planning |
| `ckrv spec list` | List all specs | See available specs |

## Workflow Phases

```
┌──────────────────────────────────────────────────────────────┐
│                     SPECIFY (spec new)                       │
│  Input: Natural language description                        │
│  Output: spec.md with user stories, requirements            │
└─────────────────────────┬────────────────────────────────────┘
                          │
                          ▼
┌──────────────────────────────────────────────────────────────┐
│                    CLARIFY (spec clarify)                    │
│  Input: spec.md with [NEEDS CLARIFICATION] markers          │
│  Output: Updated spec.md with resolved values               │
│  (Skip if no markers)                                       │
└─────────────────────────┬────────────────────────────────────┘
                          │
                          ▼
┌──────────────────────────────────────────────────────────────┐
│                    DESIGN (spec design)                      │
│  Input: spec.yaml                                           │
│  Output: research.md, design.md, data-model.md              │
│  (Optional - useful for complex features)                   │
│  NOTE: design.md is NOT plan.yaml (execution plan)          │
└─────────────────────────┤────────────────────────────────────┘
                          │
                          ▼
┌──────────────────────────────────────────────────────────────┐
│                     TASKS (spec tasks)                       │
│  Input: spec.md (and plan.md if exists)                     │
│  Output: tasks.yaml with actionable tasks                   │
└─────────────────────────┬────────────────────────────────────┘
                          │
                          ▼
┌──────────────────────────────────────────────────────────────┐
│                   IMPLEMENT (task run)                       │
│  Input: tasks.yaml, source files                            │
│  Output: Code changes, completed tasks                      │
└──────────────────────────────────────────────────────────────┘
```

## Output Files

After running the full workflow:

```
.specs/001-feature-name/
├── spec.yaml            # Feature specification (YAML - renders on UI)
├── research.md          # Technical decisions (from design)
├── design.md            # Technical design (from design command)
├── data-model.md        # Entity definitions (from design)
├── tasks.yaml           # Implementation tasks
├── plan.yaml            # Execution plan (for ckrv run)
└── checklists/
    └── requirements.md  # Quality validation
```

**Note**: `design.md` is a technical design document. `plan.yaml` is the execution plan used by `ckrv run` for task orchestration.

## Tips

### Better Specs

1. **Be specific**: "Add user auth" → "Add OAuth2 authentication with Google and GitHub providers"
2. **Include constraints**: "...with rate limiting and session timeout after 30 minutes"
3. **Mention scale**: "...supporting 10,000 concurrent users"

### Faster Iteration

```bash
# Regenerate tasks with force flag
ckrv spec tasks --force

# Skip clarify if you're confident
ckrv spec tasks  # works even with markers
```

### JSON Output

```bash
# For scripting/automation
ckrv spec new "feature" --json
ckrv spec list --json
```

## Troubleshooting

### "Claude Code not available"
- Ensure Claude Code is installed: `which claude`
- Check Docker is running: `docker ps`
- Try: `ckrv spec new "test" --verbose`

### "Spec already exists"
- Use a different name: `ckrv spec new "..." --name different-name`
- Or delete existing: `rm -rf .specs/001-old-spec`

### "Tasks validation failed"
- Review spec for completeness: `cat .specs/*/spec.md`
- Regenerate: `ckrv spec tasks --force`
