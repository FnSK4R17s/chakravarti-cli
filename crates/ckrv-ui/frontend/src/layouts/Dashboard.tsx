import React, { type ReactNode } from 'react';
import { FileText, Layers, Zap, ChevronRight, Loader2, Container, Bot, Cloud, ListTodo, Workflow, Rocket, GitCompare } from 'lucide-react';
import { useConnection, type ConnectionStatus } from '../hooks/useConnection';
import { useQuery } from '@tanstack/react-query';
import { useNavigation } from '../App';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';

interface DockerStatus {
    available: boolean;
    message: string;
}

interface CloudStatus {
    authenticated: boolean;
    email?: string;
    message: string;
}

interface DashboardLayoutProps {
    children: ReactNode;
}

export const DashboardLayout: React.FC<DashboardLayoutProps> = ({ children }) => {
    const { status } = useConnection(5000);
    const { currentPage, setCurrentPage } = useNavigation();

    const { data: dockerStatus } = useQuery<DockerStatus>({
        queryKey: ['docker'],
        queryFn: async () => {
            const res = await fetch('/api/docker');
            return res.json();
        },
        refetchInterval: 10000, // Check every 10 seconds
    });

    const { data: cloudStatus } = useQuery<CloudStatus>({
        queryKey: ['cloud'],
        queryFn: async () => {
            const res = await fetch('/api/cloud');
            return res.json();
        },
        refetchInterval: 15000, // Check every 15 seconds
    });

    // Page titles
    const pageTitles: Record<string, string> = {
        dashboard: 'Dashboard',
        agents: 'Agent Manager',
        specs: 'Specifications',
        plan: 'Execution Plan',
        tasks: 'Task Orchestration',
        runner: 'Execution Runner',
        diff: 'Diff Viewer',
    };
    const pageTitle = pageTitles[currentPage] || 'Dashboard';

    return (
        <div className="flex h-screen w-full bg-background">
            {/* Sidebar Navigation */}
            <aside
                className="w-16 flex flex-col items-center py-6 gap-2 bg-muted border-r border-border"
            >
                {/* Logo */}
                <div
                    className="w-10 h-10 rounded-lg flex items-center justify-center mb-6 font-mono font-bold text-sm"
                    style={{
                        background: 'linear-gradient(135deg, var(--accent-cyan), var(--accent-purple))',
                        color: 'var(--bg-primary)'
                    }}
                >
                    CK
                </div>

                {/* Navigation Icons */}
                <nav className="flex flex-col gap-1 w-full items-center flex-1" role="navigation" aria-label="Main navigation">
                    <NavIcon
                        icon={<Layers size={20} />}
                        label="Dashboard"
                        active={currentPage === 'dashboard'}
                        onClick={() => setCurrentPage('dashboard')}
                        testId="nav-dashboard"
                    />
                    <NavIcon
                        icon={<Bot size={20} />}
                        label="Agents"
                        active={currentPage === 'agents'}
                        onClick={() => setCurrentPage('agents')}
                        testId="nav-agents"
                    />
                    <NavIcon
                        icon={<FileText size={20} />}
                        label="Specs"
                        active={currentPage === 'specs'}
                        onClick={() => setCurrentPage('specs')}
                        testId="nav-specs"
                    />
                    <NavIcon
                        icon={<ListTodo size={20} />}
                        label="Tasks"
                        active={currentPage === 'tasks'}
                        onClick={() => setCurrentPage('tasks')}
                        testId="nav-tasks"
                    />
                    <NavIcon
                        icon={<Workflow size={20} />}
                        label="Plan"
                        active={currentPage === 'plan'}
                        onClick={() => setCurrentPage('plan')}
                        testId="nav-plan"
                    />
                    <NavIcon
                        icon={<Rocket size={20} />}
                        label="Runner"
                        active={currentPage === 'runner'}
                        onClick={() => setCurrentPage('runner')}
                        testId="nav-runner"
                    />
                    <NavIcon
                        icon={<GitCompare size={20} />}
                        label="Diff"
                        active={currentPage === 'diff'}
                        onClick={() => setCurrentPage('diff')}
                        testId="nav-diff"
                    />
                </nav>

                {/* Bottom section */}
                <div className="flex flex-col gap-1 items-center">
                    <NavIcon icon={<Zap size={20} />} label="Quick Run (coming soon)" disabled testId="nav-quick-run" />
                </div>
            </aside>

            {/* Main Content Area */}
            <div className="flex-1 flex flex-col overflow-hidden">
                {/* Header */}
                <header
                    className="h-14 flex items-center justify-between px-6 bg-muted border-b border-border"
                >
                    <div className="flex items-center gap-3">
                        <span
                            className="font-mono text-sm px-2 py-1 rounded bg-secondary text-secondary-foreground"
                        >
                            ckrv
                        </span>
                        <ChevronRight size={14} className="text-muted-foreground" />
                        <h1 className="text-lg font-semibold text-foreground">
                            {pageTitle}
                        </h1>
                    </div>

                    <div className="flex items-center gap-3">
                        {/* Cloud status */}
                        <CloudIndicator status={cloudStatus} />
                        {/* Docker status */}
                        <DockerIndicator status={dockerStatus} />
                        {/* Connection status */}
                        <ConnectionIndicator status={status} />
                    </div>
                </header>

                {/* Page Content */}
                <main
                    className="flex-1 overflow-hidden p-4 bg-background"
                >
                    <div className="h-full">
                        {children}
                    </div>
                </main>
            </div>
        </div>
    );
};

