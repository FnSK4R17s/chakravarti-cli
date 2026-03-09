# Rust Build Time Optimization

**Created**: 2026-03-09
**Status**: Tasks Generated

## Problem Statement

As CKRV grows (13 workspace crates, 3 final binaries), build times are creeping up. Per matklad's guidance: build times are a multiplier for everything, including build times themselves. Small regressions go unnoticed until they compound. We need to establish a baseline, apply best practices, and set up guardrails to prevent regression.

This brainstorm is informed by matklad's "Fast Rust Builds" article (saved in our NotebookLM knowledge base) and a full audit of the codebase.

## Current State

### Build Timing Baseline (release, `-p ckrv-cli`)

| Crate | Duration | Role |
|-------|----------|------|
| **ckrv-transport** | 11.15s | API types + handlers (heaviest) |
| **ckrv-cli** | 10.45s + 10.28s | Entry point (compiled twice: lib + bin) |
| **ckrv-sandbox** | 6.00s | Docker/agent providers |
| **ckrv-ui** | 5.84s | Web dashboard server |
| **git2** | 4.57s | External dep (C bindings) |
| **openssl** | 4.32s | External dep (vendored C) |
| **ckrv-core** | 3.32s | Orchestration engine (9 dependents) |
| **openssl-sys** | 3.11s | External dep (C build) |
| **reqwest** | 3.08s | HTTP client |
| **ckrv-metrics** | 2.33s | Cost/time tracking |
| **ckrv-git** | 0.68s | Git operations |
| **ckrv-verify** | 0.63s | Test pipeline |
| **ckrv-spec** | 0.58s | Spec parsing |

**Top 3 bottlenecks**: ckrv-transport (11.15s), ckrv-cli (10.45s), ckrv-sandbox (6.00s)

### What's Already Working Well

- **CI caching**: `Swatinem/rust-cache@v2` with per-job keys (clippy, test, build)
- **Warning handling**: `RUSTFLAGS="-D warnings"` in CI, not `#![deny(warnings)]` in code
- **Workspace lints**: Centralized clippy/rust rules in root Cargo.toml
- **Trait objects**: Only 2 uses (AgentProvider factory, EventHandler) — minimal, justified
- **No custom proc macros**: Zero custom derives — clean external-only approach
- **Layer 0 parallelism**: ckrv-git, ckrv-sandbox, ckrv-metrics have zero inter-workspace deps

### Dependency Graph

```
Layer 0 (parallel):  ckrv-git, ckrv-sandbox, ckrv-metrics
Layer 1 (core):      ckrv-core → ckrv-sandbox
Layer 2 (features):  ckrv-spec → ckrv-core, ckrv-verify → {ckrv-core, ckrv-sandbox}
Layer 3 (transport): ckrv-transport → {ckrv-core, ckrv-sandbox, ckrv-git, ckrv-metrics}
Layer 4 (apps):      ckrv-ui → ckrv-transport, ckrv-tauri → ckrv-transport
Layer 5 (entry):     ckrv-cli → {7 crates}, ckrv-mcp → ckrv-cli
```

Central bottleneck: ckrv-core (3.32s) blocks 9 downstream crates.

## Issues Found

### Issue 1: No Cargo Profiles (5 min fix, HIGH impact)

**Problem**: Zero `[profile]` sections in root Cargo.toml. All builds use Rust defaults.

**Impact**: Dependencies recompile unoptimized in dev mode. No strip/LTO for release.

**Fix**:
```toml
[profile.dev]
debug = 1                    # Reduced debuginfo (default is 2)

[profile.dev.package."*"]
opt-level = 2                # Optimize deps — compiled once, cached forever

[profile.release]
lto = "thin"                 # Faster linking than full LTO
codegen-units = 1            # Better optimization
strip = true                 # Smaller binaries
```

**Why it matters**: `[profile.dev.package."*"] opt-level = 2` is the single highest-impact one-liner for Rust build times. Deps compile with optimizations once and are cached by cargo. Your own code stays at opt-level 0 for fast iteration.

---

### Issue 2: No Linker Configuration (10 min fix, HIGH impact)

**Problem**: No `.cargo/config.toml`. Using default `ld` linker, which is slow.

**Impact**: Linking phase takes significantly longer than necessary.

**Fix**: Install `mold` and create `.cargo/config.toml`:
```toml
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]
```

**Consideration**: This only affects Linux/WSL2. For cross-platform, we can also add:
```toml
[target.x86_64-apple-darwin]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]

[target.aarch64-apple-darwin]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]
```

**CI note**: `mold` is available on `ubuntu-latest` via `apt install mold`, or we skip it in CI and only optimize local builds.

---

### Issue 3: CARGO_INCREMENTAL Not Disabled in CI (5 min fix, MEDIUM impact)

**Problem**: CI builds are from-scratch. Incremental compilation adds overhead and inflates `./target` (hurts cache hit rate).

