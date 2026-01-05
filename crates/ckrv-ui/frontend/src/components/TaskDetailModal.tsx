import React, { useState, useEffect, useRef } from 'react';
import { useQuery } from '@tanstack/react-query';
import {
    X, Play, CheckCircle2, Circle, AlertTriangle, GitBranch,
    Zap, Brain, Cpu, Link2, Clock, Bot, ChevronDown, Loader2,
    RotateCcw, Terminal as TerminalIcon
} from 'lucide-react';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import '@xterm/xterm/css/xterm.css';
import type { AgentConfig } from './AgentManager';

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

// Badge Components
const RiskBadge: React.FC<{ risk: string }> = ({ risk }) => {
    const styles: Record<string, string> = {
        low: 'bg-green-900/50 text-green-300 border-green-700',
        medium: 'bg-amber-900/50 text-amber-300 border-amber-700',
        high: 'bg-orange-900/50 text-orange-300 border-orange-700',
        critical: 'bg-red-900/50 text-red-300 border-red-700'
    };
    return (
        <span className={`text-xs font-medium px-2 py-0.5 rounded border ${styles[risk] || styles.low}`}>
            {risk}
        </span>
    );
};

const ModelTierBadge: React.FC<{ tier: string }> = ({ tier }) => {
    const styles: Record<string, { bg: string; icon: React.ElementType }> = {
        light: { bg: 'bg-sky-900/50 text-sky-300', icon: Zap },
        standard: { bg: 'bg-violet-900/50 text-violet-300', icon: Cpu },
        heavy: { bg: 'bg-fuchsia-900/50 text-fuchsia-300', icon: Brain }
    };
    const { bg, icon: Icon } = styles[tier] || styles.standard;
    return (
        <span className={`text-xs font-medium px-2 py-0.5 rounded flex items-center gap-1 ${bg}`}>
            <Icon size={12} />
            {tier}
        </span>
    );
};

const StatusBadge: React.FC<{ status: string }> = ({ status }) => {
    const styles: Record<string, { bg: string; icon: React.ElementType; label: string }> = {
        pending: { bg: 'bg-gray-700 text-gray-300', icon: Circle, label: 'Pending' },
        running: { bg: 'bg-blue-900/50 text-blue-300', icon: Play, label: 'Running' },
        completed: { bg: 'bg-green-900/50 text-green-300', icon: CheckCircle2, label: 'Completed' },
        failed: { bg: 'bg-red-900/50 text-red-300', icon: AlertTriangle, label: 'Failed' }
    };
    const { bg, icon: Icon, label } = styles[status] || styles.pending;
    return (
        <div className={`text-sm font-medium px-3 py-1.5 rounded flex items-center gap-2 ${bg}`}>
            <Icon size={16} />
            {label}
        </div>
    );
};

