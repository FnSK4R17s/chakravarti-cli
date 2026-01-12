import React, { useEffect, useState, useRef } from 'react';
import { type OrchestrationEvent } from '../types';
import { useMutation } from '@tanstack/react-query';
import {
    Terminal, AlertCircle, CheckCircle2, ChevronRight,
    Play, Pause, Trash2, Download, Filter, Sparkles, Loader2, X
} from 'lucide-react';
import { useCommandResult } from './CommandPalette';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Input } from '@/components/ui/input';
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select';

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

    const errorCount = logs.filter(l => l.type === 'error').length;

    return (
        <Card className="flex flex-col h-full">
            <CardHeader className="pb-3 shrink-0">
                <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3">
                        <Terminal size={16} className="text-primary" />
                        <CardTitle className="text-sm font-semibold">Execution Log</CardTitle>
                        <Badge variant="success" className="flex items-center gap-1">
                            <div className="w-1.5 h-1.5 rounded-full bg-current animate-pulse" />
                            Live
                        </Badge>
                    </div>

                    <div className="flex items-center gap-2">
                        {/* Type Filter */}
                        <Select value={typeFilter} onValueChange={setTypeFilter}>
                            <SelectTrigger className="w-[120px] h-8 text-xs">
                                <SelectValue placeholder="All Events" />
                            </SelectTrigger>
                            <SelectContent>
                                <SelectItem value="all">All Events</SelectItem>
                                <SelectItem value="step_start">Steps</SelectItem>
                                <SelectItem value="success">Success</SelectItem>
                                <SelectItem value="error">Errors</SelectItem>
                                <SelectItem value="log">Logs</SelectItem>
                            </SelectContent>
                        </Select>

                        {/* Text Filter */}
                        <div className="relative">
                            <Filter
                                size={12}
                                className="absolute left-2 top-1/2 -translate-y-1/2 text-muted-foreground"
                            />
                            <Input
                                type="text"
                                placeholder="Filter..."
                                value={filter}
                                onChange={(e) => setFilter(e.target.value)}
                                className="w-32 h-8 pl-7 text-xs"
                            />
                        </div>

                        {/* Toolbar buttons */}
                        <Button
                            variant="ghost"
                            size="icon"
                            className="h-8 w-8"
                            onClick={() => setAutoScroll(!autoScroll)}
                            title={autoScroll ? 'Pause auto-scroll' : 'Resume auto-scroll'}
                        >
                            {autoScroll ? <Pause size={14} /> : <Play size={14} />}
                        </Button>

                        <Button
                            variant="ghost"
                            size="icon"
                            className="h-8 w-8"
                            onClick={handleExport}
                            title="Export logs"
                        >
                            <Download size={14} />
                        </Button>

                        <Button
                            variant="ghost"
                            size="icon"
                            className="h-8 w-8"
                            onClick={handleClear}
                            title="Clear logs"
                        >
                            <Trash2 size={14} />
                        </Button>
                    </div>
                </div>
            </CardHeader>

            {/* Log Content */}
            <CardContent className="flex-1 p-0 min-h-0">
                <ScrollArea className="h-full" ref={viewportRef}>
                    <div className="p-2 font-mono text-xs leading-relaxed">
                        {filteredLogs.length === 0 ? (
                            <EmptyLogs />
                        ) : (
                            filteredLogs.map((log, i) => (
                                <LogLine key={i} log={log} />
                            ))
                        )}
                    </div>
                </ScrollArea>
            </CardContent>

            {/* Status Bar */}
            <div className="px-4 py-2 flex items-center justify-between text-xs font-mono border-t border-border bg-muted/50">
                <div className="flex items-center gap-3">
                    <span className="text-muted-foreground">{filteredLogs.length} events</span>

                    {/* Command Result Toast */}
                    {lastResult && (
                        <Badge
                            variant={lastResult.result.success ? 'success' : 'destructive'}
                            className="flex items-center gap-2"
                        >
                            {lastResult.result.success
                                ? <CheckCircle2 size={12} />
                                : <AlertCircle size={12} />
                            }
                            <span
                                className="max-w-[200px] truncate"
                                title={lastResult.result.message || (lastResult.result.success ? 'Command completed' : 'Command failed')}
                            >
                                {lastResult.result.message || (lastResult.result.success ? 'Done' : 'Failed')}
                            </span>
                            <button
                                onClick={() => setLastResult(null)}
                                className="opacity-60 hover:opacity-100 ml-1"
                            >
                                <X size={12} />
                            </button>
                        </Badge>
                    )}
                </div>
                <div className="flex items-center gap-3">
                    {/* Fix with AI button - show when there are errors */}
                    {errorCount > 0 && (
                        <Button
                            variant="outline"
                            size="sm"
                            onClick={() => fixMutation.mutate()}
                            disabled={fixMutation.isPending}
                            className="h-7 text-xs bg-[var(--accent-purple-dim)] text-[var(--accent-purple)] border-[var(--accent-purple)] hover:bg-[var(--accent-purple)] hover:text-background"
                        >
                            {fixMutation.isPending ? (
                                <Loader2 size={12} className="mr-1.5 animate-spin" />
                            ) : (
                                <Sparkles size={12} className="mr-1.5" />
                            )}
                            {fixMutation.isPending ? 'Fixing...' : 'Fix with AI'}
                        </Button>
                    )}
                    {errorCount > 0 && (
                        <Badge variant="destructive">{errorCount} err</Badge>
                    )}
                </div>
            </div>
        </Card>
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
                return { color: 'text-[var(--accent-red)]', icon: <AlertCircle size={12} /> };
            case 'success':
                return { color: 'text-[var(--accent-green)]', icon: <CheckCircle2 size={12} /> };
            case 'warning':
                return { color: 'text-[var(--accent-amber)]', icon: <AlertCircle size={12} /> };
            case 'step_start':
                return { color: 'text-[var(--accent-cyan)]', icon: <ChevronRight size={12} /> };
            case 'step_end':
                return { color: 'text-[var(--accent-purple)]', icon: <CheckCircle2 size={12} /> };
            default:
                return { color: 'text-muted-foreground', icon: null };
        }
    };

    const style = getTypeStyle(log.type);

    return (
        <div className="flex items-start gap-2 py-1 px-2 rounded hover:bg-accent/50 transition-colors group">
            {/* Timestamp */}
            <span className="shrink-0 select-none text-muted-foreground">
                {timestamp}
            </span>

            {/* Type indicator */}
            <span className={`shrink-0 w-4 flex items-center justify-center ${style.color}`}>
                {style.icon}
            </span>

            {/* Message */}
            <span className={`flex-1 break-all ${style.color}`}>
                {log.message}
            </span>

            {/* Step name if present */}
            {log.step_name && (
                <Badge variant="secondary" className="shrink-0 text-xs">
                    {log.step_name}
                </Badge>
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
    <div className="h-full flex flex-col items-center justify-center gap-4 py-12">
        <CkrvLogo />
        <div className="text-center text-muted-foreground">
            <div className="text-sm mb-1 opacity-80">No activity yet</div>
            <div className="text-xs opacity-60">
                Run <code className="px-1.5 py-0.5 rounded bg-muted">ckrv run</code> to start
            </div>
        </div>
    </div>
);
