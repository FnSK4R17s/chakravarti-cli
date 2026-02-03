---
name: speckit-order
description: Reference for speckit workflow execution order and dependencies.
---

# Speckit Workflow Order

This skill provides the execution order for speckit workflows. Reference this when completing any `/speckit.*` workflow to know what to run next.

## Execution Order

| Step | Workflow | What It Does | Required Artifacts |
|------|----------|--------------|-------------------|
| **0** | `/speckit.constitution` | Defines project principles and non-negotiables | None (foundational) |
| **1** | `/speckit.specify` | Creates feature spec from natural language | Constitution |
| **2** | `/speckit.clarify` | Resolves ambiguities in the spec (optional) | `spec.md` |
| **3** | `/speckit.plan` | Creates technical plan with architecture | `spec.md` |
| **4** | `/speckit.tasks` | Generates actionable, ordered task list | `spec.md`, `plan.md` |
| **5** | `/speckit.analyze` | Cross-artifact consistency check (optional) | `spec.md`, `plan.md`, `tasks.md` |
| **6** | `/speckit.implement` | Executes tasks phase-by-phase | `tasks.md` (all artifacts) |

## Dependency Graph

```
                    ┌─────────────────────┐
                    │ /speckit.constitution│
                    └──────────┬──────────┘
                               │
                               ▼
                    ┌─────────────────────┐
                    │   /speckit.specify   │
                    └──────────┬──────────┘
                               │
              ┌────────────────┼────────────────┐
              │                │                │
              ▼                │                ▼
    ┌──────────────────┐       │      ┌──────────────────┐
    │ /speckit.clarify │       │      │ /speckit.checklist│
    │   (optional)     │       │      │    (anytime)      │
    └────────┬─────────┘       │      └──────────────────┘
             │                 │
             └────────┬────────┘
                      ▼
           ┌─────────────────────┐
           │    /speckit.plan    │
           └──────────┬──────────┘
                      │
                      ▼
           ┌─────────────────────┐
           │   /speckit.tasks    │
           └──────────┬──────────┘
                      │
        ┌─────────────┼─────────────┐
        │             │             │
        ▼             │             ▼
┌───────────────┐     │    ┌────────────────────┐
│/speckit.analyze│     │    │/speckit.taskstoissues│
│  (optional)   │     │    │    (optional)       │
└───────┬───────┘     │    └────────────────────┘
        │             │
        └──────┬──────┘
               ▼
    ┌─────────────────────┐
    │  /speckit.implement  │
    └─────────────────────┘
```

## Key Dependencies

- `/speckit.constitution` is foundational; run once per project (or when principles change)
- `/speckit.specify` creates `spec.md` which all other workflows depend on
- `/speckit.clarify` is optional but recommended before `/speckit.plan` (reduces rework)
- `/speckit.plan` requires `spec.md` and produces `plan.md`, `research.md`, `data-model.md`, `contracts/`
- `/speckit.tasks` requires both `spec.md` and `plan.md`
- `/speckit.analyze` is read-only; run after `/speckit.tasks` to catch issues before implementation
- `/speckit.implement` requires `tasks.md`; checks all checklists before starting
- `/speckit.checklist` can run anytime after `spec.md` exists (creates requirement quality tests)
- `/speckit.taskstoissues` converts tasks to GitHub issues (optional, requires GitHub remote)

## 🎯 The Spec Review Checkpoint (Recommended)

**Before running `/speckit.plan`, pause and review your spec.**

Checklists serve as "unit tests for requirements" — they train you to write complete specs. Running `/speckit.checklist` before planning helps you:

1. **Catch gaps early** — Cheaper to fix in text than in code
2. **Build the spec designer muscle** — Learn what "complete" looks like
3. **Reduce rework** — Plans built on solid specs don't need revision
4. **Ensure alignment** — Validate against project vision and principles

### Recommended Checklists

| Checklist | When to Use | What It Validates |
|-----------|-------------|-------------------|
| `requirements` | Always | Basic spec completeness and clarity |
| `guiding_docs alignment` | Always | Alignment with project vision/principles |
| `security` | Auth, data, payments | Security requirements coverage |
| `ux` | User-facing features | UX requirements completeness |
| `api` | Backend/integration work | API contract requirements |
| `performance` | Scale-sensitive features | Non-functional requirements |

### Recommended Flow

```
/speckit.specify  
        │
        ▼
/speckit.checklist requirements
/speckit.checklist guiding_docs alignment    ← Ensures vision alignment
/speckit.checklist <domain>                  ← Domain-specific (security, ux, etc.)
        │
        ▼
👤 Review & Fix Gaps
        │
        ▼
/speckit.plan
```

### Questions to Ask Yourself

Before moving to planning, reflect on these questions. If you can't answer confidently, your spec may need work.

#### 🎭 User & Scope
- **Who** exactly will use this feature? Are all user types defined?
- What can users do **after** this ships that they couldn't before?
- What is explicitly **out of scope**? (If unclear, scope creep will happen)
- Are there **permissions or roles** that affect who can do what?

