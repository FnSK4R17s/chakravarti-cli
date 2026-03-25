---
name: references
description: Generate references.md file or files from internet research on the topic of interest.
license: MIT
compatibility: Requires internet access and edit_file capabilities
metadata:
  author: sjankovic-wwt
  version: "1.0"
---

# References Skill

This skill defines the standard format and rules for citing internet-based sources in whitepapers produced by the Background Research Agent.

## Citation Format

Each citation entry in the bibliography (`{TOPIC}-references.md`) must follow this structure:

```
[{id}] {Author(s)}. "{Title}." *{Domain/Publisher}*. {Publication Date}. URL: {URL}. Accessed: {Access Date}.
```

### Field Definitions

| Field              | Required | Description                                                                                                  |
|--------------------|----------|--------------------------------------------------------------------------------------------------------------|
| **ID**             | Yes      | A short unique identifier used for in-text references, e.g., `[1]`, `[2]`, or `[author-year]`.              |
| **Author(s)**      | Yes      | The individual or organization that authored the content. If unknown, use `Unknown Author`.                  |
| **Title**          | Yes      | The title of the page, article, or paper. Enclosed in double quotes.                                        |
| **Domain/Publisher**| Yes     | The source platform or publisher. Italicized. See [Domain Guidelines](#domain-guidelines) below.            |
| **Publication Date**| Yes     | The date the source was published, in `YYYY-MM-DD` format. If unavailable, write `n.d.` (no date).          |
| **URL**            | Yes      | The full URL where the source can be accessed. Must be a working link at time of research.                   |
| **Access Date**    | Yes      | The date the source was accessed during research, in `YYYY-MM-DD` format.                                   |

## Domain Guidelines

The domain/publisher field should clearly identify *where* the source lives on the internet. Use the most recognizable name for the platform or publisher. Examples:

| Source Type              | Domain/Publisher Example          |
|--------------------------|-----------------------------------|
| Academic paper (arXiv)   | *arXiv (Cornell University)*      |
| Academic paper (journal) | *Nature*, *IEEE*, *ACM*           |
| Blog post (company)      | *Anthropic Blog*, *OpenAI Blog*   |
| Blog post (personal)     | *{Author}'s Blog ({domain.com})* |
| Documentation            | *Python Documentation*, *MDN Web Docs* |
| Forum / Q&A              | *Stack Overflow*, *Reddit (r/{subreddit})* |
| News article             | *The New York Times*, *TechCrunch* |
| GitHub repository        | *GitHub ({owner}/{repo})*         |
| Government / org report  | *WHO*, *NIST*, *U.S. GAO*        |
| Wiki                     | *Wikipedia*                       |

If a source doesn't fit neatly into these categories, use the site's domain name in title case (e.g., *Example.com*).

## In-Text Citation

When referencing a source within the whitepaper body, use the bracketed ID that matches the bibliography entry:

```markdown
Recent findings suggest that large language models benefit from chain-of-thought prompting [1].
Multiple studies have confirmed this effect [1][3][7].
```

## Example Bibliography Entries

```markdown
[1] Wei, Jason et al. "Chain-of-Thought Prompting Elicits Reasoning in Large Language Models." *arXiv (Cornell University)*. 2022-01-28. URL: https://arxiv.org/abs/2201.11903. Accessed: 2026-02-26.

[2] Anthropic. "Claude's Character." *Anthropic Blog*. 2024-06-10. URL: https://www.anthropic.com/research/claudes-character. Accessed: 2026-02-26.

[3] Unknown Author. "How do transformer models handle long-range dependencies?" *Stack Overflow*. n.d. URL: https://stackoverflow.com/questions/00000000. Accessed: 2026-02-26.

[4] Vaswani, Ashish et al. "Attention Is All You Need." *arXiv (Cornell University)*. 2017-06-12. URL: https://arxiv.org/abs/1706.03762. Accessed: 2026-02-26.
```

## Rules

1. **Every source must have a working URL** — the reader must be able to re-access it.
2. **Every source must identify its domain/publisher** — the reader must know *where* it comes from.
3. **Every source must list an author** — use `Unknown Author` only as a last resort.
4. **Publication date is required** — use `n.d.` only when genuinely unavailable.
5. **Access date is always required** — record the date the agent accessed the source.
6. **IDs must be sequential** — numbered `[1]`, `[2]`, `[3]`, etc., in the order they first appear in the whitepaper.
7. **Use the template** at `.agents/skills/beary/templates/references.md` for the bibliography file structure.
