/**
 * @module test/setup
 * @description
 * Global test setup for Vitest. Configures jest-dom matchers, MSW server
 * lifecycle, and browser API mocks (matchMedia, ResizeObserver).
 *
 * @context
 * Loaded automatically by Vitest via the setupFiles config. Runs before
 * every test file to ensure a consistent test environment.
 *
 * @dependencies
 * - @testing-library/jest-dom: Custom DOM matchers
 * - MSW server: API mocking lifecycle
 */

import '@testing-library/jest-dom/vitest';
import { cleanup } from '@testing-library/react';
import { afterAll, afterEach, beforeAll } from 'vitest';
import { server } from './mocks/server';

// ============================================================
// MSW SERVER LIFECYCLE
// ============================================================

beforeAll(() => server.listen({ onUnhandledRequest: 'warn' }));
afterEach(() => {
  server.resetHandlers();
  cleanup();
});
afterAll(() => server.close());

// ============================================================
// BROWSER API MOCKS
// ============================================================

/** Mock matchMedia for components using media queries */
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: (query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: () => {},
    removeListener: () => {},
    addEventListener: () => {},
    removeEventListener: () => {},
    dispatchEvent: () => false,
  }),
});

/** Mock ResizeObserver for components that observe element dimensions */
class MockResizeObserver {
  observe() {}
  unobserve() {}
  disconnect() {}
}
window.ResizeObserver = MockResizeObserver as unknown as typeof ResizeObserver;

/** Mock IntersectionObserver for components using lazy loading or visibility detection */
class MockIntersectionObserver {
  observe() {}
  unobserve() {}
  disconnect() {}
}
window.IntersectionObserver = MockIntersectionObserver as unknown as typeof IntersectionObserver;
