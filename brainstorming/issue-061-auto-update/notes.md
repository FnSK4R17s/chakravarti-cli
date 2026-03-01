# Add Auto Update for Tauri App

**Issue**: [#61](https://github.com/FnSK4R17s/chakravarti-cli/issues/61)
**Created**: 2026-02-28
**Status**: In Progress

## Problem Statement

Users running the Chakravarti desktop app (Tauri) have no mechanism to receive updates automatically. They must manually download new releases from GitHub, which creates friction and delays adoption of bug fixes and new features.

## Current State

- Tauri v2 desktop app is fully scaffolded with 50+ IPC commands
- GitHub Actions release workflow builds for macOS (arm64/x64), Linux (x64), Windows (x64)
- Signing keys (`TAURI_SIGNING_PRIVATE_KEY`) already configured as GitHub secrets
- No updater plugin or configuration exists
- Issue was explicitly deferred from v1 Tauri implementation (see issue-042 brainstorm)

## Proposed Solution

Add Tauri v2's built-in updater plugin (`tauri-plugin-updater`) with GitHub Releases as the update endpoint. The app checks for updates on startup and provides manual check/install commands for frontend integration.

## User Stories

### US1: Automatic Update Notification
**As a** desktop app user,
**I want** the app to notify me when a new version is available on startup,
**So that** I always run the latest version without manually checking GitHub.

### US2: Manual Update Check
**As a** desktop app user,
**I want** to manually trigger an update check and install,
**So that** I can update on my own schedule.

## Technical Approach

### Options Considered

| Option | Pros | Cons |
|--------|------|------|
| A: `tauri-plugin-updater` + GitHub Releases | Native Tauri v2 support, no extra infra, signing already set up | Requires `latest.json` generation in CI |
| B: CrabNebula Cloud | Managed update hosting, analytics | External dependency, cost, over-engineered for this project |
| C: Custom update server | Full control over update flow | Maintenance burden, unnecessary complexity |

### Decision

**Option A: `tauri-plugin-updater` with GitHub Releases**

Rationale:
- Zero additional infrastructure (GitHub Releases already used)
- Signing keys already configured as repo secrets
- Native Tauri v2 plugin with well-tested update flow
- `latest.json` manifest generated in existing CI workflow

## Implementation Notes

### Components Changed

1. **`Cargo.toml`** - Add `tauri-plugin-updater = "2"` dependency
2. **`tauri.conf.json`** - Add updater plugin config (endpoint + pubkey) and `createUpdaterArtifacts: "v2"`
3. **`capabilities/default.json`** - Add `updater:default` permission
4. **`commands/update.rs`** - New module: `check_for_updates` + `install_update` commands
5. **`main.rs`** - Register plugin, add commands, background update check on startup
6. **`release-tauri.yml`** - Collect updater artifacts (`.sig` + bundles), generate and upload `latest.json`

### Updater Flow

1. App starts -> 5s delay -> background check against `latest.json` on GitHub Releases
2. If update found -> emit `update-available` event to frontend (version + release notes)
3. Frontend can call `check_for_updates` (manual) or `install_update` (download + apply)
4. After install, app restarts automatically

### Signing Requirement

The updater verifies artifact signatures using a public key. The user must:
1. Generate a keypair: `cargo tauri signer generate -w ~/.tauri/chakravarti.key`
2. Set `TAURI_SIGNING_PRIVATE_KEY` as a GitHub secret (already done)
3. Add the public key to `tauri.conf.json` under `plugins.updater.pubkey`

### CI `latest.json` Format

```json
{
  "version": "v0.1.0",
  "notes": "Release notes",
  "pub_date": "2026-02-28T00:00:00Z",
  "platforms": {
    "darwin-aarch64": { "signature": "...", "url": "https://github.com/.../Chakravarti_aarch64.app.tar.gz" },
    "darwin-x86_64": { "signature": "...", "url": "https://github.com/.../Chakravarti_x86_64.app.tar.gz" },
    "linux-x86_64": { "signature": "...", "url": "https://github.com/.../Chakravarti_amd64.AppImage.tar.gz" },
    "windows-x86_64": { "signature": "...", "url": "https://github.com/.../Chakravarti_x64-setup.nsis.zip" }
  }
}
```

## Open Questions

- [x] Which update endpoint? -> GitHub Releases
- [x] Background or manual check? -> Both (background on startup + manual command)
- [ ] Public key value? -> User must generate and add to config (placeholder provided)

## Success Criteria

| Metric | Target |
|--------|--------|
| Plugin registered and compiles | Yes |
| Background check on startup | 5s delay, non-blocking |
| Manual check/install commands | Exposed as Tauri IPC |
| CI generates `latest.json` | Uploaded to GitHub Release |
| Updater artifacts produced | `.sig` files + update bundles |

## References

- [Tauri v2 Updater Plugin](https://v2.tauri.app/plugin/updater/)
- [tauri-plugin-updater crate](https://crates.io/crates/tauri-plugin-updater)
- Issue-042 brainstorm (deferred auto-update decision)
