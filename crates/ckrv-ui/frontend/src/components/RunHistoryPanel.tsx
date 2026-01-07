/**
 * Run History Panel Component
 * 
 * Displays a list of past runs for a specification with status indicators.
 */

import {
    Clock, CheckCircle2, XCircle, Loader2,
    Slash, Circle, ChevronRight, Trash2, History
} from 'lucide-react';
import type { Run, RunStatus } from '../types/history';
import { formatElapsedTime, formatRelativeTime } from '../types/history';

interface RunHistoryPanelProps {
    runs: Run[];
    selectedRunId: string | null;
    onSelectRun: (run: Run) => void;
    onDeleteRun?: (runId: string) => void;
    isLoading?: boolean;
    error?: string | null;
}

/**
 * Get status icon component
 */
function StatusIcon({ status }: { status: RunStatus }) {
    switch (status) {
        case 'completed':
            return <CheckCircle2 className="w-4 h-4 text-green-500" />;
        case 'failed':
            return <XCircle className="w-4 h-4 text-red-500" />;
        case 'running':
            return <Loader2 className="w-4 h-4 text-blue-500 animate-spin" />;
        case 'aborted':
            return <Slash className="w-4 h-4 text-orange-500" />;
        case 'pending':
        default:
            return <Circle className="w-4 h-4 text-gray-400" />;
    }
}

/**
 * Get status label with color
 */
function StatusLabel({ status }: { status: RunStatus }) {
    const colors: Record<RunStatus, string> = {
        completed: 'text-green-600 bg-green-50',
        failed: 'text-red-600 bg-red-50',
        running: 'text-blue-600 bg-blue-50',
        aborted: 'text-orange-600 bg-orange-50',
        pending: 'text-gray-600 bg-gray-50',
    };

    return (
        <span className={`px-2 py-0.5 rounded-full text-xs font-medium ${colors[status]}`}>
            {status.charAt(0).toUpperCase() + status.slice(1)}
        </span>
    );
}

/**
 * Single run item in the history list
 */
function RunItem({
    run,
    isSelected,
    onClick,
    onDelete,
}: {
    run: Run;
    isSelected: boolean;
    onClick: () => void;
    onDelete?: () => void;
}) {
    return (
        <div
            className={`
        group cursor-pointer p-3 rounded-lg border transition-all
        ${isSelected
                    ? 'border-purple-400 bg-purple-50/50 shadow-sm'
                    : 'border-gray-200 hover:border-gray-300 hover:bg-gray-50'}
      `}
            onClick={onClick}
        >
            <div className="flex items-start justify-between gap-2">
                <div className="flex items-center gap-2 min-w-0">
                    <StatusIcon status={run.status} />
                    <div className="min-w-0">
                        <div className="flex items-center gap-2">
                            <span className="text-sm font-medium text-gray-900 truncate">
                                {run.dry_run ? '🧪 Dry Run' : 'Run'}
                            </span>
                            <StatusLabel status={run.status} />
                        </div>
                        <p className="text-xs text-gray-500 mt-0.5">
                            {formatRelativeTime(run.started_at)}
                        </p>
                    </div>
                </div>

                <div className="flex items-center gap-2">
                    {onDelete && run.status !== 'running' && (
                        <button
                            onClick={(e) => {
                                e.stopPropagation();
                                onDelete();
                            }}
                            className="p-1 rounded opacity-0 group-hover:opacity-100 hover:bg-red-100 transition-opacity"
                            title="Delete run"
                        >
                            <Trash2 className="w-3.5 h-3.5 text-red-500" />
                        </button>
                    )}
                    <ChevronRight className="w-4 h-4 text-gray-400" />
                </div>
            </div>

            {/* Summary stats */}
            <div className="flex items-center gap-4 mt-2 text-xs text-gray-500">
                <span className="flex items-center gap-1">
                    <CheckCircle2 className="w-3 h-3" />
                    {run.summary.completed_batches}/{run.summary.total_batches}
                </span>
                {run.elapsed_seconds !== null && (
                    <span className="flex items-center gap-1">
                        <Clock className="w-3 h-3" />
                        {formatElapsedTime(run.elapsed_seconds)}
                    </span>
                )}
                {run.summary.branches_merged > 0 && (
                    <span className="text-green-600">
                        {run.summary.branches_merged} merged
                    </span>
                )}
            </div>
        </div>
    );
}

/**
 * Empty state when no runs exist
 */
function EmptyState() {
    return (
        <div className="flex flex-col items-center justify-center py-8 text-center">
            <div className="w-12 h-12 rounded-full bg-gray-100 flex items-center justify-center mb-3">
                <History className="w-6 h-6 text-gray-400" />
            </div>
            <h3 className="text-sm font-medium text-gray-900 mb-1">No run history</h3>
            <p className="text-xs text-gray-500 max-w-[200px]">
                Run history will appear here after you execute your first run.
            </p>
        </div>
    );
}

/**
 * Loading skeleton
 */
function LoadingSkeleton() {
    return (
        <div className="space-y-2">
            {[1, 2, 3].map((i) => (
                <div key={i} className="p-3 rounded-lg border border-gray-200 animate-pulse">
                    <div className="flex items-center gap-2">
                        <div className="w-4 h-4 rounded-full bg-gray-200" />
                        <div className="flex-1">
                            <div className="h-4 bg-gray-200 rounded w-24" />
                            <div className="h-3 bg-gray-100 rounded w-16 mt-1" />
                        </div>
                    </div>
                </div>
            ))}
        </div>
    );
}

/**
 * Run History Panel
 */
export function RunHistoryPanel({
    runs,
    selectedRunId,
    onSelectRun,
    onDeleteRun,
    isLoading = false,
    error = null,
}: RunHistoryPanelProps) {
    if (isLoading) {
        return (
            <div className="p-4">
                <h2 className="text-sm font-semibold text-gray-700 mb-3 flex items-center gap-2">
                    <History className="w-4 h-4" />
                    Run History
                </h2>
                <LoadingSkeleton />
            </div>
        );
    }

    if (error) {
        return (
            <div className="p-4">
                <h2 className="text-sm font-semibold text-gray-700 mb-3 flex items-center gap-2">
                    <History className="w-4 h-4" />
                    Run History
                </h2>
                <div className="p-3 rounded-lg bg-red-50 border border-red-200 text-sm text-red-600">
                    Failed to load history: {error}
                </div>
            </div>
        );
    }

    return (
        <div className="p-4">
            <h2 className="text-sm font-semibold text-gray-700 mb-3 flex items-center gap-2">
                <History className="w-4 h-4" />
                Run History
                {runs.length > 0 && (
                    <span className="text-xs font-normal text-gray-400">
                        ({runs.length})
                    </span>
                )}
            </h2>

            {runs.length === 0 ? (
                <EmptyState />
            ) : (
                <div className="space-y-2 max-h-[400px] overflow-y-auto">
                    {runs.map((run) => (
                        <RunItem
                            key={run.id}
                            run={run}
                            isSelected={run.id === selectedRunId}
                            onClick={() => onSelectRun(run)}
                            onDelete={onDeleteRun ? () => onDeleteRun(run.id) : undefined}
                        />
                    ))}
                </div>
            )}
        </div>
    );
}

export default RunHistoryPanel;
