# Add Mistral Vibe

**Issue**: [#29](https://github.com/FnSK4R17s/chakravarti-cli/issues/29)
**Created**: 2026-02-23
**Status**: Ready for Spec

## Problem Statement

Issue #29 requests support for “Mistral Vibe” (referencing Mistral Code). Today, users can run Claude/Codex/Kilo workflows, but there is no first-class, low-friction “Mistral Vibe” path in the default agent lineup. This creates onboarding friction for users who want to use Mistral models as part of orchestration.

## Current State

- chakravarti-cli currently supports:
  - Claude Code (native / OpenRouter / GLM)
  - Codex
  - Kilo Code (multi-provider)
- Mistral-family models may be reachable indirectly (e.g., via Kilo/OpenRouter), but there is no explicit “Mistral Vibe” guided setup in issue context.
- User intent from issue body is minimal (single Mistral link), so scope must be inferred and clarified.

## Proposed Solution

Deliver “Mistral Vibe” as a **configuration-first integration** (not a new orchestration architecture):

1. Add a documented, tested agent profile that enables Mistral Code usage through existing execution interfaces.
2. Prefer the smallest implementation that aligns with product vision (“orchestration layer, not another coding agent”).
3. Ensure users can select Mistral-capable agents from CLI/UI without ambiguity.

## User Stories

### US1: Quick Mistral Onboarding
**As a** user with Mistral credentials,
**I want** a clear way to configure a Mistral-capable agent in ckrv,
**So that** I can run specs with Mistral quickly.

### US2: Agent Routing Flexibility
**As a** power user with multiple providers,
**I want** to map certain task classes to a Mistral-capable agent,
**So that** I can optimize speed/cost/quality per batch.

### US3: Predictable UX
**As a** UI user,
**I want** Mistral-related options to be explicit and documented,
**So that** I avoid trial-and-error setup.

## Technical Approach

### Option A — Kilo-based Mistral Profile (Recommended)
Use existing `kilo_code` provider and add a documented/templated Mistral model profile.

**Pros**
- No new sandbox provider implementation required
- Fastest path to user value
- Aligned with existing multi-provider architecture

**Cons**
- Depends on Kilo provider capabilities and user’s Kilo setup
- “Mistral Vibe” branding may still feel indirect

### Option B — OpenRouter-backed Mistral Profile
Use existing Claude/OpenRouter pathway with curated Mistral model defaults.

**Pros**
- Reuses existing OpenRouter integration
- Potentially easier for users already on OpenRouter

**Cons**
- Not necessarily “native” Mistral feel
- Requires model compatibility validation per route

### Option C — New Native `mistral_*` Agent Type
Implement dedicated provider in `ckrv-sandbox`.

**Pros**
- Most explicit first-class support
- Stronger product signal for issue ask

**Cons**
- Highest implementation and maintenance cost
- Risks violating non-goal (becoming yet another agent adapter layer beyond practical need)

### Decision

Proceed with **Option A** first (Kilo-based Mistral profile), with Option B as fallback where suitable. Defer Option C unless issue acceptance criteria explicitly require a new native provider binary.

## Implementation Notes

- Keep changes primarily in:
  - Agent configuration templates/examples
  - Docs (agent guide + getting started snippets)
  - UI labels/help text where agent setup is exposed
- Validate that “Mistral Vibe” path works end-to-end for:
  - CLI-run execution selection
  - Planner/executor model mapping
  - Optional UI agent management
- Avoid heavy architectural churn; this is an integration + DX improvement issue.

## Open Questions

- [ ] Does issue #29 require a **new native provider** or just a reliable Mistral-capable workflow?
- [ ] Which concrete model IDs should be blessed as defaults for “Mistral Vibe”?
- [ ] Should “Mistral Vibe” be exposed as a named preset in UI and sample config?
- [ ] Are there authentication/env conventions unique to Mistral Code we must standardize?

## Success Criteria

| Metric | Target |
|--------|--------|
| Time to first successful Mistral-backed run | <= 10 minutes from fresh setup |
| New code surface in sandbox providers | Minimal (prefer zero new provider) |
| Documentation clarity | One copy-paste configuration path + verification step |
| User-facing parity | Mistral-capable agent selectable in same places as other agents |

## Next Steps

- [ ] Confirm expected scope in issue comments (profile vs native provider).
- [ ] Draft spec from this brainstorm focused on Option A.
- [ ] Add acceptance test checklist for Mistral-backed run path.
- [ ] If native provider is mandated, create follow-up spike spec.

## References

- https://github.com/FnSK4R17s/chakravarti-cli/issues/29
- https://mistral.ai/news/mistral-code
- `crates/docs/agent-guide.md`
- `guiding_docs/vision.md`
