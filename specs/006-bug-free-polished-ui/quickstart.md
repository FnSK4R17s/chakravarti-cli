# Quickstart: Bug-Free and Polished Chakravarti CLI UI

**Feature**: 006-bug-free-polished-ui  
**Date**: 2026-01-05

## Prerequisites

- Node.js 18+
- Rust 1.75+
- pnpm (recommended) or npm

## Development Setup

### 1. Install Dependencies

```bash
# From frontend directory
cd crates/ckrv-ui/frontend
pnpm install

# Add Playwright for E2E tests
pnpm add -D @playwright/test
npx playwright install
```

### 2. Start Development Server

```bash
# Terminal 1: Build and run Rust backend
cargo run --package ckrv-ui

# Terminal 2: Start Vite dev server (if using HMR)
cd crates/ckrv-ui/frontend
pnpm dev
```

## ⚠️ CRITICAL: Test Isolation Strategy

**Tests MUST run in temporary folders to avoid modifying working code.**

The `ckrv` commands (spec-new, tasks, run, etc.) modify the filesystem by creating/updating files. If tests run against the real codebase, they will:
- Overwrite actual specs, tasks, and plans
- Create unwanted Git branches and worktrees
- Corrupt the working directory state

### Isolation Requirements

1. **Temporary Project Directory**: Each E2E test MUST create a fresh temporary directory
2. **Git Repository Clone**: Tests MUST clone or initialize a clean Git repo in the temp folder
3. **Backend Target**: Tests MUST point the backend server at the temp directory
4. **Cleanup**: Temp directories MUST be cleaned up after each test

### Implementation Pattern

```typescript
// frontend/tests/helpers/test-project.ts
import { test as base, expect } from '@playwright/test';
import { mkdtemp, rm, cp } from 'fs/promises';
import { tmpdir } from 'os';
import { join } from 'path';
import { execSync } from 'child_process';

// Fixture that creates an isolated test project
export const test = base.extend<{ testProject: string }>({
    testProject: async ({}, use) => {
        // Create temp directory
        const tempDir = await mkdtemp(join(tmpdir(), 'ckrv-test-'));
        
        // Initialize as git repo
        execSync('git init', { cwd: tempDir });
        execSync('git config user.email "test@test.com"', { cwd: tempDir });
        execSync('git config user.name "Test"', { cwd: tempDir });
        
        // Copy fixture files
        await cp(
            join(__dirname, '../fixtures/sample-project'),
            tempDir,
            { recursive: true }
        );
        
        // Initial commit
        execSync('git add .', { cwd: tempDir });
        execSync('git commit -m "Initial commit"', { cwd: tempDir });
        
        // Use the temp project for this test
        await use(tempDir);
        
        // Cleanup after test
        await rm(tempDir, { recursive: true, force: true });
    },
});

export { expect };
```

### Backend Configuration

The backend server MUST be started with the test project directory:

```typescript
// In test setup
beforeAll(async () => {
    // Start backend pointing at temp directory
    backendProcess = spawn('cargo', ['run', '--package', 'ckrv-ui'], {
        env: {
            ...process.env,
            CKRV_PROJECT_ROOT: testProjectPath, // Point at temp folder
        },
    });
});
```

### Example Test Using Isolated Project

```typescript
import { test, expect } from './helpers/test-project';

test('create new spec does not affect main codebase', async ({ page, testProject }) => {
    // testProject is a fresh temp directory
    console.log(`Running test in isolated folder: ${testProject}`);
    
    // Navigate to UI (backend already pointing at testProject)
    await page.goto('/');
    
    // Create a new spec - this modifies files in testProject, NOT the real code
    await page.click('[data-testid="new-spec-button"]');
    await page.fill('[data-testid="spec-description"]', 'Test feature');
    await page.click('[data-testid="submit-spec"]');
    
    // Verify spec created in temp folder only
    await expect(page.locator('[data-testid="spec-list"]')).toContainText('Test feature');
    
    // After test, testProject is deleted - no impact on real codebase
});
```

```bash
cd crates/ckrv-ui/frontend

# Run all E2E tests
pnpm test:e2e

# Run specific test file
pnpm test:e2e tests/e2e/execution-runner.spec.ts

# Run with UI mode for debugging
pnpm test:e2e --ui
```

## Test Fixtures

Test fixtures are located in `frontend/tests/fixtures/`:

```text
frontend/tests/fixtures/
├── specs/
│   └── test-feature/
│       ├── spec.yaml
│       ├── tasks.yaml
│       └── plan.yaml
└── agents/
    └── test-agent.yaml
```

### Using Fixtures in Tests

```typescript
import { test, expect } from '@playwright/test';
import { loadFixture } from '../helpers/fixtures';

test('execution runner loads plan', async ({ page }) => {
    // Seed fixture data via API
    await loadFixture('specs/test-feature');
    
    // Navigate to runner
    await page.goto('/runner');
    
    // Verify batches loaded
    await expect(page.locator('[data-testid="batch-card"]')).toHaveCount(3);
});
```

## Debugging Tips

### WebSocket Issues

1. Open browser DevTools → Network → WS tab
2. Look for `/api/execution/ws` connection
3. Check message flow and connection status

### React Component Issues

1. Install React DevTools browser extension
2. Inspect component state and props
3. Check for excessive re-renders in Profiler

### XTerm.js Issues

1. Check terminal container dimensions
2. Verify FitAddon is called after layout
3. Check for ResizeObserver errors in console

## Key Files to Edit

| Bug | Primary File | Related Files |
|-----|-------------|---------------|
| BUG-001 | ExecutionRunner.tsx | - |
| BUG-002 | ExecutionRunner.tsx | - |
| BUG-003 | ExecutionRunner.tsx | LogTerminal.tsx |
| BUG-004 | New: LoadingButton.tsx | CommandPalette.tsx, SpecEditor.tsx |
| BUG-005 | ExecutionRunner.tsx | - |
| BUG-006 | New: ErrorBoundary.tsx | App.tsx |
| BUG-007 | ExecutionRunner.tsx | - |
| BUG-008 | LogTerminal.tsx | - |
| BUG-009 | index.css | All components |
| BUG-010 | Modal components | - |
| BUG-011 | All components | - |
| BUG-012 | LogTerminal.tsx | index.css |

## Code Style

- Use `var(--property-name)` for all colors
- Use animation tokens from data-model.md
- Add `data-testid` attributes for E2E test selectors
- Add `aria-label` for icon-only buttons
