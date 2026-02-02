# Chakravarti-cli Vision Document

*Version 2.0 | February 2026*

---

## What Is Chakravarti-cli?

**Chakravarti-cli (`ckrv`) is a cross-platform orchestration engine for AI coding agents.**

You write specifications. `ckrv` coordinates multiple AI agents—Claude Code, Codex, Gemini, and others—to implement them in parallel.

Here's what makes this possible: the companies behind these tools will never build cross-provider support themselves. Anthropic won't help you use Codex. OpenAI won't integrate Claude. Their business models require lock-in.

Chakravarti-cli lives in the gap between their incentives. It's the only tool that lets you use all your AI subscriptions together.

---

## Who It's For

You're the ideal `ckrv` user if you:

- Have hit Claude Code's rate limits and been frustrated
- Pay for 2+ AI coding subscriptions you can't use simultaneously  
- Are technical enough to run Docker but don't want to build orchestration infrastructure from scratch
- Want "fire and forget" execution—kick off work, walk away, come back to review

You're a solo founder or senior IC who understands that bigger models excel at planning while smaller, faster models can execute routine tasks efficiently. You want to match the right agent to the right job.

Today, you switch between tools manually. You can't parallelize. You're bottlenecked by whichever single agent you're using at the moment.

You discover `ckrv` and think: *"Finally, someone built this."*

---

## How It Works

1. **You write a spec.** Plain language description of the feature you want. This is the source of truth—AI interprets, humans decide.

2. **The planner breaks it into tasks.** Claude Opus analyzes your codebase and spec, identifies dependencies, and groups tasks into executable batches.

3. **Tasks are assigned by skill level.** Complex integration work goes to L5 agents (state-of-the-art models). Routine changes go to L3 or L4 (cheaper, faster models). You configure the mapping based on your available subscriptions.

4. **Agents execute in isolation.** Each batch runs in a Docker container on its own git worktree. No interference between agents. No risk to your main branch.

5. **Results merge automatically.** The planner handles merge conflicts and dependency ordering. You review the final PR.

The entire workflow is git-native. Branches, worktrees, PRs, code review—`ckrv` accelerates your existing process rather than replacing it.

---

## What Makes It Different

Cursor, Windsurf, Aider, Claude Code, Codex, Devin, OpenHands, Cline—they all offer agents. They're individual contractors.

**`ckrv` is the general contractor** who coordinates the electrician, plumber, and carpenter on the same job site.

**Why can't incumbents just build this?**

Anthropic won't build orchestration that routes tasks to Codex. OpenAI won't help you use Claude and vice versa. Their incentives forbid cross-provider support.

`ckrv` treats agents as interchangeable workers behind a CLI interface. We invoke the same commands you'd type yourself—`claude`, `codex`, `opencode`. Providers can't lock this down without breaking their own power users.

**The planning stage is the core intelligence.** You interact with `ckrv`, refine your spec, and the planner determines how to split work across all the agents you've configured. The orchestration layer is commodity infrastructure. The planning layer is where the value lives.

---

## The Outcome

**Ten features a day.**

Start a spec over morning coffee. Review the PR by lunch. Start another. By end of day, you've shipped what used to take a week.

When you return from a run: your feature is implemented, tested against existing tests, with new tests written for the generated code. You review the PR, assess code quality, and merge.

Then you work on the next spec. And the next.

You move from implementation to oversight, managing the results instead of guiding an agent through the process.

---

## Business Model

**Open-core.**

The orchestration engine is free and open source (MIT licensed). Single-spec execution with full planner capabilities, local Docker execution, unlimited use.

Cloud hosting is available for users who need more:

- **Multi-spec parallelism.** Running multiple specs simultaneously requires cloning repos into separate directories, managing independent git states, and coordinating merges across runs. The cloud handles this complexity.
- **No local Docker required.** Run specs from any machine without configuring containers locally. This opens the tool to users who want to plan out their work while traveling or when they don't have access to a local machine.
- **Managed infrastructure.** We handle compute, storage, and cleanup.

The open-source version proves the value. The cloud removes friction for users who want to scale.

---

## Dependencies & Risks

What has to be true for `ckrv` to work:

**CLI access remains open.** `ckrv` invokes agents via CLI. If providers lock down CLI access or require frequent re-authentication for all usage, the model becomes more complex. Current assessment: unlikely—CLI access is how their power users work, and breaking it would be self-destructive.

