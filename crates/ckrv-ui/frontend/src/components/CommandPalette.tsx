import React, { useState, createContext, useContext } from 'react';
import { useMutation, useQueryClient, useQuery } from '@tanstack/react-query';
import {
    Play, FileText, GitBranch, Rocket, Terminal,
    ChevronRight, Loader2, X, Sparkles,
    GitCompare, ShieldCheck, ExternalLink, ClipboardList
} from 'lucide-react';

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
    const { data: status } = useQuery<SystemStatus>({
        queryKey: ['status'],
        queryFn: async () => {
            const res = await fetch('/api/status');
            return res.json();
        },
        refetchInterval: 2000,
    });

    // Fetch specs to check if any exist
    const { data: specsData } = useQuery<SpecsResponse>({
        queryKey: ['specs'],
        queryFn: async () => {
            const res = await fetch('/api/specs');
            return res.json();
        },
        refetchInterval: 3000,
    });

    // Fetch tasks to check if any exist
    const { data: tasksData } = useQuery<TasksResponse>({
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
        mutationFn: () => runCommand('run', { dry_run: true }),
        onSuccess: (data) => {
            setLastResult({ command: 'plan', result: data });
            queryClient.invalidateQueries({ queryKey: ['tasks'] });
            queryClient.invalidateQueries({ queryKey: ['specs'] });
        },
        onError: () => {
            setLastResult({ command: 'plan', result: { success: false, message: 'Failed to run plan (dry-run)' } });
        }
    });

    const runMutation = useMutation({
        mutationFn: () => runCommand('run'),
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
            disabled: isInitialized, // Disable if already initialized
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
            disabled: !isInitialized, // Enable only after initialization
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
            disabled: !isInitialized || !hasSpecsWithoutTasks, // Enable only if initialized and has specs without tasks
            color: 'amber' as const,
        },
        {
            id: 'plan',
            icon: <ClipboardList size={16} />,
            label: 'Plan',
            description: `Preview execution without running agents`,
            command: 'ckrv run --dry-run',
            action: () => planMutation.mutate(),
            loading: planMutation.isPending,
            disabled: !isInitialized || !hasTasks || hasImplementation,
            color: 'cyan' as const,
        },
        {
            id: 'run',
            icon: <Rocket size={16} />,
            label: 'Run',
            description: `Execute tasks with AI agents${pendingTasks > 0 ? ` (${pendingTasks} pending)` : ''}`,
            command: 'ckrv run',
            action: () => runMutation.mutate(),
            loading: runMutation.isPending,
            disabled: !isInitialized || !hasTasks || hasImplementation,
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
            disabled: !hasImplementation, // Enable after implementation complete
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
            disabled: !hasImplementation, // Enable after implementation complete
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
            disabled: !hasImplementation, // Enable after implementation complete
            color: 'green' as const,
        },
    ];

    return (
        <>
            <div
                className="rounded-lg overflow-hidden flex flex-col"
                style={{
                    background: 'var(--bg-secondary)',
                    border: '1px solid var(--border-subtle)',
                    maxHeight: '500px',
                }}
            >
                {/* Header */}
                <div
                    className="px-4 py-3 flex items-center justify-between shrink-0"
                    style={{ borderBottom: '1px solid var(--border-subtle)' }}
                >
                    <div className="flex items-center gap-2">
                        <Terminal size={16} style={{ color: 'var(--accent-cyan)' }} />
                        <h3
                            className="font-semibold text-sm"
                            style={{ color: 'var(--text-primary)' }}
                        >
                            Commands
                        </h3>
                    </div>
                </div>

                {/* Command List */}
                <div className="p-2 space-y-1 flex-1 overflow-y-auto overflow-x-hidden custom-scrollbar min-h-0">
                    {commands.map((cmd) => (
                        <CommandButton
                            key={cmd.id}
                            {...cmd}
                        />
                    ))}
                </div>

                {/* Terminal Hint */}
                <div
                    className="px-4 py-2 text-xs shrink-0 truncate"
                    style={{
                        background: 'var(--bg-tertiary)',
                        color: 'var(--text-muted)',
                        borderTop: '1px solid var(--border-subtle)'
                    }}
                >
                    CLI: <code className="font-mono" style={{ color: 'var(--accent-cyan)' }}>ckrv --help</code>
                </div>
            </div>

            {/* New Spec Modal */}
            {showSpecModal && (
                <SpecNewModal
                    onClose={() => setShowSpecModal(false)}
                    onSubmit={(description) => specNewMutation.mutate({ description })}
                    isLoading={specNewMutation.isPending}
                />
            )}
        </>
    );
};

