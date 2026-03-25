/**
 * @module useExecutionStream
 * @description
 * Unified hook for execution log streaming that works in both web (WebSocket)
 * and Tauri (event) modes. Provides a consistent API for starting, stopping,
 * and monitoring executions.
 *
 * @context
 * Used by BarebonesExecutor and other components that need to display execution
 * logs in real-time. Automatically detects the runtime environment and uses
 * the appropriate transport mechanism.
 *
 * @dependencies
 * - React hooks: useState, useEffect, useCallback, useRef
 * - @tauri-apps/api: For Tauri event listening (optional, only in Tauri mode)
 *
 * @example
 * const {
 *   logs,
 *   status,
 *   startRun,
 *   stopRun,
 *   error
 * } = useExecutionStream();
 *
 * // Start execution
 * await startRun('my-spec');
 *
 * // Logs stream automatically
 * console.log(logs);
 */

import { useState, useEffect, useCallback, useRef } from 'react';

// ===== Types =====

/** Log entry from execution */
export interface LogEntry {
    type: 'log' | 'stepstart' | 'stepend' | 'error' | 'success';
    message?: string;
    timestamp: string;
    step_name?: string;
    status?: string;
    metadata?: Record<string, unknown>;
}

/** Batch status for progress tracking */
export interface BatchStatus {
    batchId: string;
    status: 'pending' | 'running' | 'complete' | 'error';
    progress?: number;
}

/** Execution status states */
export type ExecutionStatus = 'idle' | 'running' | 'done' | 'error';

/** Return type for useExecutionStream hook */
export interface UseExecutionStreamReturn {
    /** Accumulated log entries */
    logs: LogEntry[];
    /** Current batch statuses */
    batches: BatchStatus[];
    /** Current execution status */
    status: ExecutionStatus;
    /** Current run ID if executing */
    runId: string | null;
    /** Active spec name if executing */
    activeSpec: string | null;
    /** Start a new execution run */
    startRun: (spec: string) => Promise<string>;
    /** Stop the current execution */
    stopRun: () => Promise<void>;
    /** Last error message if any */
    error: string | null;
    /** Clear all logs */
    clearLogs: () => void;
}

// ===== Helpers =====

/** Check if running in Tauri environment */
const isTauri = (): boolean => {
    return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
};

/** Generate a unique run ID */
const generateRunId = (): string => {
    return `run-${Date.now()}-${Math.random().toString(36).substring(2, 9)}`;
};

// ===== Main Hook =====

/**
 * Unified hook for execution log streaming.
 *
 * Automatically detects whether running in Tauri or web mode and uses
 * the appropriate transport (Tauri events vs WebSocket).
 */
