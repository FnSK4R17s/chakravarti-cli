/**
 * @module TestFixModal
 * @description
 * Modal dialog for running AI-powered test fix agent. Opens a sandboxed terminal
 * where the AI agent can analyze test failures and apply fixes. Includes error
 * copying and agent-assisted debugging.
 *
 * @context
 * Opened from TestRunner when tests fail. Provides interactive terminal for AI
 * agent to diagnose and fix test issues. Uses the same terminal infrastructure
 * as AgentCliModal.
 *
 * @dependencies
 * - xterm: Terminal emulator for interactive shell
 * - shadcn/ui components: Dialog, Badge, Button, Card for consistent UI
 *
 * @example
 * <TestFixModal
 *   error={testErrorOutput}
 *   baseBranch="main"
 *   onClose={() => setShowFixModal(false)}
 * />
 */

// === IMPORTS ===
import React, { useEffect, useRef, useState } from 'react';
import { Circle, X, Bot, Wrench, Copy, Check } from 'lucide-react';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import '@xterm/xterm/css/xterm.css';
import { getTerminalTheme } from '@/lib/theme';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Badge } from '@/components/ui/badge';
import { Card } from '@/components/ui/card';
import { Button } from '@/components/ui/button';

/**
 * Props for TestFixModal component.
 * Modal for running AI-powered test fix agent in a sandboxed terminal.
 */
interface TestFixModalProps {
    /** Test error output to be analyzed and fixed */
    error: string;
    /** Base git branch for the fix context */
    baseBranch: string;
    /** Callback fired when modal is closed */
    onClose: () => void;
}

