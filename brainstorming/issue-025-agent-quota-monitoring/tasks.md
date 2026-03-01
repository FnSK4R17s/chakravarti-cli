# Tasks — Issue 025: Per-Agent Usage Quota Monitoring

Source: [notes.md](./notes.md)

---

## Task 1: Add `agent_type` field to `Metrics` struct

**Crate**: `ckrv-metrics`
**Files**: `crates/ckrv-metrics/src/lib.rs` (or wherever `Metrics` is defined)
**Depends on**: —

- Add `pub agent_type: Option<String>` to the `Metrics` struct with `#[serde(default)]` for backward compatibility.
- Ensure existing `.chakravarti/runs/{job_id}/metrics.json` files deserialize without error when `agent_type` is absent.
- Update `MetricsSummary` (display type) to include agent information.
- Add a unit test: deserialize a metrics JSON without `agent_type` field and confirm it loads as `None`.
- Add a unit test: serialize/deserialize round-trip with `agent_type` set.

---

## Task 2: Pipe agent type from `ckrv run` through to metrics collection

**Crate**: `ckrv-core`, `ckrv-cli`
**Files**: `crates/ckrv-core/src/orchestrator.rs`, `crates/ckrv-cli/src/commands/run.rs`
**Depends on**: Task 1

- Pass the `--agent` value from `RunArgs` through the orchestrator to `MetricsCollector`.
- When `MetricsCollector::finish_job()` produces a `Metrics`, set `agent_type` from the agent used.
- Confirm that saved `metrics.json` now includes `"agent_type": "claude"` (or whichever agent ran).
- Add a test: run metrics collection with agent type set, save, reload, verify field persists.

---

## Task 3: Add per-agent aggregation helpers to `ckrv-metrics`

**Crate**: `ckrv-metrics`
**Files**: `crates/ckrv-metrics/src/lib.rs` (or storage module)
**Depends on**: Task 1

- Add a function to aggregate a `Vec<Metrics>` grouped by `agent_type` — returning a map of agent name to summary (jobs, tokens, cost).
- Add a function to filter `Vec<Metrics>` by date range (requires adding a timestamp field to `Metrics` if not already present; use `finished_at: Option<chrono::DateTime<Utc>>` with `#[serde(default)]`).
- Add a function to compute period boundaries (daily/weekly/monthly) given a `QuotaPeriod` and current UTC time.
- Unit tests:
  - Aggregation with mixed agent types, including `None` (mapped to "unknown").
  - Date-range filtering with boundary conditions.
  - Period boundary calculation for each `QuotaPeriod` variant.

---

## Task 4: Extend `ckrv usage` command with agent and time filters

**Crate**: `ckrv-cli`
**Files**: `crates/ckrv-cli/src/commands/usage.rs`
**Depends on**: Task 3

- Add CLI flags to the existing `usage` command:
  - `--agent <name>` — filter results to a single agent type.
  - `--since <YYYY-MM-DD>` — filter to jobs after this date.
  - `--period <daily|weekly|monthly>` — filter to current period.
  - `--by-agent` — group output by agent instead of by model.
- When `--by-agent` is set, render a table with columns: Agent, Jobs, Tokens, Cost.
- Ensure `--json` output includes the agent grouping when `--by-agent` is used.
- Ensure existing behavior is unchanged when no new flags are provided.
- Add `long_about` and `after_help` text for the new flags per RUST_CONVENTIONS.md.

---

## Task 5: Add `AgentQuota` type and YAML storage

**Crate**: `ckrv-metrics`
**Files**: New file or extend existing storage module
**Depends on**: —

- Define types:
  - `AgentQuota { agent: String, max_tokens: Option<u64>, max_cost_usd: Option<f64>, period: QuotaPeriod }`
  - `QuotaPeriod { Daily, Weekly, Monthly, Total }` (with serde support)
  - `QuotaConfig { quotas: Vec<AgentQuota> }` (top-level for YAML file)
- Implement load/save for `.chakravarti/quotas.yaml`.
- Implement CRUD: `set_quota`, `remove_quota`, `get_quota`, `list_quotas`.
- Handle missing file gracefully (return empty config).
- Unit tests:
  - Round-trip YAML serialization.
  - Set, get, remove, list operations.
  - Load from nonexistent file returns empty config.

---

## Task 6: Add `ckrv quota` subcommand

**Crate**: `ckrv-cli`
**Files**: New `crates/ckrv-cli/src/commands/quota.rs`, update `crates/ckrv-cli/src/commands/mod.rs`
**Depends on**: Task 3, Task 5

- Add `ckrv quota` with subcommands:
  - `ckrv quota` (no subcommand) — show all quotas with current usage against limits.
  - `ckrv quota set <agent> --max-cost <usd> --max-tokens <n> --period <period>` — create/update a quota.
  - `ckrv quota check <agent>` — show remaining quota for one agent.
  - `ckrv quota remove <agent>` — delete a quota.
- Default output: table with columns Agent, Limit, Used, Remaining, Period, Status.
- Support `--json` flag for structured output (if consistent with existing conventions).
- Add `long_about` and `after_help` for the command and each subcommand per RUST_CONVENTIONS.md.
- Register the command in the main CLI dispatch.

---

## Task 7: Add quota pre-flight check to `ckrv run`

**Crate**: `ckrv-cli`, `ckrv-core`
**Files**: `crates/ckrv-cli/src/commands/run.rs`
**Depends on**: Task 2, Task 6

- Before job execution in `ckrv run`:
  1. Load quota config for the selected agent.
  2. Load metrics for the current period.
  3. If usage > 80% of limit: print a warning but continue.
  4. If usage >= 100% of limit: print an error and exit, unless `--force` is passed.
- Add `--force` flag to `ckrv run` to override quota limits.
- Update `long_about` and `after_help` for `run` command to document the new flag.
- Test: quota exceeded blocks run without `--force`; `--force` overrides the block.

---

## Dependency Graph

```
Task 1 (Metrics field)
  ├──> Task 2 (Pipe agent type)
  └──> Task 3 (Aggregation helpers)
            └──> Task 4 (usage flags)
            └──> Task 6 (quota command) <── Task 5 (quota storage)
                       └──> Task 7 (run pre-flight) <── Task 2
Task 5 (quota storage) — independent
```

## Verification

After all tasks are complete:
- `just build` succeeds with no warnings.
- `just test` passes (including new tests).
- `just lint` passes.
- `ckrv usage --by-agent` shows agent-grouped output.
- `ckrv quota set claude --max-cost 50 --period monthly` persists to quotas.yaml.
- `ckrv quota` displays current status.
- `ckrv run` with exceeded quota blocks unless `--force` is used.
