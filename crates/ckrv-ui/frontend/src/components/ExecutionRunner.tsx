import React, { useState, useEffect, useRef, useCallback, useMemo, useTransition } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import {
    Square, RotateCcw, CheckCircle2, Circle, Clock,
    AlertTriangle, Loader2, Terminal as TerminalIcon, ChevronRight, Maximize2, Minimize2,
    ArrowRight, Zap, Brain, Cpu,
    Layers, Timer, DollarSign, Rocket, GitMerge, ArrowDown
} from 'lucide-react';
import { LogTerminal } from './LogTerminal';
import { CompletionSummary } from './CompletionSummary';
import type { Terminal } from '@xterm/xterm';
import { useTimeout } from '../hooks/useTimeout';
import type { RunStatus } from '../types/history';

// Types
interface Batch {
    id: string;
    name: string;
    task_ids: string[];
    depends_on: string[];
    model_assignment: {
        default: string;
        overrides: Record<string, string>;
    };
    execution_strategy: string;
    estimated_cost: number;
    estimated_time: string;
    reasoning: string;
    status?: BatchStatus;
}

interface Spec {
    name: string;
    path: string;
    has_tasks: boolean;
    has_plan: boolean;
    task_count: number;
}

type BatchStatus = 'pending' | 'waiting' | 'running' | 'completed' | 'failed';
// T022: Added 'reconnecting' status for WebSocket reconnection handling (BUG-002)
type ExecutionStatus = 'idle' | 'starting' | 'running' | 'reconnecting' | 'completed' | 'failed' | 'aborted';

interface LogEntry {
    time: string;
    message: string;
    type: 'info' | 'success' | 'error' | 'start' | 'batch_start' | 'batch_complete' | 'batch_error';
    stream?: 'stdout' | 'stderr';
    batchId?: string;
}

interface WsMessage {
    type: string;
    message?: string;
    status?: string;
    stream?: string;
    timestamp?: string;
    // T013: Added fields for batch_status messages
    batch_id?: string;
    batch_name?: string;
    branch?: string;
    error?: string;
}

// API functions
const fetchSpecs = async (): Promise<{ specs: Spec[] }> => {
    const res = await fetch('/api/specs');
    return res.json();
};

const fetchPlan = async (spec: string): Promise<{ success: boolean; batches: Batch[] }> => {
    const res = await fetch(`/api/plans/detail?spec=${spec}`);
    return res.json();
};

const startExecution = async (spec: string, runId: string, dryRun = false): Promise<{ success: boolean; message?: string }> => {
    const res = await fetch('/api/execution/start', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ spec, run_id: runId, dry_run: dryRun }),
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

// Branch info for merge panel
interface UnmergedBranch {
    name: string;
    batch_name: string;
    ahead_commits: number;
    is_clean: boolean;
}

const fetchUnmergedBranches = async (spec: string): Promise<{
    success: boolean;
    current_branch: string;
    branches: UnmergedBranch[];
}> => {
    const res = await fetch('/api/execution/branches', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ spec }),
    });
    return res.json();
};

const mergeAllBranches = async (spec: string): Promise<{
    success: boolean;
    merged: string[];
    failed: string[];
    message: string;
}> => {
    const res = await fetch('/api/execution/merge-all', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ spec }),
    });
    return res.json();
};

// Model configuration helper
const getModelConfig = (modelId: string) => {
    if (modelId.includes('claude') || modelId.includes('gpt-4')) {
        return { name: 'Claude', short: 'CL', tier: 'heavy', color: 'amber', icon: Brain };
    } else if (modelId.includes('minimax') || modelId.includes('haiku') || modelId.includes('flash')) {
        return { name: modelId.split('/').pop() || 'Light', short: 'MM', tier: 'light', color: 'sky', icon: Zap };
    } else {
        return { name: modelId.split('/').pop() || modelId, short: 'STD', tier: 'standard', color: 'violet', icon: Cpu };
    }
};

// Status configuration
const statusConfig: Record<BatchStatus, { color: string; textColor: string; borderColor: string; icon: React.ElementType; spin?: boolean }> = {
    pending: { color: 'bg-slate-800/50', textColor: 'text-slate-400', borderColor: 'border-slate-600', icon: Circle },
    waiting: { color: 'bg-blue-900/30', textColor: 'text-blue-400', borderColor: 'border-blue-500', icon: Clock },
    running: { color: 'bg-amber-900/30', textColor: 'text-amber-400', borderColor: 'border-amber-500', icon: Loader2, spin: true },
    completed: { color: 'bg-emerald-900/30', textColor: 'text-emerald-400', borderColor: 'border-emerald-500', icon: CheckCircle2 },
    failed: { color: 'bg-red-900/30', textColor: 'text-red-400', borderColor: 'border-red-500', icon: AlertTriangle },
};

// Progress Ring Component (smaller version)
const ProgressRing: React.FC<{ progress: number; size?: number; strokeWidth?: number }> = ({
    progress, size = 48, strokeWidth = 4
}) => {
    const radius = (size - strokeWidth) / 2;
    const circumference = radius * 2 * Math.PI;
    const offset = circumference - (progress / 100) * circumference;

    return (
        <div className="relative" style={{ width: size, height: size }}>
            <svg className="transform -rotate-90" width={size} height={size}>
                <circle
                    className="text-gray-700"
                    strokeWidth={strokeWidth}
                    stroke="currentColor"
                    fill="transparent"
                    r={radius}
                    cx={size / 2}
                    cy={size / 2}
                />
                <circle
                    className="text-emerald-500 transition-all duration-500"
                    strokeWidth={strokeWidth}
                    strokeDasharray={circumference}
                    strokeDashoffset={offset}
                    strokeLinecap="round"
                    stroke="currentColor"
                    fill="transparent"
                    r={radius}
                    cx={size / 2}
                    cy={size / 2}
                />
            </svg>
            <div className="absolute inset-0 flex items-center justify-center">
                <span className="text-sm font-bold text-gray-200">{Math.round(progress)}%</span>
            </div>
        </div>
    );
};

