# NPM Package for Quick Global Install

**Issue**: [#12](https://github.com/FnSK4R17s/chakravarti-cli/issues/12)
**Priority**: P0 - Critical for Go-To-Market
**Status**: Draft

## Problem Statement

Currently, users must:
1. Clone the repository
2. Install Rust toolchain
3. Build from source (`cargo build --release`)
4. Manually add binary to PATH

This creates significant friction for adoption. Most developers expect to try new tools with a single command like `npx` or `npm install -g`.

**Current Time to First Command**: ~10-15 minutes
**Target Time to First Command**: <30 seconds

## Solution

Create an npm package (`@chakravarti/cli`) that:
1. Downloads pre-built platform-specific binaries
2. Provides a JavaScript wrapper that invokes the native binary
3. Enables instant usage via `npx @chakravarti/cli` or global install

## User Stories

### US1: Try Without Install
**As a** developer evaluating Chakravarti,
**I want** to run `npx @chakravarti/cli init` without installing anything,
**So that** I can try the tool before committing to it.

### US2: Global Install
**As a** developer adopting Chakravarti,
**I want** to run `npm install -g @chakravarti/cli`,
**So that** I can use `ckrv` from anywhere on my system.

### US3: Project Dependency
**As a** team lead,
**I want** to add `@chakravarti/cli` to `devDependencies`,
**So that** all team members use the same version.

## Technical Approach

### Option A: Binary Wrapper (Recommended)
Pre-build native binaries for each platform, download at install time.

**Pros**:
- Native performance
- Full feature parity with Rust binary
- Smaller package size (downloads only needed binary)

**Cons**:
- Must maintain CI for multiple platforms
- Binary hosting costs (GitHub Releases are free)

### Option B: WASM Compilation
Compile Rust to WASM, run via Node.js.

**Pros**:
- Single package works everywhere
- No binary downloads

**Cons**:
- WASM performance limitations
- Some features may not work (Docker, stdio)
- Larger package size

### Decision: Option A (Binary Wrapper)

The binary wrapper approach is chosen because:
1. **Full Compatibility**: Docker sandbox, stdio transport for MCP all work
2. **Performance**: Native execution speed
3. **Proven Pattern**: Used by `esbuild`, `turbo`, `biome`, `pnpm`

## Supported Platforms

| Platform | Architecture | Binary Name |
|----------|-------------|-------------|
| macOS | x64 | `ckrv-darwin-x64` |
| macOS | arm64 (M1/M2) | `ckrv-darwin-arm64` |
| Linux | x64 | `ckrv-linux-x64` |
| Linux | arm64 | `ckrv-linux-arm64` |
| Windows | x64 | `ckrv-win32-x64.exe` |

## Package Structure

```
@chakravarti/cli/
├── package.json
├── README.md
├── bin/
│   └── ckrv.js           # Entry point, invokes native binary
├── scripts/
│   └── postinstall.js    # Downloads platform binary
└── lib/
    └── platform.js       # Platform detection utilities
```

## CLI Invocation Flow

```
npx @chakravarti/cli init
         ↓
bin/ckrv.js (Node.js)
         ↓
Detect platform (darwin-arm64)
         ↓
Spawn: ~/.chakravarti/bin/ckrv-darwin-arm64 init
         ↓
Native Rust binary executes
         ↓
Output returned to user
```

## Binary Distribution

### Hosting: GitHub Releases

Binaries are attached to GitHub releases:
```
https://github.com/FnSK4R17s/chakravarti-cli/releases/download/v0.1.0/ckrv-darwin-arm64.tar.gz
```

### Installation Location

```
~/.chakravarti/
├── bin/
│   └── ckrv-{platform}   # Downloaded binary
└── .version              # Installed version for upgrade checks
```

## API Design

### package.json

```json
{
  "name": "@chakravarti/cli",
  "version": "0.1.0",
  "description": "Spec-driven agent orchestration engine",
  "bin": {
    "ckrv": "./bin/ckrv.js"
  },
  "scripts": {
    "postinstall": "node scripts/postinstall.js"
  },
  "engines": {
    "node": ">=18"
  },
  "repository": {
    "type": "git",
    "url": "https://github.com/FnSK4R17s/chakravarti-cli"
  },
  "keywords": ["cli", "ai", "agent", "spec", "orchestration"],
  "license": "MIT"
}
```

### bin/ckrv.js

```javascript
#!/usr/bin/env node
import { spawn } from 'child_process';
import { getBinaryPath, ensureBinary } from '../lib/platform.js';

async function main() {
  const binaryPath = await ensureBinary();
  const proc = spawn(binaryPath, process.argv.slice(2), {
    stdio: 'inherit',
    env: process.env,
  });
  proc.on('exit', (code) => process.exit(code ?? 0));
}

main().catch((err) => {
  console.error('Error:', err.message);
  process.exit(1);
});
```

## CI/CD Requirements

### Release Workflow

```yaml
name: Release
on:
  push:
    tags: ['v*']

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            name: ckrv-linux-x64
          - os: ubuntu-latest
            target: aarch64-unknown-linux-gnu
            name: ckrv-linux-arm64
          - os: macos-latest
            target: x86_64-apple-darwin
            name: ckrv-darwin-x64
          - os: macos-latest
            target: aarch64-apple-darwin
            name: ckrv-darwin-arm64
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            name: ckrv-win32-x64.exe
    
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - run: cargo build --release --target ${{ matrix.target }}
      - name: Upload to Release
        uses: softprops/action-gh-release@v1
        with:
          files: target/${{ matrix.target }}/release/ckrv*
```

### NPM Publish Workflow

```yaml
name: Publish NPM
on:
  workflow_dispatch:
  release:
    types: [published]

jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
          registry-url: 'https://registry.npmjs.org'
      - run: npm publish --access public
        working-directory: npm
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
```

## Success Criteria

| Metric | Target |
|--------|--------|
| `npx @chakravarti/cli --version` | Works in <30s |
| Package size (without binary) | <50KB |
| Binary download time | <10s on average connection |
| Platform coverage | macOS, Linux, Windows (x64 + arm64) |
| npm weekly downloads | Track growth |

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Binary hosting costs | Low | GitHub Releases are free |
| Platform not supported | Medium | Clear error message with build instructions |
| Version mismatch | Low | postinstall always fetches matching version |
| Download failures | Medium | Retry logic + fallback mirrors |

## Out of Scope

- Homebrew formula (future feature)
- Scoop package for Windows (future feature)  
- Linux package managers (apt, rpm) (future feature)
- Auto-updates within the binary

## Implementation Phases

### Phase 1: Core Package
- [ ] Create npm package structure
- [ ] Implement platform detection
- [ ] Implement binary download in postinstall
- [ ] Create bin/ckrv.js wrapper

### Phase 2: CI/CD
- [ ] Add cross-compilation workflow
- [ ] Add binary upload to releases
- [ ] Add npm publish workflow
- [ ] Test on all platforms

### Phase 3: Polish
- [ ] Add progress bar for downloads
- [ ] Add offline cache support
- [ ] Add checksum verification
- [ ] Write documentation

## References

- [esbuild npm package](https://github.com/evanw/esbuild/tree/main/npm)
- [turbo npm package](https://github.com/vercel/turbo/tree/main/packages/turbo)
- [biome npm package](https://github.com/biomejs/biome/tree/main/npm/biome)
