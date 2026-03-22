---
description: Research a topic and produce a whitepaper with citations
---

# Research to Whitepaper Workflow

This workflow combines internet research, citation management, and whitepaper writing into a single end-to-end process.

## Prerequisites

Before starting, you need:
- **TOPIC**: Name of the topic (used for folder and file names)
- **Description**: A longer description of the topic of interest
- **Purpose** (optional): What the user wants to accomplish with this research

## Skills Referenced

| Skill | Path | Purpose |
|-------|------|---------|
| Internet Research | `.agents/skills/beary/skills/internet-research/SKILL.md` | Research execution steps |
| References | `.agents/skills/beary/skills/references/SKILL.md` | Citation format rules |
| Whitepaper Writing | `.agents/skills/beary/skills/whitepaper-writing/SKILL.md` | Whitepaper writing steps |
| User Context Template | `.agents/skills/beary/skills/user-context-template/SKILL.md` | Create or refresh `.beary/USER.md` template |

---

## Workflow Steps

### 0. Initial Setup
 
Before beginning research, confirm the following with the user:

**0.1 Purpose (if not provided)**
If the user did not provide a purpose, ask:
*"Would you like to provide a purpose for this research? This helps focus the questions and narrative. You can skip this if you prefer broad exploration."*
 
**0.2 Research Mode Selection**
Read the `RESEARCH_MODE` value from `.beary/USER.md` (the `<!-- RESEARCH_MODE: ... -->` comment). If a valid default exists (hibernation or hyperphagia), use it and inform the user which mode is active, offering to override:
*"Using your default research mode: {MODE}. Would you like to change it for this run?"*

If no default is set, ask:
*"Which research mode would you like to use?"*
- **Hibernation**: Preserve my tokens and be judicious with amounts collected and written
- **Hyperphagia**: Prioritize quantity and variety of resources, eat those tokens yum

Record the final choice as **MODE** (HIBERNATION or HYPERPHAGIA).

**0.3 Review Presence**
Read the `REVIEW_FLOW` value from `.beary/USER.md` (the `<!-- REVIEW_FLOW: ... -->` comment). If a valid default exists (attended or unattended), use it and inform the user, offering to override:
*"Using your default review flow: {REVIEW_FLOW}. Would you like to change it for this run?"*

If no default is set, ask:
*"Would you like to be present for review checkpoints during the process? This improves quality but requires continued engagement. If you say no, I'll complete the entire workflow without stopping."*

Record the final choice as **ATTENDED** (yes) or **UNATTENDED** (no).

**Mode Summary:**
- **HIBERNATION**: Token-conservative mode. Uses minimal questions/search terms, performs sufficiency checks, and defaults to single-topic (no subtopics).
- **HYPERPHAGIA**: Token-generous mode. Uses more questions/search terms, allows subtopics, and skips sufficiency checks to maximize resource gathering.
 
---

### 1. Read User Context (Once)

If `.beary/USER.md` does not exist, create the `.beary/` directory at the project root if needed, then create `USER.md` from `.agents/skills/beary/templates/user-context-template.md` by following `.agents/skills/beary/skills/user-context-template/SKILL.md`.

Then read `.beary/USER.md` to understand the user's audience and research priorities. This context applies to the entire workflow — do not re-read it in subsequent steps.

### 2. Execute Internet Research

Follow `.agents/skills/beary/skills/internet-research/SKILL.md`, **skipping step 1** (User Context) since it was already read above. **Pass the MODE (HIBERNATION or HYPERPHAGIA) to the skill.**

This includes:
// turbo
- Directory setup (step 3)
- Initial research with General Understanding questions (step 4.1)
- In-depth research with Deeper Dive questions (step 4.2)
- Review and synthesize notes (step 5)

For citation formatting during research, refer to `.agents/skills/beary/skills/references/SKILL.md`.

### 3. Checkpoint: User Review (ATTENDED mode only)
 
**If ATTENDED:** STOP and ask the user to review...
**If UNATTENDED:** Skip this step and proceed directly to whitepaper writing.

Present:
- Summary of key findings from `notes/{TOPIC}-notes.md`
- Number of sources cited in `whitepaper/{TOPIC}-references.md`
- Any unresolved contradictions or gaps

Ask: *"Would you like me to proceed with writing the whitepaper, or would you like to adjust the research first?"*

Do not proceed until the user confirms.

### 4. Execute Whitepaper Writing

Follow `.agents/skills/beary/skills/whitepaper-writing/SKILL.md`, **skipping steps 1 and 2** (User Context and Read Prompt) since context was already established.

This includes:
- Read notes and references (step 3)
- Outline the whitepaper (step 4)
- Write the whitepaper (step 5)
- Review (step 6)

### 5. Move Completed Whitepaper to Output Directory

Research should run inside `beary-scratchpad/{TOPIC}` to avoid cluttering the project root.

Check `.beary/USER.md` for the output path configuration:
- Look for the `OUTPUT_PATH:` value in the "Output Path" section
- If not specified or file doesn't exist, use `whitepaper-output/` as the default
- The path can be either:
  - **Relative** to the project root (e.g., `whitepaper-output`, `docs/research`)
  - **Absolute** (e.g., `/Users/username/Documents/research`) - required for paths outside the repository
- Verify the path is valid and accessible
- If the path fails to resolve or is inaccessible, fall back to `whitepaper-output/` and notify the user
- Create the output directory if it doesn't exist
// turbo
- Move `beary-scratchpad/{TOPIC}/whitepaper/` and `beary-scratchpad/{TOPIC}/notes/` into `{OUTPUT_PATH}/{TOPIC}/`

This keeps completed research separate from active work and prevents accidental commits.

### 6. Final Delivery

**If ATTENDED:** Present the completed whitepaper and ask if they would like any revisions.
**If UNATTENDED:** Notify the user that the workflow is complete.

Provide file locations:
- Whitepaper: `{OUTPUT_PATH}/{TOPIC}/whitepaper/{TOPIC}-whitepaper.md`
- References: `{OUTPUT_PATH}/{TOPIC}/whitepaper/{TOPIC}-references.md`
- Notes: `{OUTPUT_PATH}/{TOPIC}/notes/{TOPIC}-notes.md`
- Scratchpad workspace: `beary-scratchpad/{TOPIC}/`

---

## Context Management Tips

If the research phase consumes significant context:
1. After completing General Understanding, summarize key findings before proceeding to Deeper Dive
2. For complex topics with multiple subtopics, complete and summarize one subtopic at a time
3. Before starting whitepaper writing, ensure notes summaries are complete — rely on summaries rather than re-reading all raw notes
