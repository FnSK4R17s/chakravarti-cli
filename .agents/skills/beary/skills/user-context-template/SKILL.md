---
name: user-context-template
description: Generate or refresh `.beary/USER.md` from a clean template for BEARY research preferences (audience, priorities, source quality rules, and output path). Use when `.beary/USER.md` is missing, incomplete, or the user asks to reset/tune research defaults.
---

# User Context Template

Create or update `.beary/USER.md` using the template at `.agents/skills/beary/templates/user-context-template.md`.

## Questionnaire (ask user before writing)

Ask these in order, then write/update USER.md from answers:

1. **Audience level**
   - “Who is this research usually for? (beginner / intermediate / expert / mixed)”
   - “Any specific role to optimize for? (e.g., engineer, PM, founder, student)”

2. **Purpose style**
   - “Should BEARY optimize for broad exploration or decision support?”
   - “Any recurring outcomes you want? (e.g., implementation plan, comparison table, risks, recommendations)”

3. **Source preferences**
   - “Rank your preferred sources (docs, academic papers, standards, vendor blogs, news, community forums).”
   - “Any sources/domains to always avoid?”

4. **Freshness expectations**
   - “For fast-moving topics, how recent should sources be by default? (30/60/90 days)”

5. **Depth and skepticism**
   - “Should BEARY prioritize concise summaries or deep technical detail?”
   - “How strict should contradiction checks be? (light / normal / strict)”

6. **Research mode**
   - “Which research mode do you prefer by default?”
   - **Hibernation**: Conservative — fewer questions/searches, sufficiency checks, no subtopics unless needed.
   - **Hyperphagia**: Generous — more questions/searches, no early stopping, subtopics encouraged.

7. **Review flow**
   - “Do you prefer to review research at checkpoints (attended) or let BEARY run end-to-end (unattended)?”

8. **Output path**
   - “Where should completed research be stored by default?”
   - “Use relative project path or absolute path?”

9. **Final confirmation**
   - Show a short summary of captured preferences and ask: “Apply these as your default USER.md?”

## Rules

- Keep the same section order as the template.
- Preserve explicit user customizations if they already exist.
- If a field is unknown, keep the template default.
- Ensure the output path block includes the `<!-- OUTPUT_PATH: ... -->` line.
- Remove `Note: You are currently using the default USER.md template.` once user-specific answers are captured.

## Output

Write a complete `.beary/USER.md` that is human-editable and ready for future runs.
