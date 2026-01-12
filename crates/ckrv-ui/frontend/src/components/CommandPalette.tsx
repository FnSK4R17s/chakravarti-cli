import React, { useState, createContext, useContext } from 'react';
import { useMutation, useQueryClient, useQuery } from '@tanstack/react-query';
import {
    Play, FileText, GitBranch, Rocket, Terminal,
    ChevronRight, Loader2, Sparkles,
    GitCompare, ShieldCheck, ExternalLink, ClipboardList
} from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Skeleton } from '@/components/ui/skeleton';
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog';
import { cn } from '@/lib/utils';

// Context for sharing command results with LogViewer
interface CommandResultContextType {
    lastResult: { command: string; result: { success: boolean; message?: string } } | null;
    setLastResult: (result: { command: string; result: { success: boolean; message?: string } } | null) => void;
}

export const CommandResultContext = createContext<CommandResultContextType>({
    lastResult: null,
    setLastResult: () => { },
});

export const useCommandResult = () => useContext(CommandResultContext);

interface CommandResult {
    success: boolean;
    message?: string;
}

interface SystemStatus {
    is_ready: boolean;
    active_branch: string;
    mode: string;
}

interface Spec {
    name: string;
    path: string;
    has_tasks: boolean;
    has_plan: boolean;
    has_implementation: boolean;
    implementation_branch: string | null;
}

interface SpecsResponse {
    specs: Spec[];
    count: number;
}

interface Task {
    id: string;
    status: string;
}

interface TasksResponse {
    tasks: Task[];
    spec_id: string;
}

