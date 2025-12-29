import React, { useEffect, useState, useRef } from 'react';
import { type OrchestrationEvent } from '../types';
import { useMutation } from '@tanstack/react-query';
import { 
    Terminal, AlertCircle, CheckCircle2, ChevronRight, 
    Play, Pause, Trash2, Download, Filter, Sparkles, Loader2, X
} from 'lucide-react';
import { useCommandResult } from './CommandPalette';

export const LogViewer: React.FC = () => {
    const [logs, setLogs] = useState<OrchestrationEvent[]>([]);
    const [filter, setFilter] = useState<string>('');
    const [autoScroll, setAutoScroll] = useState(true);
    const [typeFilter, setTypeFilter] = useState<string>('all');
    const viewportRef = useRef<HTMLDivElement>(null);
    const { lastResult, setLastResult } = useCommandResult();

    // Fix with AI mutation
    const fixMutation = useMutation({
        mutationFn: async () => {
            const res = await fetch('/api/command/fix', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ check: true }),
            });
            if (!res.ok) throw new Error('Fix command failed');
            return res.json();
        },
    });

    useEffect(() => {
        const evtSource = new EventSource('/api/events');

        evtSource.onmessage = (event) => {
            try {
                const parsed = JSON.parse(event.data) as OrchestrationEvent;
                setLogs(prev => [...prev, parsed]);
            } catch (e) {
                console.error("Failed to parse event", e);
            }
        };

        return () => {
            evtSource.close();
        };
    }, []);

    useEffect(() => {
        if (autoScroll && viewportRef.current) {
            viewportRef.current.scrollTop = viewportRef.current.scrollHeight;
        }
    }, [logs, autoScroll]);

    const filteredLogs = logs.filter(log => {
        const matchesText = filter === '' || 
            log.message.toLowerCase().includes(filter.toLowerCase()) ||
            log.type.toLowerCase().includes(filter.toLowerCase());
        const matchesType = typeFilter === 'all' || log.type.toLowerCase() === typeFilter;
        return matchesText && matchesType;
    });

    const handleClear = () => setLogs([]);

    const handleExport = () => {
        const content = filteredLogs.map(log => 
            `[${new Date(log.timestamp).toISOString()}] [${log.type.toUpperCase()}] ${log.message}`
        ).join('\n');
        
        const blob = new Blob([content], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `ckrv-logs-${Date.now()}.txt`;
        a.click();
    };

    return (
        <div 
            className="flex flex-col h-full rounded-lg overflow-hidden"
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
                <div className="flex items-center gap-3">
                    <Terminal size={16} style={{ color: 'var(--accent-cyan)' }} />
                    <h3 
                        className="font-semibold text-sm"
                        style={{ color: 'var(--text-primary)' }}
                    >
                        Execution Log
                    </h3>
                    <div 
                        className="flex items-center gap-1 text-xs"
                        style={{ color: 'var(--text-muted)' }}
                    >
                        <div className="status-dot running"></div>
                        Live
                    </div>
                </div>

                <div className="flex items-center gap-2">
                    {/* Type Filter */}
                    <select
                        value={typeFilter}
                        onChange={(e) => setTypeFilter(e.target.value)}
                        className="text-xs px-2 py-1 rounded border-none outline-none cursor-pointer"
                        style={{ 
                            background: 'var(--bg-tertiary)',
                            color: 'var(--text-secondary)'
                        }}
                    >
                        <option value="all">All Events</option>
                        <option value="step_start">Steps</option>
                        <option value="success">Success</option>
                        <option value="error">Errors</option>
                        <option value="log">Logs</option>
                    </select>

                    {/* Text Filter */}
                    <div className="relative">
                        <Filter 
                            size={12} 
                            className="absolute left-2 top-1/2 -translate-y-1/2"
                            style={{ color: 'var(--text-muted)' }}
                        />
                        <input
                            type="text"
                            placeholder="Filter..."
                            value={filter}
                            onChange={(e) => setFilter(e.target.value)}
                            className="w-32 pl-7 pr-2 py-1 text-xs rounded border-none outline-none"
                            style={{ 
                                background: 'var(--bg-tertiary)',
                                color: 'var(--text-primary)'
                            }}
                        />
                    </div>

                    {/* Auto-scroll toggle */}
                    <button
                        onClick={() => setAutoScroll(!autoScroll)}
                        className="p-1.5 rounded transition-colors"
                        style={{ 
                            background: autoScroll ? 'var(--accent-cyan-dim)' : 'var(--bg-tertiary)',
                            color: autoScroll ? 'var(--accent-cyan)' : 'var(--text-muted)'
                        }}
                        title={autoScroll ? 'Pause auto-scroll' : 'Resume auto-scroll'}
                    >
                        {autoScroll ? <Pause size={14} /> : <Play size={14} />}
                    </button>

                    {/* Export */}
                    <button
                        onClick={handleExport}
                        className="p-1.5 rounded transition-colors"
                        style={{ 
                            background: 'var(--bg-tertiary)',
                            color: 'var(--text-muted)'
                        }}
                        title="Export logs"
                    >
                        <Download size={14} />
                    </button>

                    {/* Clear */}
                    <button
                        onClick={handleClear}
                        className="p-1.5 rounded transition-colors"
                        style={{ 
                            background: 'var(--bg-tertiary)',
                            color: 'var(--text-muted)'
                        }}
                        title="Clear logs"
                    >
                        <Trash2 size={14} />
                    </button>
                </div>
            </div>

            {/* Log Content */}
            <div
                ref={viewportRef}
                className="flex-1 overflow-y-auto p-2 font-mono text-xs leading-relaxed"
                style={{ background: 'var(--bg-primary)' }}
            >
                {filteredLogs.length === 0 ? (
                    <EmptyLogs />
                ) : (
                    filteredLogs.map((log, i) => (
                        <LogLine key={i} log={log} />
                    ))
                )}
            </div>

            {/* Status Bar */}
            <div 
                className="px-4 py-2 flex items-center justify-between text-xs font-mono"
                style={{ 
                    background: 'var(--bg-tertiary)',
                    borderTop: '1px solid var(--border-subtle)',
                    color: 'var(--text-muted)'
                }}
            >
                <div className="flex items-center gap-3">
                    <span>{filteredLogs.length} events</span>
                    
                    {/* Command Result Toast */}
                    {lastResult && (
                        <div 
                            className="flex items-center gap-2 px-2 py-1 rounded"
                            style={{ 
                                background: lastResult.result.success ? 'var(--accent-green-dim)' : 'var(--accent-red-dim)',
                                border: `1px solid ${lastResult.result.success ? 'var(--accent-green)' : 'var(--accent-red)'}`
                            }}
                        >
                            {lastResult.result.success 
                                ? <CheckCircle2 size={12} style={{ color: 'var(--accent-green)' }} />
                                : <AlertCircle size={12} style={{ color: 'var(--accent-red)' }} />
                            }
                            <span 
                                className="max-w-[200px] truncate"
                                style={{ color: lastResult.result.success ? 'var(--accent-green)' : 'var(--accent-red)' }}
                                title={lastResult.result.message || (lastResult.result.success ? 'Command completed' : 'Command failed')}
                            >
                                {lastResult.result.message || (lastResult.result.success ? 'Done' : 'Failed')}
                            </span>
                            <button 
                                onClick={() => setLastResult(null)}
                                className="opacity-60 hover:opacity-100"
                                style={{ color: 'var(--text-secondary)' }}
                            >
                                <X size={12} />
                            </button>
                        </div>
                    )}
                </div>
                <div className="flex items-center gap-3">
                    {/* Fix with AI button - show when there are errors */}
                    {logs.filter(l => l.type === 'error').length > 0 && (
                        <button
                            onClick={() => fixMutation.mutate()}
                            disabled={fixMutation.isPending}
                            className="flex items-center gap-1.5 px-2.5 py-1 rounded-md transition-all"
                            style={{
                                background: 'var(--accent-purple-dim)',
                                color: 'var(--accent-purple)',
                                border: '1px solid var(--accent-purple)',
                                cursor: fixMutation.isPending ? 'not-allowed' : 'pointer',
                                opacity: fixMutation.isPending ? 0.7 : 1,
                            }}
                            title="Run AI to fix verification errors"
                        >
                            {fixMutation.isPending ? (
                                <Loader2 size={12} className="animate-spin" />
                            ) : (
                                <Sparkles size={12} />
                            )}
                            <span className="font-medium">
                                {fixMutation.isPending ? 'Fixing...' : 'Fix with AI'}
                            </span>
                        </button>
                    )}
                    {logs.filter(l => l.type === 'error').length > 0 && (
                        <span style={{ color: 'var(--accent-red)' }}>
                            {logs.filter(l => l.type === 'error').length} err
                        </span>
                    )}
                </div>
            </div>
        </div>
    );
};

