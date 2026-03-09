/**
 * @module useConnection.test
 * @description
 * Tests for the useConnection hook. Validates connectivity detection via /health,
 * consecutive failure threshold behavior, status transitions, periodic checking,
 * and cleanup on unmount.
 *
 * @context
 * Uses MSW for HTTP mocking. The hook pings /health (not /api/status) to avoid
 * duplicating the React Query status poll. A consecutive failure threshold of 2
 * prevents status flicker from transient network errors.
 *
 * @dependencies
 * - @testing-library/react: renderHook, waitFor, act
 * - vitest: describe, it, expect, vi, afterEach
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
  it('transitions to "connected" when /health returns 200', async () => {
    // Base handler already returns 200 for /health
    const { result } = renderHook(() => useConnection());

    await waitFor(() => expect(result.current.status).toBe('connected'));
  });

  it('sets lastChecked to a Date after check completes', async () => {
    const { result } = renderHook(() => useConnection());

    await waitFor(() => expect(result.current.lastChecked).not.toBeNull());
    expect(result.current.lastChecked).toBeInstanceOf(Date);
  });

  it('checkNow() triggers an immediate re-check and updates status', async () => {
    // Start with /health erroring — use short interval so 2nd failure fires quickly
    server.use(
      http.get('/health', () => HttpResponse.error()),
    );

    const { result } = renderHook(() => useConnection(200));

    // Wait until we get disconnected (after 2+ failures from initial check + short interval)
    await waitFor(() => expect(result.current.status).toBe('disconnected'));

    // Override to return success now
    server.use(
      http.get('/health', () => new HttpResponse('OK', { status: 200 })),
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

  // ============================================================
  // Consecutive failure threshold
  // ============================================================

  describe('consecutive failure threshold', () => {
    it('stays connected after a single failure if previously connected', async () => {
      // Start connected via default handler
      const { result } = renderHook(() => useConnection(60000));

      await waitFor(() => expect(result.current.status).toBe('connected'));

      // Override with error for next check
      server.use(
        http.get('/health', () => HttpResponse.error()),
      );

      // Trigger one failing check
      await act(async () => {
        result.current.checkNow();
      });

      // Should still be connected — threshold is 2
      expect(result.current.status).toBe('connected');
    });

    it('transitions to disconnected after 2 consecutive failures', async () => {
      // Start connected via default handler
      const { result } = renderHook(() => useConnection(60000));

      await waitFor(() => expect(result.current.status).toBe('connected'));

      // Override with error
      server.use(
        http.get('/health', () => HttpResponse.error()),
      );

      // Trigger first failing check
      await act(async () => {
        result.current.checkNow();
      });

      // Still connected after 1 failure
      expect(result.current.status).toBe('connected');

      // Trigger second failing check
      await act(async () => {
        result.current.checkNow();
      });

      // Now disconnected after 2 consecutive failures
      expect(result.current.status).toBe('disconnected');
    });

    it('transitions to disconnected after 2 consecutive non-ok responses', async () => {
      // Start connected via default handler
      const { result } = renderHook(() => useConnection(60000));

      await waitFor(() => expect(result.current.status).toBe('connected'));

      // Override with 500 status
      server.use(
        http.get('/health', () => new HttpResponse(null, { status: 500 })),
      );

      // Trigger first failing check
      await act(async () => {
        result.current.checkNow();
      });

      // Still connected after 1 failure
      expect(result.current.status).toBe('connected');

      // Trigger second failing check
      await act(async () => {
        result.current.checkNow();
      });

      // Now disconnected after 2 consecutive failures
      expect(result.current.status).toBe('disconnected');
    });

    it('resets failure counter on successful response', async () => {
      // Start connected via default handler
      const { result } = renderHook(() => useConnection(60000));

      await waitFor(() => expect(result.current.status).toBe('connected'));

      // Override with error
      server.use(
        http.get('/health', () => HttpResponse.error()),
      );

      // Trigger one failing check (failure count = 1)
      await act(async () => {
        result.current.checkNow();
      });

      expect(result.current.status).toBe('connected');

      // Restore success handler — resets failure counter to 0
      server.use(
        http.get('/health', () => new HttpResponse('OK', { status: 200 })),
      );

      await act(async () => {
        result.current.checkNow();
      });

      expect(result.current.status).toBe('connected');

      // Now override with error again
      server.use(
        http.get('/health', () => HttpResponse.error()),
      );

      // One failure after reset — should still be connected
      await act(async () => {
        result.current.checkNow();
      });

      expect(result.current.status).toBe('connected');
    });
  });
});
