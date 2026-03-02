/**
 * @module CommandPalette
 * @description
 * Command interface providing quick actions for the chakravarti CLI workflow.
 * Displays available commands based on current workflow state with a context
 * for sharing command results with other components.
 *
 * @context
 * Rendered in the dashboard sidebar or main area. Provides initialization and
 * spec creation actions. Commands are automatically enabled/disabled based on
 * workflow state.
 *
 * @dependencies
 * - useQuery/useMutation: React Query for command execution
 * - SpecNewDialog: Modal for creating new specifications
 * - shadcn/ui components: Card, Button, Dialog for consistent UI
 *
 * @example
 * <CommandPalette />
 *
 * // Use with context to share command results
 * const { lastResult } = useCommandResult();
 */

// === IMPORTS ===
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

// ============================================================
// CONTEXT
// ============================================================

// Context for sharing command results with LogViewer
interface CommandResultContextType {
    lastResult: { command: string; result: { success: boolean; message?: string } } | null;
    setLastResult: (result: { command: string; result: { success: boolean; message?: string } } | null) => void;
}

// eslint-disable-next-line react-refresh/only-export-components
export const CommandResultContext = createContext<CommandResultContextType>({
    lastResult: null,
    setLastResult: () => { },
});

// eslint-disable-next-line react-refresh/only-export-components
export const useCommandResult = () => useContext(CommandResultContext);

// ============================================================
// TYPES
// ============================================================

interface CommandResult {
    success: boolean;
    message?: string;
}

interface SystemStatus {
    is_ready: boolean;
    active_branch: string;
    mode: string;
}

// ============================================================
// MAIN COMPONENT
// ============================================================

export const CommandPalette: React.FC = () => {
    const queryClient = useQueryClient();
    const { setLastResult } = useCommandResult();
    /** Whether the new spec modal is visible */
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

// ============================================================
// SUB-COMPONENTS
// ============================================================

/**
 * Props for SpecNewDialog component.
 * Modal dialog for creating new specifications via AI generation.
 */
export interface SpecNewDialogProps {
    /** Whether the dialog is currently open */
    open: boolean;
    /** Callback fired when dialog open state changes */
    onOpenChange: (open: boolean) => void;
    /** Callback fired when user submits the description form */
    onSubmit: (description: string) => void;
    /** Whether the spec creation is in progress */
    isLoading: boolean;
}

export const SpecNewDialog: React.FC<SpecNewDialogProps> = ({ open, onOpenChange, onSubmit, isLoading }) => {
    /** Feature description input text */
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
                        <div className="p-2 rounded-lg bg-success/20">
                            <Sparkles size={20} className="text-success" />
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

/**
 * Props for CommandButton component.
 * Renders a clickable command action button with icon and description.
 */
interface CommandButtonProps {
    /** Icon element displayed in the button */
    icon: React.ReactNode;
    /** Button label text */
    label: string;
    /** Description shown below the label */
    description: string;
    /** CLI command string shown in tooltip */
    command: string;
    /** Click handler for executing the command */
    action?: () => void;
    /** Whether the button is disabled */
    disabled?: boolean;
    /** Whether the command is currently executing */
    loading?: boolean;
    /** Color theme for the button icon */
    color: 'cyan' | 'green' | 'amber' | 'purple';
}

const CommandButton: React.FC<CommandButtonProps> = ({
    icon, label, description, command, action, disabled, loading, color
}) => {
    const colorClasses = {
        cyan: 'bg-info/20 text-info hover:border-info',
        green: 'bg-success/20 text-success hover:border-success',
        amber: 'bg-warning/20 text-warning hover:border-warning',
        purple: 'bg-primary/20 text-primary hover:border-primary',
    };

    const arrowColors = {
        cyan: 'text-info',
        green: 'text-success',
        amber: 'text-warning',
        purple: 'text-primary',
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
