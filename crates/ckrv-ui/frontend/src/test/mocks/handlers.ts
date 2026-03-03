/**
 * @module test/mocks/handlers
 * @description
 * Base MSW request handlers for common API endpoints. These provide
 * sensible defaults for all tests. Individual tests can override
 * specific handlers via server.use().
 *
 * @context
 * Loaded by the MSW server in setup.ts. Only contains base/happy-path
 * handlers. Error and edge-case handlers belong in individual test files.
 *
 * @dependencies
 * - msw: HTTP request interception
 * - fixtures: Factory functions for response data
 */

import { http, HttpResponse } from 'msw';
import { createSystemStatus, createAgent, createSpec, createDockerStatus } from './fixtures';

export const handlers = [
  // GET /api/status
  http.get('/api/status', () => {
    return HttpResponse.json(createSystemStatus());
  }),

  // GET /api/agents
  http.get('/api/agents', () => {
    return HttpResponse.json({
      agents: [
        createAgent(),
        createAgent({
          name: 'codex-agent',
          display_name: 'OpenAI Codex',
          agent_type: 'codex',
          model: null,
          is_default: false,
        }),
      ],
    });
  }),

  // GET /api/specs
  http.get('/api/specs', () => {
    return HttpResponse.json({
      specs: [
        createSpec(),
        createSpec({
          name: '043-dashboard',
          title: 'Dashboard Redesign',
          status: 'planned',
          has_plan: true,
          has_tasks: true,
          task_count: 5,
        }),
      ],
      count: 2,
    });
  }),

  // GET /api/docker
  http.get('/api/docker', () => {
    return HttpResponse.json(createDockerStatus());
  }),
];
