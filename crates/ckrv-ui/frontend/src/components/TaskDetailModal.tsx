import React, { useState, useEffect, useRef } from 'react';
import { useQuery } from '@tanstack/react-query';
import {
    X, Play, CheckCircle2, Circle, AlertTriangle, GitBranch,
    Zap, Brain, Cpu, Link2, Clock, Bot, Loader2,
    RotateCcw, Terminal as TerminalIcon
} from 'lucide-react';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import '@xterm/xterm/css/xterm.css';
import type { AgentConfig } from './AgentManager';
import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
    DialogFooter,
} from '@/components/ui/dialog';
import { Card, CardContent, CardHeader } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select';

// Types
interface Task {
    id: string;
    phase: string;
    title: string;
    description: string;
    file: string;
    user_story: string | null;
    parallel: boolean;
    complexity: number;
    model_tier: string;
    estimated_tokens: number;
    risk: string;
    context_required: string[];
    status: string;
}

interface TaskDetailModalProps {
    task: Task;
    specName: string;
    onClose: () => void;
    onStatusChange: (taskId: string, status: string) => void;
}

// API functions
const fetchAgents = async (): Promise<{ agents: AgentConfig[] }> => {
    const res = await fetch('/api/agents');
    return res.json();
};

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

// Badge Components using shadcn Badge
const RiskBadge: React.FC<{ risk: string }> = ({ risk }) => {
    const variants: Record<string, "success" | "warning" | "destructive" | "secondary"> = {
        low: 'success',
        medium: 'warning',
        high: 'warning',
        critical: 'destructive'
    };
    return (
        <Badge variant={variants[risk] || 'secondary'}>
            {risk}
        </Badge>
    );
};

const ModelTierBadge: React.FC<{ tier: string }> = ({ tier }) => {
    const icons: Record<string, React.ElementType> = {
        light: Zap,
        standard: Cpu,
        heavy: Brain
    };
    const Icon = icons[tier] || Cpu;
    return (
        <Badge variant="info" className="flex items-center gap-1">
            <Icon size={12} />
            {tier}
        </Badge>
    );
};

const StatusBadge: React.FC<{ status: string }> = ({ status }) => {
    const variants: Record<string, "secondary" | "info" | "success" | "destructive"> = {
        pending: 'secondary',
        running: 'info',
        completed: 'success',
        failed: 'destructive'
    };
    const icons: Record<string, React.ElementType> = {
        pending: Circle,
        running: Play,
        completed: CheckCircle2,
        failed: AlertTriangle
    };
    const Icon = icons[status] || Circle;
    return (
        <Badge variant={variants[status] || 'secondary'} className="flex items-center gap-2 px-3 py-1.5">
            <Icon size={16} />
            {status.charAt(0).toUpperCase() + status.slice(1)}
        </Badge>
    );
};

// Agent Selector using shadcn Select
const AgentSelector: React.FC<{
    selectedAgent: AgentConfig | null;
    onSelect: (agent: AgentConfig) => void;
    recommendedTier: string;
}> = ({ selectedAgent, onSelect, recommendedTier }) => {
    const { data: agentsData, isLoading } = useQuery({
        queryKey: ['agents'],
        queryFn: fetchAgents,
    });

    const agents = agentsData?.agents?.filter(a => a.enabled) || [];
    const defaultAgent = agents.find(a => a.is_default) || agents[0];

    // Auto-select default agent
    useEffect(() => {
        if (!selectedAgent && defaultAgent) {
            onSelect(defaultAgent);
        }
    }, [defaultAgent, selectedAgent, onSelect]);

    // Get recommended agents based on task tier
    const getAgentRecommendation = (agent: AgentConfig): 'recommended' | 'capable' | 'overkill' | null => {
        const tierLevels: Record<string, number> = { light: 1, standard: 3, heavy: 5 };
        const taskLevel = tierLevels[recommendedTier] || 3;

        if (agent.level === taskLevel) return 'recommended';
        if (agent.level >= taskLevel) return agent.level > taskLevel + 1 ? 'overkill' : 'capable';
        return null;
    };

    if (isLoading) {
        return (
            <div className="flex items-center gap-2 text-muted-foreground">
                <Loader2 size={16} className="animate-spin" />
                Loading agents...
            </div>
        );
    }

    if (agents.length === 0) {
        return (
            <div className="text-[var(--accent-amber)] text-sm flex items-center gap-2">
                <AlertTriangle size={16} />
                No agents configured. Add one in Agent Manager.
            </div>
        );
    }

    return (
        <Select
            value={selectedAgent?.id || ''}
            onValueChange={(value) => {
                const agent = agents.find(a => a.id === value);
                if (agent) onSelect(agent);
            }}
        >
            <SelectTrigger className="w-full">
                <SelectValue placeholder="Select an agent">
                    {selectedAgent && (
                        <div className="flex items-center gap-2">
                            <Bot size={16} className="text-primary" />
                            <span>{selectedAgent.name}</span>
                            <span className="text-muted-foreground">• {selectedAgent.agent_type} • Level {selectedAgent.level}</span>
                        </div>
                    )}
                </SelectValue>
            </SelectTrigger>
            <SelectContent>
                {agents.map(agent => {
                    const rec = getAgentRecommendation(agent);
                    return (
                        <SelectItem key={agent.id} value={agent.id}>
                            <div className="flex items-center justify-between w-full gap-4">
                                <div className="flex items-center gap-2">
                                    <Bot size={16} className={agent.is_default ? 'text-[var(--accent-amber)]' : 'text-muted-foreground'} />
                                    <div>
                                        <div className="flex items-center gap-2">
                                            <span className="font-medium">{agent.name}</span>
                                            {agent.is_default && (
                                                <Badge variant="warning" className="text-xs">default</Badge>
                                            )}
                                        </div>
                                        <div className="text-xs text-muted-foreground">
                                            {agent.agent_type} • Level {agent.level}
                                            {agent.openrouter?.model && ` • ${agent.openrouter.model}`}
                                        </div>
                                    </div>
                                </div>
                                {rec && (
                                    <Badge variant={rec === 'recommended' ? 'success' : rec === 'capable' ? 'info' : 'warning'}>
                                        {rec}
                                    </Badge>
                                )}
                            </div>
                        </SelectItem>
                    );
                })}
            </SelectContent>
        </Select>
    );
};

