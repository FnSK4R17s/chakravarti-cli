---
last_commit: c1bb442
last_updated: 2026-01-21
---

# Tasks: Comprehensive Code Documentation

## Phase 1: Setup

- [x] **T1.1**: Create `crates/docs/` directory structure
- [x] **T1.2**: Create `docs/` subfolder in each of the 10 crates [P]

## Phase 2: Top-Level Documentation

- [x] **T2.1**: Create `crates/docs/architecture.md` with crate dependency diagram
- [x] **T2.2**: Create `crates/docs/getting-started.md` onboarding guide
- [x] **T2.3**: Create `crates/docs/cli-commands.md` command reference
- [x] **T2.4**: Create `crates/docs/agent-guide.md` agent extensibility guide

## Phase 3: Per-Crate Documentation

### Core Crates [P]

- [x] **T3.1**: Create `crates/ckrv-cli/docs/README.md`
- [x] **T3.2**: Create `crates/ckrv-core/docs/README.md`
- [x] **T3.3**: Create `crates/ckrv-git/docs/README.md`
- [x] **T3.4**: Create `crates/ckrv-sandbox/docs/README.md`
- [x] **T3.5**: Create `crates/ckrv-spec/docs/README.md`

### Supporting Crates [P]

- [x] **T3.6**: Create `crates/ckrv-metrics/docs/README.md`
- [x] **T3.7**: Create `crates/ckrv-model/docs/README.md`
- [x] **T3.8**: Create `crates/ckrv-integrations/docs/README.md`
- [x] **T3.9**: Create `crates/ckrv-verify/docs/README.md`
- [x] **T3.10**: Create `crates/ckrv-ui/docs/README.md`

### API Reference

- [x] **T3.11**: Create `crates/ckrv-ui/docs/api-reference.md`

## Phase 4: Verification

- [x] **T4.1**: Run `cargo doc --deny warnings --no-deps`
- [x] **T4.2**: Verify all docs have frontmatter with `last_commit`
- [x] **T4.3**: Check folder structure is complete