const LogLine: React.FC<{ log: OrchestrationEvent }> = ({ log }) => {
    const timestamp = new Date(log.timestamp).toLocaleTimeString([], { 
        hour12: false, 
        hour: '2-digit', 
        minute: '2-digit', 
        second: '2-digit' 
    });

    const getTypeStyle = (type: string) => {
        switch (type.toLowerCase()) {
            case 'error':
                return { color: 'var(--accent-red)', icon: <AlertCircle size={12} /> };
            case 'success':
                return { color: 'var(--accent-green)', icon: <CheckCircle2 size={12} /> };
            case 'warning':
                return { color: 'var(--accent-amber)', icon: <AlertCircle size={12} /> };
            case 'step_start':
                return { color: 'var(--accent-cyan)', icon: <ChevronRight size={12} /> };
            case 'step_end':
                return { color: 'var(--accent-purple)', icon: <CheckCircle2 size={12} /> };
            default:
                return { color: 'var(--text-muted)', icon: null };
        }
    };

    const style = getTypeStyle(log.type);

    return (
        <div 
            className="flex items-start gap-2 py-1 px-2 rounded hover:bg-white/5 transition-colors group"
        >
            {/* Timestamp */}
            <span 
                className="shrink-0 select-none"
                style={{ color: 'var(--text-muted)' }}
            >
                {timestamp}
            </span>

            {/* Type indicator */}
            <span 
                className="shrink-0 w-4 flex items-center justify-center"
                style={{ color: style.color }}
            >
                {style.icon}
            </span>

            {/* Message */}
            <span 
                className="flex-1 break-all"
                style={{ color: style.color }}
            >
                {log.message}
            </span>

            {/* Step name if present */}
            {log.step_name && (
                <span 
                    className="shrink-0 px-1.5 py-0.5 rounded text-xs"
                    style={{ 
                        background: 'var(--bg-surface)',
                        color: 'var(--text-muted)'
                    }}
                >
                    {log.step_name}
                </span>
            )}
        </div>
    );
};

