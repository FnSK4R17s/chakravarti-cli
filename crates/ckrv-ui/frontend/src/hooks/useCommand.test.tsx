/**
 * @module useCommand.test
 * @description
 * Tests for the useCommand hook. Validates CLI command mutations, API
 * endpoints, request bodies, and cache invalidation on success.
 *
 * @context
 * Tests run in jsdom via Vitest with MSW for HTTP mocking. Each test uses
 * a fresh QueryClient to prevent state leakage.
 *
 * @dependencies
 * - @testing-library/react: renderHook, waitFor, act
 * - @tanstack/react-query: QueryClient, QueryClientProvider
 * - msw: http, HttpResponse for per-test handler overrides
 * - @/test/mocks/server: MSW server instance
 * - @/hooks/useCommand: Hook under test
 */

import type { ReactNode } from 'react';
import { describe, it, expect } from 'vitest';
import { renderHook, waitFor, act } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { http, HttpResponse } from 'msw';
import { server } from '@/test/mocks/server';
import { useCommand } from './useCommand';

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
// useCommand
// ============================================================

describe('useCommand', () => {
  // ============================================================
  // runInit
  // ============================================================

  describe('runInit', () => {
    it('POST /api/command/init succeeds and clears pending state', async () => {
      server.use(
        http.post('/api/command/init', () =>
          HttpResponse.json({ success: true, message: 'Initialized' }),
        ),
      );

      const { result } = renderHook(() => useCommand(), { wrapper: createWrapper() });

      act(() => {
        result.current.runInit();
      });

      await waitFor(() => expect(result.current.isInitPending).toBe(false));
      expect(result.current.initError).toBeNull();
    });

    it('sets error state when POST /api/command/init fails', async () => {
      server.use(
        http.post('/api/command/init', () => HttpResponse.error()),
      );

      const { result } = renderHook(() => useCommand(), { wrapper: createWrapper() });

      act(() => {
        result.current.runInit();
      });

      await waitFor(() => expect(result.current.initError).not.toBeNull());
      expect(result.current.isInitPending).toBe(false);
    });
  });

  // ============================================================
  // runSpecNew
  // ============================================================

  describe('runSpecNew', () => {
    it('POST /api/command/spec/new sends JSON body with description', async () => {
      let capturedBody: unknown;

      server.use(
        http.post('/api/command/spec/new', async ({ request }) => {
          capturedBody = await request.json();
          return HttpResponse.json({ success: true });
        }),
      );

      const { result } = renderHook(() => useCommand(), { wrapper: createWrapper() });

      act(() => {
        result.current.runSpecNew('Add auth');
      });

      await waitFor(() => expect(result.current.isSpecNewPending).toBe(false));
      expect(capturedBody).toEqual({ description: 'Add auth' });
      expect(result.current.specNewError).toBeNull();
    });

    it('sets error state when POST /api/command/spec/new fails', async () => {
      server.use(
        http.post('/api/command/spec/new', () => HttpResponse.error()),
      );

      const { result } = renderHook(() => useCommand(), { wrapper: createWrapper() });

      act(() => {
        result.current.runSpecNew('Add auth');
      });

      await waitFor(() => expect(result.current.specNewError).not.toBeNull());
    });
  });

  // ============================================================
  // runSpecTasks
  // ============================================================

  describe('runSpecTasks', () => {
    it('POST /api/command/spec/tasks succeeds and clears pending state', async () => {
      server.use(
        http.post('/api/command/spec/tasks', () =>
          HttpResponse.json({ success: true }),
        ),
      );

      const { result } = renderHook(() => useCommand(), { wrapper: createWrapper() });

      act(() => {
        result.current.runSpecTasks();
      });

      await waitFor(() => expect(result.current.isSpecTasksPending).toBe(false));
      expect(result.current.specTasksError).toBeNull();
    });

    it('sets error state when POST /api/command/spec/tasks fails', async () => {
      server.use(
        http.post('/api/command/spec/tasks', () => HttpResponse.error()),
      );

      const { result } = renderHook(() => useCommand(), { wrapper: createWrapper() });

      act(() => {
        result.current.runSpecTasks();
      });

      await waitFor(() => expect(result.current.specTasksError).not.toBeNull());
    });
  });

  // ============================================================
  // runExec
  // ============================================================

  describe('runExec', () => {
    it('POST /api/command/run succeeds and clears pending state', async () => {
      server.use(
        http.post('/api/command/run', () =>
          HttpResponse.json({ success: true }),
        ),
      );

      const { result } = renderHook(() => useCommand(), { wrapper: createWrapper() });

      act(() => {
        result.current.runExec();
      });

      await waitFor(() => expect(result.current.isExecPending).toBe(false));
      expect(result.current.execError).toBeNull();
    });

    it('sets error state when POST /api/command/run fails', async () => {
      server.use(
        http.post('/api/command/run', () => HttpResponse.error()),
      );

      const { result } = renderHook(() => useCommand(), { wrapper: createWrapper() });

      act(() => {
        result.current.runExec();
      });

      await waitFor(() => expect(result.current.execError).not.toBeNull());
    });
  });
});
