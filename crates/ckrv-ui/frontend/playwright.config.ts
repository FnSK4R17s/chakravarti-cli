import { defineConfig, devices } from '@playwright/test';
import path from 'path';

/**
 * Playwright Configuration for Chakravarti CLI UI E2E Tests
 *
 * CRITICAL: Tests run in isolated temporary directories per TR-007.
 * See tests/helpers/test-project.ts for the isolation implementation.
 *
 * The backend server is started automatically via the webServer option.
 * - CI: Uses a pre-built binary downloaded from the rust-build job artifact.
 *        Set CKRV_BINARY env var to the absolute path of the binary.
 * - Local: Falls back to `cargo run`. If a server is already running on
 *          port 3000, Playwright reuses it (reuseExistingServer).
 */
const workspaceRoot = path.resolve(__dirname, '../../..');

export default defineConfig({
    testDir: './tests/e2e',

    /* Run tests in files in parallel */
    fullyParallel: true,

    /* Fail the build on CI if you accidentally left test.only in the source code */
    forbidOnly: !!process.env.CI,

    /* Retry on CI only */
    retries: process.env.CI ? 2 : 0,

    /* Opt out of parallel tests on CI for stability */
    workers: process.env.CI ? 1 : undefined,

    /* Reporter configuration */
    reporter: [
        ['html', { open: 'never' }],
        ['list'],
    ],

    /* Shared settings for all projects */
    use: {
        /* Base URL for tests - backend server */
        baseURL: process.env.CKRV_TEST_URL || 'http://localhost:3000',

        /* Collect trace when retrying the failed test */
        trace: 'on-first-retry',

        /* Screenshot on failure */
        screenshot: 'only-on-failure',

        /* Video on failure */
        video: 'on-first-retry',
    },

    /* Test timeout - 60 seconds per TR-005 */
    timeout: 60000,

    /* Expect timeout for assertions */
    expect: {
        timeout: 5000,
    },

    /* Configure projects for major browsers */
    projects: [
        {
            name: 'chromium',
            use: { ...devices['Desktop Chrome'] },
        },
        {
            name: 'firefox',
            use: { ...devices['Desktop Firefox'] },
        },
        {
            name: 'webkit',
            use: { ...devices['Desktop Safari'] },
        },
    ],

    /* Start the real backend server before running E2E tests */
    webServer: {
        command: process.env.CKRV_BINARY
            ? `${process.env.CKRV_BINARY} ui --port 3000`
            : 'cargo run -p ckrv-cli -- ui --port 3000',
        cwd: workspaceRoot,
        url: 'http://localhost:3000/api/status',
        reuseExistingServer: !process.env.CI,
        timeout: 120_000,
    },
});
