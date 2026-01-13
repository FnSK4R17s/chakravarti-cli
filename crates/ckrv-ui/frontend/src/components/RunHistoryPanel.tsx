/**
 * Run History Panel Component
 * 
 * Displays a list of past runs for a specification with status indicators.
 */

import {
    Clock, CheckCircle2, XCircle, Loader2,
    Slash, Circle, ChevronRight, Trash2, History
} from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Skeleton } from '@/components/ui/skeleton';
import { Alert, AlertDescription } from '@/components/ui/alert';
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
            return <CheckCircle2 className="w-4 h-4 text-accent-green" />;
        case 'failed':
            return <XCircle className="w-4 h-4 text-destructive" />;
        case 'running':
            return <Loader2 className="w-4 h-4 text-accent-cyan animate-spin" />;
        case 'aborted':
            return <Slash className="w-4 h-4 text-accent-amber" />;
        case 'pending':
        default:
            return <Circle className="w-4 h-4 text-muted-foreground" />;
    }
}

/**
 * Get status badge using shadcn Badge
 */
function StatusLabel({ status }: { status: RunStatus }) {
    const variants: Record<RunStatus, "success" | "destructive" | "info" | "warning" | "secondary"> = {
        completed: 'success',
        failed: 'destructive',
        running: 'info',
        aborted: 'warning',
        pending: 'secondary',
    };

    return (
        <Badge variant={variants[status]}>
            {status.charAt(0).toUpperCase() + status.slice(1)}
        </Badge>
    );
}

/**
 * Single run item using Card
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
        <Card
            className={`
                group cursor-pointer transition-all
                ${isSelected
                    ? 'ring-1 ring-primary border-primary/50'
                    : 'hover:border-muted-foreground/50 hover:bg-accent/50'}
            `}
            onClick={onClick}
        >
            <CardContent className="p-3">
                <div className="flex items-start justify-between gap-2">
                    <div className="flex items-center gap-2 min-w-0">
                        <StatusIcon status={run.status} />
                        <div className="min-w-0">
                            <div className="flex items-center gap-2">
                                <span className="text-sm font-medium text-foreground truncate">
                                    {run.dry_run ? '🧪 Dry Run' : 'Run'}
                                </span>
                                <StatusLabel status={run.status} />
                            </div>
                            <p className="text-xs text-muted-foreground mt-0.5">
                                {formatRelativeTime(run.started_at)}
                            </p>
                        </div>
                    </div>

                    <div className="flex items-center gap-2">
                        {onDelete && run.status !== 'running' && (
                            <Button
                                variant="ghost"
                                size="icon"
                                className="h-6 w-6 opacity-0 group-hover:opacity-100 transition-opacity"
                                onClick={(e) => {
                                    e.stopPropagation();
                                    onDelete();
                                }}
                                title="Delete run"
                            >
                                <Trash2 className="w-3.5 h-3.5 text-destructive" />
                            </Button>
                        )}
                        <ChevronRight className="w-4 h-4 text-muted-foreground" />
                    </div>
                </div>

                {/* Summary stats */}
                <div className="flex items-center gap-4 mt-2 text-xs text-muted-foreground">
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
                        <Badge variant="success" className="text-xs">
                            {run.summary.branches_merged} merged
                        </Badge>
                    )}
                </div>
            </CardContent>
        </Card>
    );
}

/**
 * Empty state when no runs exist
 */
function EmptyState() {
    return (
        <div className="flex flex-col items-center justify-center py-8 text-center">
            <div className="w-12 h-12 rounded-full bg-muted flex items-center justify-center mb-3">
                <History className="w-6 h-6 text-muted-foreground" />
            </div>
            <h3 className="text-sm font-medium text-foreground mb-1">No run history</h3>
            <p className="text-xs text-muted-foreground max-w-[200px]">
                Run history will appear here after you execute your first run.
            </p>
        </div>
    );
}

/**
 * Loading skeleton using shadcn Skeleton
 */
function LoadingSkeleton() {
    return (
        <div className="space-y-2">
            {[1, 2, 3].map((i) => (
                <Card key={i}>
                    <CardContent className="p-3">
                        <div className="flex items-center gap-2">
                            <Skeleton className="w-4 h-4 rounded-full" />
                            <div className="flex-1">
                                <Skeleton className="h-4 w-24" />
                                <Skeleton className="h-3 w-16 mt-1" />
                            </div>
                        </div>
                    </CardContent>
                </Card>
            ))}
        </div>
    );
}

/**
 * Run History Panel using shadcn Card and ScrollArea
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
            <Card>
                <CardHeader className="py-3">
                    <CardTitle className="text-sm flex items-center gap-2">
                        <History className="w-4 h-4" />
                        Run History
                    </CardTitle>
                </CardHeader>
                <CardContent>
                    <LoadingSkeleton />
                </CardContent>
            </Card>
        );
    }

    if (error) {
        return (
            <Card>
                <CardHeader className="py-3">
                    <CardTitle className="text-sm flex items-center gap-2">
                        <History className="w-4 h-4" />
                        Run History
                    </CardTitle>
                </CardHeader>
                <CardContent>
                    <Alert variant="destructive">
                        <AlertDescription>
                            Failed to load history: {error}
                        </AlertDescription>
                    </Alert>
                </CardContent>
            </Card>
        );
    }

    return (
        <Card>
            <CardHeader className="py-3">
                <CardTitle className="text-sm flex items-center gap-2">
                    <History className="w-4 h-4" />
                    Run History
                    {runs.length > 0 && (
                        <Badge variant="secondary" className="text-xs">
                            {runs.length}
                        </Badge>
                    )}
                </CardTitle>
            </CardHeader>
            <CardContent>
                {runs.length === 0 ? (
                    <EmptyState />
                ) : (
                    <ScrollArea className="max-h-[400px]">
                        <div className="space-y-2 pr-2">
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
                    </ScrollArea>
                )}
            </CardContent>
        </Card>
    );
}

export default RunHistoryPanel;
