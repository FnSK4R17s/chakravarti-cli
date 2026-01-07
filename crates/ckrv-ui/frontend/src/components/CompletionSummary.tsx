/**
 * Completion Summary Component
 * 
 * Displays a clear summary when execution runs complete, including
 * batch statistics, elapsed time, and merged branches.
 */

import {
    CheckCircle2, XCircle, Clock, GitMerge,
    Trophy, AlertTriangle, Layers, Timer
} from 'lucide-react';
import type { RunSummary, RunStatus } from '../types/history';
import { formatElapsedTime } from '../types/history';

interface CompletionSummaryProps {
    status: RunStatus;
    summary: RunSummary;
    elapsedSeconds: number | null;
    dryRun?: boolean;
    mergedBranches?: string[];
    error?: string | null;
    onClose?: () => void;
    onViewHistory?: () => void;
}

/**
 * Success Summary - All batches completed
 */
function SuccessSummary({
    summary,
    elapsedSeconds,
    dryRun,
    mergedBranches = []
}: {
    summary: RunSummary;
    elapsedSeconds: number | null;
    dryRun?: boolean;
    mergedBranches?: string[];
}) {
    return (
        <div className="bg-gradient-to-br from-emerald-900/30 to-emerald-800/10 border border-emerald-500/40 rounded-xl p-6">
            <div className="flex items-center gap-4 mb-4">
                <div className="w-14 h-14 rounded-full bg-emerald-500/20 flex items-center justify-center">
                    <Trophy className="w-7 h-7 text-emerald-400" />
                </div>
                <div>
                    <h2 className="text-xl font-bold text-emerald-300">
                        {dryRun ? '🧪 Dry Run Complete' : 'Execution Complete!'}
                    </h2>
                    <p className="text-emerald-400/80 text-sm">
                        All {summary.total_batches} batches completed successfully
                    </p>
                </div>
            </div>

            <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mt-6">
                <StatCard
                    icon={Layers}
                    label="Batches"
                    value={`${summary.completed_batches}/${summary.total_batches}`}
                    color="emerald"
                />
                <StatCard
                    icon={Timer}
                    label="Duration"
                    value={formatElapsedTime(elapsedSeconds)}
                    color="blue"
                />
                <StatCard
                    icon={GitMerge}
                    label="Merged"
                    value={String(summary.branches_merged)}
                    color="purple"
                />
                <StatCard
                    icon={CheckCircle2}
                    label="Tasks"
                    value={String(summary.tasks_completed)}
                    color="teal"
                />
            </div>

            {!dryRun && mergedBranches.length > 0 && (
                <div className="mt-6 p-4 bg-gray-800/50 rounded-lg">
                    <h3 className="text-sm font-medium text-gray-300 mb-2 flex items-center gap-2">
                        <GitMerge className="w-4 h-4" />
                        Merged Branches
                    </h3>
                    <div className="flex flex-wrap gap-2">
                        {mergedBranches.map((branch) => (
                            <span
                                key={branch}
                                className="px-2 py-1 text-xs bg-purple-500/20 text-purple-300 rounded border border-purple-500/30"
                            >
                                {branch}
                            </span>
                        ))}
                    </div>
                </div>
            )}
        </div>
    );
}

/**
 * Partial Success Summary - Some batches failed
 */
function PartialSuccessSummary({
    summary,
    elapsedSeconds,
    error
}: {
    summary: RunSummary;
    elapsedSeconds: number | null;
    error?: string | null;
}) {
    return (
        <div className="bg-gradient-to-br from-orange-900/30 to-orange-800/10 border border-orange-500/40 rounded-xl p-6">
            <div className="flex items-center gap-4 mb-4">
                <div className="w-14 h-14 rounded-full bg-orange-500/20 flex items-center justify-center">
                    <AlertTriangle className="w-7 h-7 text-orange-400" />
                </div>
                <div>
                    <h2 className="text-xl font-bold text-orange-300">Partial Success</h2>
                    <p className="text-orange-400/80 text-sm">
                        {summary.completed_batches} of {summary.total_batches} batches completed
                    </p>
                </div>
            </div>

            <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mt-6">
                <StatCard
                    icon={CheckCircle2}
                    label="Completed"
                    value={String(summary.completed_batches)}
                    color="emerald"
                />
                <StatCard
                    icon={XCircle}
                    label="Failed"
                    value={String(summary.failed_batches)}
                    color="red"
                />
                <StatCard
                    icon={Clock}
                    label="Pending"
                    value={String(summary.pending_batches)}
                    color="gray"
                />
                <StatCard
                    icon={Timer}
                    label="Duration"
                    value={formatElapsedTime(elapsedSeconds)}
                    color="blue"
                />
            </div>

            {error && (
                <div className="mt-6 p-4 bg-red-900/30 rounded-lg border border-red-500/30">
                    <h3 className="text-sm font-medium text-red-300 mb-1">Error</h3>
                    <p className="text-xs text-red-400/80 font-mono">{error}</p>
                </div>
            )}
        </div>
    );
}