// Embedded Terminal Component wrapped in Card
const EmbeddedTerminal: React.FC<{
    agent: AgentConfig;
    task: Task;
    specName: string;
    onComplete: (success: boolean) => void;
}> = ({ agent, task, specName, onComplete }) => {
    const terminalRef = useRef<HTMLDivElement>(null);
    const xtermRef = useRef<Terminal | null>(null);
    const wsRef = useRef<WebSocket | null>(null);
    const fitAddonRef = useRef<FitAddon | null>(null);
    const sessionIdRef = useRef(`task-${task.id}-${Date.now()}`);
    const [status, setStatus] = useState<'connecting' | 'connected' | 'error' | 'disconnected'>('connecting');

    useEffect(() => {
        let mounted = true;

        const init = async () => {
            if (!terminalRef.current) return;

            // Create xterm instance
            const term = new Terminal({
                cursorBlink: true,
                fontSize: 13,
                fontFamily: 'Menlo, Monaco, "Courier New", monospace',
                theme: {
                    background: '#0d1117',
                    foreground: '#c9d1d9',
                    cursor: '#58a6ff',
                    cursorAccent: '#0d1117',
                    selectionBackground: '#3392FF44',
                    black: '#484f58',
                    red: '#ff7b72',
                    green: '#3fb950',
                    yellow: '#d29922',
                    blue: '#58a6ff',
                    magenta: '#bc8cff',
                    cyan: '#39c5cf',
                    white: '#b1bac4',
                },
                convertEol: true,
                scrollback: 5000,
            });

            const fitAddon = new FitAddon();
            term.loadAddon(fitAddon);
            term.open(terminalRef.current);
            fitAddon.fit();

            xtermRef.current = term;
            fitAddonRef.current = fitAddon;

            term.writeln('\x1b[1;36m▌ Starting task execution...\x1b[0m');
            term.writeln(`\x1b[90mTask: ${task.id} - ${task.title}\x1b[0m`);
            term.writeln(`\x1b[90mAgent: ${agent.name} (${agent.agent_type})\x1b[0m`);
            term.writeln('');

            try {
                // Start terminal session
                const result = await startTerminalSession(sessionIdRef.current, agent);

                if (!mounted || !result.success) {
                    setStatus('error');
                    term.writeln('\x1b[1;31m✗ Failed to start session\x1b[0m');
                    return;
                }

                // Connect WebSocket
                const wsProtocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
                const ws = new WebSocket(`${wsProtocol}//${window.location.host}/api/terminal/ws/${sessionIdRef.current}`);

                ws.onopen = () => {
                    if (!mounted) return;
                    setStatus('connected');

                    // Send the task command
                    const taskCommand = buildTaskCommand(task);
                    term.writeln(`\x1b[1;32m▌ Executing: ${taskCommand}\x1b[0m\n`);
                    ws.send(taskCommand + '\n');
                };

                ws.onmessage = (event) => {
                    if (!mounted) return;
                    term.write(event.data);
                };

                ws.onerror = () => {
                    if (!mounted) return;
                    setStatus('error');
                    term.writeln('\x1b[1;31m✗ Connection error\x1b[0m');
                };

                ws.onclose = () => {
                    if (!mounted) return;
                    setStatus('disconnected');
                };

                wsRef.current = ws;

                // Handle terminal input
                term.onData((data) => {
                    if (ws.readyState === WebSocket.OPEN) {
                        ws.send(data);
                    }
                });

            } catch (err) {
                setStatus('error');
                term.writeln(`\x1b[1;31m✗ Error: ${err}\x1b[0m`);
            }
        };

        init();

        // Handle resize
        const handleResize = () => fitAddonRef.current?.fit();
        window.addEventListener('resize', handleResize);

        return () => {
            mounted = false;
            window.removeEventListener('resize', handleResize);
            wsRef.current?.close();
            xtermRef.current?.dispose();
            stopTerminalSession(sessionIdRef.current);
        };
    }, [agent, task, specName]);

    return (
        <Card className="flex flex-col h-full">
            <CardHeader className="py-2 px-4 flex flex-row items-center justify-between border-b border-border">
                <div className="flex items-center gap-3">
                    <TerminalIcon size={16} className="text-muted-foreground" />
                    <span className="text-sm text-muted-foreground">Task Execution</span>
                    <span className={`w-2 h-2 rounded-full ${status === 'connected' ? 'bg-[var(--accent-green)]' :
                        status === 'connecting' ? 'bg-[var(--accent-amber)] animate-pulse' :
                            status === 'error' ? 'bg-destructive' : 'bg-muted-foreground'
                        }`} />
                </div>
                <div className="flex items-center gap-2">
                    <Button size="sm" variant="default" onClick={() => onComplete(true)}>
                        Mark Complete
                    </Button>
                    <Button size="sm" variant="destructive" onClick={() => onComplete(false)}>
                        Mark Failed
                    </Button>
                </div>
            </CardHeader>
            <CardContent className="flex-1 p-0">
                <div ref={terminalRef} className="w-full h-full bg-[#0d1117] p-2" />
            </CardContent>
        </Card>
    );
};

