---
description: Generate bugfix tasks from bugs found in a brainstorm's post-implementation review and link them back to the brainstorm notes.
---

## Purpose

Extract bugs documented in a brainstorming `notes.md` (typically under a "Post-Implementation Review" section) and generate a numbered `bugfixNN.md` file with actionable fix tasks. Then update the brainstorm `notes.md` to reference the generated bugfix file.

## Prerequisites

- A brainstorm folder exists (e.g., `brainstorming/<feature>/notes.md`)
- The `notes.md` contains a section with documented bugs (typically "Post-Implementation Review" or similar)
- Each bug should have: file location, issue description, impact, and proposed fix

## Workflow Steps

### Step 1: Locate the brainstorm

Identify the brainstorm folder. The user may specify it directly, or you can infer it from context (active document, recent conversation).

```
BRAINSTORM_DIR = brainstorming/<feature-slug>/
```

Verify `notes.md` exists in that directory.

### Step 2: Determine the bugfix number

Count existing `bugfix*.md` files in the brainstorm folder to determine the next number:

// turbo
```bash
ls brainstorming/<feature-slug>/bugfix*.md 2>/dev/null | wc -l
```

The new file will be `bugfix<NN>.md` where `<NN>` is the next number, zero-padded to 2 digits (e.g., `bugfix01.md`, `bugfix02.md`).

### Step 3: Extract bugs from notes.md

Read the brainstorm `notes.md` and find all documented bugs. Look for sections that contain:
- Bug descriptions with **File**, **Issue**, **Impact**, and **Fix** subsections
- Any section titled "Post-Implementation Review", "Bugs Found", "Issues Found", or similar
- UX improvements documented alongside bugs (include these as separate tasks)

For each bug found, extract:
1. **Title**: Short descriptive name
2. **Severity**: Critical (runtime crash/data loss), High (broken feature), Medium (wrong behavior), Low (cosmetic/warning)
3. **File(s)**: Which files need changes
4. **Problem**: What's wrong
5. **Proposed fix**: The solution (code snippets if available in notes.md)
6. **Estimate**: Time estimate based on complexity

### Step 4: Generate the bugfix file

Create `brainstorming/<feature-slug>/bugfix<NN>.md` using the template at `.templates/brainstorm.bugfix.md` as inspiration but filling in the actual content.

The file structure should be:

```markdown
# <Feature Name> — Bugfix Tasks (<NN>)

**Brainstorm**: [notes.md](./notes.md)
**Created**: <YYYY-MM-DD>
**Source**: Post-implementation review in notes.md

## Bugfix Overview

| # | Bug | Severity | Estimate |
|---|-----|----------|----------|
| BF-01 | <bug title> | Critical/High/Medium/Low | Xm |
| BF-02 | <bug title> | ... | ... |

---

## BF-01: <Bug Title>

**Severity**: Critical | High | Medium | Low
**File(s)**: `path/to/file.rs`
**Estimate**: Xm

### Problem

<Description of what's wrong, with code snippets showing the current broken code>

### Fix

<Description of the fix, with code snippets showing the corrected code>

### Acceptance Criteria

- [ ] <Specific testable criterion>
- [ ] <Another criterion>

---

## BF-02: <Bug Title>
...

## Verification

After all bugfixes are applied:

- [ ] `cargo build -p <crate>` succeeds with no errors
- [ ] `cargo clippy -p <crate> -- -D warnings` passes (or warnings reduced)
- [ ] `cargo test -p <crate>` passes
- [ ] Manual smoke test of affected functionality
```

### Step 5: Update the brainstorm notes.md

Add a reference to the generated bugfix file in the brainstorm `notes.md`. Insert it in two places:

1. **Near the top** (after the header/metadata area), add to the file's metadata links:
   ```markdown
   **Bugfixes**: [bugfix<NN>.md](./bugfix<NN>.md)
   ```

2. **In the Post-Implementation Review section**, add a note pointing to the bugfix tasks:
   ```markdown
   > ℹ️ Bugfix tasks generated: [bugfix<NN>.md](./bugfix<NN>.md)
   ```

### Step 6: Report

Output a summary:
- Path to the generated bugfix file
- Number of bugs extracted
- Severity breakdown (e.g., "1 Critical, 2 Medium, 1 Low")
- Total estimated fix time
- Any bugs that were unclear and need clarification

## Rules

1. **Every bug gets its own task** — don't combine multiple bugs into one task
2. **Include the proposed fix** — if `notes.md` has code snippets for the fix, include them in the bugfix task
3. **UX improvements are tasks too** — if the review found UX improvements alongside bugs, include them as separate tasks with severity "Low" or "Medium"
4. **Preserve context** — reference the original bug description in notes.md so readers can trace back
5. **Be specific** — each acceptance criterion should be testable (not "works correctly" but "function handles None input without panicking")
6. **Order by severity** — Critical bugs first, then High, Medium, Low