// Individual Batch Log Panel Component
const BatchLogPanel: React.FC<{
    batch: Batch;
    logs: LogEntry[];
    isExpanded: boolean;
    onToggleExpand: () => void;
}> = ({ batch, logs, isExpanded, onToggleExpand }) => {
    const logsContainerRef = useRef<HTMLDivElement>(null);
    const logsEndRef = useRef<HTMLDivElement>(null);
    // T027: Track if user has manually scrolled away from bottom (BUG-007)
    const [isUserScrolled, setIsUserScrolled] = useState(false);
    const isUserScrolledRef = useRef(false);

    const status = batch.status || 'pending';
    const statusCfg = statusConfig[status];
    const StatusIcon = statusCfg.icon;
    const modelConfig = getModelConfig(batch.model_assignment.default);
    const ModelIcon = modelConfig.icon;

    // T027: Detect user scroll to pause auto-scroll (BUG-007)
    const handleScroll = useCallback(() => {
        const container = logsContainerRef.current;
        if (!container) return;

        // Check if scrolled to bottom (within 50px threshold)
        const isAtBottom = container.scrollHeight - container.scrollTop - container.clientHeight < 50;

        if (isAtBottom) {
            // User scrolled back to bottom, resume auto-scroll
            isUserScrolledRef.current = false;
            setIsUserScrolled(false);
        } else {
            // User scrolled away from bottom
            isUserScrolledRef.current = true;
            setIsUserScrolled(true);
        }
    }, []);

    // T027: Only auto-scroll if user hasn't manually scrolled (BUG-007)
    useEffect(() => {
        if (!isUserScrolledRef.current && logsEndRef.current) {
            logsEndRef.current.scrollIntoView({ behavior: 'smooth' });
        }
    }, [logs]);

    // T028: Scroll to bottom when button is clicked (BUG-007)
    const handleScrollToBottom = useCallback(() => {
        if (logsEndRef.current) {
            logsEndRef.current.scrollIntoView({ behavior: 'smooth' });
        }
        isUserScrolledRef.current = false;
        setIsUserScrolled(false);
    }, []);

    const colorMap: Record<string, string> = {
        sky: 'bg-sky-900/30 text-sky-300',
        violet: 'bg-violet-900/30 text-violet-300',
        amber: 'bg-amber-900/30 text-amber-300',
    };

    return (
        <div className={`
            flex flex-col rounded-lg border overflow-hidden transition-all duration-300
            ${statusCfg.color} ${statusCfg.borderColor}
            ${isExpanded ? 'col-span-2 row-span-2' : ''}
        `}>
            {/* Header */}
            <div className="flex items-center justify-between px-3 py-2 bg-gray-900/50 border-b border-gray-700/50">
                <div className="flex items-center gap-2 min-w-0">
                    <div className={`p-1 rounded ${statusCfg.textColor}`}>
                        <StatusIcon size={14} className={statusCfg.spin ? 'animate-spin' : ''} />
                    </div>
                    <span className="font-medium text-sm text-gray-200 truncate">{batch.name}</span>
                    <div className={`flex items-center gap-1 px-1.5 py-0.5 rounded text-xs font-medium ${colorMap[modelConfig.color] || 'bg-gray-700 text-gray-300'}`}>
                        <ModelIcon size={10} />
                        <span>{modelConfig.short}</span>
                    </div>
                </div>
                <div className="flex items-center gap-1">
                    <span className="text-xs text-gray-500">{logs.length} lines</span>
                    <button
                        onClick={onToggleExpand}
                        className="p-1 hover:bg-gray-700 rounded transition-colors"
                        aria-label={isExpanded ? 'Minimize panel' : 'Maximize panel'}
                    >
                        {isExpanded ? <Minimize2 size={12} /> : <Maximize2 size={12} />}
                    </button>
                </div>
            </div>

            {/* Log content with scroll detection */}
            <div
                ref={logsContainerRef}
                onScroll={handleScroll}
                className="flex-1 overflow-y-auto bg-gray-900/30 p-2 font-mono text-xs min-h-0 relative"
                data-testid="batch-log-container"
            >
                {logs.length === 0 ? (
                    <div className="text-gray-500 text-center py-4 flex items-center justify-center gap-2">
                        {status === 'pending' && <Circle size={12} />}
                        {status === 'waiting' && <Clock size={12} className="animate-pulse" />}
                        {status === 'running' && <Loader2 size={12} className="animate-spin" />}
                        <span>
                            {status === 'pending' && 'Waiting to start...'}
                            {status === 'waiting' && 'Dependencies in progress...'}
                            {status === 'running' && 'Agent working...'}
                            {status === 'completed' && 'Completed'}
                            {status === 'failed' && 'Failed'}
                        </span>
                    </div>
                ) : (
                    logs.map((log, i) => {
                        const textColor = {
                            success: 'text-emerald-400',
                            batch_complete: 'text-emerald-400',
                            error: 'text-red-400',
                            batch_error: 'text-red-400',
                            start: 'text-amber-400',
                            batch_start: 'text-cyan-400',
                            info: 'text-gray-400',
                        }[log.type] || 'text-gray-400';

                        return (
                            <div key={i} className={`py-0.5 ${textColor} break-all`}>
                                <span className="text-gray-600 mr-2">{log.time}</span>
                                {log.message}
                            </div>
                        );
                    })
                )}
                <div ref={logsEndRef} />
            </div>

            {/* T028: Scroll to bottom button - shown when user has scrolled up (BUG-007) */}
            {isUserScrolled && logs.length > 0 && (
                <button
                    onClick={handleScrollToBottom}
                    className="absolute bottom-12 right-4 p-2 rounded-full bg-gray-800/90 hover:bg-gray-700 border border-gray-600 shadow-lg transition-all duration-200 flex items-center gap-1 text-xs text-gray-300"
                    style={{ backdropFilter: 'blur(4px)' }}
                    aria-label="Scroll to bottom"
                    data-testid="scroll-to-bottom-button"
                >
                    <ArrowDown size={14} />
                    <span>New logs</span>
                </button>
            )}
        </div>
    );
};

