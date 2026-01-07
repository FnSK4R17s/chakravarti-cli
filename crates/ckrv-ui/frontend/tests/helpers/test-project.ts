/**
 * Test Project Helper
 * 
 * Creates isolated temporary directories for E2E tests to prevent
 * CLI commands from modifying the working codebase.
 * 
 * CRITICAL: All E2E tests MUST use this fixture per TR-007.
 */

import { test as base, expect } from '@playwright/test';
import { mkdtemp, rm, cp, mkdir, writeFile } from 'fs/promises';
import { tmpdir } from 'os';
import { join } from 'path';
import { execSync } from 'child_process';

export interface TestProjectFixture {
    /** Absolute path to the isolated test project directory */
    testProject: string;
    /** Port the test backend is running on */
    backendPort: number;
}

/**
 * Extended Playwright test with isolated test project fixture.
 * 
 * Usage:
 * ```typescript
 * import { test, expect } from './helpers/test-project';
 * 
 * test('my test', async ({ page, testProject }) => {
 *     // testProject is an isolated temp directory
 *     await page.goto('/');
 * });
 * ```
 */
export const test = base.extend<TestProjectFixture>({
    testProject: async ({ }, use) => {
        // Create temp directory with unique name
        const tempDir = await mkdtemp(join(tmpdir(), 'ckrv-test-'));

        try {
            // Initialize as git repo (required for ckrv commands)
            execSync('git init', { cwd: tempDir, stdio: 'pipe' });
            execSync('git config user.email "test@test.com"', { cwd: tempDir, stdio: 'pipe' });
            execSync('git config user.name "Test User"', { cwd: tempDir, stdio: 'pipe' });

            // Copy fixture files to temp directory
            const fixturesPath = join(__dirname, '../fixtures/sample-project');
            try {
                await cp(fixturesPath, tempDir, { recursive: true });
            } catch {
                // If fixtures don't exist, create minimal structure
                await mkdir(join(tempDir, 'specs'), { recursive: true });
                await mkdir(join(tempDir, '.chakravarti'), { recursive: true });
                await writeFile(
                    join(tempDir, 'package.json'),
                    JSON.stringify({ name: 'test-project', version: '1.0.0' }, null, 2)
                );
            }

            // Initial commit (required for git operations)
            execSync('git add .', { cwd: tempDir, stdio: 'pipe' });
            execSync('git commit -m "Initial commit"', { cwd: tempDir, stdio: 'pipe' });

            // Use the temp project for this test
            await use(tempDir);
        } finally {
            // Cleanup after test completes
            await rm(tempDir, { recursive: true, force: true });
        }
    },

    backendPort: async ({ }, use) => {
        // Each test gets a unique port to avoid conflicts
        // In practice, tests may share a backend or spin up their own
        const port = 3000 + Math.floor(Math.random() * 1000);
        await use(port);
    },
});

export { expect };

/**
 * Helper to wait for backend to be ready
 */
export async function waitForBackend(port: number, timeout = 30000): Promise<void> {
    const start = Date.now();
    while (Date.now() - start < timeout) {
        try {
            const response = await fetch(`http://localhost:${port}/api/status`);
            if (response.ok) return;
        } catch {
            // Backend not ready yet
        }
        await new Promise(resolve => setTimeout(resolve, 100));
    }
    throw new Error(`Backend did not start within ${timeout}ms`);
}

/**
 * Helper to seed test data into the test project
 */
export async function seedTestData(
    testProject: string,
    specName: string,
    specContent: string
): Promise<void> {
    const specDir = join(testProject, 'specs', specName);
    await mkdir(specDir, { recursive: true });
    await writeFile(join(specDir, 'spec.yaml'), specContent);

    // Commit the seeded data
    execSync('git add .', { cwd: testProject, stdio: 'pipe' });
    execSync(`git commit -m "Add spec: ${specName}"`, { cwd: testProject, stdio: 'pipe' });
}
