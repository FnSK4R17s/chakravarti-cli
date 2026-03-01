# Organise all code related tasks under the `ckrv code` command

**Issue**: [#26](https://github.com/FnSK4R17s/chakravarti-cli/issues/26)
**Created**: 2026-02-23
**Status**: Tasks Generated

## Problem Statement

`ckrv code` should align with the **Code page workflow tabs** in `ckrv ui`, not with separate Test/QA pages.

The Code page currently represents a tight pipeline:

- Spec
- Tasks
- Plan
- Run

But the CLI exposes these as separate top-level commands (`spec`, `plan`, `run`) plus task generation under `spec tasks`, which weakens UI/CLI mental-model parity.

## Current State

### UI structure (verified)

- Code page tabs are defined as `spec | tasks | plan | run` in `frontend/src/types.ts`.
- Code page renders `SpecEditor`, `TaskEditor`, `PlanEditor`, `BarebonesExecutor` in `frontend/src/components/CodePage.tsx`.
- Test and QA are separate top-level pages in dashboard navigation (`test`, `qa`) in `frontend/src/layouts/Dashboard.tsx` and `frontend/src/App.tsx`.

### API wiring for Code page

- Spec tab APIs: `/api/specs/*` (detail, validate, clarify, design, tasks)
- Tasks tab APIs: `/api/tasks/*`, plus plan generation trigger
- Plan tab APIs: `/api/plans/*`
- Run tab APIs: `/api/execution/*` and plan generation trigger

### Important implication

`verify`, `fix`, `test`, and `qa` are **not** part of the Code page tab model. `test` and `qa` are their own pages and workflows.

## Proposed Solution

Make `ckrv code` mirror the Code page workflow only:

```bash
ckrv code <SUBCOMMAND>
```

V1 subcommands:

- `ckrv code spec ...`
- `ckrv code tasks ...`
- `ckrv code plan ...`
- `ckrv code run ...`

Optional alias for review step if desired:

- `ckrv code diff ...`

Out of scope for `ckrv code` namespace:

- `verify`, `fix`, `test`, `qa`

## User Stories

### US1: UI-to-CLI parity for Code workflow
**As a** user switching between UI and CLI,
**I want** Code page tabs to map to `ckrv code` subcommands,
**So that** I can move between interfaces without translation overhead.

### US2: Clear separation of workflows
**As a** developer,
**I want** Test/QA to remain separate from Code tab workflows,
**So that** command grouping reflects product IA rather than technical implementation details.

### US3: Non-breaking migration
**As a** team with existing scripts,
**I want** legacy top-level commands to keep working during transition,
**So that** migration can be incremental.

## Technical Approach

### Options Considered

| Option | Pros | Cons |
|--------|------|------|
| **A: `code` mirrors Code page tabs only** | Correct UI parity, clean IA | Requires explicit handling of `tasks` naming |
| **B: `code` includes all engineering commands** | Broad umbrella | Breaks parity with current UI information architecture |
| **C: Keep current flat CLI** | No migration work | Ongoing discoverability and parity mismatch |

### Decision

**Option A**: `ckrv code` maps to Code page workflow (`spec/tasks/plan/run`, optionally `diff`).

## Implementation Notes

### Command shape

Add new top-level group:

```text
ckrv code spec ...
ckrv code tasks ...
ckrv code plan ...
ckrv code run ...
```

Design note for `tasks`:

- Either implement `ckrv code tasks` as a thin alias to `ckrv spec tasks`
- Or create an explicit tasks command surface and route to existing generation logic

### Routing strategy

Use thin delegation in `crates/ckrv-cli/src/commands/code.rs`:

- `crate::spec::execute(...)` (for `spec` and possibly `tasks` alias path)
- `crate::plan::execute(...)`
- `crate::run::execute(...)`
- optionally `crate::diff::execute(...)`

### Compatibility

Keep top-level commands functional during migration:

- `ckrv spec ...`
- `ckrv plan ...`
- `ckrv run ...`
- `ckrv diff ...` (if included under `code`)

Do not re-scope `verify/fix/test/qa` under `code` in this issue.

## Open Questions

- [ ] Should V1 include `ckrv code diff`, or keep diff top-level-only?
- [ ] Should `ckrv code tasks` support only generation initially, or include future task operations?
- [ ] Should legacy commands remain visible in help or become hidden aliases later?
- [ ] Should command palette in UI gradually show `ckrv code ...` wording?

## Success Criteria

| Metric | Target |
|--------|--------|
| `ckrv code` mirrors Code page tabs (`spec/tasks/plan/run`) | Yes |
| No accidental inclusion of Test/QA commands in `code` scope | 100% |
| Existing scripts remain functional during rollout | 0 breaking changes |
| Documentation reflects UI-aligned command taxonomy | 100% updated |

## Next Steps

- [ ] Freeze V1 scope: `spec/tasks/plan/run` (+ optional `diff`)
- [ ] Draft formal spec via `/speckit.specify`
- [ ] Implement `commands/code.rs` with thin delegation
- [ ] Add CLI parsing tests for new namespace and legacy compatibility
- [ ] Update CLI docs and examples to reflect UI-aligned taxonomy

## References

- `crates/ckrv-ui/frontend/src/components/CodePage.tsx`
- `crates/ckrv-ui/frontend/src/types.ts`
- `crates/ckrv-ui/frontend/src/layouts/Dashboard.tsx`
- `crates/ckrv-ui/frontend/src/App.tsx`
- `crates/ckrv-cli/src/lib.rs`
- `crates/ckrv-cli/src/main.rs`
- `crates/docs/cli-commands.md`