interface NavIconProps {
    icon: ReactNode;
    label: string;
    active?: boolean;
    disabled?: boolean;
    onClick?: () => void;
    testId?: string;
}

const NavIcon: React.FC<NavIconProps> = ({ icon, label, active, disabled, onClick, testId }) => (
    <Tooltip>
        <TooltipTrigger asChild>
            <Button
                variant="ghost"
                size="icon"
                className={`w-10 h-10 relative ${active ? 'bg-accent text-primary' : 'text-muted-foreground hover:text-foreground'}`}
                onClick={disabled ? undefined : onClick}
                disabled={disabled}
                data-testid={testId}
                aria-label={label}
            >
                {icon}
                {active && (
                    <div
                        className="absolute left-0 top-1/2 -translate-y-1/2 w-0.5 h-5 rounded-r bg-primary"
                    />
                )}
            </Button>
        </TooltipTrigger>
        <TooltipContent side="right">
            <p>{label}</p>
        </TooltipContent>
    </Tooltip>
);

interface ConnectionIndicatorProps {
    status: ConnectionStatus;
}

const ConnectionIndicator: React.FC<ConnectionIndicatorProps> = ({ status }) => {
    const getStatusConfig = () => {
        switch (status) {
            case 'connected':
                return {
                    variant: 'success' as const,
                    label: 'Connected',
                    showPulse: true,
                };
            case 'connecting':
                return {
                    variant: 'warning' as const,
                    label: 'Connecting...',
                    showPulse: false,
                };
            case 'disconnected':
            default:
                return {
                    variant: 'destructive' as const,
                    label: 'Disconnected',
                    showPulse: false,
                };
        }
    };

    const config = getStatusConfig();

    return (
        <Tooltip>
            <TooltipTrigger asChild>
                <Badge variant={config.variant} className="flex items-center gap-2 cursor-default">
                    {status === 'connecting' ? (
                        <Loader2 size={12} className="animate-spin" />
                    ) : (
                        <div
                            className={`w-2 h-2 rounded-full ${config.showPulse ? 'animate-pulse' : ''}`}
                            style={{
                                backgroundColor: 'currentColor',
                                boxShadow: config.showPulse ? '0 0 8px currentColor' : 'none',
                            }}
                        />
                    )}
                    {config.label}
                </Badge>
            </TooltipTrigger>
            <TooltipContent>
                <p>Server connection status</p>
            </TooltipContent>
        </Tooltip>
    );
};

interface DockerIndicatorProps {
    status?: DockerStatus;
}

const DockerIndicator: React.FC<DockerIndicatorProps> = ({ status }) => {
    const isAvailable = status?.available ?? false;
    const message = status?.message ?? 'Checking Docker status...';

    const variant = isAvailable ? 'default' : 'destructive';

    return (
        <Tooltip>
            <TooltipTrigger asChild>
                <Badge variant={variant} className="flex items-center gap-2 cursor-default">
                    <Container size={12} />
                    Docker
                    {!status && <Loader2 size={10} className="animate-spin" />}
                </Badge>
            </TooltipTrigger>
            <TooltipContent>
                <p>{message}</p>
            </TooltipContent>
        </Tooltip>
    );
};

interface CloudIndicatorProps {
    status?: CloudStatus;
}

const CloudIndicator: React.FC<CloudIndicatorProps> = ({ status }) => {
    const isAuthenticated = status?.authenticated ?? false;
    const email = status?.email;
    const baseMessage = status?.message ?? 'Checking cloud status...';

    // Show email in tooltip if authenticated
    const tooltipMessage = isAuthenticated && email
        ? `${baseMessage} (${email})`
        : baseMessage;

    const variant = isAuthenticated ? 'info' : 'warning';

    return (
        <Tooltip>
            <TooltipTrigger asChild>
                <Badge variant={variant} className="flex items-center gap-2 cursor-default">
                    <Cloud size={12} />
                    Cloud
                    {!status && <Loader2 size={10} className="animate-spin" />}
                </Badge>
            </TooltipTrigger>
            <TooltipContent>
                <p>{tooltipMessage}</p>
            </TooltipContent>
        </Tooltip>
    );
};
