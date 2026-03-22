---
name: internet-research
description: Generate notes.md file or files from internet research on the topic of interest.
license: MIT
compatibility: Requires internet access and edit_file capabilities. Optionally uses subagents/agent teams for parallel research when available.
metadata:
  author: sjankovic-wwt
  version: "1.0"
---

# Internet Research Skill

This skill defines the standard format and rules for conducting internet research on the topic of interest.

By *research*, we mean internet searches of relevant literature, blogs, technical troubleshooting forums, documentation, code repositories, company releases, etc. Research should take place over multiple actions, involving systematic approach to the topic and multiple searches.

# User Context (Optional)

If `.beary/USER.md` exists, read it to understand the user's specific needs, audience, and priorities before beginning research.

# Prerequisites
You must be prompted specifically to begin research. You need:
- Name of the topic of interest to use in creating files and folders
- A longer description of the topic of interest

Optionally, the user may tell you what the purpose of the research is. This can be used to narrow down the search.

# Research Modes

This skill supports two research modes that control token usage and research depth:

## HIBERNATION Mode (Default)
Token-conservative mode. Beary is saving up tokens for winter.
- **Start minimal** — Begin with 2 questions and 2 search terms each. Add more only if information is insufficient.
- **Evaluate after each question** — Before starting the next question, assess if you already have enough information to answer the research goal.
- **Stop early when satisfied** — If 2 questions with 2-3 search terms yield comprehensive answers, do not continue to the maximum.
- **Default to straightforward** — Assume the topic does NOT need subtopics unless it is clearly too broad to cover in a single notes file.
- **Sufficiency checks enabled** — Always check if gathered information is sufficient before proceeding.

## HYPERPHAGIA Mode
Token-generous mode. Beary is bulking up before hibernation — eat those tokens!
- **Start generous** — Begin with 3-4 questions and 3 search terms each.
- **No sufficiency checks** — Do not stop early; gather as much information as possible.
- **Subtopics encouraged** — If the topic has any natural subdivisions, break it into subtopics for deeper coverage.
- **Maximize variety** — Prioritize diverse sources and perspectives over efficiency.

# Parallel Execution with Subagents

If your host agent supports subagents, agent teams, or agent swarms (e.g., Claude Code's Agent tool, Windsurf Cascade's parallel actions, or any multi-agent orchestration), use them to parallelize research. This dramatically reduces wall-clock time without sacrificing quality.

## When to Parallelize

- **Research questions within the same phase** (General Understanding or Deeper Dive) are independent and can be researched in parallel.
- **Search terms within a single question** are independent and can be searched in parallel.
- **Subtopic research** (Complex Topics in step 4.2.2.2) — each subtopic can be assigned to a separate subagent.

## How to Parallelize

1. **Fan out by question:** After generating research questions (step 4.1.1 or 4.2.1), launch one subagent per question. Each subagent receives:
   - The question and its search terms
   - The citation format rules (from `.agents/skills/beary/skills/references/SKILL.md`)
   - The user context (audience, priorities)
   - Instructions to return: (a) notes organized by question with inline citations, and (b) a list of reference entries in the standard format

2. **Merge results:** Once all subagents return, the orchestrator:
   - Appends each subagent's notes into the appropriate section of `{TOPIC}-notes.md`
   - Merges all reference entries into `{TOPIC}-references.md`, re-numbering IDs sequentially and updating in-text citations to match
   - Runs the synthesis step (summary paragraph) over the combined notes

3. **Fan out by subtopic:** For complex topics, launch one subagent per subtopic. Each subagent handles all questions within its subtopic and returns a completed `{subtopic}-notes.md` and its reference entries.

## Constraints

- **HIBERNATION mode with sufficiency checks:** Parallelizing all questions defeats early stopping. In HIBERNATION mode, prefer running questions sequentially or in small batches (2 at a time) so sufficiency checks remain meaningful.
- **HYPERPHAGIA mode:** Full parallelization is encouraged — there are no sufficiency checks to respect.
- **Citation ID conflicts:** Each subagent should use placeholder IDs (e.g., `[Q1-1]`, `[Q1-2]`). The orchestrator re-numbers them sequentially during the merge step.
- **If subagents are not available:** Fall back to the sequential execution steps below. The workflow is designed to work either way.

# Execution Steps

## 1. Read User Context
If `.beary/USER.md` exists, read it first to understand the user's audience and research priorities. This will inform how you frame questions and what sources to prioritize.

## 2. Read the Prompt
Read the prompt to understand the topic of interest and the purpose of the research.

## 3. Directory Setup
Using the given TOPIC, create a working folder under `beary-scratchpad/{TOPIC}`. Within this folder, create `notes` and `whitepaper` directories. Create the following files using templates from `.agents/skills/beary/templates/`:

