import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright Configuration for Chakravarti CLI UI E2E Tests
 * 
 * CRITICAL: Tests run in isolated temporary directories per TR-007.
 * See tests/helpers/test-project.ts for the isolation implementation.
 */
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

    /* Global setup - can be used to start backend server */
    // globalSetup: './tests/global-setup.ts',

    /* Global teardown - cleanup */
    // globalTeardown: './tests/global-teardown.ts',

    /* Run local dev server before starting the tests */
    // Uncomment to auto-start backend:
    // webServer: {
    //     command: 'CKRV_PROJECT_ROOT=$(mktemp -d) cargo run --package ckrv-ui',
    //     url: 'http://localhost:3000/api/status',
    //     reuseExistingServer: !process.env.CI,
    //     timeout: 120000,
    // },
});
