import React, { useState } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { type SystemStatus } from '../types';
import { GitBranch, FolderGit, Settings, Loader2, AlertCircle, CheckCircle2, Play, GitBranchPlus } from 'lucide-react';

const fetchStatus = async (): Promise<SystemStatus> => {
    const res = await fetch('/api/status');
    if (!res.ok) throw new Error('Failed to fetch status');
    return res.json();
};

export const StatusWidget: React.FC = () => {
    const queryClient = useQueryClient();
    const [isRunningGitInit, setIsRunningGitInit] = useState(false);
    const [gitInitResult, setGitInitResult] = useState<{ success: boolean; message: string } | null>(null);

    const { data: status, error, isLoading } = useQuery({
        queryKey: ['status'],
        queryFn: fetchStatus,
        refetchInterval: 5000,
    });

    const runGitInit = async () => {
        setIsRunningGitInit(true);
        setGitInitResult(null);
        try {
            const res = await fetch('/api/command/git-init', { method: 'POST' });
            const data = await res.json();
            setGitInitResult(data);
            // Refresh status after git init
            queryClient.invalidateQueries({ queryKey: ['status'] });
        } catch (err) {
            setGitInitResult({ success: false, message: String(err) });
        } finally {
            setIsRunningGitInit(false);
        }
    };

    if (isLoading) return <StatusLoading />;
    if (error || !status) return <StatusError />;

    const getModeLabel = (mode: string) => {
        switch (mode.toLowerCase()) {
            case 'running': return 'Running';
            case 'planning': return 'Planning';
            case 'promoting': return 'Promoting';
            default: return 'Ready';
        }
    };

    const getModeStyle = (mode: string) => {
        switch (mode.toLowerCase()) {
            case 'running': 
                return { 
                    bg: 'var(--accent-green-dim)', 
                    color: 'var(--accent-green)',
                    dot: 'running'
                };
            case 'planning': 
                return { 
                    bg: 'var(--accent-cyan-dim)', 
                    color: 'var(--accent-cyan)',
                    dot: 'planning'
                };
            case 'promoting': 
                return { 
                    bg: 'var(--accent-purple-dim)', 
                    color: 'var(--accent-purple)',
                    dot: 'promoting'
                };
            default: 
                return { 
                    bg: 'var(--bg-surface)', 
                    color: 'var(--text-secondary)',
                    dot: 'idle'
                };
        }
    };

    const modeStyle = getModeStyle(status.mode);
    
    // Check if it's a git repo (if branch is "none" or empty, it's not)
    const isGitRepo = status.active_branch && status.active_branch !== 'none' && status.active_branch !== '';

    return (
        <div 
            className="rounded-lg overflow-hidden"
            style={{ 
                background: 'var(--bg-secondary)',
                border: '1px solid var(--border-subtle)'
            }}
        >
            {/* Header */}
            <div 
                className="px-4 py-3 flex items-center justify-between"
                style={{ borderBottom: '1px solid var(--border-subtle)' }}
            >
                <h3 
                    className="font-semibold text-sm"
                    style={{ color: 'var(--text-primary)' }}
                >
                    Repository Status
                </h3>
                <div 
                    className="flex items-center gap-2 px-2 py-1 rounded-full text-xs font-medium"
                    style={{ 
                        background: modeStyle.bg,
                        color: modeStyle.color
                    }}
                >
                    <div className={`status-dot ${modeStyle.dot}`}></div>
                    {getModeLabel(status.mode)}
                </div>
            </div>

            {/* Content */}
            <div className="p-4 space-y-3">
                {/* Git Repository Status */}
                {!isGitRepo ? (
                    <div className="flex flex-col gap-2">
                        <div className="flex items-center justify-between">
                            <div 
                                className="flex items-center gap-2 text-sm"
                                style={{ color: 'var(--text-muted)' }}
                            >
                                <GitBranchPlus size={16} />
                                <span>Git Repository</span>
                            </div>
                            <div className="flex items-center gap-2">
                                <AlertCircle size={14} style={{ color: 'var(--accent-amber)' }} />
                                <span 
                                    className="text-sm"
                                    style={{ color: 'var(--accent-amber)' }}
                                >
                                    Not initialized
                                </span>
                            </div>
                        </div>
                        <button
                            onClick={runGitInit}
                            disabled={isRunningGitInit}
                            className="flex items-center justify-center gap-2 px-3 py-2 rounded text-sm font-medium transition-all"
                            style={{ 
                                background: isRunningGitInit ? 'var(--bg-surface)' : 'var(--accent-cyan-dim)',
                                color: isRunningGitInit ? 'var(--text-muted)' : 'var(--accent-cyan)',
                                border: '1px solid var(--border-subtle)',
                                cursor: isRunningGitInit ? 'not-allowed' : 'pointer'
                            }}
                        >
                            {isRunningGitInit ? (
                                <>
                                    <Loader2 size={14} className="animate-spin" />
                                    Initializing...
                                </>
                            ) : (
                                <>
                                    <Play size={14} />
                                    Initialize Git Repository
                                </>
                            )}
                        </button>
                        {gitInitResult && (
                            <div 
                                className="text-xs px-2 py-1 rounded"
                                style={{ 
                                    background: gitInitResult.success ? 'var(--accent-green-dim)' : 'var(--accent-red-dim)',
                                    color: gitInitResult.success ? 'var(--accent-green)' : 'var(--accent-red)'
                                }}
                            >
                                {gitInitResult.message}
                            </div>
                        )}
                    </div>
                ) : (
                    <>
                        {/* Initialization Status */}
                        <StatusRow
                            icon={<FolderGit size={16} />}
                            label="Initialized"
                            value={status.is_ready ? 'Yes' : 'No'}
                            status={status.is_ready ? 'success' : 'warning'}
                            hint={!status.is_ready ? 'Run: ckrv init' : undefined}
                        />

                        {/* Branch */}
                        <StatusRow
                            icon={<GitBranch size={16} />}
                            label="Branch"
                            value={status.active_branch}
                            mono
                        />
                    </>
                )}

                {/* Feature */}
                {status.feature_number && (
                    <StatusRow
                        icon={<Settings size={16} />}
                        label="Spec"
                        value={status.feature_number}
                        mono
                    />
                )}
            </div>

            {/* Quick Info Footer */}
            <div 
                className="px-4 py-2 text-xs font-mono truncate"
                style={{ 
                    background: 'var(--bg-tertiary)',
                    color: 'var(--text-muted)',
                    borderTop: '1px solid var(--border-subtle)'
                }}
                title="$ ckrv --version"
            >
                <span style={{ color: 'var(--accent-cyan)' }}>$</span> ckrv <span className="hidden sm:inline">--version</span>
            </div>
        </div>
    );
};

