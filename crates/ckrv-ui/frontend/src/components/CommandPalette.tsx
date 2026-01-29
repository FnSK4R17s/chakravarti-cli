import React, { useState, createContext, useContext } from 'react';
import { useMutation, useQueryClient, useQuery } from '@tanstack/react-query';
import {
    Play, Terminal,
    ChevronRight, Loader2, Sparkles
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

    // Derive workflow state
    const isInitialized = status?.is_ready ?? false;

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
    ];

    const isLoading = isLoadingStatus;

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

export interface SpecNewDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    onSubmit: (description: string) => void;
    isLoading: boolean;
}

export const SpecNewDialog: React.FC<SpecNewDialogProps> = ({ open, onOpenChange, onSubmit, isLoading }) => {
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
                        <div className="p-2 rounded-lg bg-accent-green-dim">
                            <Sparkles size={20} className="text-accent-green" />
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
        cyan: 'bg-accent-cyan-dim text-accent-cyan hover:border-accent-cyan hover:glow-cyan',
        green: 'bg-accent-green-dim text-accent-green hover:border-accent-green hover:glow-green',
        amber: 'bg-accent-amber-dim text-accent-amber hover:border-accent-amber hover:glow-amber',
        purple: 'bg-accent-purple-dim text-accent-purple hover:border-accent-purple hover:glow-purple',
    };

    const arrowColors = {
        cyan: 'text-accent-cyan',
        green: 'text-accent-green',
        amber: 'text-accent-amber',
        purple: 'text-accent-purple',
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