interface SpecNewModalProps {
    onClose: () => void;
    onSubmit: (description: string) => void;
    isLoading: boolean;
}

const SpecNewModal: React.FC<SpecNewModalProps> = ({ onClose, onSubmit, isLoading }) => {
    const [description, setDescription] = useState('');

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        if (description.trim()) {
            onSubmit(description.trim());
        }
    };

    return (
        <div
            className="fixed inset-0 flex items-center justify-center z-50"
            style={{ background: 'rgba(0, 0, 0, 0.7)' }}
            onClick={onClose}
        >
            <div
                className="w-full max-w-lg mx-4 rounded-xl overflow-hidden shadow-2xl"
                style={{
                    background: 'var(--bg-secondary)',
                    border: '1px solid var(--border-default)'
                }}
                onClick={e => e.stopPropagation()}
            >
                {/* Modal Header */}
                <div
                    className="px-6 py-4 flex items-center justify-between"
                    style={{ borderBottom: '1px solid var(--border-subtle)' }}
                >
                    <div className="flex items-center gap-3">
                        <div
                            className="p-2 rounded-lg"
                            style={{ background: 'var(--accent-green-dim)' }}
                        >
                            <Sparkles size={20} style={{ color: 'var(--accent-green)' }} />
                        </div>
                        <div>
                            <h2
                                className="font-semibold text-lg"
                                style={{ color: 'var(--text-primary)' }}
                            >
                                New Specification
                            </h2>
                            <p
                                className="text-xs"
                                style={{ color: 'var(--text-muted)' }}
                            >
                                Describe your feature and AI will generate a spec
                            </p>
                        </div>
                    </div>
                    <button
                        onClick={onClose}
                        className="p-2 rounded-lg transition-colors hover:bg-opacity-50"
                        style={{ color: 'var(--text-muted)' }}
                    >
                        <X size={20} />
                    </button>
                </div>

                {/* Modal Body */}
                <form onSubmit={handleSubmit} className="p-6 space-y-4">
                    {/* Description Input */}
                    <div className="space-y-2">
                        <label
                            className="text-sm font-medium flex items-center gap-2"
                            style={{ color: 'var(--text-primary)' }}
                        >
                            Description
                            <span
                                className="text-xs font-normal px-1.5 py-0.5 rounded"
                                style={{
                                    background: 'var(--accent-amber-dim)',
                                    color: 'var(--accent-amber)'
                                }}
                            >
                                required
                            </span>
                        </label>
                        <textarea
                            value={description}
                            onChange={(e) => setDescription(e.target.value)}
                            placeholder="e.g., Add user authentication with OAuth2 support"
                            rows={3}
                            className="w-full px-4 py-3 rounded-lg text-sm resize-none focus:outline-none transition-all"
                            style={{
                                background: 'var(--bg-tertiary)',
                                border: '1px solid var(--border-subtle)',
                                color: 'var(--text-primary)',
                            }}
                            onFocus={(e) => {
                                e.currentTarget.style.borderColor = 'var(--accent-green)';
                                e.currentTarget.style.boxShadow = '0 0 0 3px var(--accent-green-dim)';
                            }}
                            onBlur={(e) => {
                                e.currentTarget.style.borderColor = 'var(--border-subtle)';
                                e.currentTarget.style.boxShadow = 'none';
                            }}
                            autoFocus
                        />
                        <p
                            className="text-xs"
                            style={{ color: 'var(--text-muted)' }}
                        >
                            Describe what feature you want to build. Be specific about requirements.
                        </p>
                    </div>

                    {/* Actions */}
                    <div className="flex items-center justify-end gap-3 pt-2">
                        <button
                            type="button"
                            onClick={onClose}
                            className="px-4 py-2 rounded-lg text-sm font-medium transition-all"
                            style={{
                                color: 'var(--text-secondary)',
                                background: 'var(--bg-tertiary)',
                                border: '1px solid var(--border-subtle)'
                            }}
                        >
                            Cancel
                        </button>
                        <button
                            type="submit"
                            disabled={!description.trim() || isLoading}
                            className="px-4 py-2 rounded-lg text-sm font-medium flex items-center gap-2 transition-all"
                            style={{
                                background: description.trim() && !isLoading
                                    ? 'var(--accent-green)'
                                    : 'var(--bg-tertiary)',
                                color: description.trim() && !isLoading
                                    ? 'var(--bg-primary)'
                                    : 'var(--text-muted)',
                                border: '1px solid transparent',
                                cursor: !description.trim() || isLoading ? 'not-allowed' : 'pointer',
                                opacity: !description.trim() || isLoading ? 0.6 : 1,
                            }}
                        >
                            {isLoading ? (
                                <>
                                    <Loader2 size={16} className="animate-spin" />
                                    Creating...
                                </>
                            ) : (
                                <>
                                    <Sparkles size={16} />
                                    Create Specification
                                </>
                            )}
                        </button>
                    </div>
                </form>

                {/* Footer hint */}
                <div
                    className="px-6 py-3 text-xs font-mono"
                    style={{
                        background: 'var(--bg-tertiary)',
                        color: 'var(--text-muted)',
                        borderTop: '1px solid var(--border-subtle)'
                    }}
                >
                    <span style={{ color: 'var(--accent-cyan)' }}>$</span> ckrv spec new "{description || '...'}"
                </div>
            </div>
        </div>
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
    const colorMap = {
        cyan: { accent: 'var(--accent-cyan)', dim: 'var(--accent-cyan-dim)' },
        green: { accent: 'var(--accent-green)', dim: 'var(--accent-green-dim)' },
        amber: { accent: 'var(--accent-amber)', dim: 'var(--accent-amber-dim)' },
        purple: { accent: 'var(--accent-purple)', dim: 'var(--accent-purple-dim)' },
    };

    const colors = colorMap[color];

    return (
        <button
            onClick={action}
            disabled={disabled || loading}
            className="w-full p-3 rounded-lg flex items-center gap-3 transition-all duration-200 group text-left relative"
            style={{
                background: 'var(--bg-tertiary)',
                border: '1px solid var(--border-subtle)',
                opacity: disabled ? 0.5 : 1,
                cursor: disabled ? 'not-allowed' : 'pointer',
            }}
            onMouseEnter={(e) => {
                if (!disabled) {
                    e.currentTarget.style.borderColor = colors.accent;
                    e.currentTarget.style.boxShadow = `0 0 20px ${colors.dim}`;
                }
            }}
            onMouseLeave={(e) => {
                e.currentTarget.style.borderColor = 'var(--border-subtle)';
                e.currentTarget.style.boxShadow = 'none';
            }}
            title={command}
        >
            {/* Icon */}
            <div
                className="p-2 rounded-lg transition-all shrink-0"
                style={{
                    background: colors.dim,
                    color: colors.accent
                }}
            >
                {loading ? <Loader2 size={16} className="animate-spin" /> : icon}
            </div>

            {/* Content */}
            <div className="flex-1 min-w-0 overflow-hidden">
                <div
                    className="font-medium text-sm truncate"
                    style={{ color: 'var(--text-primary)' }}
                >
                    {label}
                </div>
                <div
                    className="text-xs truncate"
                    style={{ color: 'var(--text-muted)' }}
                >
                    {description}
                </div>
            </div>

            {/* Arrow */}
            <div className="shrink-0">
                <ChevronRight
                    size={14}
                    className="transition-transform group-hover:translate-x-0.5"
                    style={{ color: disabled ? 'var(--text-muted)' : colors.accent }}
                />
            </div>
        </button>
    );
};