// Agent Selector Component
const AgentSelector: React.FC<{
    selectedAgent: AgentConfig | null;
    onSelect: (agent: AgentConfig) => void;
    recommendedTier: string;
}> = ({ selectedAgent, onSelect, recommendedTier }) => {
    const [isOpen, setIsOpen] = useState(false);
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
            <div className="flex items-center gap-2 text-gray-500">
                <Loader2 size={16} className="animate-spin" />
                Loading agents...
            </div>
        );
    }

    if (agents.length === 0) {
        return (
            <div className="text-amber-400 text-sm flex items-center gap-2">
                <AlertTriangle size={16} />
                No agents configured. Add one in Agent Manager.
            </div>
        );
    }

    return (
        <div className="relative">
            <button
                onClick={() => setIsOpen(!isOpen)}
                className="w-full px-4 py-3 bg-gray-800 border border-gray-600 rounded-lg flex items-center justify-between hover:border-gray-500 transition-colors"
                title={selectedAgent?.description}
            >
                <div className="flex items-center gap-3">
                    <Bot size={20} className="text-cyan-400" />
                    <div className="text-left">
                        <div className="font-medium text-gray-200">
                            {selectedAgent?.name || 'Select Agent'}
                        </div>
                        {selectedAgent && (
                            <div className="text-xs text-gray-500">
                                {selectedAgent.agent_type} • Level {selectedAgent.level}
                            </div>
                        )}
                    </div>
                </div>
                <ChevronDown size={20} className={`text-gray-500 transition-transform ${isOpen ? 'rotate-180' : ''}`} />
            </button>

            {isOpen && (
                <div className="absolute z-50 mt-2 w-full bg-gray-800 border border-gray-700 rounded-lg shadow-xl overflow-hidden">
                    {agents.map(agent => {
                        const rec = getAgentRecommendation(agent);
                        return (
                            <button
                                key={agent.id}
                                onClick={() => {
                                    onSelect(agent);
                                    setIsOpen(false);
                                }}
                                className={`w-full px-4 py-3 flex items-center justify-between hover:bg-gray-700 transition-colors ${selectedAgent?.id === agent.id ? 'bg-gray-700' : ''
                                    }`}
                                title={agent.description}
                            >
                                <div className="flex items-center gap-3">
                                    <Bot size={18} className={agent.is_default ? 'text-amber-400' : 'text-gray-400'} />
                                    <div className="text-left">
                                        <div className="font-medium text-gray-200 flex items-center gap-2">
                                            {agent.name}
                                            {agent.is_default && (
                                                <span className="text-xs bg-amber-900/50 text-amber-300 px-1.5 py-0.5 rounded">default</span>
                                            )}
                                        </div>
                                        <div className="text-xs text-gray-500">
                                            {agent.agent_type} • Level {agent.level}
                                            {agent.openrouter?.model && ` • ${agent.openrouter.model}`}
                                        </div>
                                    </div>
                                </div>
                                {rec && (
                                    <span className={`text-xs px-2 py-0.5 rounded ${rec === 'recommended' ? 'bg-green-900/50 text-green-300' :
                                        rec === 'capable' ? 'bg-blue-900/50 text-blue-300' :
                                            'bg-amber-900/50 text-amber-300'
                                        }`}>
                                        {rec}
                                    </span>
                                )}
                            </button>
                        );
                    })}
                </div>
            )}
        </div>
    );
};

// Embedded Terminal Component
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
        <div className="flex flex-col h-full">
            <div className="flex items-center justify-between px-4 py-2 bg-gray-900 border-b border-gray-700">
                <div className="flex items-center gap-3">
                    <TerminalIcon size={16} className="text-gray-500" />
                    <span className="text-sm text-gray-400">Task Execution</span>
                    <span className={`w-2 h-2 rounded-full ${status === 'connected' ? 'bg-green-500' :
                        status === 'connecting' ? 'bg-amber-500 animate-pulse' :
                            status === 'error' ? 'bg-red-500' : 'bg-gray-500'
                        }`} />
                </div>
                <div className="flex items-center gap-2">
                    <button
                        onClick={() => onComplete(true)}
                        className="text-xs px-3 py-1 bg-green-600 hover:bg-green-500 text-white rounded"
                    >
                        Mark Complete
                    </button>
                    <button
                        onClick={() => onComplete(false)}
                        className="text-xs px-3 py-1 bg-red-600 hover:bg-red-500 text-white rounded"
                    >
                        Mark Failed
                    </button>
                </div>
            </div>
            <div ref={terminalRef} className="flex-1 bg-[#0d1117] p-2" />
        </div>
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

