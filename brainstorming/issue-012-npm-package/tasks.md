# NPM Package Implementation Tasks

**Spec**: [spec.md](./spec.md)
**Issue**: [#12](https://github.com/FnSK4R17s/chakravarti-cli/issues/12)

## Phase 1: Core Package

### Task 1.1: Create Package Structure
**Priority**: P0 | **Estimate**: 1h

Create the npm package directory structure:

```bash
mkdir -p npm/@chakravarti/cli/{bin,lib,scripts}
```

Files to create:
- `npm/@chakravarti/cli/package.json`
- `npm/@chakravarti/cli/README.md`
- `npm/@chakravarti/cli/.npmignore`

**Acceptance Criteria**:
- [ ] package.json has correct metadata
- [ ] bin field points to `./bin/ckrv.js`
- [ ] postinstall script configured

---

### Task 1.2: Implement Platform Detection
**Priority**: P0 | **Estimate**: 1h

Create `lib/platform.js`:

```javascript
export function getPlatformInfo() {
  const platform = process.platform; // darwin, linux, win32
  const arch = process.arch; // x64, arm64
  return { platform, arch };
}

export function getBinaryName() {
  const { platform, arch } = getPlatformInfo();
  const ext = platform === 'win32' ? '.exe' : '';
  return `ckrv-${platform}-${arch}${ext}`;
}

export function getBinaryUrl(version) {
  const name = getBinaryName();
  return `https://github.com/FnSK4R17s/chakravarti-cli/releases/download/v${version}/${name}.tar.gz`;
}
```

**Acceptance Criteria**:
- [ ] Correctly detects all supported platforms
- [ ] Returns correct binary name for each platform
- [ ] Generates valid download URLs

---

### Task 1.3: Implement Binary Download
**Priority**: P0 | **Estimate**: 2h

Create `scripts/postinstall.js`:

```javascript
import { getBinaryUrl, getBinaryName } from '../lib/platform.js';
import https from 'https';
import fs from 'fs';
import path from 'path';
import { createGunzip } from 'zlib';
import { extract } from 'tar';

async function downloadBinary() {
  const version = require('../package.json').version;
  const url = getBinaryUrl(version);
  const binDir = path.join(os.homedir(), '.chakravarti', 'bin');
  
  // Download, extract, make executable
}
```

**Acceptance Criteria**:
- [ ] Downloads correct binary for platform
- [ ] Extracts tar.gz
- [ ] Sets executable permission on Unix
- [ ] Shows progress during download
- [ ] Handles errors gracefully

---

### Task 1.4: Create Binary Wrapper
**Priority**: P0 | **Estimate**: 1h

Create `bin/ckrv.js`:

```javascript
#!/usr/bin/env node
import { spawn } from 'child_process';
import { getBinaryPath } from '../lib/platform.js';

const binaryPath = getBinaryPath();
const proc = spawn(binaryPath, process.argv.slice(2), {
  stdio: 'inherit',
});

proc.on('exit', (code) => process.exit(code ?? 0));
```

**Acceptance Criteria**:
- [ ] Passes all arguments to native binary
- [ ] Inherits stdio (colors, interactivity work)
- [ ] Exits with same code as binary
- [ ] Clear error if binary not found

---

## Phase 2: CI/CD

### Task 2.1: Cross-Compilation Workflow
**Priority**: P0 | **Estimate**: 2h

Create `.github/workflows/release.yml`:

- Build for all 5 platforms
- Upload binaries to GitHub Release
- Create checksums file

**Acceptance Criteria**:
- [ ] Builds for darwin-x64, darwin-arm64, linux-x64, linux-arm64, win32-x64
- [ ] Binaries are attached to release
- [ ] SHA256 checksums included

---

### Task 2.2: NPM Publish Workflow
**Priority**: P0 | **Estimate**: 1h

Create `.github/workflows/npm-publish.yml`:

- Triggered on release publish
- Publishes to npm registry

**Acceptance Criteria**:
- [ ] Publishes package on release
- [ ] Correct version from tag
- [ ] Public access

---

### Task 2.3: Platform Testing
**Priority**: P1 | **Estimate**: 2h

Test on all platforms:
- macOS Intel
- macOS Apple Silicon
- Ubuntu x64
- Windows x64

**Acceptance Criteria**:
- [ ] `npx @chakravarti/cli --version` works
- [ ] `npx @chakravarti/cli init` works
- [ ] Global install works

---

## Phase 3: Polish

### Task 3.1: Progress Bar
**Priority**: P2 | **Estimate**: 1h

Add download progress indicator:

```
Downloading ckrv-darwin-arm64... [████████░░] 80% (4.2MB/5.3MB)
```

**Acceptance Criteria**:
- [ ] Shows percentage and size
- [ ] Updates smoothly
- [ ] Falls back to simple output if not TTY

---

### Task 3.2: Checksum Verification
**Priority**: P1 | **Estimate**: 1h

Verify downloaded binary against checksums:

```javascript
import crypto from 'crypto';

function verifyChecksum(filePath, expectedHash) {
  const hash = crypto.createHash('sha256');
  const data = fs.readFileSync(filePath);
  return hash.update(data).digest('hex') === expectedHash;
}
```

**Acceptance Criteria**:
- [ ] Downloads checksums file
- [ ] Verifies binary before use
- [ ] Clear error on mismatch

---

### Task 3.3: Documentation
**Priority**: P1 | **Estimate**: 1h

Update README with npm install instructions:

```markdown
## Installation

### Quick Try (no install)
\`\`\`bash
npx @chakravarti/cli init
\`\`\`

### Global Install
\`\`\`bash
npm install -g @chakravarti/cli
ckrv init
\`\`\`

### Project Dependency
\`\`\`bash
npm install -D @chakravarti/cli
npx ckrv init
\`\`\`
```

**Acceptance Criteria**:
- [ ] Clear installation instructions
- [ ] Troubleshooting section
- [ ] Platform support listed

---

## Task Summary

| Phase | Tasks | Estimate |
|-------|-------|----------|
| Phase 1 | 4 tasks | 5h |
| Phase 2 | 3 tasks | 5h |
| Phase 3 | 3 tasks | 3h |
| **Total** | **10 tasks** | **13h** |

## Dependencies

```
Task 1.1 ──→ Task 1.2 ──→ Task 1.3 ──→ Task 1.4
                                          ↓
Task 2.1 ─────────────────────────────────┤
                                          ↓
Task 2.2 ─────────────────────────────→ Task 2.3
                                          ↓
                           Task 3.1, 3.2, 3.3 (parallel)
```