export const CommandPalette: React.FC = () => {
    const queryClient = useQueryClient();
    const { setLastResult } = useCommandResult();
    const [showSpecModal, setShowSpecModal] = useState(false);

    // Fetch status to determine workflow state
    const { data: status, isLoading: isLoadingStatus } = useQuery<SystemStatus>({
        queryKey: ['status'],
        queryFn: async () => {
            const res = await fetch('/api/status');
            return res.json();
        },
        refetchInterval: 2000,
    });

    // Fetch specs to check if any exist
    const { data: specsData, isLoading: isLoadingSpecs } = useQuery<SpecsResponse>({
        queryKey: ['specs'],
        queryFn: async () => {
            const res = await fetch('/api/specs');
            return res.json();
        },
        refetchInterval: 3000,
    });

    // Fetch tasks to check if any exist
    const { data: tasksData, isLoading: isLoadingTasks } = useQuery<TasksResponse>({
        queryKey: ['tasks'],
        queryFn: async () => {
            const res = await fetch('/api/tasks');
            return res.json();
        },
        refetchInterval: 3000,
    });

    // Derive workflow state
    const isInitialized = status?.is_ready ?? false;
    const specs = specsData?.specs ?? [];
    const specsWithoutTasks = specs.filter(s => !s.has_tasks);
    const hasSpecsWithoutTasks = specsWithoutTasks.length > 0;
    const tasks = tasksData?.tasks ?? [];
    const hasTasks = tasks.length > 0;
    const pendingTasks = tasks.filter(t => t.status !== 'completed').length;

    // Check if any spec has completed implementation (code merged and ready for review)
    const hasImplementation = specs.some(s => s.has_implementation);

    // Check if any spec already has a plan
    const hasPlan = specs.some(s => s.has_plan);

    const runCommand = async (endpoint: string, body?: object): Promise<CommandResult> => {
        const res = await fetch(`/api/command/${endpoint}`, {
            method: 'POST',
            headers: body ? { 'Content-Type': 'application/json' } : undefined,
            body: body ? JSON.stringify(body) : undefined,
        });
        if (!res.ok) throw new Error('Command failed');
        return res.json();
    };

    const initMutation = useMutation({
        mutationFn: () => runCommand('init'),
        onSuccess: (data) => {
            setLastResult({ command: 'init', result: data });
            queryClient.invalidateQueries({ queryKey: ['status'] });
        },
        onError: () => {
            setLastResult({ command: 'init', result: { success: false, message: 'Failed to initialize' } });
        }
    });

    const specNewMutation = useMutation({
        mutationFn: (params: { description: string; name?: string }) =>
            runCommand('spec-new', params),
        onSuccess: (data) => {
            setLastResult({ command: 'spec-new', result: data });
            setShowSpecModal(false);
            queryClient.invalidateQueries({ queryKey: ['status'] });
            queryClient.invalidateQueries({ queryKey: ['specs'] });
        },
        onError: () => {
            setLastResult({ command: 'spec-new', result: { success: false, message: 'Failed to create specification' } });
        }
    });

    const specTasksMutation = useMutation({
        mutationFn: () => runCommand('spec-tasks'),
        onSuccess: (data) => {
            setLastResult({ command: 'spec-tasks', result: data });
            queryClient.invalidateQueries({ queryKey: ['tasks'] });
            queryClient.invalidateQueries({ queryKey: ['specs'] });
        },
        onError: () => {
            setLastResult({ command: 'spec-tasks', result: { success: false, message: 'Failed to generate tasks' } });
        }
    });

    const planMutation = useMutation({
        mutationFn: () => runCommand('plan'),
        onSuccess: (data) => {
            setLastResult({ command: 'plan', result: data });
            queryClient.invalidateQueries({ queryKey: ['tasks'] });
            queryClient.invalidateQueries({ queryKey: ['specs'] });
        },
        onError: () => {
            setLastResult({ command: 'plan', result: { success: false, message: 'Failed to generate plan' } });
        }
    });

    const runMutation = useMutation({
        mutationFn: () => runCommand('execute'),
        onSuccess: (data) => {
            setLastResult({ command: 'run', result: data });
            queryClient.invalidateQueries({ queryKey: ['tasks'] });
            queryClient.invalidateQueries({ queryKey: ['status'] });
            queryClient.invalidateQueries({ queryKey: ['specs'] });
        },
        onError: () => {
            setLastResult({ command: 'run', result: { success: false, message: 'Failed to execute tasks' } });
        }
    });

    const diffMutation = useMutation({
        mutationFn: () => runCommand('diff', { stat: true, files: true }),
        onSuccess: (data) => {
            setLastResult({ command: 'diff', result: data });
        },
        onError: () => {
            setLastResult({ command: 'diff', result: { success: false, message: 'Failed to get diff' } });
        }
    });

    const verifyMutation = useMutation({
        mutationFn: () => runCommand('verify', {}),
        onSuccess: (data) => {
            setLastResult({ command: 'verify', result: data });
            queryClient.invalidateQueries({ queryKey: ['specs'] });
        },
        onError: () => {
            setLastResult({ command: 'verify', result: { success: false, message: 'Verification failed' } });
        }
    });

    const promoteMutation = useMutation({
        mutationFn: () => runCommand('promote', { push: true }),
        onSuccess: (data) => {
            setLastResult({ command: 'promote', result: data });
            queryClient.invalidateQueries({ queryKey: ['specs'] });
        },
        onError: () => {
            setLastResult({ command: 'promote', result: { success: false, message: 'Failed to create PR' } });
        }
    });

    const commands = [
        {
            id: 'init',
            icon: <Play size={16} />,
            label: 'Initialize',
            description: 'Setup .specs and .chakravarti directories',
            command: 'ckrv init',
            action: () => initMutation.mutate(),
            loading: initMutation.isPending,
            disabled: isInitialized,
            color: 'cyan' as const,
        },
        {
            id: 'spec-new',
            icon: <FileText size={16} />,
            label: 'New Spec',
            description: 'Create AI-generated specification',
            command: 'ckrv spec new "..."',
            action: () => setShowSpecModal(true),
            loading: specNewMutation.isPending,
            disabled: !isInitialized,
            color: 'green' as const,
        },
        {
            id: 'spec-tasks',
            icon: <GitBranch size={16} />,
            label: 'Generate Tasks',
            description: 'Create implementation tasks from spec',
            command: 'ckrv spec tasks',
            action: () => specTasksMutation.mutate(),
            loading: specTasksMutation.isPending,
            disabled: !isInitialized || !hasSpecsWithoutTasks,
            color: 'amber' as const,
        },
        {
            id: 'plan',
            icon: <ClipboardList size={16} />,
            label: hasPlan ? 'Plan Exists' : 'Plan',
            description: hasPlan ? 'Execution plan already generated' : 'Generate execution plan in Docker',
            command: 'ckrv plan',
            action: () => planMutation.mutate(),
            loading: planMutation.isPending,
            disabled: !isInitialized || !hasTasks || hasImplementation || hasPlan,
            color: 'cyan' as const,
        },
        {
            id: 'run',
            icon: <Rocket size={16} />,
            label: 'Run',
            description: hasPlan
                ? `Execute batches in Docker${pendingTasks > 0 ? ` (${pendingTasks} pending)` : ''}`
                : 'Generate a plan first',
            command: 'ckrv run',
            action: () => runMutation.mutate(),
            loading: runMutation.isPending,
            disabled: !isInitialized || !hasTasks || hasImplementation || !hasPlan,
            color: 'purple' as const,
        },
        {
            id: 'diff',
            icon: <GitCompare size={16} />,
            label: 'View Diff',
            description: 'Review changes between branches',
            command: 'ckrv diff',
            action: () => diffMutation.mutate(),
            loading: diffMutation.isPending,
            disabled: !hasImplementation,
            color: 'cyan' as const,
        },
        {
            id: 'verify',
            icon: <ShieldCheck size={16} />,
            label: 'Verify',
            description: 'Run tests, lint, and type checks',
            command: 'ckrv verify',
            action: () => verifyMutation.mutate(),
            loading: verifyMutation.isPending,
            disabled: !hasImplementation,
            color: 'amber' as const,
        },
        {
            id: 'promote',
            icon: <ExternalLink size={16} />,
            label: 'Create PR',
            description: 'Push and create pull request',
            command: 'ckrv promote --push',
            action: () => promoteMutation.mutate(),
            loading: promoteMutation.isPending,
            disabled: !hasImplementation,
            color: 'green' as const,
        },
    ];

    const isLoading = isLoadingStatus || isLoadingSpecs || isLoadingTasks;

    return (
        <>
            <Card className="flex flex-col flex-1 min-h-0">
                <CardHeader className="pb-3 shrink-0">
                    <div className="flex items-center gap-2">
                        <Terminal size={16} className="text-primary" />
                        <CardTitle className="text-sm font-semibold">Commands</CardTitle>
                    </div>
                </CardHeader>

                <CardContent className="flex-1 overflow-y-auto space-y-1 min-h-0">
                    {isLoading ? (
                        // Loading skeleton
                        <>
                            {[...Array(4)].map((_, i) => (
                                <div key={i} className="flex items-center gap-3 p-3">
                                    <Skeleton className="h-8 w-8 rounded-lg" />
                                    <div className="flex-1 space-y-2">
                                        <Skeleton className="h-4 w-24" />
                                        <Skeleton className="h-3 w-40" />
                                    </div>
                                </div>
                            ))}
                        </>
                    ) : (
                        commands.map((cmd) => (
                            <CommandButton
                                key={cmd.id}
                                {...cmd}
                            />
                        ))
                    )}
                </CardContent>

                {/* Terminal Hint */}
                <div className="px-4 py-2 text-xs shrink-0 truncate border-t border-border bg-muted/50">
                    CLI: <code className="font-mono text-primary">ckrv --help</code>
                </div>
            </Card>

            {/* New Spec Dialog */}
            <SpecNewDialog
                open={showSpecModal}
                onOpenChange={setShowSpecModal}
                onSubmit={(description) => specNewMutation.mutate({ description })}
                isLoading={specNewMutation.isPending}
            />
        </>
    );
};

