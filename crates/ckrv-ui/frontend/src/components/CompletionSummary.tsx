/**
 * @module CompletionSummary
 * @description
 * Summary display component shown when execution runs complete. Shows batch statistics,
 * elapsed time, merged branches, and appropriate feedback based on success/failure status.
 * Supports success, partial success, and failure states with distinct visual styling.
 *
 * @context
 * Rendered in ExecutionRunner after a run completes. Provides clear visual feedback
 * and actionable next steps based on execution outcome.
 *
 * @dependencies
 * - RunSummary, RunStatus: Types from history module
 * - formatElapsedTime: Utility for duration formatting
 * - shadcn/ui components: Card, Badge, Button for consistent UI
 *
 * @example
 * <CompletionSummary
 *   status="completed"
 *   summary={runSummary}
 *   elapsedSeconds={123}
 *   mergedBranches={['feature-a', 'feature-b']}
 * />
 */

// === IMPORTS ===
import {
    CheckCircle2, XCircle, Clock, GitMerge,
    Trophy, AlertTriangle, Layers, Timer
} from 'lucide-react';
import { Card, CardContent } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import type { RunSummary, RunStatus } from '../types/history';
import { formatElapsedTime } from '../types/history';

// ============================================================
// TYPES
// ============================================================

/**
 * Props for CompletionSummary component.
 * Summary display shown when execution runs complete with statistics.
 */
interface CompletionSummaryProps {
    /** Final status of the run (completed, failed, aborted) */
    status: RunStatus;
    /** Summary statistics including batch counts and branch merges */
    summary: RunSummary;
    /** Total elapsed time in seconds, or null if not available */
    elapsedSeconds: number | null;
    /** Whether this was a dry run without actual changes */
    dryRun?: boolean;
    /** List of branch names that were merged during execution */
    mergedBranches?: string[];
    /** Error message if execution failed */
    error?: string | null;
    /** Callback to close the summary panel */
    onClose?: () => void;
    /** Callback to navigate to run history view */
    onViewHistory?: () => void;
}

// ============================================================
// SUB-COMPONENTS
// ============================================================

/**
 * Stat Card Component using shadcn Card
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
        emerald: 'border-success/30 text-success',
        blue: 'border-info/30 text-info',
        purple: 'border-primary/30 text-primary',
        teal: 'border-info/30 text-info',
        red: 'border-destructive/30 text-destructive',
        gray: 'border-border text-muted-foreground',
        orange: 'border-warning/30 text-warning',
    };

    return (
        <Card className={`${colorClasses[color]}`}>
            <CardContent className="p-4">
                <div className="flex items-center gap-2 mb-1">
                    <Icon className="w-4 h-4 opacity-70" />
                    <span className="text-xs opacity-70">{label}</span>
                </div>
                <div className="text-2xl font-bold">{value}</div>
            </CardContent>
        </Card>
    );
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
        <Card className="border-success/40 bg-card">
            <CardContent className="p-6 bg-success/20 rounded-lg">
                <div className="flex items-center gap-4 mb-4">
                    <div className="w-14 h-14 rounded-full bg-success/20 flex items-center justify-center">
                        <Trophy className="w-7 h-7 text-success" />
                    </div>
                    <div>
                        <h2 className="text-xl font-bold text-success">
                            {dryRun ? '🧪 Dry Run Complete' : 'Execution Complete!'}
                        </h2>
                        <p className="text-success/80 text-sm">
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
                    <div className="mt-6 p-4 bg-muted/50 rounded-lg">
                        <h3 className="text-sm font-medium text-foreground mb-2 flex items-center gap-2">
                            <GitMerge className="w-4 h-4" />
                            Merged Branches
                        </h3>
                        <div className="flex flex-wrap gap-2">
                            {mergedBranches.map((branch) => (
                                <Badge key={branch} variant="info">
                                    {branch}
                                </Badge>
                            ))}
                        </div>
                    </div>
                )}
            </CardContent>
        </Card>
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
        <Card className="border-warning/40 bg-card">
            <CardContent className="p-6 bg-warning/20 rounded-lg">
                <div className="flex items-center gap-4 mb-4">
                    <div className="w-14 h-14 rounded-full bg-warning/20 flex items-center justify-center">
                        <AlertTriangle className="w-7 h-7 text-warning" />
                    </div>
                    <div>
                        <h2 className="text-xl font-bold text-warning">Partial Success</h2>
                        <p className="text-warning/80 text-sm">
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
                    <div className="mt-6 p-4 bg-destructive/10 rounded-lg border border-destructive/30">
                        <h3 className="text-sm font-medium text-destructive mb-1">Error</h3>
                        <p className="text-xs text-destructive/80 font-mono">{error}</p>
                    </div>
                )}
            </CardContent>
        </Card>
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
        <Card className="border-destructive/40 bg-card">
            <CardContent className="p-6 bg-destructive/10 rounded-lg">
                <div className="flex items-center gap-4 mb-4">
                    <div className="w-14 h-14 rounded-full bg-destructive/20 flex items-center justify-center">
                        <XCircle className="w-7 h-7 text-destructive" />
                    </div>
                    <div>
                        <h2 className="text-xl font-bold text-destructive">Execution Failed</h2>
                        <p className="text-destructive/80 text-sm">
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
                    <div className="mt-6 p-4 bg-destructive/10 rounded-lg border border-destructive/30">
                        <h3 className="text-sm font-medium text-destructive mb-1">Error Details</h3>
                        <p className="text-xs text-destructive/80 font-mono whitespace-pre-wrap">{error}</p>
                    </div>
                )}
            </CardContent>
        </Card>
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
                <Button
                    variant="ghost"
                    size="icon"
                    onClick={onClose}
                    className="absolute top-2 right-2"
                    title="Close summary"
                >
                    ×
                </Button>
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
                    <Button variant="outline" onClick={onViewHistory}>
                        View Run History
                    </Button>
                </div>
            )}
        </div>
    );
}

export default CompletionSummary;
