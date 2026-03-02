/**
 * @module useTimeout.test
 * @description
 * Tests for useTimeout and useDelayedCallback hooks. Validates timeout
 * management, cleanup on unmount, and conditional execution.
 *
 * @context
 * Uses fake timers for deterministic timeout testing. No MSW or QueryClient needed.
 *
 * @dependencies
 * - @testing-library/react: renderHook, act
 * - vitest: describe, it, expect, vi, beforeEach, afterEach
 */

import { renderHook, act } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { useTimeout, useDelayedCallback } from './useTimeout';

describe('useTimeout', () => {
    beforeEach(() => {
        vi.useFakeTimers();
    });

    afterEach(() => {
        vi.useRealTimers();
    });

    it('set() fires callback after delay', () => {
        const cb = vi.fn();
        const { result } = renderHook(() => useTimeout());

        act(() => {
            result.current.set(cb, 1000);
        });

        expect(cb).not.toHaveBeenCalled();

        act(() => {
            vi.advanceTimersByTime(1000);
        });

        expect(cb).toHaveBeenCalledOnce();
    });

    it('clear() cancels a specific timeout', () => {
        const cb = vi.fn();
        const { result } = renderHook(() => useTimeout());

        let id: number;
        act(() => {
            id = result.current.set(cb, 1000);
        });

        act(() => {
            result.current.clear(id);
        });

        act(() => {
            vi.advanceTimersByTime(2000);
        });

        expect(cb).not.toHaveBeenCalled();
    });

    it('clearAll() cancels all active timeouts', () => {
        const cb1 = vi.fn();
        const cb2 = vi.fn();
        const { result } = renderHook(() => useTimeout());

        act(() => {
            result.current.set(cb1, 1000);
            result.current.set(cb2, 3000);
        });

        act(() => {
            result.current.clearAll();
        });

        act(() => {
            vi.advanceTimersByTime(5000);
        });

        expect(cb1).not.toHaveBeenCalled();
        expect(cb2).not.toHaveBeenCalled();
    });

    it('cleanup on unmount clears all timeouts', () => {
        const cb = vi.fn();
        const { result, unmount } = renderHook(() => useTimeout());

        act(() => {
            result.current.set(cb, 1000);
        });

        unmount();

        act(() => {
            vi.advanceTimersByTime(2000);
        });

        expect(cb).not.toHaveBeenCalled();
    });
});

describe('useDelayedCallback', () => {
    beforeEach(() => {
        vi.useFakeTimers();
    });

    afterEach(() => {
        vi.useRealTimers();
    });

    it('fires callback after delay when enabled', () => {
        const cb = vi.fn();
        renderHook(() => useDelayedCallback(cb, 500));

        act(() => {
            vi.advanceTimersByTime(500);
        });

        expect(cb).toHaveBeenCalledOnce();
    });

    it('does not fire callback when disabled', () => {
        const cb = vi.fn();
        renderHook(() => useDelayedCallback(cb, 500, false));

        act(() => {
            vi.advanceTimersByTime(1000);
        });

        expect(cb).not.toHaveBeenCalled();
    });

    it('cancels previous timeout when re-rendered with new delay', () => {
        const cb = vi.fn();
        const { rerender } = renderHook(
            ({ delay }) => useDelayedCallback(cb, delay),
            { initialProps: { delay: 1000 } }
        );

        act(() => {
            vi.advanceTimersByTime(500); // halfway through original
        });

        rerender({ delay: 2000 }); // reset with new delay

        act(() => {
            vi.advanceTimersByTime(1500); // past original deadline, before new
        });

        expect(cb).not.toHaveBeenCalled();

        act(() => {
            vi.advanceTimersByTime(500); // now at 2000ms from rerender
        });

        expect(cb).toHaveBeenCalledOnce();
    });
});