// Spec List View
const SpecListView: React.FC<{ specs: Spec[]; onSelect: (name: string) => void; isLoading: boolean }> = ({
    specs, onSelect, isLoading
}) => {
    const specsWithPlan = specs.filter(s => s.has_plan);

    if (isLoading) {
        return (
            <div className="flex items-center justify-center h-64">
                <Loader2 className="animate-spin text-gray-500" size={24} />
            </div>
        );
    }

    if (specsWithPlan.length === 0) {
        return (
            <div className="text-center py-12 text-gray-500">
                <Rocket size={48} className="mx-auto mb-4 opacity-50" />
                <p>No specs with execution plans found</p>
                <p className="text-sm mt-2">Run <code className="bg-gray-800 px-2 py-0.5 rounded">ckrv run --dry-run</code> to generate a plan</p>
            </div>
        );
    }

    return (
        <div className="space-y-2">
            {specsWithPlan.map((spec) => (
                <button
                    key={spec.name}
                    onClick={() => onSelect(spec.name)}
                    className="w-full text-left p-4 bg-gray-800/50 hover:bg-gray-800 rounded-lg border border-gray-700 transition-colors"
                >
                    <div className="flex items-center justify-between">
                        <div className="flex items-center gap-3">
                            <Rocket size={20} className="text-cyan-400" />
                            <div>
                                <h3 className="font-medium text-gray-200">{spec.name}</h3>
                                <div className="flex items-center gap-2 mt-1">
                                    <span className="text-xs bg-green-900/50 text-green-300 px-2 py-0.5 rounded">ready to run</span>
                                    <span className="text-xs text-gray-500">{spec.task_count} tasks</span>
                                </div>
                            </div>
                        </div>
                        <ChevronRight size={20} className="text-gray-500" />
                    </div>
                </button>
            ))}
        </div>
    );
};