| File (under `beary-scratchpad/{TOPIC}/`) | Template |
|------|----------|
| `notes/{TOPIC}-research-questions.md` | `.agents/skills/beary/templates/topic-research-questions.md` |
| `notes/{TOPIC}-notes.md` | `.agents/skills/beary/templates/topic-notes.md` |
| `whitepaper/{TOPIC}-references.md` | `.agents/skills/beary/templates/references.md` |
| `whitepaper/{TOPIC}-whitepaper.md` | `.agents/skills/beary/templates/whitepaper.md` |

## 4. Research

### 4.1. Initial Research
You must gain a general understanding of the topic before diving in deeper.

#### 4.1.1. General Research Questions
Before beginning research, use the given TOPIC, description, and purpose to generate research questions. List your questions in the `beary-scratchpad/{TOPIC}/notes/{TOPIC}-research-questions.md` file under the heading "General Understanding". 

**HIBERNATION mode:** Generate **2** *general research questions* (add a 3rd only if the topic clearly requires it). Include **2** internet search terms per question (add more only if initial searches are insufficient).

**HYPERPHAGIA mode:** Generate **3-4** *general research questions*. Include **3** internet search terms per question.

You may use your general knowledge of software engineering, machine learning, and other relevant technical topics to inform these questions.

#### 4.1.2 Notes on General Understanding
Your answers should go under the heading "General Understanding" in the `beary-scratchpad/{TOPIC}/notes/{TOPIC}-notes.md` file. *Organize your notes by question, not source.*

> **Parallel execution:** If subagents are available, fan out research questions in parallel per the "Parallel Execution with Subagents" section above. The steps below describe the work each agent (or subagent) performs per question.

For each research question, complete the following steps.

1. Create a subsection for the question in the "General Understanding" section of the `beary-scratchpad/{TOPIC}/notes/{TOPIC}-notes.md` file.
2. Using your question's first search term, find some sources on the internet that answer your questions. 
3. Read the sources. For each source that answers your questions, add the source to the list of citations in the `beary-scratchpad/{TOPIC}/whitepaper/{TOPIC}-references.md` file. 
4. Write down your answers to the questions in the "General Understanding" section of the `beary-scratchpad/{TOPIC}/notes/{TOPIC}-notes.md` file *with citation*.
5. Repeat steps 2-4 for each search term under the question.
6. **Sufficiency check (HIBERNATION mode only):** Before starting the next question, review what you have learned. If the research goal can already be answered with the information gathered, you may skip remaining questions and proceed to synthesis. **In HYPERPHAGIA mode, skip this check and continue to all questions.**
7. Repeat steps 1-6 for each research question.

Once all general research questions have been answered (or skipped due to sufficiency), synthesize the "General Understanding" section:
1. Review all answers and identify themes, agreements, and contradictions across sources.
2. Add a summary paragraph at the end of the "General Understanding" section with key takeaways. Ensure citations remain accurate.

### 4.2 In-depth Research
#### 4.2.1 In-depth Research Questions
In order to generate deeper questions, use the General Understanding Section of `beary-scratchpad/{TOPIC}/notes/{TOPIC}-notes.md` in the following way:
- Read the Section, making particular notes of where there are sources with multiple (more than 2) citations. Treat these as more reliable sources. Also take note of conflicting sources. Treat these as places for deeper understanding.
- Create a new section in the `beary-scratchpad/{TOPIC}/notes/{TOPIC}-research-questions.md` file under the heading "Deeper Dive".
**HIBERNATION mode:**
- **Default to straightforward.** Only classify as "complex" if the topic clearly has 3+ distinct subtopics that cannot be covered coherently in a single notes file. Most topics should remain straightforward.
  - If the topic is straightforward (default), generate **1-2** *in-depth research questions* with **2** internet search terms for each question.
  - If the topic is genuinely complex, break it down into subtopics. Give each one a heading. Then, for each subtopic, generate a list of 1-2 *in-depth research questions* with 2 internet search terms for each question.

**HYPERPHAGIA mode:**
- **Subtopics encouraged.** If the topic has any natural subdivisions (even 2), break it into subtopics for deeper coverage.
  - For straightforward topics, generate **2-3** *in-depth research questions* with **3** internet search terms for each question.
  - For topics with subtopics, generate **2-3** *in-depth research questions* per subtopic with **3** internet search terms for each question.

**Both modes:**
- If provided with a purpose for asking the questions, use it to generate up to 2 additional questions. If there are subtopics, add the purpose answers to the most relevant subtopic or create a new one. If there are no subtopics, it can just be a new question.
- Review the In-Depth research questions:
    - Check for quality and ability for the search terms to answer the questions.
    - If any questions are redundant, move them to the Redundant Questions Section at the bottom of the file.
    - For each question, if any internet search terms are redundant, move them to the bottom of the list of search terms and add (Redundant) to the title of the search term.

#### 4.2.2 In-depth Research Notes
Choose the following instructions based on whether your topic is straightforward or complex. Skip the other instructions.

