/**
 * @module test/mocks/server
 * @description
 * MSW server instance for Node.js (Vitest) test environment. Configures
 * the mock service worker with base request handlers.
 *
 * @context
 * Imported by test/setup.ts for lifecycle management (listen, reset, close).
 * Test files import this to add test-specific overrides via server.use().
 *
 * @dependencies
 * - msw/node: Node.js MSW server
 * - handlers: Base request handlers
 */

import { setupServer } from 'msw/node';
import { handlers } from './handlers';

export const server = setupServer(...handlers);