**Current CI** (`ci.yml`):
```yaml
env:
  CARGO_TERM_COLOR: always
  RUSTFLAGS: "-D warnings"
  # Missing: CARGO_INCREMENTAL: 0
```

**Fix**: Add to CI env:
```yaml
env:
  CARGO_TERM_COLOR: always
  RUSTFLAGS: "-D warnings"
  CARGO_INCREMENTAL: 0
```

**Also consider**: `cargo test --no-run` + `cargo test` split to separate build from test timing (per matklad).

---

### Issue 4: Triple reqwest Version Compilation (30 min fix, HIGH impact)

**Problem**: Cargo.lock contains THREE versions of reqwest:
- `reqwest 0.11.27` — workspace default (used by ckrv-model, ckrv-integrations)
- `reqwest 0.12.28` — overridden in ckrv-cli, ckrv-ui, ckrv-transport
- `reqwest 0.13.1` — pulled transitively by tauri-plugin-updater/shell

**Impact**: The entire HTTP stack compiles 2-3 times:
- `hyper` 0.14 AND 1.8 (two completely different major versions)
- `http` 0.2 AND 1.4
- `h2` 0.3 AND 0.4
- `http-body` 0.4 AND 1.0
- Plus all their transitive deps

**Fix**:
1. Update workspace default from 0.11 to 0.12
2. Remove version overrides from ckrv-cli, ckrv-ui, ckrv-transport
3. Accept 0.13 from Tauri as unavoidable (transitive, feature-gated behind `ckrv-tauri`)
4. Result: 1 reqwest for CLI builds, 2 only when building Tauri

**Estimated savings**: Eliminating the 0.11 tree saves ~3-5s of compile time (reqwest alone is 3.08s, plus duplicated hyper/http/h2).

---

### Issue 5: Monomorphization Bloat from `impl Into<String>` (1-2 hr fix, MEDIUM impact)

**Problem**: 40 public functions across the workspace use `impl Into<String>` or `impl Into<PathBuf>`. Each generic parameter gets monomorphized per-crate consumer.

**Audit results by crate**:

| Crate | Generic Params | Downstream Consumers | Total Instantiations |
|-------|---------------|---------------------|---------------------|
| **ckrv-core** | 18 | 9 | ~162 |
| **ckrv-sandbox** | 13 | 4 | ~52 |
| **ckrv-metrics** | 8 | 4 | ~32 |
| **ckrv-verify** | 4 | 2 | ~8 |
| **ckrv-git** | 1 | 2 | ~2 |
| **ckrv-cli** | 6 | 0 (leaf) | 0 |
| **Total** | **51** | — | **~256** |

**Hotspots in ckrv-core** (highest impact — 9 consumers):
- `step.rs:47` — `Step::new(id, name, step_type)` — 2 generic params
- `step_result.rs:65` — `failed(step_id, error)` — 2 generic params
- `step_result.rs:78` — `with_output(name, value)` — 2 generic params
- `prompt.rs:108` — `with_input(name, value)` — 2 generic params
- `prompt.rs:154` — `with_output(name, value)` — 2 generic params
- `job.rs:184,190` — `success(diff)`, `failure(error)` — 1 each

**Fix**: Change `impl Into<String>` to `&str` at crate boundaries:
```rust
// Before (monomorphizes in every consumer)
pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
    Self { id: id.into(), name: name.into() }
}

// After (concrete, no monomorphization)
pub fn new(id: &str, name: &str) -> Self {
    Self { id: id.to_owned(), name: name.to_owned() }
}
```

**Priority order**: ckrv-core first (18 params x 9 consumers), then ckrv-sandbox (13 x 4), then ckrv-metrics (8 x 4).

---

### Issue 6: Serde in Foundation Crates (1 hr fix, MEDIUM impact)

**Problem**: Serde (derive) is used in all 13 crates. The `syn` crate (required by serde derive) creates a compilation bottleneck.

**Audit results by crate**:

| Crate | S/D Derives | TS Derives | Dependents | Role |
|-------|------------|-----------|-----------|------|
| **ckrv-transport** | 178 | 31 | 2 | Boundary (correct) |
| **ckrv-cli** | 93 | 16 | 1 | Leaf (correct) |
| **ckrv-tauri** | 31 | 4 | 0 | Leaf (correct) |
| **ckrv-core** | 25 | 1 | **8** | Foundation (review) |
| **ckrv-ui** | 25 | 0 | 1 | Leaf (correct) |
| **ckrv-sandbox** | 1 | 0 | **6** | Foundation (good) |

**Assessment**: ckrv-core has 25 serde derives with 8 dependents. These are mostly domain types (Spec, Job, Plan, Step) that genuinely need serialization for persistence. The placement is acceptable but could be improved.

