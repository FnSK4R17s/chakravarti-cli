import React, { useState } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { type SystemStatus } from '../types';
import { GitBranch, FolderGit, Settings, Loader2, AlertCircle, CheckCircle2, Play, GitBranchPlus } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Skeleton } from '@/components/ui/skeleton';
import { Alert, AlertDescription } from '@/components/ui/alert';

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

    const getModeVariant = (mode: string): "default" | "success" | "warning" | "info" | "secondary" => {
        switch (mode.toLowerCase()) {
            case 'running': return 'success';
            case 'planning': return 'default';
            case 'promoting': return 'info';
            default: return 'secondary';
        }
    };

    const getModeLabel = (mode: string) => {
        switch (mode.toLowerCase()) {
            case 'running': return 'Running';
            case 'planning': return 'Planning';
            case 'promoting': return 'Promoting';
            default: return 'Ready';
        }
    };

    // Check if it's a git repo (if branch is "none" or empty, it's not)
    const isGitRepo = status.active_branch && status.active_branch !== 'none' && status.active_branch !== '';

    return (
        <Card>
            <CardHeader className="pb-3">
                <div className="flex items-center justify-between">
                    <CardTitle className="text-sm font-semibold">Repository Status</CardTitle>
                    <Badge variant={getModeVariant(status.mode)}>
                        {getModeLabel(status.mode)}
                    </Badge>
                </div>
            </CardHeader>

            <CardContent className="space-y-3">
                {/* Git Repository Status */}
                {!isGitRepo ? (
                    <div className="flex flex-col gap-2">
                        <div className="flex items-center justify-between">
                            <div className="flex items-center gap-2 text-sm text-muted-foreground">
                                <GitBranchPlus size={16} />
                                <span>Git Repository</span>
                            </div>
                            <Badge variant="warning" className="flex items-center gap-1">
                                <AlertCircle size={12} />
                                Not initialized
                            </Badge>
                        </div>
                        <Button
                            onClick={runGitInit}
                            disabled={isRunningGitInit}
                            variant="secondary"
                            size="sm"
                            className="w-full"
                        >
                            {isRunningGitInit ? (
                                <>
                                    <Loader2 size={14} className="mr-2 animate-spin" />
                                    Initializing...
                                </>
                            ) : (
                                <>
                                    <Play size={14} className="mr-2" />
                                    Initialize Git Repository
                                </>
                            )}
                        </Button>
                        {gitInitResult && (
                            <Badge
                                variant={gitInitResult.success ? 'success' : 'destructive'}
                                className="text-xs justify-center"
                            >
                                {gitInitResult.message}
                            </Badge>
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
            </CardContent>

            {/* Quick Info Footer */}
            <div
                className="px-4 py-2 text-xs font-mono truncate border-t border-border bg-muted/50"
                title="$ ckrv --version"
            >
                <span className="text-primary">$</span> ckrv <span className="hidden sm:inline">--version</span>
            </div>
        </Card>
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
    return (
        <div className="flex flex-col gap-1">
            <div className="flex items-center justify-between">
                <div className="flex items-center gap-2 text-sm text-muted-foreground">
                    {icon}
                    <span>{label}</span>
                </div>
                <div className="flex items-center gap-2">
                    {status === 'success' && <CheckCircle2 size={14} className="text-[var(--accent-green)]" />}
                    {status === 'warning' && <AlertCircle size={14} className="text-[var(--accent-amber)]" />}
                    <span className={`text-sm ${mono ? 'font-mono' : ''} ${status ? '' : 'text-foreground'}`}>
                        {value}
                    </span>
                </div>
            </div>
            {hint && (
                <code className="text-xs font-mono px-2 py-1 rounded block bg-muted text-primary">
                    {hint}
                </code>
            )}
        </div>
    );
};

const StatusLoading: React.FC = () => (
    <Card>
        <CardHeader className="pb-3">
            <div className="flex items-center justify-between">
                <Skeleton className="h-4 w-32" />
                <Skeleton className="h-5 w-16" />
            </div>
        </CardHeader>
        <CardContent className="space-y-3">
            <div className="flex items-center justify-between">
                <Skeleton className="h-4 w-24" />
                <Skeleton className="h-4 w-16" />
            </div>
            <div className="flex items-center justify-between">
                <Skeleton className="h-4 w-20" />
                <Skeleton className="h-4 w-32" />
            </div>
        </CardContent>
    </Card>
);

const StatusError: React.FC = () => (
    <Card className="border-destructive">
        <CardContent className="p-6">
            <Alert variant="destructive">
                <AlertCircle className="h-4 w-4" />
                <AlertDescription>Connection Error</AlertDescription>
            </Alert>
        </CardContent>
    </Card>
);