interface StatusRowProps {
    icon: React.ReactNode;
    label: string;
    value: string;
    status?: 'success' | 'warning' | 'error';
    mono?: boolean;
    hint?: string;
}

const StatusRow: React.FC<StatusRowProps> = ({ icon, label, value, status, mono, hint }) => {
    const getStatusColor = () => {
        switch (status) {
            case 'success': return 'var(--accent-green)';
            case 'warning': return 'var(--accent-amber)';
            case 'error': return 'var(--accent-red)';
            default: return 'var(--text-primary)';
        }
    };

    return (
        <div className="flex flex-col gap-1">
            <div className="flex items-center justify-between">
                <div 
                    className="flex items-center gap-2 text-sm"
                    style={{ color: 'var(--text-muted)' }}
                >
                    {icon}
                    <span>{label}</span>
                </div>
                <div className="flex items-center gap-2">
                    {status === 'success' && <CheckCircle2 size={14} style={{ color: 'var(--accent-green)' }} />}
                    {status === 'warning' && <AlertCircle size={14} style={{ color: 'var(--accent-amber)' }} />}
                    <span 
                        className={`text-sm ${mono ? 'font-mono' : ''}`}
                        style={{ color: getStatusColor() }}
                    >
                        {value}
                    </span>
                </div>
            </div>
            {hint && (
                <code 
                    className="text-xs font-mono px-2 py-1 rounded block"
                    style={{ 
                        color: 'var(--accent-cyan)',
                        background: 'var(--bg-surface)'
                    }}
                >
                    {hint}
                </code>
            )}
        </div>
    );
};

const StatusLoading: React.FC = () => (
    <div 
        className="rounded-lg p-6 flex items-center justify-center"
        style={{ 
            background: 'var(--bg-secondary)',
            border: '1px solid var(--border-subtle)'
        }}
    >
        <Loader2 size={24} className="animate-spin" style={{ color: 'var(--text-muted)' }} />
    </div>
);

const StatusError: React.FC = () => (
    <div 
        className="rounded-lg p-6 flex flex-col items-center justify-center gap-2"
        style={{ 
            background: 'var(--bg-secondary)',
            border: '1px solid var(--accent-red-dim)'
        }}
    >
        <AlertCircle size={24} style={{ color: 'var(--accent-red)' }} />
        <span 
            className="text-sm"
            style={{ color: 'var(--accent-red)' }}
        >
            Connection Error
        </span>
    </div>
);
