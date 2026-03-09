# Rust Build Time Optimization - Tasks

**Brainstorm**: [notes.md](./notes.md)
**Created**: 2026-03-09
**Baseline**: 39.17s release build (`-p ckrv-cli`)

## Task Overview

| Phase | Tasks | Estimate |
|-------|-------|----------|
| Phase 1: Config Quick Wins | 4 | 45m |
| Phase 2: Dependency Dedup | 3 | 1h |
| Phase 3: Monomorphization Reduction | 4 | 2h |
| Phase 4: Measurement & Verification | 2 | 15m |
| **Total** | **13** | **~4h** |

## Dependencies

```
Phase 1 ──────────────────────────────────────────────────►
  Task 1.1 ──► Task 1.2 (independent)
  Task 1.3 (independent)
  Task 1.4 (independent)
                    │
Phase 2 ────────────┼─────────────────────────────────────►
                    │
                    └──► Task 2.1 ──► Task 2.2 ──► Task 2.3
                                                      │
Phase 3 ──────────────────────────────────────────────┼───►
                                                      │
                    Task 3.1 ──► Task 3.2 ──► Task 3.3 ──► Task 3.4
                                                      │
Phase 4 ──────────────────────────────────────────────┼───►
                                                      │
                                              Task 4.1 ──► Task 4.2
```

---

## Phase 1: Config Quick Wins

Zero API changes. All config/profile additions.

### Task 1.1: Add Cargo Profiles to Root Cargo.toml
**Priority**: P0
**Estimate**: 10m
**Files**: `Cargo.toml`

Add `[profile.dev]`, `[profile.dev.package."*"]`, and `[profile.release]` sections to the workspace root Cargo.toml.

**Changes**:
```toml
[profile.dev]
debug = 1

[profile.dev.package."*"]
opt-level = 2

[profile.release]
lto = "thin"
codegen-units = 1
strip = true
```

**Acceptance Criteria**:
- [ ] `[profile.dev]` sets `debug = 1`
- [ ] `[profile.dev.package."*"]` sets `opt-level = 2`
- [ ] `[profile.release]` sets `lto = "thin"`, `codegen-units = 1`, `strip = true`
- [ ] `cargo build -p ckrv-cli` succeeds
- [ ] `cargo build --release -p ckrv-cli` succeeds

---

### Task 1.2: Configure Mold Linker for Local Builds
**Priority**: P0
**Estimate**: 10m
**Files**: `.cargo/config.toml`

Create `.cargo/config.toml` with mold linker for Linux and lld for macOS. This only affects local builds — CI can optionally install mold.

**Changes**:
```toml
# Faster linkers for local development
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]

[target.aarch64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]

[target.x86_64-apple-darwin]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]

[target.aarch64-apple-darwin]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]
```

