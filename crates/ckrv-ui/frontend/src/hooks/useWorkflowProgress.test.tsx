/**
 * @module useWorkflowProgress.test
 * @description
 * Tests for the useWorkflowProgress hook. Validates stage completion
 * calculation, branch-based spec auto-selection, and override behavior.
 *
 * @context
 * Tests run in jsdom via Vitest with MSW for HTTP mocking. Each test uses
 * a fresh QueryClient to prevent state leakage.
 *
 * @dependencies
 * - @testing-library/react: renderHook, waitFor
 * - @tanstack/react-query: QueryClient, QueryClientProvider
 * - msw: http, HttpResponse
 * - @/test/mocks/server: MSW server instance
 * - @/hooks/useWorkflowProgress: Hook under test
 */

import type { ReactNode } from 'react';
import { describe, it, expect } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { http, HttpResponse } from 'msw';
import { server } from '@/test/mocks/server';
import { useWorkflowProgress } from './useWorkflowProgress';

// ============================================================
// WRAPPER FACTORY
// ============================================================

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });
  return function Wrapper({ children }: { children: ReactNode }) {
    return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
  };
}

// ============================================================
// useWorkflowProgress
// ============================================================

describe('useWorkflowProgress', () => {
  it('all stages pending when no specs exist', async () => {
    server.use(
      http.get('/api/status', () =>
        HttpResponse.json({ is_ready: true, active_branch: 'main' }),
      ),
      http.get('/api/specs', () =>
        HttpResponse.json({ specs: [], count: 0 }),
      ),
    );

    const { result } = renderHook(() => useWorkflowProgress(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      const stages = result.current;
      expect(stages.every(s => s.status === 'pending')).toBe(true);
    });

    expect(result.current).toHaveLength(4);
    expect(result.current[0].status).toBe('pending');
    expect(result.current[1].status).toBe('pending');
    expect(result.current[2].status).toBe('pending');
    expect(result.current[3].status).toBe('pending');
  });

  it('auto-selects spec matching active branch exactly', async () => {
    server.use(
      http.get('/api/status', () =>
        HttpResponse.json({ is_ready: true, active_branch: '042-add-auth' }),
      ),
      http.get('/api/specs', () =>
        HttpResponse.json({
          specs: [
            {
              name: '042-add-auth',
              has_tasks: true,
              has_plan: false,
              has_implementation: false,
            },
            {
              name: '043-dashboard',
              has_tasks: false,
              has_plan: false,
              has_implementation: false,
            },
          ],
          count: 2,
        }),
      ),
    );

    const { result } = renderHook(() => useWorkflowProgress(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      // spec stage should be complete (found the spec)
      expect(result.current[0].status).toBe('complete');
    });

    // has_tasks is true for the matched spec
    expect(result.current[1].status).toBe('complete');
    // has_plan is false
    expect(result.current[2].status).toBe('pending');
    // has_implementation is false
    expect(result.current[3].status).toBe('pending');
  });

  it('falls back to first spec when branch is "main"', async () => {
    server.use(
      http.get('/api/status', () =>
        HttpResponse.json({ is_ready: true, active_branch: 'main' }),
      ),
      http.get('/api/specs', () =>
        HttpResponse.json({
          specs: [
            {
              name: '042-add-auth',
              has_tasks: true,
              has_plan: true,
              has_implementation: false,
            },
          ],
          count: 1,
        }),
      ),
    );

    const { result } = renderHook(() => useWorkflowProgress(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current[0].status).toBe('complete');
    });

    // spec complete (first spec used as fallback)
    expect(result.current[0].status).toBe('complete');
    // tasks complete
    expect(result.current[1].status).toBe('complete');
    // plan complete
    expect(result.current[2].status).toBe('complete');
    // run pending
    expect(result.current[3].status).toBe('pending');
  });

  it('matches branch suffix after "/" for feature branches', async () => {
    server.use(
      http.get('/api/status', () =>
        HttpResponse.json({ is_ready: true, active_branch: 'feature/042-add-auth' }),
      ),
      http.get('/api/specs', () =>
        HttpResponse.json({
          specs: [
            {
              name: '042-add-auth',
              has_tasks: false,
              has_plan: false,
              has_implementation: false,
            },
          ],
          count: 1,
        }),
      ),
    );

    const { result } = renderHook(() => useWorkflowProgress(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      // spec found via suffix match
      expect(result.current[0].status).toBe('complete');
    });

    // tasks/plan/run pending since flags are false
    expect(result.current[1].status).toBe('pending');
    expect(result.current[2].status).toBe('pending');
    expect(result.current[3].status).toBe('pending');
  });

  it('uses overrideSpec when provided', async () => {
    server.use(
      http.get('/api/status', () =>
        HttpResponse.json({ is_ready: true, active_branch: 'some-other-branch' }),
      ),
      http.get('/api/specs', () =>
        HttpResponse.json({
          specs: [
            {
              name: '042-add-auth',
              has_tasks: true,
              has_plan: true,
              has_implementation: true,
            },
            {
              name: '043-dashboard',
              has_tasks: false,
              has_plan: false,
              has_implementation: false,
            },
          ],
          count: 2,
        }),
      ),
    );

    const { result } = renderHook(() => useWorkflowProgress('042-add-auth'), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current[3].status).toBe('complete');
    });

    // All stages complete for 042-add-auth
    expect(result.current[0].status).toBe('complete');
    expect(result.current[1].status).toBe('complete');
    expect(result.current[2].status).toBe('complete');
    expect(result.current[3].status).toBe('complete');
  });

  it('all stages complete when spec has all flags true', async () => {
    server.use(
      http.get('/api/status', () =>
        HttpResponse.json({ is_ready: true, active_branch: 'main' }),
      ),
      http.get('/api/specs', () =>
        HttpResponse.json({
          specs: [
            {
              name: '042-add-auth',
              has_tasks: true,
              has_plan: true,
              has_implementation: true,
            },
          ],
          count: 1,
        }),
      ),
    );

    const { result } = renderHook(() => useWorkflowProgress(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current[3].status).toBe('complete');
    });

    expect(result.current[0].status).toBe('complete');
    expect(result.current[1].status).toBe('complete');
    expect(result.current[2].status).toBe('complete');
    expect(result.current[3].status).toBe('complete');
  });

  it('falls back to first spec when no branch match and marks stages by flags', async () => {
    server.use(
      http.get('/api/status', () =>
        HttpResponse.json({ is_ready: true, active_branch: 'unrelated-branch' }),
      ),
      http.get('/api/specs', () =>
        HttpResponse.json({
          specs: [
            {
              name: '042-add-auth',
              has_tasks: true,
              has_plan: false,
              has_implementation: false,
            },
          ],
          count: 1,
        }),
      ),
    );

    const { result } = renderHook(() => useWorkflowProgress(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      // spec stage is complete (first spec used as fallback)
      expect(result.current[0].status).toBe('complete');
    });

    // tasks complete (has_tasks = true)
    expect(result.current[1].status).toBe('complete');
    // plan pending
    expect(result.current[2].status).toBe('pending');
    // run pending
    expect(result.current[3].status).toBe('pending');
  });
});
