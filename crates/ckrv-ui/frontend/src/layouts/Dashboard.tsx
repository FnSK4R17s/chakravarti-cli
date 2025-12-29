import React, { type ReactNode } from 'react';
import { FileText, GitBranch, Layers, Zap, ChevronRight, Loader2, Container, Bot } from 'lucide-react';
import { useConnection, type ConnectionStatus } from '../hooks/useConnection';
import { useQuery } from '@tanstack/react-query';
import { useNavigation } from '../App';

interface DockerStatus {
    available: boolean;
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

    // Page titles
    const pageTitle = currentPage === 'agents' ? 'Agent Manager' : 'Dashboard';

    return (
        <div className="flex h-screen w-full" style={{ background: 'var(--bg-primary)' }}>
            {/* Sidebar Navigation */}
            <aside 
                className="w-16 flex flex-col items-center py-6 gap-2"
                style={{ 
                    background: 'var(--bg-secondary)',
                    borderRight: '1px solid var(--border-subtle)'
                }}
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
                <nav className="flex flex-col gap-1 w-full items-center flex-1">
                    <NavIcon 
                        icon={<Layers size={20} />} 
                        label="Dashboard" 
                        active={currentPage === 'dashboard'} 
                        onClick={() => setCurrentPage('dashboard')}
                    />
                    <NavIcon 
                        icon={<Bot size={20} />} 
                        label="Agents" 
                        active={currentPage === 'agents'}
                        onClick={() => setCurrentPage('agents')}
                    />
                    <NavIcon icon={<FileText size={20} />} label="Specs" disabled />
                    <NavIcon icon={<GitBranch size={20} />} label="Jobs" disabled />
                </nav>

                {/* Bottom section */}
                <div className="flex flex-col gap-1 items-center">
                    <NavIcon icon={<Zap size={20} />} label="Quick Run" disabled />
                </div>
            </aside>

            {/* Main Content Area */}
            <div className="flex-1 flex flex-col overflow-hidden">
                {/* Header */}
                <header 
                    className="h-14 flex items-center justify-between px-6"
                    style={{ 
                        background: 'var(--bg-secondary)',
                        borderBottom: '1px solid var(--border-subtle)'
                    }}
                >
                    <div className="flex items-center gap-3">
                        <span 
                            className="font-mono text-sm px-2 py-1 rounded"
                            style={{ 
                                background: 'var(--bg-tertiary)',
                                color: 'var(--text-secondary)'
                            }}
                        >
                            ckrv
                        </span>
                        <ChevronRight size={14} style={{ color: 'var(--text-muted)' }} />
                        <h1 
                            className="text-lg font-semibold"
                            style={{ color: 'var(--text-primary)' }}
                        >
                            {pageTitle}
                        </h1>
                    </div>

                    <div className="flex items-center gap-3">
                        {/* Docker status */}
                        <DockerIndicator status={dockerStatus} />
                        {/* Connection status */}
                        <ConnectionIndicator status={status} />
                    </div>
                </header>

                {/* Page Content */}
                <main 
                    className="flex-1 overflow-hidden p-4 bg-grid"
                    style={{ background: 'var(--bg-primary)' }}
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
}

const NavIcon: React.FC<NavIconProps> = ({ icon, label, active, disabled, onClick }) => (
    <button 
        className="w-10 h-10 rounded-lg flex items-center justify-center transition-all duration-200 relative group"
        style={{ 
            background: active ? 'var(--bg-tertiary)' : 'transparent',
            color: active ? 'var(--accent-cyan)' : disabled ? 'var(--text-muted)' : 'var(--text-muted)',
            opacity: disabled ? 0.4 : 1,
            cursor: disabled ? 'not-allowed' : 'pointer',
        }}
        title={disabled ? `${label} (coming soon)` : label}
        onClick={disabled ? undefined : onClick}
        disabled={disabled}
    >
        {icon}
        {active && (
            <div 
                className="absolute left-0 top-1/2 -translate-y-1/2 w-0.5 h-5 rounded-r"
                style={{ background: 'var(--accent-cyan)' }}
            />
        )}
        {/* Tooltip */}
        <div 
            className="absolute left-full ml-2 px-2 py-1 rounded text-xs font-medium opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none whitespace-nowrap z-50"
            style={{ 
                background: 'var(--bg-elevated)',
                color: 'var(--text-primary)',
                border: '1px solid var(--border-default)'
            }}
        >
            {label}
            {disabled && <span style={{ color: 'var(--text-muted)' }}> (soon)</span>}
        </div>
    </button>
);

interface ConnectionIndicatorProps {
    status: ConnectionStatus;
}

const ConnectionIndicator: React.FC<ConnectionIndicatorProps> = ({ status }) => {
    const getStatusConfig = () => {
        switch (status) {
            case 'connected':
                return {
                    dotClass: 'running',
                    label: 'Connected',
                    color: 'var(--accent-green)',
                    bgColor: 'var(--accent-green-dim)',
                };
            case 'connecting':
                return {
                    dotClass: '',
                    label: 'Connecting...',
                    color: 'var(--accent-amber)',
                    bgColor: 'var(--accent-amber-dim)',
                };
            case 'disconnected':
            default:
                return {
                    dotClass: '',
                    label: 'Disconnected',
                    color: 'var(--accent-red)',
                    bgColor: 'var(--accent-red-dim)',
                };
        }
    };

    const config = getStatusConfig();

    return (
        <div 
            className="flex items-center gap-2 px-3 py-1.5 rounded-full transition-all duration-300"
            style={{ 
                background: config.bgColor,
                border: `1px solid ${config.color}`,
            }}
        >
            {status === 'connecting' ? (
                <Loader2 
                    size={12} 
                    className="animate-spin" 
                    style={{ color: config.color }} 
                />
            ) : (
                <div 
                    className="w-2 h-2 rounded-full"
                    style={{ 
                        background: config.color,
                        boxShadow: status === 'connected' ? `0 0 8px ${config.color}` : 'none',
                    }}
                />
            )}
            <span 
                className="text-xs font-medium"
                style={{ color: config.color }}
            >
                {config.label}
            </span>
        </div>
    );
};

interface DockerIndicatorProps {
    status?: DockerStatus;
}

const DockerIndicator: React.FC<DockerIndicatorProps> = ({ status }) => {
    const isAvailable = status?.available ?? false;
    const message = status?.message ?? 'Checking...';

    const config = isAvailable
        ? {
            color: 'var(--accent-cyan)',
            bgColor: 'var(--accent-cyan-dim)',
            label: 'Docker',
        }
        : {
            color: 'var(--accent-red)',
            bgColor: 'var(--accent-red-dim)',
            label: 'Docker',
        };

    return (
        <div 
            className="flex items-center gap-2 px-3 py-1.5 rounded-full transition-all duration-300 cursor-help"
            style={{ 
                background: config.bgColor,
                border: `1px solid ${config.color}`,
            }}
            title={message}
        >
            <Container size={12} style={{ color: config.color }} />
            <span 
                className="text-xs font-medium"
                style={{ color: config.color }}
            >
                {config.label}
            </span>
            {!status && (
                <Loader2 
                    size={10} 
                    className="animate-spin" 
                    style={{ color: config.color }} 
                />
            )}
        </div>
    );
};
