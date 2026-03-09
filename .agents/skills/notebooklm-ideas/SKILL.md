---
name: notebooklm-ideas
description: Query the CKRV NotebookLM knowledge base for past research, references, and saved insights. Use when the user explicitly says "check notebook", "search notebooklm", "add to notebook", or "save to notebooklm". Do NOT use for brainstorming — use the brainstorming skill instead.
license: MIT
metadata:
  author: FnSK4R17s
  version: "1.1"
---

# NotebookLM Knowledge Base

This skill connects to a **Librarian Agent** — an AI-powered knowledge base hosted on NotebookLM. Think of it as asking a librarian who has read all your saved research, articles, and references. You ask questions, the librarian answers from what it knows.

The librarian is **read-first by default**. It retrieves and synthesizes — it doesn't capture or file things away unless you explicitly hand it something to shelve.

## How to interact

Frame queries as questions you'd ask a knowledgeable research assistant:

- "What do we know about Rust compilation bottlenecks?"
- "What did that article say about monomorphization?"
- "Do we have any references on Docker layer caching?"

The librarian will search across all saved sources and synthesize an answer with citations.

> **IMPORTANT: Read-only by default**
>
> NEVER write to this notebook (add sources, create notes) unless the user **explicitly** asks to shelve something. Phrases like "let's brainstorm", "I have an idea", or "let's explore this" do NOT mean "write to NotebookLM" — those should use the **brainstorming** skill instead.
>
> Only trigger writes on explicit requests like:
> - "save this to notebooklm"
> - "add this to the notebook"
> - "capture this in notebooklm"
> - "add this URL/file to the CKRV notebook"

> **IMPORTANT: Skill Boundaries**
>
> | Action | Correct Skill |
> |--------|---------------|
> | Brainstorm on a feature/idea | **brainstorming** (creates docs in `brainstorming/`) |
> | Plan implementation tasks | **brainstorm-to-tasks** or **speckit.tasks** |
> | Ask the librarian a question | **notebooklm-ideas** (this skill, read mode) |
> | Explicitly shelve a source | **notebooklm-ideas** (this skill, write mode) |

## Notebook

- **ID**: `d34a75b1-db04-414a-8b0e-8432437e3d71`
- **Title**: Chakravarti CLI (CKRV)
- **URL**: https://notebooklm.google.com/notebook/d34a75b1-db04-414a-8b0e-8432437e3d71

## Ask the Librarian (default)

### Query the knowledge base

Ask a question — the librarian searches all saved sources and returns a synthesized answer with citations:

```
Use mcp__notebooklm__notebook_query with:
  notebook_id: d34a75b1-db04-414a-8b0e-8432437e3d71
  query: <search terms>
```

### Browse the catalog

List everything the librarian has on file:

```
Use mcp__notebooklm__source_list_drive with:
  notebook_id: d34a75b1-db04-414a-8b0e-8432437e3d71
```

## Shelve New Material (explicit request only)

Only use these when the user explicitly asks to save/add something to the library.

### Add a text source

```
Use mcp__notebooklm__source_add with:
  notebook_id: d34a75b1-db04-414a-8b0e-8432437e3d71
  source_type: text
  text: <content>
  title: <title>
```

**Format for text ideas:**

```markdown
# <Title>

**Date**: YYYY-MM-DD
**Tags**: <comma-separated: agent, ui, dx, architecture, integration, distribution, sandbox, git, metrics>
**Status**: seed | exploring | ready-for-issue

## Description
<Content>

## Why
<Reasoning>

## Open Questions
- <Unknowns>
```

### Add a URL source

```
Use mcp__notebooklm__source_add with:
  notebook_id: d34a75b1-db04-414a-8b0e-8432437e3d71
  source_type: url
  url: <the URL>
```

### Add a file from the repo

```
Use mcp__notebooklm__source_add with:
  notebook_id: d34a75b1-db04-414a-8b0e-8432437e3d71
  source_type: file
  file_path: <absolute path to file>
```

### Create a note (curated summary)

```
Use mcp__notebooklm__note with:
  notebook_id: d34a75b1-db04-414a-8b0e-8432437e3d71
  action: create
  title: <note title>
  content: <synthesized content>
```
