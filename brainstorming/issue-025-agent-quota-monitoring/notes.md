# Per-Agent Usage Quota Monitoring

**Issue**: [#25](https://github.com/FnSK4R17s/chakravarti-cli/issues/25)
**Created**: 2026-03-01
**Status**: Draft

## Problem Statement

Chakravarti orchestrates multiple AI agents (Claude, Codex, KiloCode) but provides no per-agent visibility into usage. The existing `ckrv usage` command aggregates metrics across all jobs and breaks down by model name — but not by agent type. Users cannot answer questions like "How much have I spent on Codex this week?" or "Which agent consumes the most tokens?" There is also no mechanism to set spending or token limits per agent to prevent runaway costs.

## Current State

### What exists today

- **`ckrv usage`** — Aggregates all jobs into a single summary: total jobs, tokens, cost, duration. Breaks down by model name but not by agent type. Supports `--detailed` (per-job list) and `--json` flags.
- **`ckrv report <job-id>`** — Shows metrics for a single job (hidden command). Does not indicate which agent executed it.
- **`ckrv-metrics` crate** — Collects `Metrics` per job (job_id, spec_id, total_time_ms, token_usage, cost, step_metrics, retry_count, success). Stores JSON at `.chakravarti/runs/{job_id}/metrics.json`. Has built-in pricing for 8 models (gpt-4o, gpt-4o-mini, gpt-4-turbo, gpt-3.5-turbo, claude-3-5-sonnet, claude-3-5-haiku, claude-3-opus) with prefix-matching fallback.
- **Cloud quota** — `job_quota_remaining` tracked server-side in `User` struct. `CloudQuotaExceeded` error with reset_time and upgrade_url. No local equivalent.
- **Agent types** — `AgentType` enum: Claude, Codex, KiloCode. Each has a provider implementing the `AgentProvider` trait.

### Pain points

1. No way to see usage broken down by agent (Claude vs Codex vs KiloCode).
2. No local spending or token limits — a misconfigured loop can burn through API credits.
3. `Metrics` struct does not record which agent executed the job.
4. No time-windowed queries (e.g., "usage this week" or "usage this billing cycle").
5. Model-based breakdown is an imperfect proxy — Claude uses multiple models, so does OpenRouter.

## Proposed Solution

Add per-agent usage tracking and quota monitoring to the CLI. Three changes:

1. **Record the agent type in job metrics** so usage can be attributed to a specific agent.
2. **Extend `ckrv usage` with agent filtering and time windows** for per-agent visibility.
3. **Add a `ckrv quota` command** for setting and checking local per-agent usage limits.

## User Stories

### US1: View per-agent usage
**As a** developer using multiple agents,
**I want** to see usage broken down by agent,
**So that** I can understand cost distribution and optimize my agent selection.

### US2: Filter usage by time window
**As a** developer tracking monthly spend,
**I want** to filter usage by date range,
**So that** I can see costs for the current billing cycle.

### US3: Set per-agent spending limits
**As a** developer with API budget constraints,
**I want** to set a maximum token or dollar limit per agent,
**So that** jobs are blocked before exceeding my budget.

### US4: Check quota status before running
**As a** developer about to run a job,
**I want** to see how much quota remains for my selected agent,
**So that** I can decide whether to proceed or switch agents.

## Technical Approach

### Options Considered

| Option | Pros | Cons |
|--------|------|------|
| A: Extend existing `Metrics` + `ckrv usage` only | Minimal new surface area; builds on proven storage | No quota/limit capability |
| B: New `ckrv quota` command + separate quota store | Clean separation of concerns; quota logic is isolated | Two storage locations for related data; no improved visibility |
| C: Combine A+B — extend metrics, add quota subcommand | Full coverage of both visibility and limits | More work, but each piece is small and independent |

### Decision

**Option C** — Extend `Metrics` to include agent type (for attribution), enhance `ckrv usage` with agent/time filters (for visibility), and add `ckrv quota` for limit management (for cost control). Each piece is small and self-contained.

### Data model changes

**`Metrics` struct** — Add field:
```rust
pub agent_type: Option<String>,  // "claude", "codex", "kilo"
```

`Option<String>` for backward compatibility — existing metrics files deserialize with `agent_type: None`.

**`AgentQuota`** — New type in `ckrv-metrics`:
```rust
pub struct AgentQuota {
    pub agent: String,            // "claude", "codex", "kilo"
    pub max_tokens: Option<u64>,
    pub max_cost_usd: Option<f64>,
    pub period: QuotaPeriod,      // Daily, Weekly, Monthly, Total
}

pub enum QuotaPeriod {
    Daily,
    Weekly,
    Monthly,
    Total,
}
```

Stored at `.chakravarti/quotas.yaml` (consistent with other Chakravarti YAML config).

### CLI surface

**Enhanced `ckrv usage`:**
```
ckrv usage                          # Existing behavior (unchanged default)
ckrv usage --agent claude           # Filter to Claude jobs only
ckrv usage --agent codex --detailed # Codex jobs with per-job breakdown
ckrv usage --since 2026-02-01       # Jobs since date
ckrv usage --period monthly         # Current calendar month
ckrv usage --by-agent               # Summary table grouped by agent
```

**New `ckrv quota`:**
```
ckrv quota                          # Show all quotas and current usage against limits
ckrv quota set claude --max-cost 50 --period monthly
ckrv quota set codex --max-tokens 1000000 --period weekly
ckrv quota remove claude            # Remove quota for agent
ckrv quota check claude             # Check remaining quota for one agent
```

### Enforcement

- Before `ckrv run` executes, check if the selected agent has a quota configured.
- If current-period usage exceeds 80% of limit, warn the user.
- If current-period usage already exceeds the limit, block execution with a clear error and suggest `--force` to override.
- Enforcement is advisory (warn + `--force` override), not hard-blocking — the user always retains control.

### Crate responsibilities

| Crate | Change |
|-------|--------|
| `ckrv-metrics` | Add `agent_type` to `Metrics`; add `AgentQuota` type + YAML storage; add period-filtered aggregation helpers; add per-agent aggregation |
| `ckrv-cli` | Add `--agent`, `--since`, `--period`, `--by-agent` flags to `usage` command; add `quota` subcommand with `set`/`check`/`remove` subcommands; add `long_about` and `after_help` for new commands |
| `ckrv-core` | Pass agent type from `RunArgs` through orchestrator to `MetricsCollector` during job execution |

## Implementation Notes

- Existing `.chakravarti/runs/{job_id}/metrics.json` files deserialize with `agent_type: None` via `#[serde(default)]` — no migration needed.
- Quota config at `.chakravarti/quotas.yaml` is created on first `ckrv quota set`.
- Period calculation uses UTC dates for consistency.
- `--by-agent` output format:

```
USAGE BY AGENT
  Agent       Jobs   Tokens     Cost
  claude        12   45,000   $1.23
  codex          5   20,000   $0.45
  kilo           3   10,000   $0.12
  TOTAL         20   75,000   $1.80
```

- `ckrv quota` output format:

```
QUOTA STATUS
  Agent    Limit         Used      Remaining   Period    Status
  claude   $50.00/mo     $12.34    $37.66      monthly   OK
  codex    1M tokens/wk  450,000   550,000     weekly    OK
```

- Every new CLI command/subcommand must have `long_about` and `after_help` attributes per `RUST_CONVENTIONS.md`.
- Jobs from before this feature will show `agent: unknown` in `--by-agent` output.

## Open Questions

- [ ] Should quota warnings appear automatically in `ckrv run` output, or only when explicitly checking with `ckrv quota check`?
- [ ] Should `ckrv usage --by-agent` become the default output format, or keep the current model-based breakdown as default?
- [ ] Should quotas support per-model limits in addition to per-agent limits?

## Success Criteria

| Metric | Target |
|--------|--------|
| `ckrv usage --agent <name>` returns filtered results | Works for all 3 agent types |
| `ckrv usage --by-agent` shows agent-grouped summary | Correct aggregation |
| `ckrv usage --since` and `--period` filter by time | Correct date filtering |
| `ckrv quota set/check/remove` manages local quotas | CRUD works, persists in quotas.yaml |
| Quota enforcement warns before `ckrv run` | Warning at >80%, block at >100%, `--force` overrides |
| Backward compat with existing metrics | Old metrics.json loads with `agent_type: None` |
| All new commands have `long_about` + `after_help` | Per RUST_CONVENTIONS.md |

## Non-goals

- Billing/payment automation
- Background daemon or monitoring service
- New provider integrations beyond existing agents
- Querying remote provider APIs for quota data (this is local tracking only)

## References

- [ckrv-metrics crate](../../crates/ckrv-metrics/)
- [CLI commands doc](../../crates/docs/cli-commands.md)
- [Existing usage command](../../crates/ckrv-cli/src/commands/usage.rs)
- [Agent provider trait](../../crates/ckrv-sandbox/src/agent/mod.rs)
- [Rust conventions](../../crates/RUST_CONVENTIONS.md)