interface SpecNewDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    onSubmit: (description: string) => void;
    isLoading: boolean;
}

const SpecNewDialog: React.FC<SpecNewDialogProps> = ({ open, onOpenChange, onSubmit, isLoading }) => {
    const [description, setDescription] = useState('');

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        if (description.trim()) {
            onSubmit(description.trim());
        }
    };

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="sm:max-w-lg">
                <DialogHeader>
                    <div className="flex items-center gap-3">
                        <div className="p-2 rounded-lg bg-[var(--accent-green-dim)]">
                            <Sparkles size={20} className="text-[var(--accent-green)]" />
                        </div>
                        <div>
                            <DialogTitle>New Specification</DialogTitle>
                            <DialogDescription>
                                Describe your feature and AI will generate a spec
                            </DialogDescription>
                        </div>
                    </div>
                </DialogHeader>

                <form onSubmit={handleSubmit} className="space-y-4">
                    <div className="space-y-2">
                        <label className="text-sm font-medium flex items-center gap-2">
                            Description
                            <Badge variant="warning" className="text-xs">required</Badge>
                        </label>
                        <textarea
                            value={description}
                            onChange={(e) => setDescription(e.target.value)}
                            placeholder="e.g., Add user authentication with OAuth2 support"
                            rows={3}
                            className="w-full px-4 py-3 rounded-lg text-sm resize-none bg-muted border border-border focus:outline-none focus:ring-2 focus:ring-ring transition-all"
                            autoFocus
                        />
                        <p className="text-xs text-muted-foreground">
                            Describe what feature you want to build. Be specific about requirements.
                        </p>
                    </div>

                    <DialogFooter>
                        <Button
                            type="button"
                            variant="outline"
                            onClick={() => onOpenChange(false)}
                        >
                            Cancel
                        </Button>
                        <Button
                            type="submit"
                            disabled={!description.trim() || isLoading}
                        >
                            {isLoading ? (
                                <>
                                    <Loader2 size={16} className="mr-2 animate-spin" />
                                    Creating...
                                </>
                            ) : (
                                <>
                                    <Sparkles size={16} className="mr-2" />
                                    Create Specification
                                </>
                            )}
                        </Button>
                    </DialogFooter>
                </form>

                {/* Footer hint */}
                <div className="px-4 py-3 -mx-6 -mb-6 text-xs font-mono bg-muted border-t border-border rounded-b-lg">
                    <span className="text-primary">$</span> ckrv spec new "{description || '...'}"
                </div>
            </DialogContent>
        </Dialog>
    );
};

