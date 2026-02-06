/**
 * @module BarebonesExecutor
 * @description
 * Simplified execution runner providing a minimal interface for running specs.
 * Shows batch progress as pills, displays logs in a scrollable area, and provides
 * Run/Stop controls. Uses the unified useExecutionStream hook for transport.
 *
 * @context
 * Used as a simpler alternative to ExecutionRunner for basic execution needs.
 * Auto-selects spec based on current git branch and displays real-time execution
 * progress via WebSocket (web) or Tauri events (desktop).
 *
 * @dependencies
 * - useAutoSelectedSpec: Auto-selects spec based on current git branch
 * - useExecutionStream: Unified hook for execution streaming
 * - useQuery: React Query for fetching plan data
 * - shadcn/ui components: Card, Badge, Button for consistent UI
 *
 * @example
 * // Rendered directly as a page component
 * <BarebonesExecutor />
 *
 * // Provides Run/Stop controls with batch progress pills
 * // Logs are displayed in a terminal-style scrollable area
 */

// === IMPORTS ===
import { useState, useRef, useEffect, useCallback } from 'react';
import { useQuery } from '@tanstack/react-query';
import { useAutoSelectedSpec } from '../hooks/useAutoSelectedSpec';
import { useExecutionStream, type LogEntry } from '../hooks/useExecutionStream';
import {
    Play, Square, Loader2, Check, AlertTriangle,
    Circle, Layers, RefreshCw
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';

// === TYPES ===

/**
 * Simplified batch representation for the executor's UI.
 */
interface SimpleBatch {
    /** Unique batch identifier */
    id: string;
    /** Human-readable batch name */
    name: string;
    /** Current execution status */
    status: 'pending' | 'running' | 'done' | 'error';
}

/**
 * A log entry displayed in the execution log panel.
 */
interface LogLine {
    /** Formatted timestamp (HH:MM:SS) */
    time: string;
    /** Log message content */
    message: string;
    /** Log type for styling: info, error, success, or batch marker */
    type: 'info' | 'error' | 'success' | 'batch';
}

// === API FUNCTIONS ===

const fetchPlan = async (spec: string): Promise<{ success: boolean; batches: { id: string; name: string; status?: string }[] }> => {
    const res = await fetch(`/api/plans/detail?spec=${spec}`);
    return res.json();
};

const generatePlan = async (spec: string): Promise<{ success: boolean }> => {
    const res = await fetch('/api/command/plan-generate', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ spec }),
    });
    return res.json();
};

// === HELPER FUNCTIONS ===

const formatTime = (date: Date): string => {
    return date.toLocaleTimeString('en-US', { hour12: false });
};

/** Convert hook log entry to display log line */
const toLogLine = (entry: LogEntry): LogLine => {
    const time = entry.timestamp
        ? formatTime(new Date(entry.timestamp))
        : formatTime(new Date());

    const type: LogLine['type'] =
        entry.type === 'error' ? 'error' :
            entry.type === 'success' ? 'success' :
                entry.type === 'stepstart' || entry.type === 'stepend' ? 'batch' :
                    'info';

    const message =
        entry.message ||
        (entry.step_name ? `Step: ${entry.step_name}` : 'Unknown event');

    return { time, message, type };
};

// === MAIN COMPONENT ===

