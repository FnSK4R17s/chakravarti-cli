/**
 * @module useLogStore
 * @description
 * Log store hooks for managing execution log history with React Query integration.
 * Provides fetching, caching, timestamp tracking, and localStorage persistence
 * for WebSocket reconnection support.
 *
 * @context
 * Used in ExecutionRunner and LogViewer for managing log state. Handles reconnection
 * scenarios by tracking last-seen timestamps and fetching missed logs.
 *
 * @dependencies
 * - useQuery, useInfiniteQuery: React Query for data fetching
 * - localStorage: For timestamp persistence across page refreshes
 * - fetchLogs, fetchLogsSince: Log service API functions
 *
 * @example
 * const { lastSeenTimestamp, updateFromLogs, fetchMissedLogs } = useLogStore(executionId);
 */

// === IMPORTS ===
import { useQuery, useInfiniteQuery, useQueryClient } from '@tanstack/react-query';
import { useCallback, useRef, useState, useEffect } from 'react';
import { fetchLogs, fetchTailLogs, fetchLogsSince, deleteLogs } from '../services/logService';
import type { LogEntry, LogHistoryResponse } from '../types/log';

// T039: LocalStorage key prefix for execution timestamps
const STORAGE_KEY_PREFIX = 'ckrv-log-timestamp-';

/**
 * T039: Get the localStorage key for an execution ID
 */
function getStorageKey(executionId: string): string {
    return `${STORAGE_KEY_PREFIX}${executionId}`;
}

/**
 * T039: Persist lastSeenTimestamp to localStorage
 */
function persistTimestamp(executionId: string, timestamp: string): void {
    try {
        localStorage.setItem(getStorageKey(executionId), timestamp);
    } catch (error) {
        console.warn('Failed to persist timestamp to localStorage:', error);
    }
}

/**
 * T040: Load lastSeenTimestamp from localStorage
 */
function loadPersistedTimestamp(executionId: string): string | null {
    try {
        return localStorage.getItem(getStorageKey(executionId));
    } catch (error) {
        console.warn('Failed to load timestamp from localStorage:', error);
        return null;
    }
}

/**
 * T043: Clean up localStorage entry for an execution
 */
function clearPersistedTimestamp(executionId: string): void {
    try {
        localStorage.removeItem(getStorageKey(executionId));
    } catch (error) {
        console.warn('Failed to clear timestamp from localStorage:', error);
    }
}

/**
 * T043: Clean up all stale localStorage entries (older than 7 days)
 */
function cleanupStaleTimestamps(): void {
    try {
        const keysToRemove: string[] = [];
        const sevenDaysAgo = Date.now() - 7 * 24 * 60 * 60 * 1000;

        for (let i = 0; i < localStorage.length; i++) {
            const key = localStorage.key(i);
            if (key && key.startsWith(STORAGE_KEY_PREFIX)) {
                const value = localStorage.getItem(key);
                if (value) {
                    const timestamp = new Date(value).getTime();
                    if (timestamp < sevenDaysAgo || isNaN(timestamp)) {
                        keysToRemove.push(key);
                    }
                }
            }
        }

        keysToRemove.forEach(key => localStorage.removeItem(key));
    } catch (error) {
        console.warn('Failed to cleanup stale timestamps:', error);
    }
}

/**
 * T021: Hook for fetching log history with React Query
 *
 * @param executionId - The execution run ID
 * @param enabled - Whether to enable the query (default: true)
 */
export function useLogHistory(executionId: string | null, enabled: boolean = true) {
    return useQuery({
        queryKey: ['log-history', executionId],
        queryFn: () => fetchLogs(executionId!, {}),
        enabled: enabled && !!executionId,
        staleTime: 30000, // 30 seconds
        refetchOnWindowFocus: false,
    });
}

/**
 * Hook for fetching the tail (most recent) logs
 *
 * @param executionId - The execution run ID
 * @param count - Number of recent logs to fetch
 * @param enabled - Whether to enable the query
 */
export function useLogTail(executionId: string | null, count: number = 10, enabled: boolean = true) {
    return useQuery({
        queryKey: ['log-tail', executionId, count],
        queryFn: () => fetchTailLogs(executionId!, count),
        enabled: enabled && !!executionId,
        staleTime: 5000, // 5 seconds
        refetchOnWindowFocus: false,
    });
}

/**
 * T031: Infinite query hook for paginated log loading
 *
 * @param executionId - The execution run ID
 * @param pageSize - Number of logs per page
 * @param enabled - Whether to enable the query
 */
