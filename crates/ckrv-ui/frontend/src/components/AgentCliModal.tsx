/**
 * @module AgentCliModal
 * @description
 * Modal dialog providing an interactive terminal for executing AI agent commands
 * in a sandboxed Docker container. Supports full xterm.js terminal with copy/paste,
 * WebSocket communication, and agent-specific configurations.
 *
 * @context
 * Opened from AgentManager when a user clicks "Open Terminal" on an agent. Provides
 * a real terminal experience with the selected agent's configuration applied.
 *
 * @dependencies
 * - xterm: Terminal emulator for interactive shell
 * - AgentConfig: Type from AgentManager for agent configuration
 * - shadcn/ui components: Dialog, Badge, Button for consistent UI
 *
 * @example
 * <AgentCliModal
 *   agent={selectedAgent}
 *   onClose={() => setShowTerminal(false)}
 * />
 */

// === IMPORTS ===
import React, { useEffect, useRef, useState } from 'react';
import { Terminal as TerminalIcon, Circle, X } from 'lucide-react';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import '@xterm/xterm/css/xterm.css';
import { getTerminalTheme } from '@/lib/theme';
import type { AgentConfig } from './AgentManager';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Badge } from '@/components/ui/badge';
import { Card } from '@/components/ui/card';
import { Button } from '@/components/ui/button';

// ============================================================
// API FUNCTIONS
// ============================================================

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

    // === STATE ===
    /** WebSocket connection status */
    const [status, setStatus] = useState<'connecting' | 'connected' | 'error' | 'disconnected'>('connecting');
    /** Docker container ID running the agent */
    const [containerId, setContainerId] = useState<string | null>(null);

    // ============================================================
    // EFFECTS
    // ============================================================

    // Terminal initialization: wait for DOM, create xterm, and connect WebSocket
    useEffect(() => {
        let mounted = true;

        const init = async () => {
            // Wait for terminal ref to be available (Dialog renders asynchronously as a portal)
            let attempts = 0;
            while (!terminalRef.current && attempts < 20 && mounted) {
                await new Promise(resolve => setTimeout(resolve, 100));
                attempts++;
            }

            if (!mounted) return;
            if (!terminalRef.current) {
                return;
            }

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
                    } catch (e) {
                        // Ignore fit errors during resize
                    }
                }
            });
            resizeObserver.observe(terminalRef.current);

            // Initial fit with a slight delay
            setTimeout(() => fitAddon.fit(), 50);

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

            console.log('[AgentCliModal] xterm initialized, starting session...');
            term.writeln('\x1b[33m# Starting sandbox terminal...\x1b[0m');

            // Start terminal session
            try {
                console.log('[AgentCliModal] Calling /api/terminal/start with session_id:', sessionIdRef.current);
                const res = await startTerminalSession(sessionIdRef.current, agent);
                console.log('[AgentCliModal] API response:', res);
                if (!mounted) return;

                if (res.success) {
                    setContainerId(res.container_id || null);
                    term.writeln(`\x1b[32m# Container: ${res.container_id?.slice(0, 12) || 'unknown'}\x1b[0m`);

                    // Show agent configuration
                    if (agent.agent_type === 'claude_open_router' && agent.openrouter) {
                        term.writeln(`\x1b[35m# Mode: OpenRouter\x1b[0m`);
                        term.writeln(`\x1b[35m# Model: ${agent.openrouter.model}\x1b[0m`);
                    } else if (agent.agent_type === 'codex') {
                        term.writeln(`\x1b[32m# Mode: OpenAI Codex\x1b[0m`);
                    } else {
                        term.writeln(`\x1b[36m# Mode: Native Claude\x1b[0m`);
                    }

                    // Check if running in Tauri mode (no WebSocket available)
                    const isTauriMode = res.mode === 'tauri' || (window as any).__TAURI__;

                    if (isTauriMode && res.container_id) {
                        // Tauri mode: use PTY for interactive terminal
                        // See: crates/ckrv-ui/frontend/src/hooks/useTauriPty.ts for architecture
                        term.writeln('\x1b[33m# Running in Tauri desktop mode (PTY)\x1b[0m');

                        try {
                            // Dynamic import tauri-pty to avoid loading in web mode
                            const { spawn } = await import('tauri-pty');

                            // Spawn docker exec with PTY - this gives full interactive terminal
                            const pty = await spawn('docker', [
                                'exec',
                                '-it',
                                res.container_id,
                                '/bin/bash',
                                '-l'  // Login shell for proper environment
                            ], {
                                cols: term.cols,
                                rows: term.rows,
                            });

                            term.writeln('\x1b[32m# Connected! Type commands below.\x1b[0m\r\n');
                            setStatus('connected');

                            // PTY data handler - data comes as array buffer, wrap in Uint8Array
                            // See: https://github.com/Tnze/tauri-plugin-pty/blob/main/examples/vanilla/src/index.js
                            pty.onData((data: ArrayLike<number>) => {
                                if (!mounted) return;
                                term.write(new Uint8Array(data));
                            });

                            term.onData((data: string) => {
                                pty.write(data);
                            });

                            // Handle terminal resize
                            term.onResize(({ cols, rows }) => {
                                pty.resize(cols, rows);
                            });

                            // Store PTY reference for cleanup
                            (term as any).__pty = pty;

                        } catch (ptyError) {
                            console.error('[AgentCliModal] PTY spawn failed:', ptyError);
                            term.writeln(`\x1b[31m# PTY error: ${ptyError}\x1b[0m`);
                            term.writeln('\x1b[33m# Falling back to IPC polling mode...\x1b[0m');

                            // Fallback to polling if PTY fails
                            setStatus('connected');

                            const pollOutput = async () => {
                                if (!mounted) return;
                                try {
                                    const readRes = await fetch('/api/terminal/read', {
                                        method: 'POST',
                                        headers: { 'Content-Type': 'application/json' },
                                        body: JSON.stringify({ session_id: sessionIdRef.current }),
                                    });
                                    const data = await readRes.json();
                                    if (data && data.data) {
                                        term.write(data.data);
                                    }
                                } catch (e) {
                                    // Ignore read errors
                                }
                                if (mounted) {
                                    setTimeout(pollOutput, 100);
                                }
                            };
                            pollOutput();

                            term.onData(async (data) => {
                                try {
                                    await fetch('/api/terminal/write', {
                                        method: 'POST',
                                        headers: { 'Content-Type': 'application/json' },
                                        body: JSON.stringify({ session_id: sessionIdRef.current, data }),
                                    });
                                } catch (e) {
                                    // Ignore write errors
                                }
                            });
                        }
                    } else {
                        // Web mode: use WebSocket
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
                    }
                } else {
                    setStatus('error');
                    term.writeln(`\x1b[31m# Error: ${res.message || res.error}\x1b[0m`);
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
            // Kill PTY if it exists (Tauri mode)
            if (xtermRef.current && (xtermRef.current as any).__pty) {
                try {
                    (xtermRef.current as any).__pty.kill();
                } catch (e) {
                    // Ignore kill errors
                }
            }
            xtermRef.current?.dispose();
            stopTerminalSession(sessionIdRef.current).catch(() => { });
        };
    }, [agent.id]);

    // ============================================================
    // HANDLERS
    // ============================================================

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
            <DialogContent
                className="max-w-4xl h-[85vh] flex flex-col p-0 gap-0"
                onEscapeKeyDown={(e) => e.preventDefault()}
                onInteractOutside={(e) => e.preventDefault()}
            >
                <DialogHeader className="px-4 py-3 shrink-0 border-b border-border bg-muted">
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
                        {/* Close button */}
                        <div className="flex-1" />
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
