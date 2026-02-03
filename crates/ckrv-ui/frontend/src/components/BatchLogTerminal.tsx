/**
 * @module BatchLogTerminal
 * @description
 * Terminal-style component for displaying logs from a single batch execution.
 * Features consistent dark theme styling, auto-scroll to bottom, and compact
 * display optimized for carousel view.
 *
 * @context
 * Used within BatchLogCarousel to display individual batch logs. Shows batch
 * name, status indicator, model used, and scrollable log content with
 * color-coded entries.
 *
 * @dependencies
 * - lucide-react: Icons for status indicators
 *
 * @example
 * <BatchLogTerminal
 *   batchId="batch-1"
 *   batchName="Stage 1"
 *   batchIndex={0}
 *   status="running"
 *   logs={batchLogs}
 *   model="claude-sonnet-4"
 * />
 */

// === IMPORTS ===
import React, { useRef, useEffect } from 'react';
import { Loader2, CheckCircle2, AlertTriangle, Clock } from 'lucide-react';

// ============================================================
// CONSTANTS
// ============================================================

// Consistent gray theme matching the app's dark design
const BATCH_THEME = {
    bg: 'bg-gray-900/50',
    border: 'border-gray-700',
    text: 'text-gray-300',
    header: 'bg-[#252526]'
};


// ============================================================
// TYPES
// ============================================================

export type BatchStatus = 'pending' | 'waiting' | 'running' | 'completed' | 'failed';

export interface BatchLogEntry {
    time: string;
    message: string;
    type: 'info' | 'success' | 'error' | 'log' | 'start' | 'batch_start' | 'batch_complete' | 'batch_error';
}

export interface BatchLogTerminalProps {
    /** Batch ID */
    batchId: string;
    /** Batch name/title */
    batchName: string;
    /** Batch index (for color cycling) */
    batchIndex: number;
    /** Current status of the batch */
    status: BatchStatus;
    /** Log entries for this batch */
    logs: BatchLogEntry[];
    /** Optional branch name (shown when completed) */
    branch?: string;
    /** Whether to auto-scroll to bottom */
    autoScroll?: boolean;
    /** Model used for this batch (e.g., "claude-sonnet-4-20250514") */
    model?: string;
}


// ============================================================
// SUB-COMPONENTS
// ============================================================

const StatusIcon: React.FC<{ status: BatchStatus }> = ({ status }) => {
    switch (status) {
        case 'running':
            return <Loader2 size={14} className="animate-spin" />;
        case 'completed':
            return <CheckCircle2 size={14} />;
        case 'failed':
            return <AlertTriangle size={14} />;
        case 'waiting':
            return <Clock size={14} />;
        default:
            return null;
    }
};

export const BatchLogTerminal: React.FC<BatchLogTerminalProps> = ({
    batchId: _batchId, // Used for future features like log persistence
    batchName,
    batchIndex: _batchIndex, // Reserved for future use
    status,
    logs,
    branch,
    autoScroll = true,
    model,
}) => {
    const scrollRef = useRef<HTMLDivElement>(null);

    // Use consistent theme (no color cycling)
    const theme = BATCH_THEME;

    // Auto-scroll to bottom when new logs arrive
    useEffect(() => {
        if (autoScroll && scrollRef.current) {
            scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
        }
    }, [logs, autoScroll]);

    const statusColors: Record<BatchStatus, string> = {
        pending: 'text-gray-400',
        waiting: 'text-gray-400',
        running: 'text-amber-400',
        completed: 'text-emerald-400',
        failed: 'text-red-400',
    };

    // Format model name for display (e.g., "claude-sonnet-4-20250514" -> "Claude Sonnet 4")
    const formatModelName = (modelId: string | undefined): string => {
        if (!modelId) return '';
        // Handle common model patterns
        if (modelId.includes('claude')) {
            const parts = modelId.split('-');
            // claude-sonnet-4, claude-opus-4, etc.
            if (parts.length >= 2) {
                const variant = parts[1].charAt(0).toUpperCase() + parts[1].slice(1);
                const version = parts[2] || '';
                return `Claude ${variant}${version ? ' ' + version : ''}`;
            }
            return 'Claude';
        }
        if (modelId.includes('gpt')) {
            return modelId.toUpperCase().replace(/-/g, ' ');
        }
        // Default: capitalize first letter
        return modelId.split('/').pop()?.split('-').map(w =>
            w.charAt(0).toUpperCase() + w.slice(1)
        ).join(' ') || modelId;
    };

    return (
        <div className={`flex flex-col h-full rounded-lg border ${theme.border} ${theme.bg} overflow-hidden`}>
            {/* Batch Header */}
            <div className={`flex items-center justify-between px-3 py-2 ${theme.header} border-b ${theme.border}`}>
                <div className="flex items-center gap-2">
                    <span className={`font-medium text-sm ${theme.text}`}>{batchName}</span>

                    {/* Model Chip */}
                    {model && (
                        <span className="px-2 py-0.5 text-[10px] font-medium rounded-full bg-gray-700 text-gray-300 border border-gray-600">
                            {formatModelName(model)}
                        </span>
                    )}

                    {branch && status === 'completed' && (
                        <span className="text-xs text-muted-foreground">
                            {branch}
                        </span>
                    )}
                </div>
                <div className={`flex items-center gap-1.5 text-xs ${statusColors[status]}`}>
                    <StatusIcon status={status} />
                    <span className="capitalize">{status}</span>
                </div>
            </div>


            {/* Log Content */}
            <div
                ref={scrollRef}
                className="flex-1 min-h-0 overflow-y-auto p-2 font-mono text-xs scrollbar-thin scrollbar-thumb-gray-600 scrollbar-track-gray-800"
                style={{ maxHeight: 'calc(100% - 44px)' }}
            >
                {logs.length === 0 ? (
                    <div className="flex items-center justify-center h-full text-muted-foreground">
                        {status === 'pending' ? 'Waiting to start...' : 'No logs yet'}
                    </div>
                ) : (
                    logs.map((log, i) => (
                        <div key={i} className="flex gap-2 leading-relaxed">
                            <span className="text-muted-foreground shrink-0">{log.time}</span>
                            <span className={getLogTypeColor(log.type)}>{log.message}</span>
                        </div>
                    ))
                )}
            </div>
        </div>
    );
};

function getLogTypeColor(type: BatchLogEntry['type']): string {
    switch (type) {
        case 'success':
        case 'batch_complete':
            return 'text-emerald-400';
        case 'error':
        case 'batch_error':
            return 'text-red-400';
        case 'start':
        case 'batch_start':
            return 'text-amber-400';
        default:
            return 'text-foreground';
    }
}

export default BatchLogTerminal;