export function useExecutionStream(): UseExecutionStreamReturn {
    // State
    const [logs, setLogs] = useState<LogEntry[]>([]);
    const [batches, setBatches] = useState<BatchStatus[]>([]);
    const [status, setStatus] = useState<ExecutionStatus>('idle');
    const [runId, setRunId] = useState<string | null>(null);
    /** Active spec name for the current execution */
    const [activeSpec, setActiveSpec] = useState<string | null>(null);
    const [error, setError] = useState<string | null>(null);

    // Refs for cleanup
    const wsRef = useRef<WebSocket | null>(null);
    const unlistenersRef = useRef<Array<() => void>>([]);

    // Clear logs
    const clearLogs = useCallback(() => {
        setLogs([]);
        setBatches([]);
        setError(null);
    }, []);

    // Handle incoming log event
    const handleLogEvent = useCallback((event: LogEntry) => {
        setLogs((prev) => [...prev, event]);

        // Update status based on event type
        if (event.type === 'success') {
            setStatus('done');
        } else if (event.type === 'error') {
            setStatus('error');
            setError(event.message || 'Unknown error');
        }
    }, []);

    // Start execution - Web mode (WebSocket)
    const startRunWeb = useCallback(async (spec: string): Promise<string> => {
        setStatus('running');
        setActiveSpec(spec);
        clearLogs();

        // Call the start endpoint -- backend generates the run_id
        const response = await fetch('/api/execution/start', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ spec }),
        });

        if (!response.ok) {
            const errorText = await response.text();
            setError(errorText);
            setStatus('error');
            setActiveSpec(null);
            throw new Error(errorText);
        }

        const result = await response.json();
        const backendRunId = result.run_id as string;
        setRunId(backendRunId);

        // Connect to WebSocket for log streaming using the backend-issued run_id
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const ws = new WebSocket(`${protocol}//${window.location.host}/api/execution/ws?run_id=${backendRunId}`);

        ws.onmessage = (event) => {
            try {
                const data = JSON.parse(event.data) as LogEntry;
                handleLogEvent(data);
            } catch (e) {
                console.error('Failed to parse WebSocket message:', e);
            }
        };

        ws.onerror = (event) => {
            console.error('WebSocket error:', event);
            setError('WebSocket connection error');
        };

        ws.onclose = () => {
            if (status === 'running') {
                setStatus('done');
            }
        };

        wsRef.current = ws;

        return backendRunId;
    }, [clearLogs, handleLogEvent, status]);

    // Start execution - Tauri mode (events)
    const startRunTauri = useCallback(async (spec: string): Promise<string> => {
        const newRunId = generateRunId();
        setRunId(newRunId);
        setActiveSpec(spec);
        setStatus('running');
        clearLogs();

        try {
            // Dynamic import to avoid bundling Tauri API in web builds
            const { invoke } = await import('@tauri-apps/api/core');
            const { listen } = await import('@tauri-apps/api/event');

            // Set up event listeners before starting
            const eventTypes = ['execution:log', 'execution:step_start', 'execution:step_end', 'execution:error', 'execution:success'];

            for (const eventType of eventTypes) {
                const unlisten = await listen<LogEntry>(eventType, (event) => {
                    handleLogEvent(event.payload);
                });
                unlistenersRef.current.push(unlisten);
            }

            // Start the execution
            await invoke('start_execution', { spec, runId: newRunId });

            return newRunId;
        } catch (e) {
            const errorMessage = e instanceof Error ? e.message : String(e);
            setError(errorMessage);
            setStatus('error');
            throw e;
        }
    }, [clearLogs, handleLogEvent]);

    // Unified start function
    const startRun = useCallback(async (spec: string): Promise<string> => {
        if (isTauri()) {
            return startRunTauri(spec);
        } else {
            return startRunWeb(spec);
        }
    }, [startRunTauri, startRunWeb]);

    // Stop execution - Web mode
    const stopRunWeb = useCallback(async (): Promise<void> => {
        if (wsRef.current) {
            wsRef.current.close();
            wsRef.current = null;
        }

        if (activeSpec) {
            await fetch('/api/execution/stop', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ spec: activeSpec, run_id: runId }),
            });
        }

        setStatus('idle');
        setRunId(null);
        setActiveSpec(null);
    }, [activeSpec, runId]);

    // Stop execution - Tauri mode
    const stopRunTauri = useCallback(async (): Promise<void> => {
        // Clean up event listeners
        for (const unlisten of unlistenersRef.current) {
            unlisten();
        }
        unlistenersRef.current = [];

        if (runId) {
            try {
                const { invoke } = await import('@tauri-apps/api/core');
                await invoke('stop_execution', { runId });
            } catch (e) {
                console.error('Failed to stop execution:', e);
            }
        }

        setStatus('idle');
        setRunId(null);
        setActiveSpec(null);
    }, [runId]);

    // Unified stop function
    const stopRun = useCallback(async (): Promise<void> => {
        if (isTauri()) {
            return stopRunTauri();
        } else {
            return stopRunWeb();
        }
    }, [stopRunTauri, stopRunWeb]);

    // Cleanup on unmount
    useEffect(() => {
        return () => {
            // Close WebSocket if open
            if (wsRef.current) {
                wsRef.current.close();
            }

            // Remove Tauri event listeners
            for (const unlisten of unlistenersRef.current) {
                unlisten();
            }
        };
    }, []);

    return {
        logs,
        batches,
        status,
        runId,
        activeSpec,
        startRun,
        stopRun,
        error,
        clearLogs,
    };
}

export default useExecutionStream;
