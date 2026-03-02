/**
 * @module useConnection.test
 * @description
 * Tests for the useConnection hook. Validates connectivity detection,
 * status transitions, periodic checking, and cleanup on unmount.
 *
 * @context
 * Uses fake timers for interval testing and MSW for HTTP mocking.
 * No QueryClient needed as hook uses raw fetch.
 *
 * @dependencies
 * - @testing-library/react: renderHook, waitFor, act
 * - vitest: describe, it, expect, vi, beforeEach, afterEach
 * - msw: http, HttpResponse
 * - @/test/mocks/server: MSW server instance
 */

import { describe, it, expect, vi, afterEach } from 'vitest';
import { renderHook, waitFor, act } from '@testing-library/react';
import { http, HttpResponse } from 'msw';
import { server } from '@/test/mocks/server';
import { useConnection } from './useConnection';

afterEach(() => {
  vi.restoreAllMocks();
});

// ============================================================
// useConnection
// ============================================================

describe('useConnection', () => {
  it('transitions to "connected" when /api/status returns 200', async () => {
    // Base handler already returns 200 for /api/status
    const { result } = renderHook(() => useConnection());

    await waitFor(() => expect(result.current.status).toBe('connected'));
  });

  it('sets "disconnected" when fetch fails', async () => {
    server.use(
      http.get('/api/status', () => HttpResponse.error()),
    );

    const { result } = renderHook(() => useConnection());

    await waitFor(() => expect(result.current.status).toBe('disconnected'));
  });

  it('sets lastChecked to a Date after check completes', async () => {
    const { result } = renderHook(() => useConnection());

    await waitFor(() => expect(result.current.lastChecked).not.toBeNull());
    expect(result.current.lastChecked).toBeInstanceOf(Date);
  });

  it('checkNow() triggers an immediate re-check and updates status', async () => {
    server.use(
      http.get('/api/status', () => HttpResponse.error()),
    );

    const { result } = renderHook(() => useConnection());

    await waitFor(() => expect(result.current.status).toBe('disconnected'));

    // Override to return success now
    server.use(
      http.get('/api/status', () => HttpResponse.json({ is_ready: true })),
    );

    act(() => {
      result.current.checkNow();
    });

    await waitFor(() => expect(result.current.status).toBe('connected'));
  });

  it('cleans up the interval on unmount', async () => {
    const clearIntervalSpy = vi.spyOn(global, 'clearInterval');

    const { result, unmount } = renderHook(() => useConnection(5000));

    await waitFor(() => expect(result.current.status).toBeDefined());

    unmount();

    expect(clearIntervalSpy).toHaveBeenCalled();
  });
});
