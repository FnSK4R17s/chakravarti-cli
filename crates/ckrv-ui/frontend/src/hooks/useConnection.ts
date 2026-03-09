/**
 * @module useConnection
 * @description
 * Hook for monitoring backend connectivity. Pings /health periodically to
 * check if the backend is alive and reports connected/disconnected/connecting states.
 * Uses a consecutive failure threshold to avoid flapping on transient errors.
 *
 * @context
 * Used for showing connection status indicators and warning users when backend
 * is unreachable. Enables graceful degradation when network fails.
 *
 * @dependencies
 * - fetch: For making health check requests
 * - AbortController: For request timeout handling
 *
 * @example
 * const { status, lastChecked, checkNow } = useConnection(5000);
 * // status: 'connected' | 'disconnected' | 'connecting'
 */

// === IMPORTS ===
import { useState, useEffect, useCallback, useRef } from 'react';

// === TYPES ===

export type ConnectionStatus = 'connected' | 'disconnected' | 'connecting';

interface UseConnectionResult {
    status: ConnectionStatus;
    lastChecked: Date | null;
    checkNow: () => void;
}

// === CONSTANTS ===

/** Number of consecutive failures before showing disconnected */
const FAILURE_THRESHOLD = 2;

/** Abort timeout for health check requests in milliseconds */
const HEALTH_CHECK_TIMEOUT_MS = 5000;

// === HOOK ===

/**
 * Hook to monitor backend connectivity.
 * Pings the /health endpoint periodically to check if backend is alive.
 * Requires multiple consecutive failures before transitioning to disconnected,
 * preventing status flicker from transient network issues.
 *
 * @param intervalMs - Interval in milliseconds between connectivity checks (default: 5000)
 * @returns Object containing:
 *   - status: Current connection status ('connected' | 'disconnected' | 'connecting')
 *   - lastChecked: Timestamp of last connectivity check or null
 *   - checkNow: Function to trigger an immediate connectivity check
 */
export const useConnection = (intervalMs: number = 5000): UseConnectionResult => {
    /** Current connection status displayed to the user */
    const [status, setStatus] = useState<ConnectionStatus>('disconnected');

    /** Timestamp of the most recent health check completion */
    const [lastChecked, setLastChecked] = useState<Date | null>(null);

    /** Tracks consecutive health check failures without triggering re-renders */
    const failureCountRef = useRef(0);

    const checkConnection = useCallback(async () => {
        try {
            const controller = new AbortController();
            const timeoutId = setTimeout(() => controller.abort(), HEALTH_CHECK_TIMEOUT_MS);

            const response = await fetch('/health', {
                method: 'GET',
                signal: controller.signal,
            });

            clearTimeout(timeoutId);

            if (response.ok) {
                failureCountRef.current = 0;
                setStatus('connected');
            } else {
                failureCountRef.current += 1;
                if (failureCountRef.current >= FAILURE_THRESHOLD) {
                    setStatus('disconnected');
                }
            }
        } catch {
            failureCountRef.current += 1;
            if (failureCountRef.current >= FAILURE_THRESHOLD) {
                setStatus('disconnected');
            }
        }
        setLastChecked(new Date());
    }, []);

    /** Sets initial connecting state and starts periodic health checks */
    useEffect(() => {
        // Initial check
        // eslint-disable-next-line react-hooks/set-state-in-effect
        setStatus('connecting');
        checkConnection();

        // Set up periodic checking
        const interval = setInterval(checkConnection, intervalMs);

        return () => {
            clearInterval(interval);
        };
    }, [checkConnection, intervalMs]);

    return {
        status,
        lastChecked,
        checkNow: checkConnection,
    };
};

