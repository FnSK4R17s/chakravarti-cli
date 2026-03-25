---
name: whitepaper-writing
description: Generate whitepaper.md file from notes.md file or files and accompanying citations via the references.md file.
license: MIT
compatibility: Requires internet access and edit_file capabilities
metadata:
  author: sjankovic-wwt
  version: "1.0"
---

# Whitepaper Writing Skill

This skill defines the standard format and rules for writing whitepapers based on notes.md file or files and accompanying citations via the references.md file.

# User Context (Optional)

If `.beary/USER.md` exists, read it to understand the user's specific needs, audience, and priorities before writing the whitepaper.

# Prerequisites
You must be prompted specifically to begin writing the whitepaper. You need:
- Name of the topic of interest to use in creating files and folders
- A longer description of the topic of interest
- Access to notes.md files and accompanying references.md files to use in writing the whitepaper

Optionally, the user may tell you what the purpose of the whitepaper is. This can be used to guide the theme and scope of the whitepaper.


# Execution Steps

## 1. Read User Context
If `.beary/USER.md` exists, read it first to understand the user's audience and research priorities. This will inform how you frame questions and what sources to prioritize.

## 2. Read the Prompt
Read the prompt to understand the topic of interest and the purpose of the whitepaper.

## 3. Read the Notes and References
Read the `beary-scratchpad/{TOPIC}/notes/{TOPIC}-notes.md` file (and any `{subtopic}-notes.md` files) along with `beary-scratchpad/{TOPIC}/whitepaper/{TOPIC}-references.md` to understand the content for the whitepaper. Take note of topics that are:
- widely agreed upon
- up for debate
- interesting or creative solutions and approaches

## 4. Outline the Whitepaper
Using the `.agents/skills/beary/templates/whitepaper.md` template, create an outline in the `beary-scratchpad/{TOPIC}/whitepaper/{TOPIC}-whitepaper.md` file. Fill in the section headers based on the notes and references.

## 5. Write the Whitepaper
Fill in each section of the `beary-scratchpad/{TOPIC}/whitepaper/{TOPIC}-whitepaper.md` file based on the notes and references.

After the introduction, for each section, fill in the paper in the following steps:
1. Synthesize the research findings from the notes section with relevant citations from the references section.
2. Add in analysis and insights as to the meaning of the findings. Make note of the most important and surprising ones, if any exist. If there are points of debate, include them and explain why they are important.
3. If the user provides a purpose for the whitepaper, tailor the narrative to that purpose.
4. Discuss implementations and applicability of the findings.

For the conclusion, re-read the whitepaper and summarize and synthesize based on the existing content.

## 6. Review
Review the whitepaper.md file to ensure it is clear and concise. Make sure to:
- Eliminate redundant sentences and paragraphs. 
- Check for any typos, grammatical errors, missing citations, and logical inconsistencies.
- Make sure that there is a clear narrative in the whitepaper.

# Best Practices
- Err on the side of being concise and using the active voice
- Use any user context to support the narrative, but do not be afraid to give answers that are not in line with the user's desires. *If the user's goals are not possible, you should tell them*.
- Offering alternative solutions and viewpoints is encouraged.