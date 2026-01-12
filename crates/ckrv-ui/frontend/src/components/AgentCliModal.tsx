import React, { useEffect, useRef, useState } from 'react';
import { Terminal as TerminalIcon, X, Circle } from 'lucide-react';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import '@xterm/xterm/css/xterm.css';
import type { AgentConfig } from './AgentManager';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card } from '@/components/ui/card';

// API functions
const startTerminalSession = async (sessionId: string, agent: AgentConfig) => {
    const res = await fetch('/api/terminal/start', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ session_id: sessionId, agent }),
    });
    return res.json();
};

const stopTerminalSession = async (sessionId: string) => {
    const res = await fetch('/api/terminal/stop', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ session_id: sessionId }),
    });
    return res.json();
};

interface AgentCliModalProps {
    agent: AgentConfig;
    onClose: () => void;
}

export const AgentCliModal: React.FC<AgentCliModalProps> = ({ agent, onClose }) => {
    const terminalRef = useRef<HTMLDivElement>(null);
    const xtermRef = useRef<Terminal | null>(null);
    const wsRef = useRef<WebSocket | null>(null);
    const fitAddonRef = useRef<FitAddon | null>(null);
    const sessionIdRef = useRef(`term-${agent.id}-${Date.now()}`);

    const [status, setStatus] = useState<'connecting' | 'connected' | 'error' | 'disconnected'>('connecting');
    const [containerId, setContainerId] = useState<string | null>(null);

    useEffect(() => {
        let mounted = true;

        const init = async () => {
            if (!terminalRef.current) return;

            // Create xterm instance
            const term = new Terminal({
                cursorBlink: true,
                fontSize: 14,
                fontFamily: 'JetBrains Mono, Menlo, Monaco, Consolas, monospace',
                theme: {
                    background: '#1e1e1e',
                    foreground: '#d4d4d4',
                    cursor: '#d4d4d4',
                    cursorAccent: '#1e1e1e',
                    selectionBackground: '#264f78',
                    black: '#1e1e1e',
                    red: '#f44747',
                    green: '#608b4e',
                    yellow: '#dcdcaa',
                    blue: '#569cd6',
                    magenta: '#c586c0',
                    cyan: '#4ec9b0',
                    white: '#d4d4d4',
                    brightBlack: '#808080',
                    brightRed: '#f44747',
                    brightGreen: '#608b4e',
                    brightYellow: '#dcdcaa',
                    brightBlue: '#569cd6',
                    brightMagenta: '#c586c0',
                    brightCyan: '#4ec9b0',
                    brightWhite: '#ffffff',
                },
            });

            const fitAddon = new FitAddon();
            term.loadAddon(fitAddon);

            xtermRef.current = term;
            fitAddonRef.current = fitAddon;

            term.open(terminalRef.current);
            fitAddon.fit();

            // Enable clipboard paste support
            term.attachCustomKeyEventHandler((event) => {
                // Only handle keydown events, not keyup or repeat
                if (event.type !== 'keydown') {
                    return true;
                }

                // Handle Ctrl+V / Cmd+V for paste
                if ((event.ctrlKey || event.metaKey) && event.key === 'v') {
                    navigator.clipboard.readText().then((text) => {
                        if (wsRef.current?.readyState === WebSocket.OPEN) {
                            wsRef.current.send(text);
                        }
                    }).catch(() => {
                        // Clipboard access denied
                    });
                    return false; // Prevent default
                }
                // Handle Ctrl+C / Cmd+C for copy (allow default)
                if ((event.ctrlKey || event.metaKey) && event.key === 'c' && term.hasSelection()) {
                    const selection = term.getSelection();
                    navigator.clipboard.writeText(selection).catch(() => { });
                    return false;
                }
                return true;
            });

            term.writeln('\x1b[33m# Starting sandbox terminal...\x1b[0m');

            // Start terminal session
            try {
                const res = await startTerminalSession(sessionIdRef.current, agent);
                if (!mounted) return;

                if (res.success) {
                    setContainerId(res.container_id || null);
                    term.writeln(`\x1b[32m# Container: ${res.container_id?.slice(0, 12) || 'unknown'}\x1b[0m`);

                    // Show agent configuration
                    if (agent.agent_type === 'claude_open_router' && agent.openrouter) {
                        term.writeln(`\x1b[35m# Mode: OpenRouter\x1b[0m`);
                        term.writeln(`\x1b[35m# Model: ${agent.openrouter.model}\x1b[0m`);
                    } else {
                        term.writeln(`\x1b[36m# Mode: Native Claude\x1b[0m`);
                    }

                    term.writeln('\x1b[33m# Connecting to shell...\x1b[0m');

                    // Connect WebSocket
                    const wsProtocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
                    const wsUrl = `${wsProtocol}//${window.location.host}/api/terminal/ws?session_id=${sessionIdRef.current}`;

                    const ws = new WebSocket(wsUrl);
                    wsRef.current = ws;

                    ws.onopen = () => {
                        if (!mounted) return;
                        setStatus('connected');
                        term.writeln('\x1b[32m# Connected! Type commands below.\x1b[0m\r\n');
                    };

                    ws.onmessage = (event) => {
                        if (!mounted) return;
                        term.write(event.data);
                    };

                    ws.onerror = () => {
                        if (!mounted) return;
                        setStatus('error');
                        term.writeln('\r\n\x1b[31m# WebSocket error\x1b[0m');
                    };

                    ws.onclose = () => {
                        if (!mounted) return;
                        setStatus('disconnected');
                        term.writeln('\r\n\x1b[33m# Connection closed\x1b[0m');
                    };

                    // Send terminal input to WebSocket
                    term.onData((data) => {
                        if (ws.readyState === WebSocket.OPEN) {
                            ws.send(data);
                        }
                    });
                } else {
                    setStatus('error');
                    term.writeln(`\x1b[31m# Error: ${res.message}\x1b[0m`);
                }
            } catch (e) {
                if (!mounted) return;
                setStatus('error');
                term.writeln(`\x1b[31m# Error: ${e}\x1b[0m`);
            }
        };

        init();

        // Handle resize
        const handleResize = () => {
            fitAddonRef.current?.fit();
        };
        window.addEventListener('resize', handleResize);

        // Cleanup
        return () => {
            mounted = false;
            window.removeEventListener('resize', handleResize);
            wsRef.current?.close();
            xtermRef.current?.dispose();
            stopTerminalSession(sessionIdRef.current).catch(() => { });
        };
    }, [agent.id]);

    const handleClose = () => {
        wsRef.current?.close();
        stopTerminalSession(sessionIdRef.current).catch(() => { });
        onClose();
    };

    const getStatusVariant = (): "success" | "warning" | "destructive" | "secondary" => {
        switch (status) {
            case 'connecting': return 'warning';
            case 'connected': return 'success';
            case 'error': return 'destructive';
            case 'disconnected': return 'secondary';
        }
    };

    const statusLabel = {
        connecting: 'Connecting...',
        connected: 'Connected',
        error: 'Error',
        disconnected: 'Disconnected'
    }[status];

    return (
        <Dialog open onOpenChange={(open) => !open && handleClose()}>
            <DialogContent className="max-w-4xl h-[85vh] flex flex-col p-0 gap-0">
                <DialogHeader className="px-4 py-3 shrink-0 border-b border-border bg-muted">
                    <div className="flex items-center justify-between">
                        <div className="flex items-center gap-3">
                            <TerminalIcon size={16} className="text-muted-foreground" />
                            <DialogTitle className="text-sm">
                                Interactive Terminal: {agent.name}
                            </DialogTitle>
                            <Badge variant="info">Sandboxed</Badge>
                            <Badge variant={getStatusVariant()} className="flex items-center gap-1">
                                <Circle size={8} fill="currentColor" />
                                {statusLabel}
                            </Badge>
                            {containerId && (
                                <span className="text-[10px] font-mono text-muted-foreground">
                                    {containerId.slice(0, 12)}
                                </span>
                            )}
                        </div>
                        <Button
                            variant="ghost"
                            size="icon"
                            onClick={handleClose}
                            className="h-8 w-8"
                        >
                            <X size={16} />
                        </Button>
                    </div>
                </DialogHeader>

                {/* Terminal wrapped in Card */}
                <Card className="flex-1 m-2 overflow-hidden rounded-lg border-border">
                    <div
                        ref={terminalRef}
                        className="h-full p-2"
                        style={{ background: '#1e1e1e' }}
                    />
                </Card>
            </DialogContent>
        </Dialog>
    );
};
