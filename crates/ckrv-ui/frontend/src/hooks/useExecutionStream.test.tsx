/**
 * @module useExecutionStream.test
 * @description
 * Tests for the useExecutionStream hook. Verifies the web (WebSocket) path
 * for starting/stopping execution runs, processing log messages, and cleanup.
 *
 * @context
 * The hook uses `'__TAURI_INTERNALS__' in window` to detect Tauri mode.
 * In jsdom that property is absent, so all tests exercise the WebSocket path.
 *
 * @dependencies
 * - @testing-library/react: renderHook, act, waitFor
 * - msw / @/test/mocks/server: HTTP interception
 * - @/test/mocks/websocket: MockWebSocket stub
 * - vitest: describe, it, expect, vi, beforeEach, afterEach
 */

import type { ReactNode } from 'react';
import { renderHook, act, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { http, HttpResponse } from 'msw';
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

import { server } from '@/test/mocks/server';
import { MockWebSocket } from '@/test/mocks/websocket';
import { useExecutionStream } from './useExecutionStream';

// ============================================================
// Helpers
// ============================================================

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return function Wrapper({ children }: { children: ReactNode }) {
    return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
  };
}

// ============================================================
// MSW handlers
// ============================================================

const startHandler = http.post('/api/execution/start', () =>
  HttpResponse.json({ run_id: 'test-run-1', status: 'running' })
);

const stopHandler = http.post('/api/execution/stop', () =>
  HttpResponse.json({ success: true })
);

// ============================================================
// Setup / Teardown
// ============================================================

beforeEach(() => {
  MockWebSocket.reset();
  vi.stubGlobal('WebSocket', MockWebSocket);
  server.use(startHandler, stopHandler);
});

afterEach(() => {
  vi.unstubAllGlobals();
});

// ============================================================
// Tests
// ============================================================

describe('useExecutionStream', () => {
  it('initial state is idle with empty logs', () => {
    const { result } = renderHook(() => useExecutionStream(), {
      wrapper: createWrapper(),
    });

    expect(result.current.status).toBe('idle');
    expect(result.current.logs).toEqual([]);
    expect(result.current.error).toBeNull();
    expect(result.current.runId).toBeNull();
    expect(result.current.batches).toEqual([]);
  });

  it('startRun() calls POST /api/execution/start and opens a WebSocket', async () => {
    const { result } = renderHook(() => useExecutionStream(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.startRun('test-spec');
    });

    expect(result.current.status).toBe('running');
    expect(result.current.runId).not.toBeNull();
    // A MockWebSocket instance must have been created
    expect(MockWebSocket.instances).toHaveLength(1);
    expect(MockWebSocket.instances[0].url).toContain('/api/execution/ws');
  });

  it('WebSocket messages are processed into logs', async () => {
    const { result } = renderHook(() => useExecutionStream(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.startRun('test-spec');
    });

    const mockWs = MockWebSocket.instances[0];

    act(() => {
      mockWs.simulateOpen();
      mockWs.simulateMessage(
        JSON.stringify({
          type: 'log',
          message: 'Hello from execution',
          timestamp: '2026-01-01T00:00:00Z',
        })
      );
    });

    await waitFor(() => {
      expect(result.current.logs).toHaveLength(1);
    });

    expect(result.current.logs[0].message).toBe('Hello from execution');
    expect(result.current.logs[0].type).toBe('log');
  });

  it('error messages update status to "error"', async () => {
    const { result } = renderHook(() => useExecutionStream(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.startRun('test-spec');
    });

    const mockWs = MockWebSocket.instances[0];

    act(() => {
      mockWs.simulateOpen();
      mockWs.simulateMessage(
        JSON.stringify({
          type: 'error',
          message: 'Something went wrong',
          timestamp: '2026-01-01T00:00:00Z',
        })
      );
    });

    await waitFor(() => {
      expect(result.current.status).toBe('error');
    });

    expect(result.current.error).toBe('Something went wrong');
    expect(result.current.logs).toHaveLength(1);
  });

  it('success messages update status to "done"', async () => {
    const { result } = renderHook(() => useExecutionStream(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.startRun('test-spec');
    });

    const mockWs = MockWebSocket.instances[0];

    act(() => {
      mockWs.simulateOpen();
      mockWs.simulateMessage(
        JSON.stringify({
          type: 'success',
          message: 'Execution complete',
          timestamp: '2026-01-01T00:00:00Z',
        })
      );
    });

    await waitFor(() => {
      expect(result.current.status).toBe('done');
    });

    expect(result.current.logs).toHaveLength(1);
    expect(result.current.logs[0].type).toBe('success');
  });

  it('stopRun() closes the WebSocket and calls POST /api/execution/stop', async () => {
    const { result } = renderHook(() => useExecutionStream(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.startRun('test-spec');
    });

    const mockWs = MockWebSocket.instances[0];

    act(() => {
      mockWs.simulateOpen();
    });

    await act(async () => {
      await result.current.stopRun();
    });

    expect(result.current.status).toBe('idle');
    expect(result.current.runId).toBeNull();
    // After close() the readyState should be CLOSED
    expect(mockWs.readyState).toBe(MockWebSocket.CLOSED);
  });

  it('clearLogs() resets logs and error', async () => {
    const { result } = renderHook(() => useExecutionStream(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.startRun('test-spec');
    });

    const mockWs = MockWebSocket.instances[0];

    act(() => {
      mockWs.simulateOpen();
      mockWs.simulateMessage(
        JSON.stringify({
          type: 'error',
          message: 'oops',
          timestamp: '2026-01-01T00:00:00Z',
        })
      );
    });

    await waitFor(() => {
      expect(result.current.logs).toHaveLength(1);
    });

    act(() => {
      result.current.clearLogs();
    });

    expect(result.current.logs).toEqual([]);
    expect(result.current.error).toBeNull();
  });

  it('cleanup on unmount closes the WebSocket', async () => {
    const { result, unmount } = renderHook(() => useExecutionStream(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.startRun('test-spec');
    });

    const mockWs = MockWebSocket.instances[0];
    act(() => { mockWs.simulateOpen(); });

    // Spy on the mock's close method
    const closeSpy = vi.spyOn(mockWs, 'close');

    unmount();

    expect(closeSpy).toHaveBeenCalled();
  });
});
