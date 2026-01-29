import React, { type ReactNode, useState } from 'react';
import { Code2, Layers, ChevronRight, Loader2, Container, Bot, Cloud, FlaskConical, ShieldCheck, PanelLeftClose, PanelLeft, Settings, GitBranch } from 'lucide-react';
import { useConnection, type ConnectionStatus } from '../hooks/useConnection';
import { useQuery } from '@tanstack/react-query';
import { useNavigation } from '../App';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { cn } from '@/lib/utils';

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
    const [sidebarExpanded, setSidebarExpanded] = useState(false);

    // Fetch status to check if initialized and get branch
    const { data: systemStatus } = useQuery<{ is_ready: boolean; active_branch?: string }>({
        queryKey: ['status'],
        queryFn: async () => {
            const res = await fetch('/api/status');
            return res.json();
        },
        refetchInterval: 5000,
    });

    const currentBranch = systemStatus?.active_branch ?? 'main';

    const { data: dockerStatus } = useQuery<DockerStatus>({
        queryKey: ['docker'],
        queryFn: async () => {
            const res = await fetch('/api/docker');
            return res.json();
        },
        refetchInterval: 10000,
    });

    const { data: cloudStatus } = useQuery<CloudStatus>({
        queryKey: ['cloud'],
        queryFn: async () => {
            const res = await fetch('/api/cloud');
            return res.json();
        },
        refetchInterval: 15000,
    });

    // Page titles
    const pageTitles: Record<string, string> = {
        dashboard: 'Dashboard',
        agents: 'Agent Manager',
        code: 'Code',
        test: 'Test Runner',
        qa: 'QA Reviewer',
        settings: 'Settings',
    };
    const pageTitle = pageTitles[currentPage] || 'Dashboard';

    return (
        <div className="flex h-screen w-full bg-background">
            {/* Sidebar Navigation */}
            <aside
                className={cn(
                    "flex flex-col py-6 gap-2 bg-muted border-r border-border transition-all duration-300",
                    sidebarExpanded ? "w-48" : "w-16"
                )}
            >
                {/* Logo */}
                <div className={cn("flex items-center gap-3 mb-6", sidebarExpanded ? "px-4" : "justify-center")}>
                    <div
                        className="w-10 h-10 rounded-lg flex items-center justify-center font-mono font-bold text-sm shrink-0"
                        style={{
                            background: 'linear-gradient(135deg, var(--accent-cyan), var(--accent-purple))',
                            color: 'var(--bg-primary)'
                        }}
                    >
                        CK
                    </div>
                    {sidebarExpanded && (
                        <span className="font-semibold text-foreground truncate">Chakravarti</span>
                    )}
                </div>

                {/* Navigation Icons */}
                <nav className={cn("flex flex-col gap-1 flex-1", sidebarExpanded ? "px-2" : "items-center")} role="navigation" aria-label="Main navigation">
                    <NavItem
                        icon={<Layers size={20} />}
                        label="Dashboard"
                        active={currentPage === 'dashboard'}
                        onClick={() => setCurrentPage('dashboard')}
                        testId="nav-dashboard"
                        expanded={sidebarExpanded}
                    />
                    <NavItem
                        icon={<Code2 size={20} />}
                        label="Code"
                        active={currentPage === 'code'}
                        onClick={() => setCurrentPage('code')}
                        testId="nav-code"
                        expanded={sidebarExpanded}
                    />
                    <NavItem
                        icon={<FlaskConical size={20} />}
                        label="Test"
                        active={currentPage === 'test'}
                        onClick={() => setCurrentPage('test')}
                        testId="nav-test"
                        expanded={sidebarExpanded}
                    />
                    <NavItem
                        icon={<ShieldCheck size={20} />}
                        label="QA"
                        active={currentPage === 'qa'}
                        onClick={() => setCurrentPage('qa')}
                        testId="nav-qa"
                        expanded={sidebarExpanded}
                    />
                </nav>

                {/* Bottom section */}
                <div className={cn("flex flex-col gap-1", sidebarExpanded ? "px-2" : "items-center")}>
                    <NavItem
                        icon={<Bot size={20} />}
                        label="Agents"
                        active={currentPage === 'agents'}
                        onClick={() => setCurrentPage('agents')}
                        testId="nav-agents"
                        expanded={sidebarExpanded}
                    />
                    <NavItem
                        icon={<Settings size={20} />}
                        label="Settings"
                        active={currentPage === 'settings'}
                        onClick={() => setCurrentPage('settings')}
                        testId="nav-settings"
                        expanded={sidebarExpanded}
                    />

                    {/* Toggle button */}
                    <Button
                        variant="ghost"
                        size={sidebarExpanded ? "default" : "icon"}
                        className={cn(
                            "mt-2 text-muted-foreground hover:text-foreground",
                            sidebarExpanded ? "w-full justify-start gap-2" : "w-10 h-10"
                        )}
                        onClick={() => setSidebarExpanded(!sidebarExpanded)}
                        aria-label={sidebarExpanded ? "Collapse sidebar" : "Expand sidebar"}
                    >
                        {sidebarExpanded ? (
                            <>
                                <PanelLeftClose size={20} />
                                <span className="text-sm">Collapse</span>
                            </>
                        ) : (
                            <PanelLeft size={20} />
                        )}
                    </Button>
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

                    {/* Center - Branch indicator */}
                    <div className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-secondary/50 border border-border">
                        <GitBranch size={14} className="text-accent-cyan" />
                        <span className="font-mono text-sm text-foreground">{currentBranch}</span>
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

interface NavItemProps {
    icon: ReactNode;
    label: string;
    active?: boolean;
    disabled?: boolean;
    onClick?: () => void;
    testId?: string;
    expanded?: boolean;
}

const NavItem: React.FC<NavItemProps> = ({ icon, label, active, disabled, onClick, testId, expanded }) => {
    const button = (
        <Button
            variant="ghost"
            size={expanded ? "default" : "icon"}
            className={cn(
                "relative transition-all",
                expanded ? "w-full justify-start gap-3" : "w-10 h-10",
                active ? 'bg-accent text-primary' : 'text-muted-foreground hover:text-foreground'
            )}
            onClick={disabled ? undefined : onClick}
            disabled={disabled}
            data-testid={testId}
            aria-label={label}
        >
            <span className="shrink-0">{icon}</span>
            {expanded && <span className="truncate text-sm">{label}</span>}
            {active && (
                <div
                    className="absolute left-0 top-1/2 -translate-y-1/2 w-0.5 h-5 rounded-r bg-primary"
                />
            )}
        </Button>
    );

    // Only show tooltip when collapsed
    if (expanded) {
        return button;
    }

    return (
        <Tooltip>
            <TooltipTrigger asChild>
                {button}
            </TooltipTrigger>
            <TooltipContent side="right">
                <p>{label}</p>
            </TooltipContent>
        </Tooltip>
    );
};

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