const CkrvLogo: React.FC = () => {
    const lines = [
        ' ██████╗██╗  ██╗██████╗ ██╗   ██╗',
        '██╔════╝██║ ██╔╝██╔══██╗██║   ██║',
        '██║     █████╔╝ ██████╔╝██║   ██║',
        '██║     ██╔═██╗ ██╔══██╗╚██╗ ██╔╝',
        '╚██████╗██║  ██╗██║  ██║ ╚████╔╝ ',
        ' ╚═════╝╚═╝  ╚═╝╚═╝  ╚═╝  ╚═══╝  ',
    ];
    
    // Exact color mapping from CLI (ANSI 256 colors to hex)
    const colors = ['#FFFFAF', '#FFFF87', '#FFD700', '#FFAF00', '#FF8700', '#FF8700'];
    
    return (
        <pre style={{ fontFamily: 'monospace', fontSize: '11px', lineHeight: 1.2, margin: 0 }}>
            {lines.map((line, i) => (
                <div key={i} style={{ color: colors[i] }}>{line}</div>
            ))}
        </pre>
    );
};

const EmptyLogs: React.FC = () => (
    <div 
        className="h-full flex flex-col items-center justify-center gap-4"
    >
        <CkrvLogo />
        <div className="text-center" style={{ color: 'var(--text-muted)' }}>
            <div className="text-sm mb-1 opacity-80">No activity yet</div>
            <div className="text-xs opacity-60">
                Run <code 
                    className="px-1.5 py-0.5 rounded"
                    style={{ background: 'var(--bg-surface)' }}
                >
                    ckrv run
                </code> to start
            </div>
        </div>
    </div>
);