// Main Modal Component
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
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm">
            <div className="bg-gray-900 rounded-xl border border-gray-700 shadow-2xl w-[90vw] max-w-4xl h-[85vh] flex flex-col overflow-hidden">
                {/* Header */}
                <div className="shrink-0 px-6 py-4 border-b border-gray-700 flex items-center justify-between bg-gray-800/50">
                    <div className="flex items-center gap-4">
                        <span className="font-mono text-sm bg-gray-700 px-3 py-1 rounded text-gray-300">{task.id}</span>
                        <StatusBadge status={task.status} />
                        {task.parallel && (
                            <span className="text-xs bg-emerald-900/50 text-emerald-300 px-2 py-0.5 rounded flex items-center gap-1">
                                <GitBranch size={12} /> parallel
                            </span>
                        )}
                    </div>
                    <button
                        onClick={onClose}
                        className="p-2 hover:bg-gray-700 rounded-lg transition-colors"
                    >
                        <X size={20} className="text-gray-400" />
                    </button>
                </div>

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
                            <h2 className="text-xl font-semibold text-gray-100 mb-4">{task.title}</h2>

                            {/* Metadata Row */}
                            <div className="flex items-center gap-4 flex-wrap mb-6">
                                <RiskBadge risk={task.risk} />
                                <ModelTierBadge tier={task.model_tier} />
                                <div className="flex items-center gap-1 text-gray-500 text-sm">
                                    <Clock size={14} />
                                    {task.estimated_tokens} tokens
                                </div>
                                {task.user_story && (
                                    <span className="text-xs bg-purple-900/50 text-purple-300 px-2 py-0.5 rounded">
                                        {task.user_story}
                                    </span>
                                )}
                            </div>

                            {/* Target File */}
                            {task.file && (
                                <div className="mb-6">
                                    <h3 className="text-sm font-medium text-gray-400 mb-2">Target File</h3>
                                    <code className="text-sm text-cyan-400 bg-cyan-900/30 px-3 py-2 rounded block">
                                        {task.file}
                                    </code>
                                </div>
                            )}

                            {/* Description */}
                            <div className="mb-6">
                                <h3 className="text-sm font-medium text-gray-400 mb-2">Description</h3>
                                <p className="text-gray-300 whitespace-pre-wrap bg-gray-800/50 rounded-lg p-4 border border-gray-700">
                                    {task.description}
                                </p>
                            </div>

                            {/* Dependencies */}
                            {task.context_required.length > 0 && (
                                <div className="mb-6">
                                    <h3 className="text-sm font-medium text-gray-400 mb-2 flex items-center gap-2">
                                        <Link2 size={14} />
                                        Required Context
                                    </h3>
                                    <div className="flex flex-wrap gap-2">
                                        {task.context_required.map((dep, i) => (
                                            <code key={i} className="text-xs bg-gray-700 text-gray-300 px-2 py-1 rounded">
                                                {dep}
                                            </code>
                                        ))}
                                    </div>
                                </div>
                            )}

                            {/* Agent Selection (only for pending/failed tasks) */}
                            {(task.status === 'pending' || task.status === 'failed') && (
                                <div className="mb-6">
                                    <h3 className="text-sm font-medium text-gray-400 mb-2 flex items-center gap-2">
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
                    <div className="shrink-0 px-6 py-4 border-t border-gray-700 bg-gray-800/50 flex items-center justify-between">
                        <div className="flex items-center gap-2">
                            {task.status === 'failed' && (
                                <button
                                    onClick={handleRetry}
                                    className="flex items-center gap-2 px-4 py-2 bg-gray-700 hover:bg-gray-600 text-gray-200 rounded-lg transition-colors"
                                >
                                    <RotateCcw size={16} />
                                    Reset to Pending
                                </button>
                            )}
                            {task.status === 'running' && (
                                <button
                                    onClick={() => onStatusChange(task.id, 'pending')}
                                    className="flex items-center gap-2 px-4 py-2 bg-amber-600 hover:bg-amber-500 text-white rounded-lg transition-colors"
                                >
                                    <RotateCcw size={16} />
                                    Cancel
                                </button>
                            )}
                        </div>

                        <div className="flex items-center gap-3">
                            {task.status === 'pending' && (
                                <>
                                    <button
                                        onClick={handleMarkComplete}
                                        className="flex items-center gap-2 px-4 py-2 bg-gray-700 hover:bg-gray-600 text-gray-200 rounded-lg transition-colors"
                                    >
                                        <CheckCircle2 size={16} />
                                        Mark Complete
                                    </button>
                                    <button
                                        onClick={handleStartExecution}
                                        disabled={!selectedAgent}
                                        className={`flex items-center gap-2 px-6 py-2 rounded-lg font-medium transition-colors ${selectedAgent
                                            ? 'bg-cyan-600 hover:bg-cyan-500 text-white'
                                            : 'bg-gray-800 text-gray-500 cursor-not-allowed'
                                            }`}
                                    >
                                        <Play size={16} />
                                        Run Task
                                    </button>
                                </>
                            )}
                            {task.status === 'completed' && (
                                <div className="text-green-400 flex items-center gap-2">
                                    <CheckCircle2 size={18} />
                                    Task completed
                                </div>
                            )}
                            {task.status === 'failed' && (
                                <button
                                    onClick={handleStartExecution}
                                    disabled={!selectedAgent}
                                    className={`flex items-center gap-2 px-6 py-2 rounded-lg font-medium transition-colors ${selectedAgent
                                        ? 'bg-cyan-600 hover:bg-cyan-500 text-white'
                                        : 'bg-gray-800 text-gray-500 cursor-not-allowed'
                                        }`}
                                >
                                    <RotateCcw size={16} />
                                    Retry Task
                                </button>
                            )}
                        </div>
                    </div>
                )}
            </div>
        </div>
    );
};