export default function BarebonesExecutor() {
    // === HOOKS ===

    // Auto-select spec based on current branch
    const { selectedSpec, isLoading: isLoadingSpec } = useAutoSelectedSpec();

    // Unified execution streaming hook
    const {
        logs: streamLogs,
        status,
        startRun,
        stopRun,
        error,
        clearLogs,
    } = useExecutionStream();

    // === STATE ===

    /** Batch execution progress with status for each batch */
    const [batches, setBatches] = useState<SimpleBatch[]>([]);
    /** Track if plan generation is in progress */
    const [isGeneratingPlan, setIsGeneratingPlan] = useState(false);
    /** Local log lines for display */
    const [localLogs, setLocalLogs] = useState<LogLine[]>([]);

    // Refs
    const logsEndRef = useRef<HTMLDivElement>(null);

    // === QUERIES ===

    // Fetch plan for selected spec
    const { data: planData, refetch: refetchPlan } = useQuery({
        queryKey: ['plan', selectedSpec],
        queryFn: () => fetchPlan(selectedSpec!),
        enabled: !!selectedSpec,
    });

    const hasPlan = planData?.success && planData?.batches?.length > 0;

    // === EFFECTS ===

    // Convert stream logs to display logs
    useEffect(() => {
        if (streamLogs.length > 0) {
            const displayLogs = streamLogs.map(toLogLine);
            setLocalLogs(displayLogs);
        }
    }, [streamLogs]);

    // Auto-scroll logs
    useEffect(() => {
        logsEndRef.current?.scrollIntoView({ behavior: 'smooth' });
    }, [localLogs]);

    // Initialize batches from plan
    useEffect(() => {
        if (planData?.success && planData.batches) {
            setBatches(planData.batches.map(b => {
                let uiStatus: SimpleBatch['status'] = 'pending';
                if (b.status === 'completed') uiStatus = 'done';
                else if (b.status === 'running') uiStatus = 'running';
                else if (b.status === 'failed') uiStatus = 'error';

                return { id: b.id, name: b.name, status: uiStatus };
            }));
        }
    }, [planData]);

    // Update batch status from stream logs
    useEffect(() => {
        for (const log of streamLogs) {
            if (log.type === 'stepstart' && log.step_name) {
                setBatches(prev => prev.map(b =>
                    b.name === log.step_name || b.id === log.step_name
                        ? { ...b, status: 'running' }
                        : b
                ));
            } else if (log.type === 'stepend' && log.step_name) {
                const isSuccess = log.status !== 'error' && log.status !== 'failed';
                setBatches(prev => prev.map(b =>
                    b.name === log.step_name || b.id === log.step_name
                        ? { ...b, status: isSuccess ? 'done' : 'error' }
                        : b
                ));
            }
        }
    }, [streamLogs]);

    // === HANDLERS ===

    // Add local log entry
    const addLog = useCallback((message: string, type: LogLine['type'] = 'info') => {
        setLocalLogs(prev => [...prev, { time: formatTime(new Date()), message, type }]);
    }, []);

    // Handle Run
    const handleRun = async () => {
        if (!selectedSpec) return;

        // Reset state
        setLocalLogs([]);
        clearLogs();
        setBatches(prev => prev.map(b => ({ ...b, status: 'pending' })));

        addLog(`Starting execution for ${selectedSpec}...`, 'info');

        try {
            await startRun(selectedSpec);
        } catch (e) {
            addLog(`Error: ${e}`, 'error');
        }
    };

    // Handle Stop
    const handleStop = async () => {
        await stopRun();
        addLog('Execution stopped', 'info');
    };

    // Handle Generate Plan
    const handleGeneratePlan = async () => {
        if (!selectedSpec) return;

        setIsGeneratingPlan(true);
        addLog('Generating execution plan...', 'info');

        try {
            const res = await generatePlan(selectedSpec);
            if (res.success) {
                addLog('Plan generated successfully!', 'success');
                refetchPlan();
            } else {
                addLog('Failed to generate plan', 'error');
            }
        } catch (e) {
            addLog(`Error: ${e}`, 'error');
        } finally {
            setIsGeneratingPlan(false);
        }
    };

    // Calculate progress
    const completedCount = batches.filter(b => b.status === 'done').length;
    const totalCount = batches.length;
    const isRunning = status === 'running';

    // Loading state
    if (isLoadingSpec) {
        return (
            <div className="h-full flex items-center justify-center">
                <Loader2 className="animate-spin text-muted-foreground" size={32} />
            </div>
        );
    }

    // No spec selected
    if (!selectedSpec) {
        return (
            <div className="h-full flex items-center justify-center text-muted-foreground">
                <div className="text-center">
                    <Layers size={48} className="mx-auto mb-4 opacity-50" />
                    <p>No spec selected</p>
                    <p className="text-sm mt-2">Switch to a spec branch or create a new spec</p>
                </div>
            </div>
        );
    }

    return (
        <div className="h-full flex flex-col p-4 gap-4">
            {/* Header */}
            <Card>
                <CardHeader className="py-3">
                    <div className="flex items-center justify-between">
                        <div className="flex items-center gap-3">
                            <div className={`w-3 h-3 rounded-full ${status === 'running' ? 'bg-warning animate-pulse' :
                                status === 'done' ? 'bg-success' :
                                    status === 'error' ? 'bg-error' :
                                        'bg-muted'
                                }`} />
                            <CardTitle className="text-lg font-mono">{selectedSpec}</CardTitle>
                            <Badge variant="outline" className="text-xs">
                                {status === 'idle' ? 'Ready' :
                                    status === 'running' ? 'Running' :
                                        status === 'done' ? 'Complete' : 'Error'}
                            </Badge>
                        </div>

                        <div className="flex items-center gap-2">
                            {!hasPlan && (
                                <Button
                                    variant="outline"
                                    size="sm"
                                    onClick={handleGeneratePlan}
                                    disabled={isGeneratingPlan || isRunning}
                                >
                                    {isGeneratingPlan ? (
                                        <Loader2 size={16} className="animate-spin mr-2" />
                                    ) : (
                                        <Layers size={16} className="mr-2" />
                                    )}
                                    Generate Plan
                                </Button>
                            )}

                            {!isRunning ? (
                                <Button
                                    onClick={handleRun}
                                    disabled={!hasPlan}
                                    className="bg-success hover:bg-success"
                                >
                                    <Play size={16} className="mr-2" />
                                    Run
                                </Button>
                            ) : (
                                <Button
                                    variant="destructive"
                                    onClick={handleStop}
                                >
                                    <Square size={16} className="mr-2" />
                                    Stop
                                </Button>
                            )}
                        </div>
                    </div>
                </CardHeader>
            </Card>

            {/* Batch Progress */}
            {batches.length > 0 && (
                <Card>
                    <CardContent className="py-3">
                        <div className="flex items-center justify-between mb-3">
                            <span className="text-sm text-muted-foreground">
                                Batches: {completedCount}/{totalCount} complete
                            </span>
                            {isRunning && (
                                <RefreshCw size={14} className="animate-spin text-muted-foreground" />
                            )}
                        </div>
                        <div className="flex gap-2 flex-wrap">
                            {batches.map(batch => (
                                <Badge
                                    key={batch.id}
                                    variant={
                                        batch.status === 'done' ? 'default' :
                                            batch.status === 'running' ? 'secondary' :
                                                batch.status === 'error' ? 'destructive' :
                                                    'outline'
                                    }
                                    className={`gap-1 ${batch.status === 'done' ? 'bg-success' :
                                        batch.status === 'running' ? 'bg-warning' : ''
                                        }`}
                                >
                                    {batch.status === 'done' && <Check size={12} />}
                                    {batch.status === 'running' && <Loader2 size={12} className="animate-spin" />}
                                    {batch.status === 'error' && <AlertTriangle size={12} />}
                                    {batch.status === 'pending' && <Circle size={12} />}
                                    {batch.name}
                                </Badge>
                            ))}
                        </div>
                    </CardContent>
                </Card>
            )}

            {/* Logs */}
            <Card className="flex-1 flex flex-col min-h-0">
                <CardHeader className="py-2 border-b border-border">
                    <CardTitle className="text-sm font-medium">Logs</CardTitle>
                </CardHeader>
                <CardContent className="flex-1 p-0 overflow-hidden">
                    <div className="h-full overflow-y-auto bg-black/50 p-4 font-mono text-sm">
                        {localLogs.length === 0 ? (
                            <div className="text-muted-foreground">
                                {hasPlan ? 'Click "Run" to start execution...' : 'Generate a plan first...'}
                            </div>
                        ) : (
                            localLogs.map((log, i) => (
                                <div
                                    key={i}
                                    className={`leading-relaxed ${log.type === 'error' ? 'text-error' :
                                        log.type === 'success' ? 'text-success' :
                                            log.type === 'batch' ? 'text-warning' :
                                                'text-muted-foreground'
                                        }`}
                                >
                                    <span className="text-muted-foreground">[{log.time}]</span>{' '}
                                    {log.message}
                                </div>
                            ))
                        )}
                        <div ref={logsEndRef} />
                    </div>
                </CardContent>
            </Card>

            {/* Error display */}
            {error && (
                <Card className="border-error/50 bg-error/10">
                    <CardContent className="py-3 flex items-center gap-2 text-error">
                        <AlertTriangle size={16} />
                        <span className="text-sm">{error}</span>
                    </CardContent>
                </Card>
            )}
        </div>
    );
}
