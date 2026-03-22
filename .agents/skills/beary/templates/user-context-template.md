# User Context

This file contains user-specific context that customizes how the Background Research Agent (BEARY) conducts research.

## Audience

The research audience is a user with professional interest in the topic that touches an area of expertise, but is not their expertise. Do not assume any particular expertise. Assume that if the audience is asking a question, they have less expertise in this particular area.

For newer and cutting edge topics (within the last 3 months), assume that the audience has heard of the topic in passing, but does not know what it really is.

## Purpose

If the user shares the purpose of the research, use it to narrow down the search.

## Priorities

When researching, prioritize:
- Sources from academic literature, official documentation, and releases from well-known organizations and established expert media outlets
- Online wikis and technical documentation

Avoid sources from:
- Unknown blogs and unverified forums

## Research Mode

Default research mode controls token usage and depth.

- **Hibernation**: Token-conservative. Fewer questions and search terms, sufficiency checks enabled, subtopics only when clearly needed.
- **Hyperphagia**: Token-generous. More questions and search terms, no early stopping, subtopics encouraged.

<!-- RESEARCH_MODE: hibernation -->

## Review Flow

Default review flow controls whether BEARY pauses for user review at checkpoints.

- **Attended**: Pause at checkpoints for user review before proceeding.
- **Unattended**: Complete the entire workflow without stopping.

<!-- REVIEW_FLOW: attended -->

## Output Path

Specify where completed whitepaper artifacts should be moved after the workflow completes.

DEFAULT_OUTPUT_PATH: whitepaper-output

To customize, set the path below:
- **Relative paths** (e.g., `whitepaper-output`, `docs/research`) are relative to the project root
- **Absolute paths** (e.g., `/Users/username/Documents/research`) MUST be used for directories outside the repository
- If the path fails to resolve, the default `whitepaper-output/` will be used

<!-- OUTPUT_PATH: whitepaper-output -->