/**
 * Failure Summary - Execution failed
 */
function FailureSummary({
    summary,
    elapsedSeconds,
    error
}: {
    summary: RunSummary;
    elapsedSeconds: number | null;
    error?: string | null;
}) {
    return (
        <div className="bg-gradient-to-br from-red-900/30 to-red-800/10 border border-red-500/40 rounded-xl p-6">
            <div className="flex items-center gap-4 mb-4">
                <div className="w-14 h-14 rounded-full bg-red-500/20 flex items-center justify-center">
                    <XCircle className="w-7 h-7 text-red-400" />
                </div>
                <div>
                    <h2 className="text-xl font-bold text-red-300">Execution Failed</h2>
                    <p className="text-red-400/80 text-sm">
                        Execution stopped after {summary.completed_batches} batch(es)
                    </p>
                </div>
            </div>

            <div className="grid grid-cols-3 gap-4 mt-6">
                <StatCard
                    icon={CheckCircle2}
                    label="Completed"
                    value={String(summary.completed_batches)}
                    color="emerald"
                />
                <StatCard
                    icon={XCircle}
                    label="Failed"
                    value={String(summary.failed_batches)}
                    color="red"
                />
                <StatCard
                    icon={Timer}
                    label="Duration"
                    value={formatElapsedTime(elapsedSeconds)}
                    color="blue"
                />
            </div>

            {error && (
                <div className="mt-6 p-4 bg-red-900/30 rounded-lg border border-red-500/30">
                    <h3 className="text-sm font-medium text-red-300 mb-1">Error Details</h3>
                    <p className="text-xs text-red-400/80 font-mono whitespace-pre-wrap">{error}</p>
                </div>
            )}
        </div>
    );
}

/**
 * Stat Card Component
 */
function StatCard({
    icon: Icon,
    label,
    value,
    color
}: {
    icon: React.ElementType;
    label: string;
    value: string;
    color: 'emerald' | 'blue' | 'purple' | 'teal' | 'red' | 'gray' | 'orange';
}) {
    const colorClasses = {
        emerald: 'bg-emerald-900/30 border-emerald-500/30 text-emerald-400',
        blue: 'bg-blue-900/30 border-blue-500/30 text-blue-400',
        purple: 'bg-purple-900/30 border-purple-500/30 text-purple-400',
        teal: 'bg-teal-900/30 border-teal-500/30 text-teal-400',
        red: 'bg-red-900/30 border-red-500/30 text-red-400',
        gray: 'bg-gray-800/50 border-gray-600/30 text-gray-400',
        orange: 'bg-orange-900/30 border-orange-500/30 text-orange-400',
    };

    return (
        <div className={`p-4 rounded-lg border ${colorClasses[color]}`}>
            <div className="flex items-center gap-2 mb-1">
                <Icon className="w-4 h-4 opacity-70" />
                <span className="text-xs opacity-70">{label}</span>
            </div>
            <div className="text-2xl font-bold">{value}</div>
        </div>
    );
}

/**
 * Main Completion Summary Component
 */
export function CompletionSummary({
    status,
    summary,
    elapsedSeconds,
    dryRun = false,
    mergedBranches = [],
    error = null,
    onClose,
    onViewHistory,
}: CompletionSummaryProps) {
    // Determine which summary to show based on status
    const isSuccess = status === 'completed' && summary.failed_batches === 0;
    const isPartialSuccess = status === 'completed' && summary.failed_batches > 0;
    const isFailed = status === 'failed' || status === 'aborted';

    return (
        <div className="relative">
            {/* Close button */}
            {onClose && (
                <button
                    onClick={onClose}
                    className="absolute top-2 right-2 p-1.5 text-gray-400 hover:text-white hover:bg-gray-700/50 rounded transition-colors"
                    title="Close summary"
                >
                    ×
                </button>
            )}

            {isSuccess && (
                <SuccessSummary
                    summary={summary}
                    elapsedSeconds={elapsedSeconds}
                    dryRun={dryRun}
                    mergedBranches={mergedBranches}
                />
            )}

            {isPartialSuccess && (
                <PartialSuccessSummary
                    summary={summary}
                    elapsedSeconds={elapsedSeconds}
                    error={error}
                />
            )}

            {isFailed && (
                <FailureSummary
                    summary={summary}
                    elapsedSeconds={elapsedSeconds}
                    error={error}
                />
            )}

            {/* Action buttons */}
            {onViewHistory && (
                <div className="mt-4 flex justify-end">
                    <button
                        onClick={onViewHistory}
                        className="px-4 py-2 bg-gray-700/50 hover:bg-gray-600/50 text-gray-300 rounded-lg text-sm transition-colors"
                    >
                        View Run History
                    </button>
                </div>
            )}
        </div>
    );
}

export default CompletionSummary;