##### 4.2.2.1 Straightforward Topics

> **Parallel execution:** If subagents are available, fan out in-depth research questions in parallel per the "Parallel Execution with Subagents" section above.

For each research question, complete the following steps.

1. Create a new section for the question in the "Deeper Dive" section of the `beary-scratchpad/{TOPIC}/notes/{TOPIC}-notes.md` file.
2. Using your question's first search term, find some sources on the internet that answer your questions. 
3. Read the sources. For each source that answers your questions, add the source to the list of citations in the `beary-scratchpad/{TOPIC}/whitepaper/{TOPIC}-references.md` file. 
4. Write down your answers to the questions in the "Deeper Dive" section of the `beary-scratchpad/{TOPIC}/notes/{TOPIC}-notes.md` file *with citation*.
5. Repeat steps 2-4 for each search term under the question.
6. Repeat steps 1-5 for each research question.

Once all in-depth research questions have been answered, synthesize the "Deeper Dive" section:
1. Review all answers and identify themes, agreements, and contradictions across sources. Tie in to concepts from the "General Understanding" section.
2. Add a summary paragraph at the end of the "Deeper Dive" section with key takeaways. Ensure citations remain accurate.

##### 4.2.2.2 Complex Topics

> **Parallel execution:** If subagents are available, assign each subtopic to a separate subagent per the "Parallel Execution with Subagents" section above.

For each subtopic, complete the following steps.
1. Create a new file called `{subtopic}-notes.md` for your subtopic using the `.agents/skills/beary/templates/subtopic-notes.md` template.
2. For each question in the subtopic, create a new section in the `{subtopic}-notes.md` file.
3. Using your question's first search term, find some sources on the internet that answer your questions. 
4. Read the sources. For each source that answers your questions, add the source to the list of citations in the `beary-scratchpad/{TOPIC}/whitepaper/{TOPIC}-references.md` file. 
5. Write down your answers to the questions in the section of the `{subtopic}-notes.md` file *with citation*.
6. Repeat steps 3-5 for each search term under the question.
7. Repeat steps 2-6 for each question in the subtopic.

Once all questions in the subtopic have been answered, synthesize the subtopic:
1. Review all answers and identify themes, agreements, and contradictions across sources.
2. Add a summary of 1-2 paragraphs at the end of the `{subtopic}-notes.md` file with key takeaways. Ensure citations remain accurate.
3. Repeat for each subtopic.


## 5. Review and Synthesize
Review the notes and citations in the `beary-scratchpad/{TOPIC}/whitepaper/{TOPIC}-references.md` file.

### 5.1 Review and Clarify
These steps will get the notes in a readable form to write the whitepaper. Follow these steps for the {TOPIC}-notes.md file and any subsequent subtopic notes files, *one file at a time*.

1. Review the notes and citations in the `beary-scratchpad/{TOPIC}/whitepaper/{TOPIC}-references.md` file. Correct any citations that are incorrect or point to non-existent sources.
2. Identify and flag any points that contradict each other, or any points that seem unsubstantiated or go against common knowledge.
3. Ensure the existing synthesis summaries are accurate and complete. Make minor edits for clarity if needed, but do not re-synthesize. 

### 5.2 Quality Check
After editing the notes, run this final check to ensure note quality.
1. Ensure that each citation is accurate and points to a real source.
2. Ensure that redundant points are removed from the notes.
3. Ensure that the notes are clear and concise.
4. Ensure that the notes are well-organized and follow their respective templates.

# Templates

| Purpose | Template Path |
|---------|---------------|
| Research questions | `.agents/skills/beary/templates/topic-research-questions.md` |
| Topic notes (straightforward) | `.agents/skills/beary/templates/topic-notes.md` |
| Subtopic notes (complex) | `.agents/skills/beary/templates/subtopic-notes.md` |
| References/bibliography | `.agents/skills/beary/templates/references.md` |
| Whitepaper | `.agents/skills/beary/templates/whitepaper.md` |


# Best Practices

## HIBERNATION Mode
- **Start small, expand if needed** — Begin with 2 questions and 2 search terms. Only add more if information is clearly insufficient.
- **Evaluate sufficiency between questions** — Do not mechanically exhaust all questions if you already have good coverage.
- **Default to straightforward topics** — Subtopics add complexity and tokens. Use them only when necessary.
- Organize notes by question/topic, not source.
- When reading, prioritize quality sources over quantity. When writing, concision is key.

## HYPERPHAGIA Mode
- **Start generous** — Begin with 3-4 questions and 3 search terms. Gather broadly.
- **Skip sufficiency checks** — Complete all planned questions without early termination.
- **Embrace subtopics** — If natural divisions exist, use them to organize deeper research.
- Organize notes by question/topic, not source.
- When reading, prioritize variety and coverage. Gather diverse perspectives.






