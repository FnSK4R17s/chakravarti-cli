---
description: Build Tauri app for Windows, macOS, and Linux using GitHub Actions
---

# Tauri Cross-Platform Build Skill

Build the Tauri desktop app for all platforms without needing separate machines.

## Overview

Tauri apps need platform-specific builds because each OS has different GUI libraries:
- **Windows**: WebView2
- **macOS**: WebKit
- **Linux**: webkit2gtk

GitHub Actions provides runners for all platforms, making cross-platform builds free and automatic.

## Quick Start

1. Create the workflow file at `.github/workflows/release-tauri.yml`
2. Tag a release: `git tag v0.1.0 && git push origin v0.1.0`
3. GitHub builds all platforms automatically

## Workflow Template

```yaml
name: Release Tauri App

on:
  push:
    tags:
      - 'v*'
  workflow_dispatch:

jobs:
  create-release:
    permissions:
      contents: write
    runs-on: ubuntu-22.04
    outputs:
      release_id: ${{ steps.create-release.outputs.result }}
    steps:
      - uses: actions/checkout@v4
      - name: Create release
        id: create-release
        uses: actions/github-script@v7
        with:
          script: |
            const { data } = await github.rest.repos.createRelease({
              owner: context.repo.owner,
              repo: context.repo.repo,
              tag_name: `${{ github.ref_name }}`,
              name: `Chakravarti ${{ github.ref_name }}`,
              body: 'See the assets below to download the app for your platform.',
              draft: true,
              prerelease: false
            })
            return data.id

  build-tauri:
    needs: create-release
    permissions:
      contents: write
    strategy:
      fail-fast: false
      matrix:
        include:
          - platform: 'macos-latest'
            args: '--target aarch64-apple-darwin'
            name: 'macOS-arm64'
          - platform: 'macos-latest'
            args: '--target x86_64-apple-darwin'
            name: 'macOS-x64'
          - platform: 'ubuntu-22.04'
            args: ''
            name: 'Linux'
          - platform: 'windows-latest'
            args: ''
            name: 'Windows'
    runs-on: ${{ matrix.platform }}
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: 'npm'
          cache-dependency-path: crates/ckrv-ui/frontend/package-lock.json

      - name: Install Rust stable
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.platform == 'macos-latest' && 'aarch64-apple-darwin,x86_64-apple-darwin' || '' }}

      - name: Install dependencies (Ubuntu)
        if: matrix.platform == 'ubuntu-22.04'
        run: |
          sudo apt-get update
          sudo apt-get install -y libwebkit2gtk-4.1-dev librsvg2-dev patchelf

      - name: Install frontend dependencies
        working-directory: crates/ckrv-ui/frontend
        run: npm ci

      - name: Build Tauri app
        uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          projectPath: crates/ckrv-tauri
          releaseId: ${{ needs.create-release.outputs.release_id }}
          args: ${{ matrix.args }}

  publish-release:
    permissions:
      contents: write
    runs-on: ubuntu-22.04
    needs: [create-release, build-tauri]
    steps:
      - name: Publish release
        uses: actions/github-script@v7
        env:
          release_id: ${{ needs.create-release.outputs.release_id }}
        with:
          script: |
            github.rest.repos.updateRelease({
              owner: context.repo.owner,
              repo: context.repo.repo,
              release_id: process.env.release_id,
              draft: false,
            })
```

## Build Outputs

| Platform | Artifacts |
|----------|-----------|
| Windows | `.msi` installer, `.exe` |
| macOS | `.dmg`, `.app` bundle |
| Linux | `.deb`, `.AppImage` |

## Prerequisites

### For Local Builds

| Platform | Requirements |
|----------|--------------|
| Windows | Visual Studio Build Tools, WebView2 Runtime |
| macOS | Xcode Command Line Tools |
| Linux | `libwebkit2gtk-4.1-dev`, `librsvg2-dev`, `patchelf` |

### For CI Builds

- GitHub repository with Actions enabled
- No additional configuration needed (runners have all dependencies)

## Manual Trigger

The workflow includes `workflow_dispatch` so you can manually trigger builds:

1. Go to Actions tab in GitHub
2. Select "Release Tauri App"
3. Click "Run workflow"

## Code Signing (Optional)

For production releases, add code signing:

### macOS
```yaml
env:
  APPLE_CERTIFICATE: ${{ secrets.APPLE_CERTIFICATE }}
  APPLE_CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
  APPLE_SIGNING_IDENTITY: ${{ secrets.APPLE_SIGNING_IDENTITY }}
  APPLE_ID: ${{ secrets.APPLE_ID }}
  APPLE_PASSWORD: ${{ secrets.APPLE_PASSWORD }}
  APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
```

### Windows
```yaml
env:
  TAURI_PRIVATE_KEY: ${{ secrets.TAURI_PRIVATE_KEY }}
  TAURI_KEY_PASSWORD: ${{ secrets.TAURI_KEY_PASSWORD }}
```

## References

- [Tauri GitHub Action](https://github.com/tauri-apps/tauri-action)
- [Tauri Distribution Guide](https://tauri.app/distribute/)
- [Code Signing Setup](https://tauri.app/distribute/sign/)
