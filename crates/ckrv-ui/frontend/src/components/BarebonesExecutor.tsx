/**
 * @module BarebonesExecutor
 * @description
 * Simplified execution runner providing a minimal interface for running specs.
 * Shows batch progress as pills, displays logs in a scrollable area, and provides
 * Run/Stop controls. A lighter alternative to the full ExecutionRunner.
 *
 * @context
 * Used as a simpler alternative to ExecutionRunner for basic execution needs.
 * Auto-selects spec based on current git branch and displays real-time execution
 * progress via WebSocket connection.
 *
 * @dependencies
 * - useAutoSelectedSpec: Auto-selects spec based on current git branch
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

/** Current execution status for the barebones executor. */
type ExecutionStatus = 'idle' | 'running' | 'done' | 'error';

// === API FUNCTIONS ===

const fetchPlan = async (spec: string): Promise<{ success: boolean; batches: { id: string; name: string; status?: string }[] }> => {
    const res = await fetch(`/api/plans/detail?spec=${spec}`);
    return res.json();
};

const startExecution = async (spec: string, runId: string): Promise<{ success: boolean; message?: string }> => {
    const res = await fetch('/api/execution/start', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ spec, run_id: runId }),
    });
    return res.json();
};

const stopExecution = async (runId: string): Promise<{ success: boolean }> => {
    const res = await fetch('/api/execution/stop', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ run_id: runId }),
    });
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