// Build task command for agent
function buildTaskCommand(task: Task): string {
    // Build a prompt for the agent based on the task
    // Construct prompt
    const prompt = [
        `Execute the following development task:`,
        ``,
        `Task ID: ${task.id}`,
        `Title: ${task.title}`,
        task.file ? `Target File: ${task.file}` : '',
        ``,
        `Description:`,
        task.description,
        ``,
        task.context_required.length > 0 ? `Required Context Files: ${task.context_required.join(', ')}` : '',
    ].filter(Boolean).join('\n');

    // Return the claude command with the prompt
    return `claude -p "${prompt.replace(/"/g, '\\"')}"`;
}

// Main Modal Component using shadcn Dialog
export const TaskDetailModal: React.FC<TaskDetailModalProps> = ({
    task,
    specName,
    onClose,
    onStatusChange,
}) => {
    const [selectedAgent, setSelectedAgent] = useState<AgentConfig | null>(null);
    const [showTerminal, setShowTerminal] = useState(false);

    const handleStartExecution = () => {
        if (!selectedAgent) return;
        onStatusChange(task.id, 'running');
        setShowTerminal(true);
    };

    const handleExecutionComplete = (success: boolean) => {
        onStatusChange(task.id, success ? 'completed' : 'pending');
        setShowTerminal(false);
        if (success) {
            onClose();
        }
    };

    const handleRetry = () => {
        // Reset to pending and show agent selection again
        onStatusChange(task.id, 'pending');
        setShowTerminal(false);
    };

    const handleMarkComplete = () => {
        onStatusChange(task.id, 'completed');
        onClose();
    };

    return (
        <Dialog open={true} onOpenChange={(open) => !open && onClose()}>
            <DialogContent className="max-w-4xl h-[85vh] flex flex-col p-0 gap-0">
                {/* Header */}
                <DialogHeader className="px-6 py-4 border-b border-border flex-row items-center justify-between space-y-0">
                    <div className="flex items-center gap-4">
                        <Badge variant="secondary" className="font-mono">{task.id}</Badge>
                        <StatusBadge status={task.status} />
                        {task.parallel && (
                            <Badge variant="success" className="flex items-center gap-1">
                                <GitBranch size={12} /> parallel
                            </Badge>
                        )}
                    </div>
                    <DialogTitle className="sr-only">{task.title}</DialogTitle>
                    <Button variant="ghost" size="icon" onClick={onClose}>
                        <X size={20} />
                    </Button>
                </DialogHeader>

                {/* Content */}
                <div className="flex-1 overflow-hidden flex flex-col">
                    {showTerminal ? (
                        <EmbeddedTerminal
                            agent={selectedAgent!}
                            task={task}
                            specName={specName}
                            onComplete={handleExecutionComplete}
                        />
                    ) : (
                        <div className="flex-1 overflow-auto p-6">
                            {/* Title */}
                            <h2 className="text-xl font-semibold text-foreground mb-4">{task.title}</h2>

                            {/* Metadata Row */}
                            <div className="flex items-center gap-4 flex-wrap mb-6">
                                <RiskBadge risk={task.risk} />
                                <ModelTierBadge tier={task.model_tier} />
                                <div className="flex items-center gap-1 text-muted-foreground text-sm">
                                    <Clock size={14} />
                                    {task.estimated_tokens} tokens
                                </div>
                                {task.user_story && (
                                    <Badge variant="info">{task.user_story}</Badge>
                                )}
                            </div>

                            {/* Target File */}
                            {task.file && (
                                <div className="mb-6">
                                    <h3 className="text-sm font-medium text-muted-foreground mb-2">Target File</h3>
                                    <code className="text-sm text-[var(--accent-cyan)] bg-[var(--accent-cyan-dim)] px-3 py-2 rounded block">
                                        {task.file}
                                    </code>
                                </div>
                            )}

                            {/* Description */}
                            <div className="mb-6">
                                <h3 className="text-sm font-medium text-muted-foreground mb-2">Description</h3>
                                <Card>
                                    <CardContent className="p-4">
                                        <p className="text-foreground whitespace-pre-wrap">
                                            {task.description}
                                        </p>
                                    </CardContent>
                                </Card>
                            </div>

                            {/* Dependencies */}
                            {task.context_required.length > 0 && (
                                <div className="mb-6">
                                    <h3 className="text-sm font-medium text-muted-foreground mb-2 flex items-center gap-2">
                                        <Link2 size={14} />
                                        Required Context
                                    </h3>
                                    <div className="flex flex-wrap gap-2">
                                        {task.context_required.map((dep, i) => (
                                            <Badge key={i} variant="secondary" className="font-mono">
                                                {dep}
                                            </Badge>
                                        ))}
                                    </div>
                                </div>
                            )}

                            {/* Agent Selection (only for pending/failed tasks) */}
                            {(task.status === 'pending' || task.status === 'failed') && (
                                <div className="mb-6">
                                    <h3 className="text-sm font-medium text-muted-foreground mb-2 flex items-center gap-2">
                                        <Bot size={14} />
                                        Select Agent
                                    </h3>
                                    <AgentSelector
                                        selectedAgent={selectedAgent}
                                        onSelect={setSelectedAgent}
                                        recommendedTier={task.model_tier}
                                    />
                                </div>
                            )}
                        </div>
                    )}
                </div>

                {/* Footer Actions */}
                {!showTerminal && (
                    <DialogFooter className="px-6 py-4 border-t border-border flex items-center justify-between sm:justify-between">
                        <div className="flex items-center gap-2">
                            {task.status === 'failed' && (
                                <Button variant="outline" onClick={handleRetry}>
                                    <RotateCcw size={16} className="mr-2" />
                                    Reset to Pending
                                </Button>
                            )}
                            {task.status === 'running' && (
                                <Button variant="outline" onClick={() => onStatusChange(task.id, 'pending')}>
                                    <RotateCcw size={16} className="mr-2" />
                                    Cancel
                                </Button>
                            )}
                        </div>

                        <div className="flex items-center gap-3">
                            {task.status === 'pending' && (
                                <>
                                    <Button variant="outline" onClick={handleMarkComplete}>
                                        <CheckCircle2 size={16} className="mr-2" />
                                        Mark Complete
                                    </Button>
                                    <Button onClick={handleStartExecution} disabled={!selectedAgent}>
                                        <Play size={16} className="mr-2" />
                                        Run Task
                                    </Button>
                                </>
                            )}
                            {task.status === 'completed' && (
                                <div className="text-[var(--accent-green)] flex items-center gap-2">
                                    <CheckCircle2 size={18} />
                                    Task completed
                                </div>
                            )}
                            {task.status === 'failed' && (
                                <Button onClick={handleStartExecution} disabled={!selectedAgent}>
                                    <RotateCcw size={16} className="mr-2" />
                                    Retry Task
                                </Button>
                            )}
                        </div>
                    </DialogFooter>
                )}
            </DialogContent>
        </Dialog>
    );
};