export function useInfiniteLogHistory(
    executionId: string | null,
    pageSize: number = 100,
    enabled: boolean = true
) {
    return useInfiniteQuery({
        queryKey: ['log-history-infinite', executionId],
        queryFn: ({ pageParam = 0 }) => fetchLogs(executionId!, {
            offset: pageParam as number,
            limit: pageSize,
        }),
        getNextPageParam: (lastPage: LogHistoryResponse) => {
            if (lastPage.has_more) {
                return lastPage.offset + lastPage.logs.length;
            }
            return undefined;
        },
        initialPageParam: 0,
        enabled: enabled && !!executionId,
        staleTime: 30000,
        refetchOnWindowFocus: false,
    });
}

/**
 * T022, T039-T040: Hook for tracking lastSeenTimestamp during execution
 *
 * Maintains the most recent timestamp from received logs,
 * which can be used for WebSocket reconnection.
 * Now also persists to localStorage for page refresh recovery.
 */
export function useLastSeenTimestamp(executionId: string | null) {
    const [lastSeenTimestamp, setLastSeenTimestamp] = useState<string | null>(null);
    const timestampRef = useRef<string | null>(null);

    // T039: Update timestamp and persist to localStorage
    const updateTimestamp = useCallback((timestamp: string) => {
        timestampRef.current = timestamp;
        setLastSeenTimestamp(timestamp);

        // T039: Persist to localStorage
        if (executionId) {
            persistTimestamp(executionId, timestamp);
        }
    }, [executionId]);

    // Update from array of log entries
    const updateFromLogs = useCallback((logs: LogEntry[]) => {
        if (logs.length > 0) {
            const lastLog = logs[logs.length - 1];
            updateTimestamp(lastLog.timestamp);
        }
    }, [updateTimestamp]);

    // Get current timestamp (useful in callbacks to avoid stale closures)
    const getTimestamp = useCallback(() => timestampRef.current, []);

    // T040: Load from localStorage on execution ID change
    useEffect(() => {
        if (executionId) {
            const persisted = loadPersistedTimestamp(executionId);
            if (persisted) {
                timestampRef.current = persisted;
                setLastSeenTimestamp(persisted);
            } else {
                timestampRef.current = null;
                setLastSeenTimestamp(null);
            }
        } else {
            timestampRef.current = null;
            setLastSeenTimestamp(null);
        }
    }, [executionId]);

    // T043: Run cleanup of stale timestamps on mount
    useEffect(() => {
        cleanupStaleTimestamps();
    }, []);

    return {
        lastSeenTimestamp,
        updateTimestamp,
        updateFromLogs,
        getTimestamp,
    };
}

/**
 * Combined hook for log store functionality
 *
 * Provides history fetching, timestamp tracking, cache management,
 * and localStorage persistence.
 */
export function useLogStore(executionId: string | null) {
    const queryClient = useQueryClient();
    const { lastSeenTimestamp, updateTimestamp, updateFromLogs, getTimestamp } =
        useLastSeenTimestamp(executionId);

    // Fetch logs since the last seen timestamp (for reconnection)
    const fetchMissedLogs = useCallback(async () => {
        if (!executionId || !lastSeenTimestamp) {
            return null;
        }

        try {
            const result = await fetchLogsSince(executionId, lastSeenTimestamp);
            if (result.logs.length > 0) {
                updateFromLogs(result.logs);
            }
            return result;
        } catch (error) {
            console.error('Failed to fetch missed logs:', error);
            return null;
        }
    }, [executionId, lastSeenTimestamp, updateFromLogs]);

    // Invalidate log history cache
    const invalidateHistory = useCallback(() => {
        if (executionId) {
            queryClient.invalidateQueries({ queryKey: ['log-history', executionId] });
            queryClient.invalidateQueries({ queryKey: ['log-tail', executionId] });
        }
    }, [executionId, queryClient]);

    // Clear log history cache
    const clearHistory = useCallback(() => {
        if (executionId) {
            queryClient.removeQueries({ queryKey: ['log-history', executionId] });
            queryClient.removeQueries({ queryKey: ['log-tail', executionId] });
        }
    }, [executionId, queryClient]);

    // T043: Delete logs and clean up localStorage
    const deleteExecutionLogs = useCallback(async () => {
        if (!executionId) {
            return null;
        }

        try {
            const result = await deleteLogs(executionId);
            if (result.success) {
                // Clean up localStorage entry
                clearPersistedTimestamp(executionId);
                // Clear caches
                clearHistory();
            }
            return result;
        } catch (error) {
            console.error('Failed to delete execution logs:', error);
            return null;
        }
    }, [executionId, clearHistory]);

    return {
        lastSeenTimestamp,
        updateTimestamp,
        updateFromLogs,
        getTimestamp,
        fetchMissedLogs,
        invalidateHistory,
        clearHistory,
        deleteExecutionLogs,
    };
}

export default useLogStore;
