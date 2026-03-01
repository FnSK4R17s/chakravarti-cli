---
command: spec new
generated_from: commands/spec.rs
last_commit: f92f604
---

# ckrv spec new

Create a new specification using AI from a natural language description.

## Description

Create a new feature specification from a natural language description.

Generates a structured spec.md file in the specs/ directory containing:
- Feature overview and goals
- Acceptance criteria
- Technical requirements and constraints

A short name is auto-generated from the description if not provided.

Requires an active AI provider configuration. The AI may ask clarifying questions if the description is ambiguous.

## Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<description>` | Yes | Natural language description of the feature |

## Options

| Flag | Description |
|------|-------------|
| `--name`, `-n` | Optional short name for the spec (auto-generated if not provided) |

## Examples

```bash
# Create a spec from an inline description
ckrv spec new "Add user authentication with OAuth2"

# Create a spec with an explicit short name
ckrv spec new "Add user authentication" --name auth-oauth2

# Create a spec with a detailed multi-word description
ckrv spec new "Implement rate limiting for the public API endpoints"
```
