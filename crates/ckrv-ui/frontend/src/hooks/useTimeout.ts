/**
 * useTimeout Hook (T013)
 * 
 * Custom hook for managing timeouts with automatic cleanup on unmount.
 * Prevents memory leaks from orphaned setTimeout calls.
 * 
 * Addresses BUG-005: Auto-Collapse Timer Creates Memory Leak
 */

import { useRef, useCallback, useEffect } from 'react';

export interface UseTimeoutReturn {
    /** Set a timeout that will be automatically cleaned up */
    set: (callback: () => void, delay: number) => number;
    /** Clear a specific timeout by ID */
    clear: (id: number) => void;
    /** Clear all managed timeouts */
    clearAll: () => void;
}

export function useTimeout(): UseTimeoutReturn {
    // Track all active timeout IDs
    const timeoutIdsRef = useRef<Set<number>>(new Set());

    // Set a new timeout
    const set = useCallback((callback: () => void, delay: number): number => {
        const id = window.setTimeout(() => {
            // Remove from tracked set when executed
            timeoutIdsRef.current.delete(id);
            callback();
        }, delay);

        // Track the timeout ID
        timeoutIdsRef.current.add(id);
        return id;
    }, []);

    // Clear a specific timeout
    const clear = useCallback((id: number): void => {
        window.clearTimeout(id);
        timeoutIdsRef.current.delete(id);
    }, []);

    // Clear all timeouts
    const clearAll = useCallback((): void => {
        timeoutIdsRef.current.forEach(id => {
            window.clearTimeout(id);
        });
        timeoutIdsRef.current.clear();
    }, []);

    // Cleanup on unmount
    useEffect(() => {
        return () => {
            // Clear all timeouts when component unmounts
            timeoutIdsRef.current.forEach(id => {
                window.clearTimeout(id);
            });
            timeoutIdsRef.current.clear();
        };
    }, []);

    return { set, clear, clearAll };
}

/**
 * Convenience hook for a single delayed callback
 */
export function useDelayedCallback(
    callback: () => void,
    delay: number,
    enabled: boolean = true
): void {
    const { set, clearAll } = useTimeout();
    const callbackRef = useRef(callback);

    // Keep callback ref updated
    useEffect(() => {
        callbackRef.current = callback;
    }, [callback]);

    useEffect(() => {
        if (enabled) {
            set(() => callbackRef.current(), delay);
        }
        return () => {
            clearAll();
        };
    }, [enabled, delay, set, clearAll]);
}

export default useTimeout;
