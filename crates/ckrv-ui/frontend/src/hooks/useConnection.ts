import { useState, useEffect, useCallback } from 'react';

export type ConnectionStatus = 'connected' | 'disconnected' | 'connecting';

interface UseConnectionResult {
    status: ConnectionStatus;
    lastChecked: Date | null;
    checkNow: () => void;
}

/**
 * Hook to monitor backend connectivity.
 * Pings the /api/status endpoint periodically to check if backend is alive.
 */
export const useConnection = (intervalMs: number = 5000): UseConnectionResult => {
    const [status, setStatus] = useState<ConnectionStatus>('disconnected');
    const [lastChecked, setLastChecked] = useState<Date | null>(null);

    const checkConnection = useCallback(async () => {
        try {
            const controller = new AbortController();
            const timeoutId = setTimeout(() => controller.abort(), 3000);

            const response = await fetch('/api/status', {
                method: 'GET',
                signal: controller.signal,
            });

            clearTimeout(timeoutId);

            if (response.ok) {
                setStatus('connected');
            } else {
                setStatus('disconnected');
            }
        } catch {
            setStatus('disconnected');
        }
        setLastChecked(new Date());
    }, []);

    useEffect(() => {
        // Initial check
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

