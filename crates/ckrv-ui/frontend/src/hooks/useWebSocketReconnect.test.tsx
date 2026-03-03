/**
 * @module useWebSocketReconnect.test
 * @description
 * Tests for the useWebSocketReconnect hook. Covers connection lifecycle,
 * exponential-backoff reconnection, max-retry exhaustion, manual reconnect,
 * send behaviour, and cleanup on unmount.
 *
 * @context
 * Fake timers are used so that retry delays (default: 5 000 ms initial,
 * multiplier 2) can be advanced synchronously without real waiting.
 *
 * @dependencies
 * - @testing-library/react: renderHook, act
 * - @/test/mocks/websocket: MockWebSocket stub
 * - vitest: describe, it, expect, vi, beforeEach, afterEach
 */

import { renderHook, act } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

import { MockWebSocket } from '@/test/mocks/websocket';
import { useWebSocketReconnect } from './useWebSocketReconnect';

// ============================================================
// Setup / Teardown
// ============================================================

beforeEach(() => {
  MockWebSocket.reset();
  vi.stubGlobal('WebSocket', MockWebSocket);
  vi.useFakeTimers();
});

afterEach(() => {
  vi.runAllTimers();
  vi.useRealTimers();
  vi.unstubAllGlobals();
});

// ============================================================
// Tests
// ============================================================

describe('useWebSocketReconnect', () => {
  it('initial state is disconnected', () => {
    const { result } = renderHook(() => useWebSocketReconnect());

    expect(result.current.state.status).toBe('disconnected');
    expect(result.current.state.retryCount).toBe(0);
    expect(result.current.state.retryCountdown).toBe(0);
  });

  it('connect() creates a WebSocket and transitions connecting → connected', () => {
    const { result } = renderHook(() => useWebSocketReconnect());

    act(() => {
      result.current.connect('ws://test.example.com/ws');
    });

    // Should be 'connecting' immediately after connect()
    expect(result.current.state.status).toBe('connecting');
    expect(MockWebSocket.instances).toHaveLength(1);
    expect(MockWebSocket.instances[0].url).toBe('ws://test.example.com/ws');

    // Simulate the socket opening
    act(() => {
      MockWebSocket.instances[0].simulateOpen();
    });

    expect(result.current.state.status).toBe('connected');
    expect(result.current.state.retryCount).toBe(0);
  });

  it('disconnect() closes the WebSocket and transitions to disconnected', () => {
    const { result } = renderHook(() => useWebSocketReconnect());

    act(() => { result.current.connect('ws://test.example.com/ws'); });
    act(() => { MockWebSocket.instances[0].simulateOpen(); });

    expect(result.current.state.status).toBe('connected');

    act(() => { result.current.disconnect(); });

    expect(result.current.state.status).toBe('disconnected');
    expect(MockWebSocket.instances[0].readyState).toBe(MockWebSocket.CLOSED);
  });

  it('auto-reconnect triggers after initialDelay with exponential backoff', () => {
    const { result } = renderHook(() =>
      useWebSocketReconnect({ maxRetries: 3, initialDelay: 5000, backoffMultiplier: 2 })
    );

    act(() => { result.current.connect('ws://test.example.com/ws'); });
    act(() => { MockWebSocket.instances[0].simulateOpen(); });
    expect(result.current.state.status).toBe('connected');

    // Simulate an unexpected close
    act(() => { MockWebSocket.instances[0].simulateClose(); });

    expect(result.current.state.status).toBe('reconnecting');
    expect(result.current.state.retryCount).toBe(1);

    // Advance by initialDelay (5 000 ms) to trigger the first retry
    act(() => { vi.advanceTimersByTime(5000); });

    // A second MockWebSocket instance should now exist
    expect(MockWebSocket.instances).toHaveLength(2);
    expect(MockWebSocket.instances[1].url).toBe('ws://test.example.com/ws');
  });

  it('max retries exhausted → disconnected with lastError set', () => {
    const onClose = vi.fn();
    const { result } = renderHook(() =>
      useWebSocketReconnect({ maxRetries: 1, initialDelay: 1000, onClose })
    );

    act(() => { result.current.connect('ws://test.example.com/ws'); });
    act(() => { MockWebSocket.instances[0].simulateOpen(); });

    // First close → reconnecting (retryCount 0 < maxRetries 1, so schedules retry)
    act(() => { MockWebSocket.instances[0].simulateClose(); });
    expect(result.current.state.status).toBe('reconnecting');

    // Advance to trigger the retry connection
    act(() => { vi.advanceTimersByTime(1000); });
    expect(MockWebSocket.instances).toHaveLength(2);

    // Second close → retryCount 1 equals maxRetries 1 → exhausted
    act(() => { MockWebSocket.instances[1].simulateClose(); });

    expect(result.current.state.status).toBe('disconnected');
    expect(result.current.state.lastError).toMatch(/maximum retry/i);
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('reconnect() manually triggers a new connection', () => {
    const { result } = renderHook(() => useWebSocketReconnect());

    act(() => { result.current.connect('ws://test.example.com/ws'); });
    act(() => { MockWebSocket.instances[0].simulateOpen(); });
    act(() => { result.current.disconnect(); });

    expect(result.current.state.status).toBe('disconnected');

    act(() => { result.current.reconnect(); });

    expect(result.current.state.status).toBe('connecting');
    expect(MockWebSocket.instances).toHaveLength(2);
  });

  it('send() when connected sends data through the socket', () => {
    const { result } = renderHook(() => useWebSocketReconnect());

    act(() => { result.current.connect('ws://test.example.com/ws'); });
    act(() => { MockWebSocket.instances[0].simulateOpen(); });

    const sendSpy = vi.spyOn(MockWebSocket.instances[0], 'send');

    act(() => { result.current.send('{"type":"ping"}'); });

    expect(sendSpy).toHaveBeenCalledWith('{"type":"ping"}');
  });

  it('send() when disconnected logs a warning and does not throw', () => {
    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});
    const { result } = renderHook(() => useWebSocketReconnect());

    // Never connect → socket is null / not OPEN
    expect(() => {
      act(() => { result.current.send('{"type":"ping"}'); });
    }).not.toThrow();

    expect(warnSpy).toHaveBeenCalledWith(
      expect.stringContaining('not connected')
    );

    warnSpy.mockRestore();
  });

  it('cleanup on unmount closes the WebSocket and clears timers', () => {
    const { result, unmount } = renderHook(() => useWebSocketReconnect());

    act(() => { result.current.connect('ws://test.example.com/ws'); });
    act(() => { MockWebSocket.instances[0].simulateOpen(); });

    const closeSpy = vi.spyOn(MockWebSocket.instances[0], 'close');

    unmount();

    expect(closeSpy).toHaveBeenCalled();
  });
});