#### 📋 Requirements Clarity
- Can each requirement be **tested**? ("User can X" vs. vague "System should be intuitive")
- Are numbers **quantified**? ("Fast" → "< 200ms", "Many" → "Up to 1000")
- Are **edge cases** defined? (Empty states, max limits, concurrent access)
- Would two developers **implement this the same way**? If not, it's ambiguous.

#### 🔄 Flows & States
- What's the **happy path**? Can you walk through it step by step?
- What happens when things **go wrong**? (Network failure, invalid input, timeout)
- Are **loading, empty, and error states** defined for UI features?
- If data changes, what **triggers** the change and what gets **notified**?

#### 🔐 Non-Functional Concerns
- Are **performance requirements** specified? (Latency, throughput, scale)
- Are **security requirements** explicit? (Auth, data protection, audit)
- Is **accessibility** addressed? (Keyboard nav, screen readers, color contrast)
- Are **data retention and privacy** rules defined?

#### 🔗 Dependencies & Integration
- What **external systems** does this touch? Are failure modes defined?
- Are **API contracts** specified? (Request/response shapes, error codes)
- What needs to exist **before** this can work? (Dependencies)
- What **breaks** if this changes? (Downstream impacts)

### The Training Effect

```
Week 1:  Checklist reveals 12 gaps    😰  "I missed so much!"
Week 5:  Checklist reveals 4 gaps     🤔  "Getting better..."
Week 10: Checklist reveals 1 gap      😊  "Almost there!"
Week 20: Checklist reveals 0 gaps     🧠  "I think in complete specs now"
```

Over time, you'll **anticipate** what the checklist asks and write specs that pass on the first try.

---

## When to Run Each

| Starting Point | Run These Workflows |
|----------------|---------------------|
| **New project** | `/speckit.constitution` → `/speckit.specify` → ... |
| **New feature (quick)** | `/speckit.specify` → `/speckit.plan` → `/speckit.tasks` → `/speckit.implement` |
| **New feature (thorough)** | `/speckit.specify` → `/speckit.clarify` → `/speckit.plan` → `/speckit.tasks` → `/speckit.analyze` → `/speckit.implement` |
| **Existing spec needs clarity** | `/speckit.clarify` |
| **Existing plan, need tasks** | `/speckit.tasks` → `/speckit.implement` |
| **Before implementation** | `/speckit.analyze` (optional quality gate) |
| **Want GitHub issues** | `/speckit.taskstoissues` (after tasks exist) |
| **Need requirement checklists** | `/speckit.checklist` (anytime after spec) |

## Workflow Categories

### Core Pipeline (Required)
1. `/speckit.specify` — Creates the feature specification
2. `/speckit.plan` — Creates technical design and architecture
3. `/speckit.tasks` — Breaks plan into actionable tasks
4. `/speckit.implement` — Executes the tasks

### Quality Gates (Optional)
- `/speckit.clarify` — Reduces ambiguity before planning (recommended)
- `/speckit.analyze` — Cross-artifact consistency check before implementation
- `/speckit.checklist` — Creates "unit tests for requirements" in any domain

### Integration (Optional)
- `/speckit.taskstoissues` — Exports tasks to GitHub Issues

### Project Setup (One-time)
- `/speckit.constitution` — Defines project principles (run once, update as needed)

## Typical Full Cycle

```bash
# 1. One-time project setup (if not done)
/speckit.constitution

# 2. Start a new feature
/speckit.specify Add user authentication with OAuth2

# 3. Clarify ambiguities (optional but recommended)
/speckit.clarify

# 4. Create requirement quality checklists (RECOMMENDED)
/speckit.checklist requirements
/speckit.checklist security   # if security-sensitive feature

# 5. 👤 HUMAN REVIEW: Check off items, fix spec gaps
#    - Review each checklist item against your spec
#    - Mark items as [x] complete or identify gaps
#    - Update spec.md to address any gaps found
#    - This is where you build the "spec designer muscle"!

# 6. Create technical plan (now based on validated spec)
/speckit.plan Using TypeScript, PostgreSQL, and JWT

# 7. Generate tasks
/speckit.tasks

# 8. Quality check (optional - catches cross-artifact issues)
/speckit.analyze

# 9. Implement!
/speckit.implement
```

## Quick Reference: Artifacts Created

| Workflow | Creates/Updates |
|----------|-----------------|
| `/speckit.constitution` | `.specify/memory/constitution.md` |
| `/speckit.specify` | `specs/<NNN>-<name>/spec.md`, `checklists/requirements.md` |
| `/speckit.clarify` | Updates `spec.md` with `## Clarifications` section |
| `/speckit.plan` | `plan.md`, `research.md`, `data-model.md`, `contracts/`, `quickstart.md` |
| `/speckit.tasks` | `tasks.md` |
| `/speckit.checklist` | `checklists/<domain>.md` |
| `/speckit.analyze` | None (read-only analysis report) |
| `/speckit.implement` | Actual code files per `tasks.md` |
| `/speckit.taskstoissues` | GitHub Issues (external) |