**Fix options**:
1. **Feature-gate serde on ckrv-core**: `serde = { version = "1.0", features = ["derive"], optional = true }` with `#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]`. Only transport/cli activate it.
2. **Accept current state**: 25 derives in core is not extreme. Focus on bigger wins first.

**Recommendation**: Defer this. The 25 derives in ckrv-core are justified. Focus on issues 1-4 first.

---

### Issue 7: Dual Proc Macro Passes in ckrv-transport (30 min fix, MEDIUM impact)

**Problem**: ckrv-transport has 178 serde derives AND 31 ts-rs derives. Each type gets two proc macro passes. This crate is already the slowest at 11.15s.

**Current state**: ts-rs is behind the `typescript` feature flag, but ckrv-ui enables it by default when importing ckrv-transport.

**Fix**: Ensure ts-rs derives only compile when explicitly generating TypeScript:
- Only enable `typescript` feature during `just ui-types` or equivalent
- Default feature set should NOT include `typescript`
- Check if ckrv-ui's Cargo.toml enables it by default

---

### Issue 8: ckrv-core → ckrv-sandbox Sequential Bottleneck (2-4 hr fix, HIGH impact)

**Problem**: ckrv-core depends on ckrv-sandbox, creating a sequential chain. 9 crates must wait for both to compile before starting.

**Timeline**:
```
0s ──── ckrv-sandbox (6s) ──── ckrv-core (3.3s) ──── [9 crates can start at 9.3s]
```

**Fix**: Extract `ckrv-core-types` with zero dependencies:
```
0s ──── ckrv-core-types (0.5s) ──── [9 crates can start at 0.5s]
        ckrv-sandbox (6s) ─────────── ckrv-core (3.3s)
```

Types to move to ckrv-core-types:
- `Spec`, `Plan`, `Step`, `Job`, `Attempt`, `AgentTask`
- `Config`, `WorkflowStep`, `AgentType`
- Error types that don't depend on sandbox

Downstream crates that only need types (ckrv-spec, ckrv-metrics, ckrv-git, ckrv-integrations) could depend on `ckrv-core-types` instead of `ckrv-core`, unblocking their compilation.

**Risk**: API churn. All internal crate imports change. Worth it for a 13-crate workspace.

---

## CI-Specific Improvements

### Split test compilation from test execution
```yaml
- run: cargo test --workspace --exclude ckrv-tauri --no-run
- run: cargo test --workspace --exclude ckrv-tauri
```
This surfaces build vs. test time separately for monitoring.

### Add mold linker in CI
```yaml
- name: Install mold linker
  run: sudo apt-get install -y mold
```
Then use .cargo/config.toml for faster linking.

---

## Options Considered

| Option | Pros | Cons |
|--------|------|------|
| A: Quick wins only (Issues 1-4) | 20 min effort, immediate results | Leaves structural issues |
| B: Quick wins + monomorphization (1-5) | 2 hr, addresses code-level bloat | Requires API changes in ckrv-core |
| C: Full overhaul (1-8) | Maximum improvement, future-proof | 4-8 hrs, significant refactor |

### Decision

**Phase 1 (now)**: Issues 1-4 — Cargo profiles, mold linker, CI incremental, reqwest dedup. ~30 min, all config changes, zero API breakage.

**Phase 2 (next session)**: Issues 5+7 — Monomorphization reduction + ts-rs feature gating. ~2 hrs, minor API changes.

**Phase 3 (when needed)**: Issue 8 — ckrv-core-types extraction. Deferred until build times hit a pain threshold or we add more crates.

Issue 6 (serde in core) is acceptable as-is. 25 derives in a foundation crate is not extreme.

## Success Criteria

| Metric | Current | Target (Phase 1) | Target (Phase 2) |
|--------|---------|-------------------|-------------------|
| Release build (`-p ckrv-cli`) | **39.17s** | ~30s | ~25s |
| ckrv-transport compile | 11.15s | 11s | <8s |
| CI total (clippy+test+build) | ~30 min | ~25 min | ~20 min |
| reqwest versions in Cargo.lock | 3 | 2 | 2 |

## Next Steps

- [ ] Phase 1: Add Cargo profiles to root Cargo.toml
- [ ] Phase 1: Create .cargo/config.toml with mold linker
- [ ] Phase 1: Add CARGO_INCREMENTAL=0 to ci.yml
- [ ] Phase 1: Unify reqwest to 0.12 workspace-wide
- [ ] Measure: Re-run `cargo build --release --timings` after Phase 1
- [ ] Phase 2: Replace `impl Into<String>` with `&str` in ckrv-core
- [ ] Phase 2: Tighten ts-rs feature gate in ckrv-transport

## References

- matklad's "Fast Rust Builds" — saved in CKRV NotebookLM knowledge base
- `cargo build --timings` output: `target/cargo-timings/cargo-timing.html`
- Crate dependency graph: see architecture docs at `crates/docs/architecture.md`
