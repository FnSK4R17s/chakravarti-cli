# Iterative Planning

**Issue**: [#7](https://github.com/FnSK4R17s/chakravarti-cli/issues/7)
**Created**: 2026-02-17
**Status**: Draft

## Problem Statement

Chakravarti's planning pipeline is currently **one-shot and linear**: spec -> plan -> run. Once a plan is generated, it's static. If execution reveals problems, there's no mechanism to refine the plan mid-flight. This means:

- Plans that look good on paper but fail in practice require full re-runs
- Users can't steer execution based on intermediate results
- Failed batches trigger rollback instead of adaptation
- No learning carries forward between execution attempts

Real-world software development is inherently iterative — the plan *always* changes once you start coding. Chakravarti should embrace this.

## Current State

### The Linear Pipeline

```
spec.yaml → clarify → design.md → tasks.yaml → plan.yaml → run → done/fail
              ↑                                                    |
              └── no feedback path back ───────────────────────────┘
```

### What Exists Today

| Capability | Status |
|------------|--------|
| Plan generation from tasks | One-shot via `ckrv plan` |
| Plan resumption on interrupt | Yes — tracks batch status |
| Mid-execution plan refinement | No |
| Failure-triggered replanning | No — rolls back |
| Human review checkpoints | No |
| Feedback from execution to plan | No |
| Task partial completion tracking | No — binary pending/completed |

### Key Pain Points

1. **Plan is immutable during execution** — `ckrv run` loads plan.yaml once and never mutates it
2. **Failures are terminal** — a failed batch stops the run, no option to replan around it
3. **No intermediate feedback** — batches 1-3 might succeed but reveal that batch 4's approach is wrong
4. **Regeneration loses context** — `ckrv plan --force` starts fresh, doesn't know what was tried before

## Proposed Solution

Make the planning loop a **cycle, not a line**. After each execution phase, collect signals (success, failure, diff quality, test results) and feed them back into the planner. Allow both automatic replanning and human-in-the-loop checkpoints.

### Target Architecture

```
spec.yaml → plan.yaml → execute batch N
                ↑              |
                |         [checkpoint]
                |              |
                |    ┌─── success ──→ next batch
                |    │
                └────┤── failure ──→ replan batch N with context
                     │
                     └── user halt ──→ interactive refinement
```

## User Stories

### US1: Automatic Replanning on Failure
**As a** developer running `ckrv run`,
**I want** failed batches to trigger automatic replanning with failure context,
**So that** the system can recover without me restarting the entire run.

### US2: Checkpoint Review
**As a** developer overseeing agent work,
**I want** to pause execution between batches and review/adjust the remaining plan,
**So that** I can steer the implementation based on what I see so far.

### US3: Progressive Plan Refinement
**As a** developer iterating on a feature,
**I want** the planner to incorporate results from completed batches when planning future ones,
**So that** later tasks benefit from what was learned during earlier execution.

### US4: Plan History
**As a** developer debugging a failed run,
**I want** to see the full history of plan revisions and what triggered each change,
**So that** I can understand why the plan evolved the way it did.

## Technical Approach

### Options Considered

| Option | Pros | Cons |
|--------|------|------|
| **A: Replan-on-failure only** | Minimal changes, addresses biggest pain point | Doesn't cover proactive refinement |
| **B: Full iterative loop with checkpoints** | Comprehensive, covers all user stories | More complex, bigger change surface |
| **C: Interactive `ckrv plan` with dry-run simulation** | Users can iterate *before* execution | Doesn't help during execution |
| **D: Hybrid (A + C with optional B checkpoints)** | Incremental delivery, covers critical paths | Needs clear phase boundaries |

### Decision

**Option D: Hybrid approach** — delivers value incrementally:

1. **Phase 1**: Replan-on-failure (automatic retry with context)
2. **Phase 2**: Interactive plan review (`ckrv plan --interactive`)
3. **Phase 3**: Mid-execution checkpoints (`ckrv run --checkpoint`)
4. **Phase 4**: Plan history and audit trail

## Implementation Notes

### Phase 1: Replan-on-Failure

**Where to change**: `crates/ckrv-cli/src/commands/run.rs` (batch execution loop, ~lines 1011-1349)

Current failure flow:
```rust
// Pseudocode of current behavior
for batch in plan.batches {
    let result = execute_batch(batch).await;
    if result.is_err() {
        rollback(batch);
        return Err("batch failed");  // <-- stops here
    }
}
```

Proposed flow:
```rust
for batch in plan.batches {
    let mut attempts = 0;
    loop {
        let result = execute_batch(batch).await;
        match result {
            Ok(_) => break,
            Err(e) if attempts < max_retries => {
                let context = collect_failure_context(&e, &batch);
                batch = replan_batch(batch, context).await?;
                attempts += 1;
            }
            Err(e) => return Err(e),
        }
    }
}
```

Key additions:
- `FailureContext` struct — captures error output, failing tests, relevant diffs
- `replan_batch()` — sends failure context + original task back to Claude for a revised plan
- `max_retries` config — default 2, configurable via `--max-retries`
- Track attempt history in `plan.yaml` for each batch

### Phase 2: Interactive Plan Review

**New subcommand**: `ckrv plan --interactive` or `ckrv plan review`

Flow:
1. Display current plan.yaml in a structured format
2. Let user approve, modify, or request replanning of individual batches
3. Support `--dry-run` to show what would execute without running
4. Could use the existing UI infrastructure (`ckrv ui`) for visual plan editing

### Phase 3: Mid-Execution Checkpoints

**Config**: `checkpoint: after_each_batch | after_phase | never` in spec or plan

At each checkpoint:
1. Pause execution
2. Show batch results summary (files changed, tests, warnings)
3. Present remaining plan
4. User can: continue, skip batch, reorder, add tasks, abort
5. Terminal UI or web UI (via `ckrv ui`) for the review interface

### Phase 4: Plan History

- Store plan revisions as `plan.v1.yaml`, `plan.v2.yaml`, etc. (or a single file with revision history)
- Each revision records: trigger (failure/user/auto), timestamp, what changed, why
- `ckrv plan history` command to view the revision log
- Useful for debugging and understanding agent decision-making

### Data Model Changes

```yaml
# plan.yaml additions
metadata:
  version: 2          # bump for new format
  revision: 3         # incremented on each replan
  history:
    - revision: 1
      trigger: "initial"
      timestamp: "2026-02-17T10:00:00Z"
    - revision: 2
      trigger: "batch_failure"
      batch_id: "batch-3"
      context: "test_api_auth failed: missing middleware"
      timestamp: "2026-02-17T10:15:00Z"

batches:
  - id: "batch-3"
    status: "pending"
    attempts: 1        # NEW: track retry count
    last_failure:      # NEW: context from last failure
      error: "..."
      failing_tests: [...]
```

### Configuration

```yaml
# In spec.yaml or .chakravarti/config.yaml
planning:
  max_retries: 2              # per-batch retry limit
  checkpoint: after_each_batch  # never | after_each_batch | after_phase
  auto_replan: true            # attempt automatic replanning on failure
  require_approval: false      # require human approval for replanned batches
```

## Open Questions

- [ ] Should replanning be limited to the failing batch, or can it restructure remaining batches too?
- [ ] How much failure context is useful to send to the LLM without overwhelming it?
- [ ] Should checkpoint pauses have a timeout (auto-continue after N minutes)?
- [ ] How does this interact with `ckrv fix`? Is fix a micro-replan within a batch?
- [ ] Should plan history be stored in the spec dir or in a separate `.chakravarti/history/` location?
- [ ] Cost implications — replanning means additional LLM calls. Should there be a budget cap?

## Success Criteria

| Metric | Target |
|--------|--------|
| Failed batches auto-recovered via replan | > 50% of recoverable failures |
| Runs requiring full restart after failure | < 20% (down from 100%) |
| User can review/modify plan before execution | Phase 2 complete |
| Mid-execution steering available | Phase 3 complete |
| Full plan revision history accessible | Phase 4 complete |

## Next Steps

- [ ] Validate approach against current `run.rs` execution loop
- [ ] Prototype `FailureContext` collection from a real failed batch
- [ ] Define the replan prompt template (what context does Claude need?)
- [ ] Decide on Phase 1 scope boundaries
- [ ] Create spec via `/speckit.specify` when ready

## References

- Current plan command: `crates/ckrv-cli/src/commands/plan.rs`
- Current run command: `crates/ckrv-cli/src/commands/run.rs` (batch loop ~L1011-1349)
- Plan data model: `crates/ckrv-core/src/plan.rs`
- Planner trait (unused): `crates/ckrv-core/src/planner.rs`