interface CommandButtonProps {
    icon: React.ReactNode;
    label: string;
    description: string;
    command: string;
    action?: () => void;
    disabled?: boolean;
    loading?: boolean;
    color: 'cyan' | 'green' | 'amber' | 'purple';
}

const CommandButton: React.FC<CommandButtonProps> = ({
    icon, label, description, command, action, disabled, loading, color
}) => {
    const colorClasses = {
        cyan: 'bg-[var(--accent-cyan-dim)] text-[var(--accent-cyan)] hover:border-[var(--accent-cyan)] hover:shadow-[0_0_20px_var(--accent-cyan-dim)]',
        green: 'bg-[var(--accent-green-dim)] text-[var(--accent-green)] hover:border-[var(--accent-green)] hover:shadow-[0_0_20px_var(--accent-green-dim)]',
        amber: 'bg-[var(--accent-amber-dim)] text-[var(--accent-amber)] hover:border-[var(--accent-amber)] hover:shadow-[0_0_20px_var(--accent-amber-dim)]',
        purple: 'bg-[var(--accent-purple-dim)] text-[var(--accent-purple)] hover:border-[var(--accent-purple)] hover:shadow-[0_0_20px_var(--accent-purple-dim)]',
    };

    const arrowColors = {
        cyan: 'text-[var(--accent-cyan)]',
        green: 'text-[var(--accent-green)]',
        amber: 'text-[var(--accent-amber)]',
        purple: 'text-[var(--accent-purple)]',
    };

    return (
        <button
            onClick={action}
            disabled={disabled || loading}
            className={cn(
                "w-full p-3 rounded-lg flex items-center gap-3 transition-all duration-200 group text-left",
                "bg-accent border border-border",
                disabled ? "opacity-50 cursor-not-allowed" : "cursor-pointer hover:border-primary"
            )}
            title={command}
        >
            {/* Icon */}
            <div className={cn("p-2 rounded-lg transition-all shrink-0", colorClasses[color].split(' ').slice(0, 2).join(' '))}>
                {loading ? <Loader2 size={16} className="animate-spin" /> : icon}
            </div>

            {/* Content */}
            <div className="flex-1 min-w-0 overflow-hidden">
                <div className="font-medium text-sm truncate text-foreground">
                    {label}
                </div>
                <div className="text-xs truncate text-muted-foreground">
                    {description}
                </div>
            </div>

            {/* Arrow */}
            <div className="shrink-0">
                <ChevronRight
                    size={14}
                    className={cn(
                        "transition-transform group-hover:translate-x-0.5",
                        disabled ? "text-muted-foreground" : arrowColors[color]
                    )}
                />
            </div>
        </button>
    );
};