// Generate unique run ID
const generateRunId = (): string => {
    return `run-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
};

// === MAIN COMPONENT ===

export default function BarebonesExecutor() {
    // === STATE ===

    // --- Spec Selection ---
    // Auto-select spec based on current branch
    const { selectedSpec, isLoading: isLoadingSpec } = useAutoSelectedSpec();

    // --- Execution State ---
    /** Current execution status: idle, running, done, or error */
    const [status, setStatus] = useState<ExecutionStatus>('idle');
    /** Batch execution progress with status for each batch */
    const [batches, setBatches] = useState<SimpleBatch[]>([]);
    /** Log entries from the execution */
    const [logs, setLogs] = useState<LogLine[]>([]);
    /** Error message if execution failed */
    const [error, setError] = useState<string | null>(null);
    /** Track if plan generation is in progress */
    const [isGeneratingPlan, setIsGeneratingPlan] = useState(false);

    // Refs
    const wsRef = useRef<WebSocket | null>(null);
    const runIdRef = useRef<string>('');
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

    // Auto-scroll logs
    useEffect(() => {
        logsEndRef.current?.scrollIntoView({ behavior: 'smooth' });
    }, [logs]);

    // === HANDLERS ===

    // Add log entry
    const addLog = useCallback((message: string, type: LogLine['type'] = 'info') => {
        setLogs(prev => [...prev, { time: formatTime(new Date()), message, type }]);
    }, []);

    // === EFFECTS (continued) ===

    // Initialize batches from plan - read actual status from YAML
    useEffect(() => {
        if (planData?.success && planData.batches) {
            setBatches(planData.batches.map(b => {
                // Map YAML status to UI status
                let uiStatus: SimpleBatch['status'] = 'pending';
                if (b.status === 'completed') uiStatus = 'done';
                else if (b.status === 'running') uiStatus = 'running';
                else if (b.status === 'failed') uiStatus = 'error';

                return {
                    id: b.id,
                    name: b.name,
                    status: uiStatus
                };
            }));
        }
    }, [planData]);

    // WebSocket message handler
    const handleWsMessage = useCallback((event: MessageEvent) => {
        try {
            const msg = JSON.parse(event.data);
            const msgType = msg.type || '';
            const message = msg.message || '';

            switch (msgType) {
                case 'info':
                case 'log':
                case 'start':
                    // Regular log messages
                    if (message) addLog(message, 'info');
                    break;

                case 'success':
                    if (message) addLog(message, 'success');
                    break;

                case 'error':
                    if (message) addLog(message, 'error');
                    if (msg.status === 'failed' || msg.status === 'aborted') {
                        setStatus('error');
                        setError(message || 'Execution failed');
                    }
                    break;

                case 'batch_start':
                    addLog(`▶ Starting batch: ${msg.batch_name || msg.batch_id}`, 'batch');
                    setBatches(prev => prev.map(b =>
                        b.id === msg.batch_id ? { ...b, status: 'running' } : b
                    ));
                    break;

                case 'batch_complete':
                    addLog(`✓ Batch complete: ${msg.batch_name || msg.batch_id}`, 'success');
                    setBatches(prev => prev.map(b =>
                        b.id === msg.batch_id ? { ...b, status: 'done' } : b
                    ));
                    break;

                case 'batch_error':
                    addLog(`✗ Batch failed: ${msg.batch_name || msg.batch_id} - ${msg.error || message}`, 'error');
                    setBatches(prev => prev.map(b =>
                        b.id === msg.batch_id ? { ...b, status: 'error' } : b
                    ));
                    break;

                case 'batch_status':
                    // Update batch status based on msg.status
                    if (msg.batch_id && msg.status) {
                        const batchStatus = msg.status === 'completed' ? 'done' :
                            msg.status === 'failed' ? 'error' :
                                msg.status === 'running' ? 'running' : 'pending';
                        setBatches(prev => prev.map(b =>
                            b.id === msg.batch_id ? { ...b, status: batchStatus } : b
                        ));
                    }
                    break;

                case 'status':
                    // Handle execution status messages (running/completed/failed)
                    if (msg.status === 'completed') {
                        addLog('✓ Execution complete!', 'success');
                        setStatus('done');
                    } else if (msg.status === 'failed') {
                        addLog('Execution failed', 'error');
                        setStatus('error');
                    }
                    break;

                case 'complete':
                case 'execution_complete':
                    addLog('✓ Execution complete!', 'success');
                    setStatus('done');
                    break;

                case 'history_complete':
                    // History backfill complete, ignore
                    break;

                default:
                    // Unknown type but has message, show it
                    if (message) addLog(message, 'info');
            }
        } catch (e) {
            // Raw text message
            if (typeof event.data === 'string' && event.data.trim()) {
                addLog(event.data, 'info');
            }
        }
    }, [addLog]);

    // Connect WebSocket
    const connectWebSocket = useCallback((runId: string) => {
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const wsUrl = `${protocol}//${window.location.host}/api/execution/ws?run_id=${encodeURIComponent(runId)}`;

        const ws = new WebSocket(wsUrl);

        ws.onopen = () => {
            addLog('Connected to execution stream', 'info');
        };

        ws.onmessage = handleWsMessage;

        ws.onerror = () => {
            addLog('WebSocket error', 'error');
        };

        ws.onclose = () => {
            if (status === 'running') {
                addLog('Connection closed', 'info');
            }
        };

        wsRef.current = ws;
    }, [addLog, handleWsMessage, status]);

    // Handle Run
    const handleRun = async () => {
        if (!selectedSpec) return;

        // Reset state
        setLogs([]);
        setError(null);
        setBatches(prev => prev.map(b => ({ ...b, status: 'pending' })));
        setStatus('running');

        const runId = generateRunId();
        runIdRef.current = runId;

        addLog(`Starting execution for ${selectedSpec}...`, 'info');

        try {
            const res = await startExecution(selectedSpec, runId);
            if (res.success) {
                connectWebSocket(runId);
            } else {
                addLog(`Failed to start: ${res.message || 'Unknown error'}`, 'error');
                setStatus('error');
                setError(res.message || 'Failed to start execution');
            }
        } catch (e) {
            addLog(`Error: ${e}`, 'error');
            setStatus('error');
            setError(String(e));
        }
    };

    // Handle Stop
    const handleStop = async () => {
        if (wsRef.current) {
            wsRef.current.close();
            wsRef.current = null;
        }

        if (runIdRef.current) {
            await stopExecution(runIdRef.current);
        }

        addLog('Execution stopped', 'info');
        setStatus('idle');
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

    // Cleanup on unmount
    useEffect(() => {
        return () => {
            if (wsRef.current) {
                wsRef.current.close();
            }
        };
    }, []);

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
                        {logs.length === 0 ? (
                            <div className="text-muted-foreground">
                                {hasPlan ? 'Click "Run" to start execution...' : 'Generate a plan first...'}
                            </div>
                        ) : (
                            logs.map((log, i) => (
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