// Format elapsed time
const formatElapsedTime = (seconds: number): string => {
    if (seconds < 60) return `${seconds}s`;
    const h = Math.floor(seconds / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    const s = seconds % 60;

    if (h > 0) {
        return `${h.toString().padStart(2, '0')}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
    }
    return `${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
};

// Calculate grid layout based on number of active batches
function getGridLayout(count: number): { cols: number; rows: number } {
    if (count === 0) return { cols: 0, rows: 0 };
    if (count === 1) return { cols: 1, rows: 1 };
    if (count === 2) return { cols: 2, rows: 1 };
    if (count <= 4) return { cols: 2, rows: 2 };
    if (count <= 6) return { cols: 3, rows: 2 };
    if (count <= 9) return { cols: 3, rows: 3 };
    return { cols: 4, rows: 3 }; // Max 12
}

// Main Execution Runner
export default function ExecutionRunner() {
    const queryClient = useQueryClient();
    const [selectedSpecName, setSelectedSpecName] = useState<string | null>(null);
    const [batches, setBatches] = useState<Batch[]>([]);
    const [executionStatus, setExecutionStatus] = useState<ExecutionStatus>('idle');
    const [batchLogs, setBatchLogs] = useState<Record<string, LogEntry[]>>({});
    const [completedBatches, setCompletedBatches] = useState<Set<string>>(new Set());
    const [elapsedTime, setElapsedTime] = useState(0);
    const [expandedBatchId, setExpandedBatchId] = useState<string | null>(null);
    const [orchestratorMinimized, setOrchestratorMinimized] = useState(false);
    const [unmergedBranches, setUnmergedBranches] = useState<UnmergedBranch[]>([]);
    const [isMerging, setIsMerging] = useState(false);
    const [mergeResult, setMergeResult] = useState<{ success: boolean; message: string } | null>(null);
    // Track when batches completed (for auto-collapse after 5s)
    const [batchCompletedAt, setBatchCompletedAt] = useState<Record<string, number>>({});

    // T030: Completion summary state
    const [showCompletionSummary, setShowCompletionSummary] = useState(false);
    const [completionError, setCompletionError] = useState<string | null>(null);

    // T024: WebSocket reconnection state (BUG-002)
    const [wsRetryCount, setWsRetryCount] = useState(0);
    const [wsRetryCountdown, setWsRetryCountdown] = useState(0);

    // T021: Use transition for non-urgent state updates (BUG-001)
    const [isPending, startTransition] = useTransition();

    // T026: Use timeout hook for automatic cleanup (BUG-005)
    const { set: setTimeout, clearAll: clearAllTimeouts } = useTimeout();

    const wsRef = useRef<WebSocket | null>(null);
    const runIdRef = useRef<string>('');
    const timerRef = useRef<number | null>(null);
    const startTimeRef = useRef<number>(0);
    const terminalRef = useRef<Terminal | null>(null);

    // T020: Message batching refs for requestAnimationFrame (BUG-001)
    const pendingMessagesRef = useRef<WsMessage[]>([]);
    const rafIdRef = useRef<number | null>(null);

    // Fetch specs
    const { data: specsData, isLoading: isLoadingSpecs } = useQuery({
        queryKey: ['specs'],
        queryFn: fetchSpecs,
    });

    // Fetch plan when spec is selected
    const { data: planData } = useQuery({
        queryKey: ['plan', selectedSpecName],
        queryFn: () => fetchPlan(selectedSpecName!),
        enabled: !!selectedSpecName,
    });

    // Update batches when plan data changes
    useEffect(() => {
        if (planData?.batches) {
            setBatches(planData.batches.map(b => {
                let status: BatchStatus = 'pending';
                if (b.status === 'completed') status = 'completed';
                else if (b.status === 'running') status = 'running';
                else if (b.status === 'failed') status = 'failed';
                return { ...b, status };
            }));
            const logMap: Record<string, LogEntry[]> = {};
            planData.batches.forEach(b => { logMap[b.id] = []; });
            setBatchLogs(logMap);
            const completed = new Set<string>();
            planData.batches.forEach(b => {
                if (b.status === 'completed') completed.add(b.name);
            });
            setCompletedBatches(completed);
        }
    }, [planData]);

    // T020/T026: Cleanup effect for RAF, WebSocket, and timeouts (BUG-001, BUG-005)
    useEffect(() => {
        return () => {
            // Cancel any pending requestAnimationFrame
            if (rafIdRef.current) {
                cancelAnimationFrame(rafIdRef.current);
                rafIdRef.current = null;
            }
            // Close WebSocket connection
            if (wsRef.current) {
                wsRef.current.close();
                wsRef.current = null;
            }
            // Clear all managed timeouts (handled by useTimeout hook)
            clearAllTimeouts();
        };
    }, [clearAllTimeouts]);

    useEffect(() => {
        if (executionStatus === 'running' || executionStatus === 'reconnecting') {
            startTimeRef.current = Date.now();
            timerRef.current = window.setInterval(() => {
                setElapsedTime(Math.floor((Date.now() - startTimeRef.current) / 1000));
            }, 1000);
        } else {
            if (timerRef.current) {
                clearInterval(timerRef.current);
                timerRef.current = null;
            }
        }
        return () => {
            if (timerRef.current) clearInterval(timerRef.current);
        };
    }, [executionStatus]);

    const addLog = useCallback((message: string, type: LogEntry['type'] = 'info', batchId?: string) => {
        const time = new Date().toLocaleTimeString('en-US', {
            hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit'
        });
        const entry: LogEntry = { time, message, type, batchId };

        // Write to terminal
        if (terminalRef.current) {
            let color = '\x1b[37m'; // White default
            if (type === 'success' || type === 'batch_complete') color = '\x1b[32m'; // Green
            else if (type === 'error' || type === 'batch_error') color = '\x1b[31m'; // Red
            else if (type === 'start') color = '\x1b[33m'; // Yellow
            else if (type === 'batch_start') color = '\x1b[36m'; // Cyan

            // Clean up message newlines for consistent display
            const cleanMessage = message.replace(/\n/g, '\r\n');
            terminalRef.current.writeln(`\x1b[90m${time}\x1b[0m ${color}${cleanMessage}\x1b[0m`);
        }

        if (batchId) {
            setBatchLogs(prev => ({
                ...prev,
                [batchId]: [...(prev[batchId] || []), entry]
            }));
        }
    }, []);

    const currentBatchRef = useRef<string | null>(null);

    const updateBatchFromLog = useCallback((message: string, type: string) => {
        const spawnMatch = message.match(/Spawning batch:\s*(.+)/i);
        const completeMatch = message.match(/Mission completed:\s*(.+)/i);
        const mergeSuccessMatch = message.match(/Successfully merged batch\s*'?([^']+)'?/i);
        // T010: Add pattern for backend's actual format: "Batch <id> completed on branch <branch>"
        const batchCompleteMatch = message.match(/Batch\s+(\S+)\s+completed on branch\s+(\S+)/i);

        let batchName: string | null = null;
        let newStatus: BatchStatus | null = null;
        let isStart = false;
        let isComplete = false;

        if (spawnMatch) {
            batchName = spawnMatch[1].trim();
            newStatus = 'running';
            isStart = true;
        } else if (completeMatch) {
            batchName = completeMatch[1].trim();
            newStatus = 'completed';
            isComplete = true;
        } else if (mergeSuccessMatch) {
            batchName = mergeSuccessMatch[1].trim();
            newStatus = 'completed';
            isComplete = true;
        } else if (batchCompleteMatch) {
            // T010: Handle backend's batch complete format (matches by batch ID)
            batchName = batchCompleteMatch[1].trim();
            newStatus = 'completed';
            isComplete = true;
        }

        if (batchName && newStatus) {
            setBatches(prev => {
                const batch = prev.find(b => {
                    return b.name.trim().toLowerCase() === batchName!.trim().toLowerCase();
                });

                if (batch) {
                    addLog(message, type as LogEntry['type'], batch.id);

                    if (isStart) currentBatchRef.current = batch.id;
                    if (isComplete) {
                        setCompletedBatches(p => new Set(p).add(batchName!));
                        if (currentBatchRef.current === batch.id) currentBatchRef.current = null;

                        // Record completion time for auto-collapse
                        const completedBatchId = batch.id;
                        setBatchCompletedAt(prev => ({ ...prev, [completedBatchId]: Date.now() }));

                        // T026: Use managed timeout for auto-collapse (BUG-005)
                        // This timeout will be automatically cleaned up on unmount
                        setTimeout(() => {
                            setBatchCompletedAt(prev => ({ ...prev })); // Force re-render
                        }, 5100);
                    }

                    return prev.map(b => b.id === batch.id ? { ...b, status: newStatus! } : b);
                }
                return prev;
            });

        } else if (currentBatchRef.current) {
            setBatchLogs(prev => {
                const batchId = currentBatchRef.current;
                if (batchId && prev[batchId]) {
                    const time = new Date().toLocaleTimeString('en-US', {
                        hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit'
                    });
                    return {
                        ...prev,
                        [batchId]: [...prev[batchId], { time, message, type: type as LogEntry['type'] }]
                    };
                }
                return prev;
            });
        }
    }, [addLog]);

    // T020: Process batched messages using requestAnimationFrame (BUG-001)
    const processBatchedMessages = useCallback(() => {
        const messages = pendingMessagesRef.current;
        if (messages.length === 0) return;

        // Clear pending messages before processing
        pendingMessagesRef.current = [];

        // Process all pending messages in a single batch
        startTransition(() => {
            messages.forEach(data => {
                // Handle explicit status messages
                if (data.type === 'status') {
                    if (data.status === 'running') {
                        setExecutionStatus('running');
                        setWsRetryCount(0);
                        setWsRetryCountdown(0);
                        setShowCompletionSummary(false);
                        setCompletionError(null);
                    } else if (data.status === 'completed') {
                        setExecutionStatus('completed');
                        addLog(data.message || 'Execution completed', 'success');
                        currentBatchRef.current = null;
                        // T031: Show completion summary
                        setShowCompletionSummary(true);
                        setCompletionError(null);
                    } else if (data.status === 'failed') {
                        setExecutionStatus('failed');
                        addLog(data.message || 'Execution failed', 'error');
                        currentBatchRef.current = null;
                        // T031: Show failure summary
                        setShowCompletionSummary(true);
                        setCompletionError(data.error || data.message || 'Execution failed');
                    } else if (data.status === 'aborted') {
                        setExecutionStatus('aborted');
                        addLog('Execution aborted', 'error');
                        currentBatchRef.current = null;
                        // T031: Show aborted summary
                        setShowCompletionSummary(true);
                        setCompletionError('Execution was aborted by user');
                    }
                }
                // T013: Handle explicit batch_status messages
                else if (data.type === 'batch_status' && data.batch_id) {
                    const batchId = data.batch_id;
                    const batchStatus = data.status as BatchStatus;

                    setBatches(prev => prev.map(b => {
                        // Match by ID or by name containing the batch ID
                        if (b.id === batchId || b.name.toLowerCase().includes(batchId.toLowerCase())) {
                            if (batchStatus === 'completed') {
                                setCompletedBatches(p => new Set(p).add(b.name));
                                // Record completion time for auto-collapse
                                setBatchCompletedAt(prevState => ({ ...prevState, [b.id]: Date.now() }));
                            }
                            return { ...b, status: batchStatus };
                        }
                        return b;
                    }));
                }
                // T007: Fallback handling for 'start' type to set running status
                else if (data.type === 'start') {
                    setExecutionStatus('running');
                    setWsRetryCount(0);
                    setWsRetryCountdown(0);
                    if (data.message) {
                        addLog(data.message, 'info');
                    }
                }
                // T016: Fallback handling for 'success' type to set completed status
                else if (data.type === 'success') {
                    setExecutionStatus('completed');
                    if (data.message) {
                        addLog(data.message, 'success');
                    }
                    currentBatchRef.current = null;
                }
                // Handle regular log messages
                else if (data.message) {
                    const logType = (data.type as LogEntry['type']) || 'info';
                    addLog(data.message, logType);
                    updateBatchFromLog(data.message, data.type);
                }
            });
        });
    }, [addLog, updateBatchFromLog, startTransition]);

    const connectWebSocket = useCallback((runId: string, retryCount: number = 0) => {
        const wsProtocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const wsUrl = `${wsProtocol}//${window.location.host}/api/execution/ws?run_id=${runId}`;

        // T024: Maximum retry configuration (BUG-002)
        const MAX_RETRIES = 3;
        const RETRY_DELAYS = [5000, 10000, 20000]; // Exponential backoff

        console.log(`Attempting to connect to WebSocket: ${wsUrl} (attempt ${retryCount + 1})`);
        const ws = new WebSocket(wsUrl);
        wsRef.current = ws;

        ws.onopen = () => {
            console.log('WebSocket connection established');
            addLog('Connected to execution stream', 'info');
            setWsRetryCount(0);
            setWsRetryCountdown(0);
            if (retryCount > 0) {
                setExecutionStatus('running'); // Restore from 'reconnecting'
            }
        };

        // T020: Batch messages using requestAnimationFrame (BUG-001)
        ws.onmessage = (event) => {
            try {
                const data: WsMessage = JSON.parse(event.data);
                console.log('Received WebSocket message:', data);

                // Add to pending messages
                pendingMessagesRef.current.push(data);

                // Schedule processing on next animation frame if not already scheduled
                if (!rafIdRef.current) {
                    rafIdRef.current = requestAnimationFrame(() => {
                        rafIdRef.current = null;
                        processBatchedMessages();
                    });
                }
            } catch {
                console.log('Received raw WebSocket message:', event.data);
                addLog(event.data, 'info');
            }
        };

        // T023/T024: WebSocket error handling with reconnection (BUG-002)
        ws.onerror = (error) => {
            console.error('WebSocket error:', error);
            addLog('WebSocket connection error', 'error');
        };

        // T023/T024: WebSocket close with reconnection logic (BUG-002)
        ws.onclose = (event) => {
            console.log('WebSocket connection closed', event);

            // Only attempt reconnection if execution was in progress and not intentionally closed
            if (executionStatus === 'running' || executionStatus === 'reconnecting') {
                if (retryCount < MAX_RETRIES) {
                    const delay = RETRY_DELAYS[retryCount] || RETRY_DELAYS[RETRY_DELAYS.length - 1];
                    const delaySeconds = Math.ceil(delay / 1000);

                    setExecutionStatus('reconnecting');
                    setWsRetryCount(retryCount + 1);
                    setWsRetryCountdown(delaySeconds);

                    addLog(`Connection lost. Reconnecting in ${delaySeconds}s (attempt ${retryCount + 2}/${MAX_RETRIES + 1})...`, 'info');

                    // Start countdown
                    let countdown = delaySeconds;
                    const countdownInterval = window.setInterval(() => {
                        countdown -= 1;
                        setWsRetryCountdown(countdown);
                        if (countdown <= 0) {
                            clearInterval(countdownInterval);
                        }
                    }, 1000);

                    // Schedule reconnection using managed timeout
                    setTimeout(() => {
                        clearInterval(countdownInterval);
                        connectWebSocket(runId, retryCount + 1);
                    }, delay);
                } else {
                    // All retries exhausted
                    setExecutionStatus('failed');
                    setWsRetryCount(0);
                    setWsRetryCountdown(0);
                    addLog('Connection failed after maximum retry attempts', 'error');
                }
            } else {
                addLog('Execution stream closed', 'info');
            }
        };
    }, [addLog, processBatchedMessages, executionStatus, setTimeout]);

    const handleRun = useCallback(async () => {
        if (!selectedSpecName) return;

        const runId = `run-${Date.now()}`;
        runIdRef.current = runId;

        setExecutionStatus('starting');

        // T025: Fix terminal clear race condition (BUG-003)
        // Use requestAnimationFrame to ensure terminal is ready before clearing
        if (terminalRef.current) {
            requestAnimationFrame(() => {
                try {
                    terminalRef.current?.clear();
                } catch (e) {
                    console.warn('Terminal clear failed, will retry:', e);
                    // Fallback: try again after a short delay
                    window.setTimeout(() => {
                        terminalRef.current?.clear();
                    }, 100);
                }
            });
        }

        setBatchLogs(prev => {
            const reset: Record<string, LogEntry[]> = {};
            Object.keys(prev).forEach(k => { reset[k] = []; });
            return reset;
        });
        setCompletedBatches(new Set());
        setElapsedTime(0);
        currentBatchRef.current = null;
        setBatches(prev => prev.map(b => ({ ...b, status: 'pending' as BatchStatus })));

        addLog('🚀 Starting execution...', 'start');

        try {
            const res = await startExecution(selectedSpecName, runId);
            if (res.success) {
                connectWebSocket(runId);
            } else {
                addLog(`Failed to start: ${res.message}`, 'error');
                setExecutionStatus('failed');
            }
        } catch (e) {
            addLog(`Error: ${e}`, 'error');
            setExecutionStatus('failed');
        }
    }, [selectedSpecName, addLog, connectWebSocket]);

    const handlePlan = useCallback(async () => {
        if (!selectedSpecName) return;

        const runId = `plan-${Date.now()}`;
        runIdRef.current = runId;

        setExecutionStatus('starting');
        terminalRef.current?.clear();
        setBatchLogs(prev => {
            const reset: Record<string, LogEntry[]> = {};
            Object.keys(prev).forEach(k => { reset[k] = []; });
            return reset;
        });
        setCompletedBatches(new Set());
        setElapsedTime(0);
        currentBatchRef.current = null;
        setBatches(prev => prev.map(b => ({ ...b, status: 'pending' as BatchStatus })));

        addLog('📋 Running execution plan (dry-run)...', 'start');

        try {
            const res = await startExecution(selectedSpecName, runId, true);
            if (res.success) {
                connectWebSocket(runId);
            } else {
                addLog(`Failed to start plan: ${res.message}`, 'error');
                setExecutionStatus('failed');
            }
        } catch (e) {
            addLog(`Error: ${e}`, 'error');
            setExecutionStatus('failed');
        }
    }, [selectedSpecName, addLog, connectWebSocket]);

    const handleStop = useCallback(async () => {
        if (runIdRef.current) {
            await stopExecution(runIdRef.current);
            wsRef.current?.send(JSON.stringify({ action: 'abort' }));
        }
        addLog('🛑 Stopping execution...', 'error');
    }, [addLog]);

    const handleReset = useCallback(() => {
        setBatches(prev => prev.map(b => ({ ...b, status: 'pending' as BatchStatus })));
        terminalRef.current?.clear();
        setBatchLogs(prev => {
            const reset: Record<string, LogEntry[]> = {};
            Object.keys(prev).forEach(k => { reset[k] = []; });
            return reset;
        });
        setCompletedBatches(new Set());
        setElapsedTime(0);
        setExecutionStatus('idle');
        currentBatchRef.current = null;
        runIdRef.current = '';
        if (selectedSpecName) {
            queryClient.invalidateQueries({ queryKey: ['plan', selectedSpecName] });
        }
    }, [selectedSpecName, queryClient]);

    const handleBack = useCallback(() => {
        wsRef.current?.close();
        setSelectedSpecName(null);
        handleReset();
    }, [handleReset]);


    const progress = batches.length > 0 ? (completedBatches.size / batches.length) * 100 : 0;
    const estimatedCost = batches.filter(b => completedBatches.has(b.name)).reduce((sum, b) => sum + b.estimated_cost, 0);

    const activeBatches = useMemo(() => {
        const now = Date.now();
        const COLLAPSE_DELAY_MS = 5000; // 5 seconds

        return batches.filter(b => {
            if (b.status === 'running' || b.status === 'waiting') return true;
            if (b.status === 'failed') return true;
            if (b.status === 'completed') {
                // Show completed batches for 5 seconds after completion
                const completedTime = batchCompletedAt[b.id];
                if (completedTime && (now - completedTime) < COLLAPSE_DELAY_MS) {
                    return true;
                }
                return false;
            }
            return false;
        });
    }, [batches, batchCompletedAt]);

    const loadUnmergedBranches = useCallback(async () => {
        if (!selectedSpecName) return;
        try {
            const result = await fetchUnmergedBranches(selectedSpecName);
            if (result.success) {
                setUnmergedBranches(result.branches);
            }
        } catch (e) {
            console.error('Failed to load unmerged branches:', e);
        }
    }, [selectedSpecName]);

    useEffect(() => {
        if (selectedSpecName && (executionStatus === 'idle' || executionStatus === 'completed')) {
            loadUnmergedBranches();
        }
    }, [selectedSpecName, executionStatus, loadUnmergedBranches]);

    const handleMergeAll = useCallback(async () => {
        if (!selectedSpecName) return;
        setIsMerging(true);
        setMergeResult(null);
        try {
            const result = await mergeAllBranches(selectedSpecName);
            setMergeResult({ success: result.success, message: result.message });
            if (result.success || result.merged.length > 0) {
                await loadUnmergedBranches();
            }
        } catch (e) {
            setMergeResult({ success: false, message: `Error: ${e}` });
        } finally {
            setIsMerging(false);
        }
    }, [selectedSpecName, loadUnmergedBranches]);

    if (!selectedSpecName) {
        return (
            <div className="flex-1 p-6 relative">
                <div className="max-w-4xl mx-auto space-y-6">
                    <header className="mb-8">
                        <h2 className="text-xl font-semibold text-gray-200 mb-2">Execution Runner</h2>
                        <p className="text-gray-400 text-sm">Select a specification to plan and execute</p>
                    </header>
                    <SpecListView
                        specs={specsData?.specs || []}
                        onSelect={setSelectedSpecName}
                        isLoading={isLoadingSpecs}
                    />
                </div>
            </div>
        );
    }

    const { cols, rows } = getGridLayout(activeBatches.length);

    return (
        <div className="flex-1 flex flex-col h-full overflow-hidden bg-gray-950">
            {/* Header Toolbar */}
            <div className="h-14 shrink-0 border-b border-gray-800 bg-gray-900/50 flex items-center justify-between px-4">
                <div className="flex items-center gap-3">
                    <button
                        onClick={handleBack}
                        className="p-1.5 hover:bg-gray-800 rounded-lg text-gray-400 transition-colors"
                    >
                        <ArrowRight className="rotate-180" size={18} />
                    </button>
                    <div className="flex flex-col">
                        <span className="font-semibold text-gray-200">{selectedSpecName}</span>
                        <div className="flex items-center gap-2 text-xs text-gray-500">
                            <span>{batches.length} batches</span>
                            <span>•</span>
                            <span>{formatElapsedTime(elapsedTime)}</span>
                        </div>
                    </div>
                </div>

                <div className="flex items-center gap-3">
                    {executionStatus === 'idle' || executionStatus === 'completed' || executionStatus === 'failed' || executionStatus === 'aborted' ? (
                        <>
                            <button
                                onClick={handlePlan}
                                className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-gray-800 hover:bg-gray-700 text-gray-300 font-medium text-xs transition-colors"
                                data-testid="dry-run-button"
                                aria-label="Run dry run"
                            >
                                <Layers size={14} />
                                Dry Run
                            </button>
                            <button
                                onClick={handleRun}
                                className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-white font-medium text-xs shadow-lg shadow-emerald-900/20 transition-all hover:scale-105 active:scale-95"
                                data-testid="run-button"
                                aria-label="Start execution"
                            >
                                <Zap size={14} fill="currentColor" />
                                Run Execution
                            </button>
                        </>
                    ) : (
                        <button
                            onClick={handleStop}
                            className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-red-900/50 hover:bg-red-900 border border-red-800 text-red-300 font-medium text-xs transition-colors"
                            data-testid="stop-button"
                            aria-label="Stop execution"
                        >
                            <Square size={14} fill="currentColor" />
                            Stop
                        </button>
                    )}

                    {/* T024: WebSocket reconnection indicator (BUG-002) */}
                    {executionStatus === 'reconnecting' && (
                        <div
                            className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-amber-900/50 border border-amber-800 text-amber-300 text-xs font-medium"
                            data-testid="reconnecting-indicator"
                        >
                            <Loader2 size={14} className={isPending ? 'animate-spin' : 'animate-spin'} />
                            <span>Reconnecting in {wsRetryCountdown}s</span>
                            <span className="text-amber-500">(attempt {wsRetryCount}/{3})</span>
                        </div>
                    )}

                    <button
                        onClick={handleReset}
                        className="p-2 rounded-lg hover:bg-gray-800 text-gray-400 transition-colors"
                        title="Reset state"
                        aria-label="Reset execution state"
                    >
                        <RotateCcw size={16} />
                    </button>
                </div>
            </div>

            {/* Main Content Area */}
            <div className="flex-1 overflow-hidden p-4 relative">
                <div className="h-full flex flex-col gap-4">
                    {/* Top Section: Progress & Stats */}
                    <div className="flex gap-4 shrink-0 h-24">
                        {/* Overall Progress */}
                        <div className="w-64 bg-gray-900/50 rounded-xl border border-gray-800 p-4 flex items-center gap-4">
                            <ProgressRing progress={progress} size={56} strokeWidth={6} />
                            <div>
                                <div className="text-2xl font-bold text-gray-200">
                                    {completedBatches.size}/{batches.length}
                                </div>
                                <div className="text-xs text-gray-500 font-medium">BATCHES COMPLETED</div>
                            </div>
                        </div>

                        {/* Stats Cards */}
                        <div className="flex-1 grid grid-cols-2 gap-4">
                            <div className="bg-gray-900/50 rounded-xl border border-gray-800 p-4 flex flex-col justify-center relative overflow-hidden group">
                                <div className="absolute right-2 top-2 p-1.5 bg-gray-800/50 rounded-lg text-gray-500 group-hover:bg-cyan-900/30 group-hover:text-cyan-400 transition-colors">
                                    <Timer size={16} />
                                </div>
                                <div className="text-2xl font-mono font-bold text-gray-200 tabular-nums">
                                    {formatElapsedTime(elapsedTime)}
                                </div>
                                <div className="text-xs text-gray-500 font-medium mt-1">ELAPSED TIME</div>
                            </div>

                            <div className="bg-gray-900/50 rounded-xl border border-gray-800 p-4 flex flex-col justify-center relative overflow-hidden group">
                                <div className="absolute right-2 top-2 p-1.5 bg-gray-800/50 rounded-lg text-gray-500 group-hover:bg-emerald-900/30 group-hover:text-emerald-400 transition-colors">
                                    <DollarSign size={16} />
                                </div>
                                <div className="text-2xl font-mono font-bold text-gray-200 tabular-nums">
                                    ${estimatedCost.toFixed(4)}
                                </div>
                                <div className="text-xs text-gray-500 font-medium mt-1">ESTIMATED COST</div>
                            </div>
                        </div>
                    </div>

                    {/* Middle Section: Batch Grid (if active) */}
                    {activeBatches.length > 0 && (
                        <div className="flex-1 min-h-0 bg-gray-900/30 rounded-xl border border-gray-800/50 p-2 overflow-y-auto">
                            <div className={`grid grid-cols-${cols} grid-rows-${rows} gap-2 h-full`}>
                                {activeBatches.map(batch => (
                                    <BatchLogPanel
                                        key={batch.id}
                                        batch={batch}
                                        logs={batchLogs[batch.id] || []}
                                        isExpanded={expandedBatchId === batch.id}
                                        onToggleExpand={() => setExpandedBatchId(expandedBatchId === batch.id ? null : batch.id)}
                                    />
                                ))}
                            </div>
                        </div>
                    )}

                    {/* Bottom Section: Orchestrator Log (Always visible) */}
                    <div className={`
                         rounded-xl border border-gray-700 overflow-hidden shrink-0 flex flex-col
                         transition-all duration-300 shadow-xl
                         ${orchestratorMinimized ? 'h-10' : 'h-64'}
                     `} style={{ background: '#1e1e1e' }}>
                        <div className="px-4 py-2 bg-[#252526] border-b border-[#333] flex items-center justify-between shrink-0 h-10">
                            <h3 className="font-semibold flex items-center gap-2 text-sm text-gray-300">
                                <TerminalIcon size={14} className="text-emerald-400" />
                                Orchestrator Log
                            </h3>
                            <button
                                onClick={() => setOrchestratorMinimized(!orchestratorMinimized)}
                                className="p-1 hover:bg-white/10 rounded transition-colors text-gray-400"
                            >
                                {orchestratorMinimized ? <Maximize2 size={12} /> : <Minimize2 size={12} />}
                            </button>
                        </div>
                        <div className={`flex-1 relative ${orchestratorMinimized ? 'hidden' : 'block'}`}>
                            <LogTerminal
                                onMount={(term) => { terminalRef.current = term; }}
                            />
                        </div>
                    </div>
                </div>
            </div>

            {/* T030: Completion Summary Overlay */}
            {showCompletionSummary && (executionStatus === 'completed' || executionStatus === 'failed' || executionStatus === 'aborted') && (
                <div className="absolute inset-0 bg-black/50 flex items-center justify-center z-50 animate-fade-in">
                    <div className="w-full max-w-2xl mx-4 animate-slide-up">
                        <CompletionSummary
                            status={executionStatus as RunStatus}
                            summary={{
                                total_batches: batches.length,
                                completed_batches: completedBatches.size,
                                failed_batches: batches.filter(b => b.status === 'failed').length,
                                pending_batches: batches.filter(b => b.status === 'pending').length,
                                tasks_completed: batches.reduce((sum, b) =>
                                    b.status === 'completed' ? sum + b.task_ids.length : sum, 0
                                ),
                                branches_merged: unmergedBranches.filter(b => b.is_clean).length,
                            }}
                            elapsedSeconds={elapsedTime}
                            dryRun={runIdRef.current?.includes('dry')}
                            error={completionError}
                            onClose={() => setShowCompletionSummary(false)}
                        />
                    </div>
                </div>
            )}

            {/* Merge Panel - Slide over */}
            {unmergedBranches.length > 0 && (executionStatus === 'completed' || executionStatus === 'idle') && (
                <div className="absolute right-4 bottom-20 w-80 bg-gray-900 border border-gray-700 rounded-xl shadow-2xl overflow-hidden animate-slide-up">
                    <div className="p-3 border-b border-gray-700 flex items-center justify-between bg-gray-800/50">
                        <div className="flex items-center gap-2">
                            <GitMerge size={16} className="text-purple-400" />
                            <span className="font-medium text-sm text-gray-200">Merge Candidates</span>
                        </div>
                        <span className="text-xs bg-purple-900/30 text-purple-300 px-1.5 py-0.5 rounded-full border border-purple-800">
                            {unmergedBranches.length}
                        </span>
                    </div>
                    <div className="max-h-64 overflow-y-auto">
                        {unmergedBranches.map(branch => (
                            <div key={branch.name} className="p-3 border-b border-gray-800 hover:bg-gray-800/30 transition-colors">
                                <div className="flex items-center justify-between mb-1">
                                    <span className="text-xs font-mono text-purple-300">{branch.name}</span>
                                    {branch.is_clean ? (
                                        <CheckCircle2 size={12} className="text-emerald-500" />
                                    ) : (
                                        <div title="Working directory not clean">
                                            <AlertTriangle size={12} className="text-amber-500" />
                                        </div>
                                    )}
                                </div>
                                <div className="text-xs text-gray-500 truncate">{branch.batch_name}</div>
                                <div className="text-[10px] text-gray-600 mt-1">
                                    {branch.ahead_commits} commits ahead
                                </div>
                            </div>
                        ))}
                    </div>
                    <div className="p-3 bg-gray-800/50">
                        {mergeResult ? (
                            <div className={`text-xs p-2 rounded mb-2 ${mergeResult.success ? 'bg-emerald-900/20 text-emerald-400' : 'bg-red-900/20 text-red-400'}`}>
                                {mergeResult.message}
                            </div>
                        ) : null}
                        <button
                            onClick={handleMergeAll}
                            disabled={isMerging}
                            className="w-full py-1.5 rounded bg-purple-600 hover:bg-purple-500 text-white text-xs font-medium transition-colors disabled:opacity-50 flex items-center justify-center gap-2"
                        >
                            {isMerging ? (
                                <>
                                    <Loader2 size={12} className="animate-spin" />
                                    Merging...
                                </>
                            ) : (
                                <>
                                    <GitMerge size={12} />
                                    Merge All Branches
                                </>
                            )}
                        </button>
                    </div>
                </div>
            )}
        </div>
    );
}