**Repos benefit from being agent-ready.** Poorly documented codebases produce inconsistent results. `ckrv` works best with inline documentation, clear module boundaries, and explicit configuration (AGENTS.md, CLAUDE.md). This is an industry-wide challenge as codebases adapt to agentic workflows.

**Users must trust AI-generated code enough to review rather than co-author.** "Fire and forget" requires confidence in the planner and the agents. Some developers aren't there yet—but the ones who are move significantly faster.

---

## Product Principles

### Specs Are the Source of Truth

Everything starts from a human-written specification. The spec solves the cold start problem and gives agents the context they need to work effectively. No spec, no execution.

### Fire and Forget

Most AI coding tools assume pair programming—you and the AI, working together in real-time. `ckrv` is designed for a different model: kick off work, walk away, come back to review. Like a dream employee who takes your briefing, assembles their team, and delivers results.

### Isolation Is Safety

All agent execution happens in Docker sandboxes on isolated git worktrees. Once the spec is right, agents work without direct access to production systems or your main branch. Safety through architecture.

### Human in the Loop (Where It Matters)

Human oversight is required at the spec generation stage and at PR review. Agents handle implementation. You handle intent and approval.

### Git-Native

Works with existing workflows: branches, worktrees, PRs, code review. `ckrv` doesn't replace your development process—it accelerates it.

---

## Non-Goals

**Chakravarti-cli is not another coding agent.**

It operates at the orchestration layer, not the execution layer. Use whatever agents you already have. `ckrv` coordinates them.

This is a feature, not a limitation. Chakravarti-cli is complementary to every AI coding tool, not competitive with them.

---

## The Bigger Bet

You're already paying for multiple AI coding tools. You can't use them together. That's the immediate problem `ckrv` solves.

The larger thesis: as AI coding matures, orchestration becomes more valuable than any single agent. The developer who manages 10 agents outperforms the developer married to one.

The trajectory is clear. Every developer will transition from writing code to guiding agents. We've already taken the first steps—nobody writes code character by character anymore. Models like Claude can one-shot most feature requests today.

`ckrv` is the tool that makes this future practical now.

---

## The Name

### Meaning

*Chakravarti* is a Sanskrit term for emperors who ruled the entire Indian subcontinent—Ashoka was a Chakravarti. The literal meaning is "one who controls all directions," the full 360-degree spectrum.

The Chakravarti's role was not to do the work of kings, but to orchestrate their efforts across the realm. The symbol of the Chakravarti is the *Dharma Chakra*—the wheel that spins relentlessly through good times and bad.

In our context: agents are kings. You are the Chakravarti, orchestrating the empire.

### Official Names

| Name | Usage |
|------|-------|
| **chakravarti-cli** | Full project name. Use in documentation, GitHub, package registries. |
| **ckrv** | CLI command and shorthand. Use in code examples, terminal output, casual references. |
| ~~chakravarti~~ | Only use when explaining the name's meaning (above). Never as standalone product name. |

**Examples:**
- ✅ "Install chakravarti-cli with `npm install -g @chakravarti/cli`"
- ✅ "Run `ckrv init` to set up your project"
- ✅ "The name *Chakravarti* means 'one who controls all directions'"
- ❌ "Chakravarti orchestrates agents" → Use "chakravarti-cli orchestrates agents" or "ckrv orchestrates agents"

### The Alias

`ckrv` is the official CLI command—a memorable shorthand that:

- **Avoids typos**: "chakravarti" is 11 characters with tricky spelling. `ckrv` is 4.
- **Types fast**: No mental overhead when running commands repeatedly.
- **Sounds right**: Pronounced "check-rev" or "see-kay-are-vee"—both work.

The alias is installed alongside the full binary. Both `ckrv` and `chakravarti-cli` resolve to the same executable.

---

## Origin Story

Chakravarti-cli was born from a simple frustration: having multiple AI coding subscriptions but no way to use them together.

The workflow already existed—spec-driven development with tasks marked for parallel execution. But no tool could orchestrate multiple agents working on different tasks simultaneously. Cursor couldn't use Gemini. Claude Code couldn't use Codex. Each agent was siloed.

The research led to key insights: containerization and git worktrees for isolation, visual task management for planning, and the general need for a control plane that treats AI agents as interchangeable workers rather than singular assistants.

Chakravarti-cli exists to maximize the spec-driven development paradigm—write the specification once, let all your agents implement it together.

---

## Vision Statement

> *"Chakravarti-cli orchestrates AI agents to transform specifications into production-ready code. You write the spec. Your agents implement it. Together."*

---

*Document Status: Complete*