**Acceptance Criteria**:
- [ ] `.cargo/config.toml` exists with linker config
- [ ] `cargo build -p ckrv-cli` succeeds on current platform
- [ ] Config gracefully degrades if mold/lld is not installed (cargo falls back)
- [ ] File added to `.gitignore` OR committed (decision: commit it — it's a recommendation, cargo ignores missing linkers)

**Note**: Developers without `mold` installed will see a linker error. Document in `crates/docs/getting-started.md` or add a comment in the config file. Alternatively, gate behind an env var.

---

### Task 1.3: Disable Incremental Compilation in CI
**Priority**: P1
**Estimate**: 10m
**Files**: `.github/workflows/ci.yml`, `.github/workflows/ci-tauri.yml`

Add `CARGO_INCREMENTAL: 0` to the CI workflow env block. Also split `cargo test` into `--no-run` + run to separate build vs test time.

**Changes to `ci.yml`**:
```yaml
env:
  CARGO_TERM_COLOR: always
  RUSTFLAGS: "-D warnings"
  CARGO_INCREMENTAL: 0
```

**Changes to `rust-test` job**:
```yaml
- run: cargo test --workspace --exclude ckrv-tauri --no-run
- run: cargo test --workspace --exclude ckrv-tauri
```

**Acceptance Criteria**:
- [ ] `CARGO_INCREMENTAL: 0` set in `ci.yml` global env
- [ ] `CARGO_INCREMENTAL: 0` set in `ci-tauri.yml` global env (if it has Rust builds)
- [ ] `rust-test` job splits `--no-run` and run steps
- [ ] CI passes on the branch

---

### Task 1.4: Install Mold Linker in CI
**Priority**: P2
**Estimate**: 15m
**Files**: `.github/workflows/ci.yml`

Add mold installation step before Rust build/test/clippy jobs. This pairs with the `.cargo/config.toml` from Task 1.2.

**Changes**: Add to each Rust job (clippy, test, build) after `dtolnay/rust-toolchain`:
```yaml
- name: Install mold linker
  run: sudo apt-get update && sudo apt-get install -y mold clang
```

**Acceptance Criteria**:
- [ ] `mold` and `clang` installed in CI before Rust compilation
- [ ] All Rust CI jobs (clippy, test, build) use mold for linking
- [ ] CI passes on the branch

---

## Phase 2: Dependency Deduplication

Unify reqwest versions to eliminate triple HTTP stack compilation.

### Task 2.1: Update Workspace reqwest to 0.12
**Priority**: P0
**Estimate**: 15m
**Files**: `Cargo.toml` (workspace root)

Update the workspace-level reqwest dependency from 0.11 to 0.12.

**Change**:
```toml
# Before
reqwest = { version = "0.11", features = ["json"] }

# After
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
```

**Note**: Using `rustls-tls` instead of default `native-tls` aligns with ckrv-ui and ckrv-transport, and avoids compiling OpenSSL for HTTP (it's still needed for git2).

**Acceptance Criteria**:
- [ ] Workspace reqwest pinned to 0.12
- [ ] `cargo check --workspace --exclude ckrv-tauri` passes

---

### Task 2.2: Remove reqwest Version Overrides from Crates
**Priority**: P0
**Estimate**: 20m
**Files**: `crates/ckrv-cli/Cargo.toml`, `crates/ckrv-ui/Cargo.toml`, `crates/ckrv-transport/Cargo.toml`, `crates/ckrv-model/Cargo.toml`, `crates/ckrv-integrations/Cargo.toml`

Replace version-specific reqwest declarations with `workspace = true` plus any additional features needed per crate.

**Changes per crate**:

| Crate | Before | After |
|-------|--------|-------|
| ckrv-cli | `reqwest = { version = "0.12", features = ["json", "stream"] }` | `reqwest = { workspace = true, features = ["stream"] }` |
| ckrv-ui | `reqwest = { version = "0.12", ... features = ["json", "rustls-tls"] }` | `reqwest = { workspace = true }` |
| ckrv-transport | `reqwest = { version = "0.12", ... features = ["json", "rustls-tls"] }` | `reqwest = { workspace = true }` |
| ckrv-model | `reqwest = { workspace = true }` | No change (inherits 0.12 now) |
| ckrv-integrations | `reqwest = { workspace = true, optional = true }` | No change (inherits 0.12 now) |

**Acceptance Criteria**:
- [ ] No crate specifies a reqwest version directly (all use `workspace = true`)
- [ ] `cargo check --workspace --exclude ckrv-tauri` passes
- [ ] ckrv-model and ckrv-integrations compile with the new reqwest API

---

### Task 2.3: Fix reqwest 0.11 → 0.12 API Breakage
**Priority**: P0
**Estimate**: 25m
**Files**: Varies — wherever `reqwest` is called in ckrv-model and ckrv-integrations

The reqwest 0.11 → 0.12 upgrade introduces breaking changes:
- `RequestBuilder::send()` return type changes
- Some `Error` variants renamed
- `Client::builder()` API tweaks
- `Body` type changes

Audit ckrv-model and ckrv-integrations for API breakage and fix.

**Acceptance Criteria**:
- [ ] `cargo check --workspace --exclude ckrv-tauri` passes with zero errors
- [ ] `cargo test --workspace --exclude ckrv-tauri` passes
- [ ] `cargo clippy --workspace --exclude ckrv-tauri -- -D warnings` passes
- [ ] Only 2 reqwest versions in Cargo.lock (0.12 + 0.13 from Tauri)

---

## Phase 3: Monomorphization Reduction

Replace `impl Into<String>` with `&str` at crate boundaries to eliminate ~256 redundant instantiations.

### Task 3.1: De-generify ckrv-core Public API
**Priority**: P1
**Estimate**: 45m
**Files**: `crates/ckrv-core/src/step.rs`, `crates/ckrv-core/src/step_result.rs`, `crates/ckrv-core/src/prompt.rs`, `crates/ckrv-core/src/job.rs`, `crates/ckrv-core/src/agent_task.rs`

Change 12 functions (18 generic params) from `impl Into<String>` to `&str`. This is the highest-impact crate (9 downstream consumers = ~162 eliminated instantiations).

**Functions to change**:
| File | Function | Params to Change |
|------|----------|-----------------|
| `step.rs:47` | `Step::new()` | `id`, `name` |
| `step.rs:61` | `with_dependency()` | `dep_id` |
| `prompt.rs:108` | `with_input()` | `name`, `value` |
| `prompt.rs:115` | `with_step_outputs()` | `step_id` |
| `prompt.rs:154` | `with_output()` | `name`, `value` |
| `job.rs:184` | `AttemptResult::success()` | `diff` |
| `job.rs:190` | `AttemptResult::failure()` | `error` |
| `step_result.rs:52` | `success()` | `step_id` |
| `step_result.rs:65` | `failed()` | `step_id`, `error` |
| `step_result.rs:78` | `with_output()` | `name`, `value` |
| `step_result.rs:85` | `with_stdout()` | `stdout` |
| `step_result.rs:92` | `with_stderr()` | `stderr` |

**Pattern**:
```rust
// Before
pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
    Self { id: id.into(), name: name.into() }
}

// After
pub fn new(id: &str, name: &str) -> Self {
    Self { id: id.to_owned(), name: name.to_owned() }
}
```

**Acceptance Criteria**:
- [ ] All 12 functions in ckrv-core use `&str` instead of `impl Into<String>`
- [ ] `cargo check --workspace --exclude ckrv-tauri` passes (callers updated if needed)
- [ ] `cargo test -p ckrv-core` passes

---

### Task 3.2: De-generify ckrv-sandbox Public API
**Priority**: P1
**Estimate**: 30m
**Files**: `crates/ckrv-sandbox/src/agent/mod.rs`, `crates/ckrv-sandbox/src/executor.rs`, `crates/ckrv-sandbox/src/docker.rs`, `crates/ckrv-sandbox/src/allowlist.rs`, `crates/ckrv-sandbox/src/env.rs`

Change 11 functions (13 generic params) from `impl Into<String>` to `&str`. 4 downstream consumers = ~52 eliminated instantiations.

**Acceptance Criteria**:
- [ ] All `impl Into<String>` in ckrv-sandbox public API changed to `&str`
- [ ] `cargo check --workspace --exclude ckrv-tauri` passes
- [ ] `cargo test -p ckrv-sandbox` passes

---

### Task 3.3: De-generify ckrv-metrics Public API
**Priority**: P2
**Estimate**: 20m
**Files**: `crates/ckrv-metrics/src/collector.rs`, `crates/ckrv-metrics/src/cost.rs`, `crates/ckrv-metrics/src/report.rs`, `crates/ckrv-metrics/src/time.rs`

Change 7 functions (8 generic params). 4 downstream consumers = ~32 eliminated instantiations.

**Acceptance Criteria**:
- [ ] All `impl Into<String>` / `impl Into<PathBuf>` in ckrv-metrics changed to `&str` / `&Path`
- [ ] `cargo check --workspace --exclude ckrv-tauri` passes
- [ ] `cargo test -p ckrv-metrics` passes

---

### Task 3.4: De-generify Remaining Crates (verify, git)
**Priority**: P2
**Estimate**: 15m
**Files**: `crates/ckrv-verify/src/verdict.rs`, `crates/ckrv-verify/src/runner.rs`, `crates/ckrv-git/src/branch.rs`

Change 5 functions (5 generic params) in ckrv-verify and ckrv-git. Low impact but completes the sweep.

**Acceptance Criteria**:
- [ ] All remaining `impl Into<String/PathBuf>` in these crates changed
- [ ] `cargo check --workspace --exclude ckrv-tauri` passes
- [ ] `cargo test --workspace --exclude ckrv-tauri` passes

---

## Phase 4: Measurement & Verification

### Task 4.1: Re-run Build Timings
**Priority**: P1
**Estimate**: 10m
**Files**: None (measurement only)

After all changes, clean rebuild with timings:

```bash
cargo clean
cargo build --release -p ckrv-cli --timings
```

Compare against baseline (39.17s). Record per-crate deltas.

**Acceptance Criteria**:
- [ ] New timing report generated at `target/cargo-timings/`
- [ ] Before/after comparison table added to brainstorm notes
- [ ] Release build time < 30s (Phase 1+2 target) or < 25s (Phase 1+2+3 target)

---

### Task 4.2: Verify CI Passes and Update Brainstorm Status
**Priority**: P1
**Estimate**: 5m
**Files**: `brainstorming/rust-build-time-optimization/notes.md`

Run full local CI equivalent and update brainstorm status.

```bash
just ci
```

**Acceptance Criteria**:
- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --workspace --exclude ckrv-tauri -- -D warnings` passes
- [ ] `cargo test --workspace --exclude ckrv-tauri` passes
- [ ] Brainstorm status updated to reflect completed phases
- [ ] reqwest versions in Cargo.lock reduced from 3 to 2

---

## Deferred Work (Future Phases)

These are captured but not tasked for this iteration:

| Item | Trigger to Revisit |
|------|--------------------|
| Extract `ckrv-core-types` crate | Build time > 40s or new crates added to workspace |
| Feature-gate serde in ckrv-core | If ckrv-core compile time exceeds 5s |
| `cargo-chef` for Docker builds | When Docker build times become a bottleneck |