export const TestFixModal: React.FC<TestFixModalProps> = ({ error, baseBranch, onClose }) => {
    // === REFS ===
    const terminalRef = useRef<HTMLDivElement>(null);
    const xtermRef = useRef<Terminal | null>(null);
    const wsRef = useRef<WebSocket | null>(null);
    const fitAddonRef = useRef<FitAddon | null>(null);
    const sessionIdRef = useRef(`test-fix-${Date.now()}`);

    // === STATE ===
    /** WebSocket connection status */
    const [status, setStatus] = useState<'connecting' | 'connected' | 'error' | 'disconnected'>('connecting');
    /** Docker container ID running the fix agent */
    const [containerId, setContainerId] = useState<string | null>(null);

    // Terminal initialization: wait for DOM, create xterm, and connect WebSocket for fix agent
    useEffect(() => {
        let mounted = true;

        const init = async () => {
            // Wait for terminal ref to be available
            let attempts = 0;
            while (!terminalRef.current && attempts < 20 && mounted) {
                await new Promise(resolve => setTimeout(resolve, 100));
                attempts++;
            }

            if (!mounted) return;
            if (!terminalRef.current) return;

            // Create xterm instance with theme-aware colors
            const term = new Terminal({
                cursorBlink: true,
                fontSize: 14,
                fontFamily: 'JetBrains Mono, Menlo, Monaco, Consolas, monospace',
                scrollback: 1000,
                cols: 120,
                rows: 30,
                convertEol: true,
                allowProposedApi: true,
                theme: getTerminalTheme(),
            });

            const fitAddon = new FitAddon();
            term.loadAddon(fitAddon);

            xtermRef.current = term;
            fitAddonRef.current = fitAddon;

            term.open(terminalRef.current);

            // Use ResizeObserver for reliable sizing
            const resizeObserver = new ResizeObserver(() => {
                if (fitAddonRef.current && terminalRef.current) {
                    try {
                        fitAddonRef.current.fit();
                    } catch {
                        // Ignore fit errors during resize
                    }
                }
            });
            resizeObserver.observe(terminalRef.current);

            setTimeout(() => fitAddon.fit(), 50);

            // Enable clipboard paste support
            term.attachCustomKeyEventHandler((event) => {
                if (event.type !== 'keydown') return true;
                if ((event.ctrlKey || event.metaKey) && event.key === 'v') {
                    navigator.clipboard.readText().then((text) => {
                        if (wsRef.current?.readyState === WebSocket.OPEN) {
                            wsRef.current.send(text);
                        }
                    }).catch(() => { });
                    return false;
                }
                if ((event.ctrlKey || event.metaKey) && event.key === 'c' && term.hasSelection()) {
                    const selection = term.getSelection();
                    navigator.clipboard.writeText(selection).catch(() => { });
                    return false;
                }
                return true;
            });

            term.writeln('\x1b[35m╔════════════════════════════════════════════════════════════════╗\x1b[0m');
            term.writeln('\x1b[35m║                  🤖 AI Test Fix Agent                          ║\x1b[0m');
            term.writeln('\x1b[35m╚════════════════════════════════════════════════════════════════╝\x1b[0m');
            term.writeln('');
            term.writeln('\x1b[33m# Starting sandbox to fix test errors...\x1b[0m');

            // Start terminal session using existing endpoint
            try {
                const res = await fetch('/api/terminal/start', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({
                        session_id: sessionIdRef.current,
                        agent: null, // Use default claude settings
                    }),
                });
                const data = await res.json();

                if (!mounted) return;

                if (data.success) {
                    setContainerId(data.container_id || null);
                    term.writeln(`\x1b[32m# Container: ${data.container_id?.slice(0, 12) || 'unknown'}\x1b[0m`);

                    // Check if running in Tauri mode
                    // eslint-disable-next-line @typescript-eslint/no-explicit-any
                    const isTauriMode = data.mode === 'tauri' || (window as any).__TAURI__;

                    if (isTauriMode && data.container_id) {
                        // Tauri mode: use PTY for interactive terminal
                        term.writeln('\x1b[33m# Running in Tauri desktop mode (PTY)\x1b[0m');

                        try {
                            const { spawn } = await import('tauri-pty');

                            const pty = await spawn('docker', [
                                'exec',
                                '-it',
                                data.container_id,
                                '/bin/bash',
                                '-l'
                            ], {
                                cols: term.cols,
                                rows: term.rows,
                            });

                            term.writeln('\x1b[32m# Connected! Shell ready.\x1b[0m');
                            term.writeln('\x1b[33m# Click "Fill Fix Command" to paste the AI fix command, then press Enter to run.\x1b[0m\r\n');
                            setStatus('connected');

                            // PTY data handler - data comes as array buffer, wrap in Uint8Array
                            pty.onData((data: ArrayLike<number>) => {
                                if (!mounted) return;
                                term.write(new Uint8Array(data));
                            });

                            term.onData((input: string) => {
                                pty.write(input);
                            });

                            term.onResize(({ cols, rows }) => {
                                pty.resize(cols, rows);
                            });

                            // eslint-disable-next-line @typescript-eslint/no-explicit-any
                            (term as any).__pty = pty;

                        } catch (ptyError) {
                            console.error('[TestFixModal] PTY spawn failed:', ptyError);
                            term.writeln(`\x1b[31m# PTY error: ${ptyError}\x1b[0m`);
                            setStatus('error');
                        }
                    } else {
                        // Web mode: use WebSocket
                        term.writeln('\x1b[33m# Connecting to shell...\x1b[0m');

                        const wsProtocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
                        const wsUrl = `${wsProtocol}//${window.location.host}/api/terminal/ws?session_id=${sessionIdRef.current}`;

                        const ws = new WebSocket(wsUrl);
                        wsRef.current = ws;

                        ws.onopen = () => {
                            if (!mounted) return;
                            setStatus('connected');
                            term.writeln('\x1b[32m# Connected! Shell ready.\x1b[0m');
                            term.writeln('\x1b[33m# Click "Fill Fix Command" to paste the AI fix command, then press Enter to run.\x1b[0m\r\n');
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
                            term.writeln('\r\n\x1b[33m# Session ended\x1b[0m');
                        };

                        term.onData((input) => {
                            if (ws.readyState === WebSocket.OPEN) {
                                ws.send(input);
                            }
                        });
                    }
                } else {
                    setStatus('error');
                    term.writeln(`\x1b[31m# Error: ${data.message || data.error}\x1b[0m`);
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
            fetch('/api/terminal/stop', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                // eslint-disable-next-line react-hooks/exhaustive-deps
                body: JSON.stringify({ session_id: sessionIdRef.current }),
            }).catch(() => { });
        };
    }, [error, baseBranch]);

    const handleClose = () => {
        wsRef.current?.close();
        fetch('/api/terminal/stop', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ session_id: sessionIdRef.current }),
        }).catch(() => { });
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
        connected: 'Ready',
        error: 'Error',
        disconnected: 'Completed'
    }[status];

    /** Whether the error text was recently copied to clipboard */
    const [copied, setCopied] = useState(false);

    const handleCopyError = async () => {
        try {
            await navigator.clipboard.writeText(error);
            setCopied(true);
            setTimeout(() => setCopied(false), 2000);
        } catch (e) {
            console.error('Failed to copy:', e);
        }
    };

    return (
        <Dialog open onOpenChange={(open) => !open && handleClose()}>
            <DialogContent
                className="max-w-4xl h-[85vh] flex flex-col p-0 gap-0"
                onEscapeKeyDown={(e) => e.preventDefault()}
                onInteractOutside={(e) => e.preventDefault()}
            >
                <DialogHeader className="px-4 py-3 shrink-0 border-b border-border bg-muted">
                    <div className="flex items-center gap-3">
                        <Bot size={16} className="text-primary" />
                        <DialogTitle className="text-sm">
                            AI Test Fix Agent
                        </DialogTitle>
                        <Badge variant="info" className="flex items-center gap-1">
                            <Wrench size={10} />
                            Auto-Fix
                        </Badge>
                        <Badge variant={getStatusVariant()} className="flex items-center gap-1">
                            <Circle size={8} fill="currentColor" />
                            {statusLabel}
                        </Badge>
                        {containerId && (
                            <span className="text-[10px] font-mono text-muted-foreground">
                                {containerId.slice(0, 12)}
                            </span>
                        )}
                        <div className="flex-1" />
                        {/* Copy Error Button */}
                        <Button
                            variant="outline"
                            size="sm"
                            onClick={handleCopyError}
                            className="flex items-center gap-2 mr-2"
                            title="Copy error logs to clipboard"
                        >
                            {copied ? (
                                <>
                                    <Check size={14} className="text-success" />
                                    Copied!
                                </>
                            ) : (
                                <>
                                    <Copy size={14} />
                                    Copy Error
                                </>
                            )}
                        </Button>
                        <Button
                            variant="ghost"
                            size="icon"
                            onClick={handleClose}
                            className="h-7 w-7 shrink-0"
                            title="Close terminal"
                        >
                            <X size={16} />
                        </Button>
                    </div>
                </DialogHeader>

                {/* Terminal wrapped in Card */}
                <div className="flex-1 m-2 overflow-hidden">
                    <Card className="h-full w-full overflow-hidden rounded-lg border-border">
                        <div
                            ref={terminalRef}
                            className="w-full h-full p-2 bg-background min-h-[400px]"
                        />
                    </Card>
                </div>
            </DialogContent>
        </Dialog>
    );
};
