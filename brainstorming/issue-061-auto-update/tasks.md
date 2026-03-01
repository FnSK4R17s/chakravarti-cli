# Tasks: Add Auto Update for Tauri App

**Issue**: [#61](https://github.com/FnSK4R17s/chakravarti-cli/issues/61)
**Brainstorm**: [notes.md](./notes.md)
**Updated**: 2026-02-28

---

## Status Legend

- [x] Done (verified in codebase)
- [ ] TODO

---

## Rust / Tauri Plugin (crates/ckrv-tauri)

- [x] **T1**: Add `tauri-plugin-updater = "2"` to `Cargo.toml`
- [x] **T2**: Configure updater in `tauri.conf.json`
  - `plugins.updater.endpoints` → GitHub Releases `latest.json` ✓
  - `plugins.updater.pubkey` placeholder added ✓
  - `bundle.createUpdaterArtifacts: "v2"` ✓
- [x] **T3**: Grant `updater:default` permission in `capabilities/default.json`
- [x] **T4**: Create `src/commands/update.rs` with `check_for_updates` + `install_update`
- [x] **T5**: Declare `pub mod update;` in `src/commands/mod.rs`
- [x] **T6**: Register updater plugin in `main.rs`
- [x] **T7**: Add background update check on startup (5s delay → `update-available` event)

### Remaining Code Work

- [ ] **T8**: Register update commands in `invoke_handler`
  - **File**: `src/main.rs` (inside `tauri::generate_handler![]`)
  - **Add**:
    ```rust
    // Update commands
    commands::update::check_for_updates,
    commands::update::install_update,
    ```
  - Without this, the frontend cannot invoke these commands over IPC.

- [ ] **T9**: Trigger app restart after successful update install
  - **File**: `src/commands/update.rs`, after `download_and_install` succeeds (~line 48)
  - Call `app.restart()` (from `tauri_plugin_process`, already a dependency)
  - The brainstorm requires "after install, app restarts automatically"

---

## CI / GitHub Actions (.github/workflows/release-tauri.yml)

- [ ] **T10**: Collect updater artifacts in build jobs
  - The build already generates updater bundles + `.sig` files (via `createUpdaterArtifacts: "v2"`) but the collect steps only grab installer bundles.
  - Update each platform's "Collect artifacts" step:
    - **macOS**: also copy `*.app.tar.gz` + `*.app.tar.gz.sig`
    - **Linux**: also copy `*.AppImage.tar.gz` + `*.AppImage.tar.gz.sig`
    - **Windows**: also copy `*.nsis.zip` + `*.nsis.zip.sig`

- [ ] **T11**: Generate and upload `latest.json` in `publish-tauri` job
  - After downloading all artifacts, build `latest.json` with:
    - `version` from git tag (`${GITHUB_REF_NAME}`)
    - `pub_date` from current timestamp
    - Per-platform entries: `signature` (from `.sig` file contents), `url` (GitHub Release download URL)
  - Platform keys: `darwin-aarch64`, `darwin-x86_64`, `linux-x86_64`, `windows-x86_64`
  - Upload `latest.json` alongside other release assets via `softprops/action-gh-release`

---

## Manual / Owner Actions

- [ ] **T12**: Set real signing public key in `tauri.conf.json`
  - Generate keypair: `cargo tauri signer generate -w ~/.tauri/chakravarti.key`
  - Paste the public key into `plugins.updater.pubkey` (replacing `REPLACE_WITH_YOUR_TAURI_SIGNING_PUBLIC_KEY`)
  - Confirm `TAURI_SIGNING_PRIVATE_KEY` GitHub secret matches the private key
  - **This is a repo-owner action, not automatable.**

---

## Dependency Order

```
T1–T7 (done)
  ├── T8  Register IPC commands
  ├── T9  Add restart after install
  ├── T10 Collect updater artifacts in CI
  │    └── T11 Generate latest.json
  └── T12 Set signing pubkey (owner)
```

T8, T9, T10 are independent of each other. T11 requires T10 (needs artifacts to read signatures). T12 is a manual prerequisite for end-to-end functionality.

## Verification

| Check | Command / Action |
|-------|-----------------|
| Compiles | `cargo build -p ckrv-tauri` after T8 + T9 |
| IPC reachable | Frontend calls `check_for_updates` → gets result (or pubkey error) |
| CI updater assets | Tag a test release → confirm `.sig` + `.tar.gz`/`.nsis.zip` in artifacts |
| `latest.json` correct | Check GitHub Release assets after `publish-tauri` completes |
| End-to-end | Install old build, tag new release, confirm update prompt + install + restart |
