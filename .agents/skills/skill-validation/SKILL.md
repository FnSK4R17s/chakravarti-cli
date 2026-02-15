---
name: skill-validation
description: How to create, validate, and manage Agent Skills using the agentskills CLI. Use when creating new skills or validating existing ones.
license: MIT
compatibility: Requires uvx (uv) installed
metadata:
  author: FnSK4R17s
  version: "1.0"
---

# Skill Validation

This skill explains how to create, validate, and manage Agent Skills in this project.

## Prerequisites

- **uv** installed (`pip install uv` or `curl -LsSf https://astral.sh/uv/install.sh | sh`)
- Skills located in `.agent/skills/`

## Skill Structure

Every skill must follow this structure:

```
skill-name/
├── SKILL.md          # Required
├── scripts/          # Optional - helper scripts
├── references/       # Optional - detailed docs  
└── assets/           # Optional - templates, images
```

## SKILL.md Format

```yaml
---
name: skill-name                    # Required: lowercase, hyphens only (1-64 chars)
description: What it does and when  # Required: 1-1024 chars
license: MIT                        # Optional
compatibility: Claude Code          # Optional
metadata:
  author: your-name
  version: "1.0"
---

# Skill Title

Instructions for the AI agent...
```

### Name Rules

- ✅ `pdf-processing`, `data-analysis`, `code-review`
- ❌ `PDF-Processing` (no uppercase)
- ❌ `-pdf` (can't start with hyphen)
- ❌ `pdf--processing` (no consecutive hyphens)

## CLI Commands

### Validate a Skill

```bash
uvx --from skills-ref agentskills validate .agent/skills/skill-name
```

Output shows any validation errors:
```
✓ skill-name is valid
```

Or errors:
```
✗ name: must be lowercase alphanumeric with hyphens
✗ description: required field missing
```

### Read Skill Properties

```bash
uvx --from skills-ref agentskills read-properties .agent/skills/skill-name
```

Outputs JSON:
```json
{
  "name": "skill-name",
  "description": "What it does",
  "license": "MIT",
  "metadata": {"author": "name", "version": "1.0"}
}
```

### Generate Prompt XML

```bash
uvx --from skills-ref agentskills to-prompt .agent/skills/*
```

Generates XML for agent system prompts:
```xml
<available_skills>
<skill>
<name>skill-name</name>
<description>What it does</description>
<location>.agent/skills/skill-name/SKILL.md</location>
</skill>
</available_skills>
```

## Validate All Skills

```bash
for skill in .agent/skills/*/; do
  echo "Validating $skill..."
  uvx --from skills-ref agentskills validate "$skill"
done
```

## Creating a New Skill

1. Create the directory:
   ```bash
   mkdir -p .agent/skills/my-new-skill
   ```

2. Create SKILL.md with frontmatter:
   ```bash
   cat > .agent/skills/my-new-skill/SKILL.md << 'EOF'
   ---
   name: my-new-skill
   description: What this skill does and when to use it.
   metadata:
     version: "1.0"
   ---
   
   # My New Skill
   
   Instructions here...
   EOF
   ```

3. Validate:
   ```bash
   uvx --from skills-ref agentskills validate .agent/skills/my-new-skill
   ```

## Fetching Skills from GitHub

To install a skill from a GitHub repo:

```bash
# Create directory
mkdir -p .agent/skills/skill-name

# Fetch SKILL.md
curl -L https://raw.githubusercontent.com/owner/repo/main/skill-name/SKILL.md \
  -o .agent/skills/skill-name/SKILL.md

# Validate
uvx --from skills-ref agentskills validate .agent/skills/skill-name
```

## References

- [Agent Skills Specification](https://agentskills.io/specification)
- [Example Skills](https://github.com/anthropics/skills)
- [skills-ref Package](https://github.com/agentskills/agentskills/tree/main/skills-ref)
