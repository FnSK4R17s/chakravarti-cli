import React, { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
    Bot,
    Plus,
    Trash2,
    Star,
    StarOff,
    Settings2,
    Zap,
    Check,
    X,
    Loader2,
    ExternalLink,
    Key,
    Cpu,
    ChevronDown,
    ChevronRight,
    Sparkles,
    TestTube,
    AlertCircle,
    Terminal
} from 'lucide-react';
import { AgentCliModal } from './AgentCliModal';

// Types
type AgentType = 'claude' | 'claude_open_router' | 'gemini' | 'codex' | 'cursor' | 'amp' | 'qwen_code' | 'opencode' | 'factory_droid' | 'copilot';

interface OpenRouterConfig {
    api_key?: string;
    model: string;
    base_url?: string;
    max_tokens?: number;
    temperature?: number;
}

export interface AgentConfig {
    id: string;
    name: string;
    agent_type: AgentType;
    level: number;          // 1-5, capability level (5 = strongest)
    is_default: boolean;
    enabled: boolean;
    description?: string;
    openrouter?: OpenRouterConfig;
    binary_path?: string;
    extra_args?: string[];
    env_vars?: Record<string, string>;
}

interface OpenRouterModel {
    id: string;
    name: string;
    description: string;
    context_length?: number;
    pricing?: string;
}

// API functions
const fetchAgents = async (): Promise<{ agents: AgentConfig[] }> => {
    const res = await fetch('/api/agents');
    return res.json();
};

const fetchModels = async (): Promise<{ models: OpenRouterModel[] }> => {
    const res = await fetch('/api/agents/models');
    return res.json();
};

const upsertAgent = async (agent: AgentConfig) => {
    const res = await fetch('/api/agents/upsert', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ agent }),
    });
    return res.json();
};

const deleteAgent = async (id: string) => {
    const res = await fetch('/api/agents/delete', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ id }),
    });
    return res.json();
};

const setDefaultAgent = async (id: string) => {
    const res = await fetch('/api/agents/set-default', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ id }),
    });
    return res.json();
};

const testAgent = async (agent: AgentConfig) => {
    const res = await fetch('/api/agents/test', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ agent }),
    });
    return res.json();
};


// Agent type display info
const AGENT_TYPE_INFO: Record<AgentType, { label: string; icon: React.ReactNode; color: string }> = {
    claude: { label: 'Claude Code', icon: <Bot size={16} />, color: 'var(--accent-amber)' },
    claude_open_router: { label: 'Claude + OpenRouter', icon: <Sparkles size={16} />, color: 'var(--accent-purple)' },
    gemini: { label: 'Gemini CLI', icon: <Cpu size={16} />, color: 'var(--accent-cyan)' },
    codex: { label: 'OpenAI Codex', icon: <Zap size={16} />, color: 'var(--accent-green)' },
    cursor: { label: 'Cursor CLI', icon: <Bot size={16} />, color: 'var(--accent-pink)' },
    amp: { label: 'Amp', icon: <Zap size={16} />, color: 'var(--accent-amber)' },
    qwen_code: { label: 'Qwen Code', icon: <Bot size={16} />, color: 'var(--accent-cyan)' },
    opencode: { label: 'Opencode', icon: <Bot size={16} />, color: 'var(--accent-green)' },
    factory_droid: { label: 'Factory Droid', icon: <Bot size={16} />, color: 'var(--accent-purple)' },
    copilot: { label: 'GitHub Copilot', icon: <Bot size={16} />, color: 'var(--text-primary)' },
};

export const AgentManager: React.FC = () => {
    const queryClient = useQueryClient();
    const [showAddModal, setShowAddModal] = useState(false);
    const [editingAgent, setEditingAgent] = useState<AgentConfig | null>(null);
    const [expandedAgent, setExpandedAgent] = useState<string | null>(null);
    const [cliAgent, setCliAgent] = useState<AgentConfig | null>(null);
    const [testResult, setTestResult] = useState<{ agentId: string; success: boolean; message: string } | null>(null);

    const { data: agentsData, isLoading } = useQuery({
        queryKey: ['agents'],
        queryFn: fetchAgents,
        refetchInterval: 5000,
    });

    const { data: modelsData } = useQuery({
        queryKey: ['openrouter-models'],
        queryFn: fetchModels,
    });

    const upsertMutation = useMutation({
        mutationFn: upsertAgent,
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['agents'] });
            setShowAddModal(false);
            setEditingAgent(null);
        },
    });

    const deleteMutation = useMutation({
        mutationFn: deleteAgent,
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['agents'] });
        },
    });

    const setDefaultMutation = useMutation({
        mutationFn: setDefaultAgent,
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['agents'] });
        },
    });

    const testMutation = useMutation({
        mutationFn: testAgent,
        onSuccess: (data, agent) => {
            setTestResult({
                agentId: agent.id,
                success: data.success,
                message: data.message,
            });
            setTimeout(() => setTestResult(null), 5000);
        },
    });

    const agents = agentsData?.agents || [];
    const models = modelsData?.models || [];

    return (
        <div className="flex flex-col h-full rounded-lg overflow-hidden" style={{ background: 'var(--bg-secondary)', border: '1px solid var(--border-subtle)' }}>
            {/* Header */}
            <div
                className="px-4 py-3 flex items-center justify-between shrink-0"
                style={{ borderBottom: '1px solid var(--border-subtle)' }}
            >
                <div className="flex items-center gap-2">
                    <Bot size={18} style={{ color: 'var(--accent-purple)' }} />
                    <h3 className="text-sm font-semibold" style={{ color: 'var(--text-primary)' }}>
                        Agent Manager
                    </h3>
                    <span
                        className="text-xs px-1.5 py-0.5 rounded"
                        style={{ background: 'var(--bg-tertiary)', color: 'var(--text-muted)' }}
                    >
                        {agents.length} agents
                    </span>
                </div>
                <button
                    onClick={() => {
                        setEditingAgent(null);
                        setShowAddModal(true);
                    }}
                    className="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-xs font-medium transition-all"
                    style={{
                        background: 'var(--accent-purple-dim)',
                        color: 'var(--accent-purple)',
                    }}
                >
                    <Plus size={14} />
                    Add Agent
                </button>
            </div>

            {/* Agent List */}
            <div className="flex-1 overflow-y-auto p-4 custom-scrollbar">
                {isLoading ? (
                    <div className="flex items-center justify-center h-32">
                        <Loader2 className="animate-spin" size={24} style={{ color: 'var(--text-muted)' }} />
                    </div>
                ) : agents.length === 0 ? (
                    <div className="flex flex-col items-center justify-center h-32 gap-2">
                        <Bot size={32} style={{ color: 'var(--text-muted)' }} />
                        <p className="text-sm" style={{ color: 'var(--text-muted)' }}>No agents configured</p>
                    </div>
                ) : (
                    <div className="space-y-3">
                        {agents.map((agent) => (
                            <AgentCard
                                key={agent.id}
                                agent={agent}
                                expanded={expandedAgent === agent.id}
                                onToggleExpand={() => setExpandedAgent(expandedAgent === agent.id ? null : agent.id)}
                                onEdit={() => {
                                    setEditingAgent(agent);
                                    setShowAddModal(true);
                                }}
                                onDelete={() => deleteMutation.mutate(agent.id)}
                                onSetDefault={() => setDefaultMutation.mutate(agent.id)}
                                onTest={() => testMutation.mutate(agent)}
                                onCli={() => setCliAgent(agent)}
                                isDeleting={deleteMutation.isPending}
                                isTesting={testMutation.isPending}
                                testResult={testResult?.agentId === agent.id ? testResult : null}
                            />
                        ))}
                    </div>
                )}
            </div>

            {/* Footer */}
            <div
                className="px-4 py-2 flex items-center justify-between text-xs shrink-0"
                style={{
                    borderTop: '1px solid var(--border-subtle)',
                    background: 'var(--bg-tertiary)',
                    color: 'var(--text-muted)'
                }}
            >
                <span className="font-mono truncate">
                    OpenRouter models: {models.length}
                </span>
                <a
                    href="https://openrouter.ai/models"
                    target="_blank"
                    rel="noopener noreferrer"
                    className="flex items-center gap-1 hover:underline"
                    style={{ color: 'var(--accent-cyan)' }}
                >
                    Browse models <ExternalLink size={10} />
                </a>
            </div>

            {/* Add/Edit Modal */}
            {showAddModal && (
                <AgentModal
                    agent={editingAgent}
                    models={models}
                    onClose={() => {
                        setShowAddModal(false);
                        setEditingAgent(null);
                    }}
                    onSave={(agent) => upsertMutation.mutate(agent)}
                    isLoading={upsertMutation.isPending}
                />
            )}

            {/* CLI Modal */}
            {cliAgent && (
                <AgentCliModal
                    agent={cliAgent}
                    onClose={() => setCliAgent(null)}
                />
            )}
        </div>
    );
};

// Agent Card Component
interface AgentCardProps {
    agent: AgentConfig;
    expanded: boolean;
    onToggleExpand: () => void;
    onEdit: () => void;
    onDelete: () => void;
    onSetDefault: () => void;
    onTest: () => void;
    onCli: () => void;
    isDeleting: boolean;
    isTesting: boolean;
    testResult: { success: boolean; message: string } | null;
}

const AgentCard: React.FC<AgentCardProps> = ({
    agent,
    expanded,
    onToggleExpand,
    onEdit,
    onDelete,
    onSetDefault,
    onTest,
    onCli,
    isDeleting,
    isTesting,
    testResult,
}) => {
    const typeInfo = AGENT_TYPE_INFO[agent.agent_type] || AGENT_TYPE_INFO.claude;

    return (
        <div
            className="rounded-lg overflow-hidden transition-all"
            style={{
                background: 'var(--bg-tertiary)',
                border: agent.is_default ? `1px solid ${typeInfo.color}` : '1px solid var(--border-subtle)',
            }}
        >
            {/* Main row */}
            <div className="px-4 py-3 flex items-center gap-3">
                <button
                    onClick={onToggleExpand}
                    className="shrink-0 p-1 rounded hover:bg-white/5 transition-colors"
                >
                    {expanded ? (
                        <ChevronDown size={14} style={{ color: 'var(--text-muted)' }} />
                    ) : (
                        <ChevronRight size={14} style={{ color: 'var(--text-muted)' }} />
                    )}
                </button>

                <div
                    className="w-8 h-8 rounded-lg flex items-center justify-center shrink-0"
                    style={{ background: `${typeInfo.color}20`, color: typeInfo.color }}
                >
                    {typeInfo.icon}
                </div>

                <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2">
                        <span className="font-medium text-sm truncate" style={{ color: 'var(--text-primary)' }}>
                            {agent.name}
                        </span>
                        {/* Level Badge */}
                        <span
                            className="text-[10px] px-1.5 py-0.5 rounded font-bold"
                            style={{
                                background: 'var(--bg-tertiary)',
                                color: agent.level >= 4 ? 'var(--accent-purple)' : 'var(--text-muted)'
                            }}
                        >
                            L{agent.level || 3}
                        </span>
                        {agent.is_default && (
                            <span
                                className="text-[10px] px-1.5 py-0.5 rounded-full font-medium"
                                style={{ background: `${typeInfo.color}20`, color: typeInfo.color }}
                            >
                                DEFAULT
                            </span>
                        )}
                        {!agent.enabled && (
                            <span
                                className="text-[10px] px-1.5 py-0.5 rounded-full font-medium"
                                style={{ background: 'var(--accent-red-dim)', color: 'var(--accent-red)' }}
                            >
                                DISABLED
                            </span>
                        )}
                    </div>
                    <p className="text-xs truncate" style={{ color: 'var(--text-muted)' }}>
                        {typeInfo.label}
                        {agent.agent_type === 'claude_open_router' && agent.openrouter && (
                            <> • {agent.openrouter.model}</>
                        )}
                    </p>
                </div>

                {/* Test Result */}
                {testResult && (
                    <div
                        className="flex items-center gap-1.5 px-2 py-1 rounded text-xs animate-fade-in"
                        style={{
                            background: testResult.success ? 'var(--accent-green-dim)' : 'var(--accent-red-dim)',
                            color: testResult.success ? 'var(--accent-green)' : 'var(--accent-red)',
                        }}
                    >
                        {testResult.success ? <Check size={12} /> : <AlertCircle size={12} />}
                        <span className="truncate max-w-[100px]">{testResult.message}</span>
                    </div>
                )}

                {/* Actions */}
                <div className="flex items-center gap-1 shrink-0">
                    <button
                        onClick={onCli}
                        className="p-1.5 rounded hover:bg-white/5 transition-colors"
                        title="Open Interactive CLI"
                    >
                        <Terminal size={14} style={{ color: 'var(--text-muted)' }} />
                    </button>
                    <button
                        onClick={onTest}
                        disabled={isTesting}
                        className="p-1.5 rounded hover:bg-white/5 transition-colors"
                        title="Test agent"
                    >
                        {isTesting ? (
                            <Loader2 size={14} className="animate-spin" style={{ color: 'var(--text-muted)' }} />
                        ) : (
                            <TestTube size={14} style={{ color: 'var(--text-muted)' }} />
                        )}
                    </button>
                    <button
                        onClick={onSetDefault}
                        className="p-1.5 rounded hover:bg-white/5 transition-colors"
                        title={agent.is_default ? 'Default agent' : 'Set as default'}
                    >
                        {agent.is_default ? (
                            <Star size={14} style={{ color: typeInfo.color }} />
                        ) : (
                            <StarOff size={14} style={{ color: 'var(--text-muted)' }} />
                        )}
                    </button>
                    <button
                        onClick={onEdit}
                        className="p-1.5 rounded hover:bg-white/5 transition-colors"
                        title="Edit agent"
                    >
                        <Settings2 size={14} style={{ color: 'var(--text-muted)' }} />
                    </button>
                    <button
                        onClick={onDelete}
                        disabled={isDeleting || agent.is_default}
                        className="p-1.5 rounded hover:bg-white/5 transition-colors disabled:opacity-50"
                        title={agent.is_default ? 'Cannot delete default agent' : 'Delete agent'}
                    >
                        <Trash2 size={14} style={{ color: 'var(--accent-red)' }} />
                    </button>
                </div>
            </div>

            {/* Expanded details */}
            {expanded && (
                <div
                    className="px-4 py-3 border-t text-xs space-y-2"
                    style={{ borderColor: 'var(--border-subtle)', background: 'var(--bg-secondary)' }}
                >
                    {agent.description && (
                        <p style={{ color: 'var(--text-secondary)' }}>{agent.description}</p>
                    )}

                    {agent.agent_type === 'claude_open_router' && agent.openrouter && (
                        <div className="space-y-1">
                            <div className="flex items-center gap-2">
                                <span style={{ color: 'var(--text-muted)' }}>Model:</span>
                                <code className="px-1.5 py-0.5 rounded" style={{ background: 'var(--bg-tertiary)', color: 'var(--accent-cyan)' }}>
                                    {agent.openrouter.model}
                                </code>
                            </div>
                            {agent.openrouter.api_key && (
                                <div className="flex items-center gap-2">
                                    <span style={{ color: 'var(--text-muted)' }}>API Key:</span>
                                    <code className="px-1.5 py-0.5 rounded" style={{ background: 'var(--bg-tertiary)', color: 'var(--text-secondary)' }}>
                                        ••••••••{agent.openrouter.api_key.slice(-4)}
                                    </code>
                                </div>
                            )}
                            {agent.openrouter.base_url && (
                                <div className="flex items-center gap-2">
                                    <span style={{ color: 'var(--text-muted)' }}>Base URL:</span>
                                    <code className="px-1.5 py-0.5 rounded truncate" style={{ background: 'var(--bg-tertiary)', color: 'var(--text-secondary)' }}>
                                        {agent.openrouter.base_url}
                                    </code>
                                </div>
                            )}
                        </div>
                    )}

                    {agent.binary_path && (
                        <div className="flex items-center gap-2">
                            <span style={{ color: 'var(--text-muted)' }}>Binary:</span>
                            <code className="px-1.5 py-0.5 rounded" style={{ background: 'var(--bg-tertiary)', color: 'var(--text-secondary)' }}>
                                {agent.binary_path}
                            </code>
                        </div>
                    )}
                </div>
            )}
        </div>
    );
};

// Agent Modal Component
interface AgentModalProps {
    agent: AgentConfig | null;
    models: OpenRouterModel[];
    onClose: () => void;
    onSave: (agent: AgentConfig) => void;
    isLoading: boolean;
}

const AgentModal: React.FC<AgentModalProps> = ({ agent, models, onClose, onSave, isLoading }) => {
    const [form, setForm] = useState<AgentConfig>(() =>
        agent || {
            id: `agent-${Date.now()}`,
            name: '',
            agent_type: 'claude_open_router',
            level: 3,  // Default to mid-tier
            is_default: false,
            enabled: true,
            description: '',
            openrouter: {
                model: 'anthropic/claude-sonnet-4',
                api_key: '',
            },
        }
    );

    // Extract provider from model ID (e.g., "anthropic/claude-sonnet-4" -> "anthropic")
    const getProvider = (modelId: string) => modelId.split('/')[0] || 'unknown';

    // Get current provider from the selected model
    const currentModelProvider = form.openrouter?.model ? getProvider(form.openrouter.model) : 'anthropic';
    const [selectedProvider, setSelectedProvider] = useState(currentModelProvider);

    // Get unique providers from models
    const providers = [...new Set(models.map(m => getProvider(m.id)))].sort((a, b) => {
        // Prioritize popular providers
        const priority = (p: string) => {
            if (p === 'anthropic') return 0;
            if (p === 'openai') return 1;
            if (p === 'google') return 2;
            if (p === 'deepseek') return 3;
            if (p === 'meta-llama') return 4;
            if (p === 'mistralai') return 5;
            if (p === 'qwen') return 6;
            return 10;
        };
        return priority(a) - priority(b);
    });

    // Get models for selected provider
    const filteredModels = models.filter(m => getProvider(m.id) === selectedProvider);

    // Format provider name for display
    const formatProvider = (provider: string) => {
        const names: Record<string, string> = {
            'anthropic': 'Anthropic',
            'openai': 'OpenAI',
            'google': 'Google',
            'deepseek': 'DeepSeek',
            'meta-llama': 'Meta (Llama)',
            'mistralai': 'Mistral AI',
            'qwen': 'Qwen (Alibaba)',
            'minimax': 'MiniMax',
            'moonshot': 'Moonshot',
            'cohere': 'Cohere',
            'x-ai': 'xAI (Grok)',
            'zhipu': 'Zhipu (GLM)',
        };
        return names[provider] || provider.charAt(0).toUpperCase() + provider.slice(1);
    };

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        // Ensure we have a valid model selected before saving
        const finalForm = { ...form };
        if (finalForm.agent_type === 'claude_open_router' && finalForm.openrouter) {
            // If current model is not in the list, pick the first available
            const currentModelExists = models.some(m => m.id === finalForm.openrouter?.model);
            if (!currentModelExists && filteredModels.length > 0) {
                finalForm.openrouter = { ...finalForm.openrouter, model: filteredModels[0].id };
            } else if (!currentModelExists && models.length > 0) {
                finalForm.openrouter = { ...finalForm.openrouter, model: models[0].id };
            }
        }
        onSave(finalForm);
    };

    const isOpenRouter = form.agent_type === 'claude_open_router';

    // Sync selected model with provider when filteredModels changes
    React.useEffect(() => {
        if (isOpenRouter && filteredModels.length > 0) {
            const currentModelInProvider = filteredModels.some(m => m.id === form.openrouter?.model);
            if (!currentModelInProvider) {
                setForm(f => ({
                    ...f,
                    openrouter: { ...f.openrouter!, model: filteredModels[0].id },
                }));
            }
        }
    }, [selectedProvider, filteredModels.length, isOpenRouter]);

    return (
        <div
            className="fixed inset-0 z-50 flex items-center justify-center p-4"
            style={{ background: 'rgba(0,0,0,0.7)' }}
            onClick={onClose}
        >
            <div
                className="w-full max-w-lg rounded-xl overflow-hidden shadow-2xl"
                style={{ background: 'var(--bg-secondary)', border: '1px solid var(--border-default)' }}
                onClick={(e) => e.stopPropagation()}
            >
                <form onSubmit={handleSubmit}>
                    {/* Header */}
                    <div
                        className="px-6 py-4 flex items-center justify-between"
                        style={{ borderBottom: '1px solid var(--border-subtle)' }}
                    >
                        <h3 className="text-lg font-semibold" style={{ color: 'var(--text-primary)' }}>
                            {agent ? 'Edit Agent' : 'Add New Agent'}
                        </h3>
                        <button
                            type="button"
                            onClick={onClose}
                            className="p-1 rounded hover:bg-white/5 transition-colors"
                        >
                            <X size={18} style={{ color: 'var(--text-muted)' }} />
                        </button>
                    </div>

                    {/* Body */}
                    <div className="px-6 py-4 space-y-4 max-h-[60vh] overflow-y-auto custom-scrollbar">
                        {/* Name */}
                        <div>
                            <label className="block text-xs font-medium mb-1.5" style={{ color: 'var(--text-secondary)' }}>
                                Name
                            </label>
                            <input
                                type="text"
                                value={form.name}
                                onChange={(e) => setForm({ ...form, name: e.target.value })}
                                placeholder="My Custom Agent"
                                required
                                className="w-full px-3 py-2 rounded-lg text-sm outline-none transition-all focus:ring-2"
                                style={{
                                    background: 'var(--bg-tertiary)',
                                    border: '1px solid var(--border-subtle)',
                                    color: 'var(--text-primary)',
                                }}
                            />
                        </div>

                        {/* Agent Type */}
                        <div>
                            <label className="block text-xs font-medium mb-1.5" style={{ color: 'var(--text-secondary)' }}>
                                Agent Type
                            </label>
                            <select
                                value={form.agent_type}
                                onChange={(e) => setForm({
                                    ...form,
                                    agent_type: e.target.value as AgentType,
                                    openrouter: e.target.value === 'claude_open_router' ? form.openrouter || { model: 'anthropic/claude-sonnet-4' } : undefined,
                                })}
                                className="w-full px-3 py-2 rounded-lg text-sm outline-none transition-all focus:ring-2"
                                style={{
                                    background: 'var(--bg-tertiary)',
                                    border: '1px solid var(--border-subtle)',
                                    color: 'var(--text-primary)',
                                }}
                            >
                                <option value="claude">Claude Code (Default CLI)</option>
                                <option value="claude_open_router">Claude Code + OpenRouter</option>
                                <option value="gemini">Gemini CLI</option>
                                <option value="codex">OpenAI Codex</option>
                                <option value="cursor">Cursor CLI</option>
                                <option value="amp">Amp</option>
                                <option value="qwen_code">Qwen Code</option>
                                <option value="opencode">Opencode</option>
                                <option value="copilot">GitHub Copilot</option>
                            </select>
                        </div>

                        {/* Capability Level */}
                        <div>
                            <label className="block text-xs font-medium mb-1.5" style={{ color: 'var(--text-secondary)' }}>
                                Capability Level
                                <span className="ml-2 text-[10px] font-normal" style={{ color: 'var(--text-muted)' }}>
                                    (1=lightest, 5=strongest)
                                </span>
                            </label>
                            <div className="flex items-center gap-2">
                                {[1, 2, 3, 4, 5].map((level) => (
                                    <button
                                        key={level}
                                        type="button"
                                        onClick={() => setForm({ ...form, level })}
                                        className="flex-1 py-2 rounded-lg text-sm font-bold transition-all"
                                        style={{
                                            background: form.level === level ? 'var(--accent-purple)' : 'var(--bg-tertiary)',
                                            color: form.level === level ? 'var(--bg-primary)' : 'var(--text-secondary)',
                                            border: form.level === level ? 'none' : '1px solid var(--border-subtle)',
                                        }}
                                    >
                                        {level}
                                    </button>
                                ))}
                            </div>
                            <p className="text-[10px] mt-1" style={{ color: 'var(--text-muted)' }}>
                                {form.level === 1 && '📄 Simple files, boilerplate, configs'}
                                {form.level === 2 && '🔧 Basic implementations, CRUD'}
                                {form.level === 3 && '⚙️ Standard development tasks'}
                                {form.level === 4 && '🧠 Complex logic, architecture'}
                                {form.level === 5 && '🚀 Planning, reasoning, critical tasks'}
                            </p>
                        </div>

                        {/* OpenRouter Config */}
                        {isOpenRouter && (
                            <div
                                className="p-4 rounded-lg space-y-4"
                                style={{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-subtle)' }}
                            >
                                <div className="flex items-center gap-2 mb-2">
                                    <Key size={14} style={{ color: 'var(--accent-purple)' }} />
                                    <span className="text-xs font-medium" style={{ color: 'var(--accent-purple)' }}>
                                        OpenRouter Configuration
                                    </span>
                                </div>

                                {/* Model Selection - Provider + Model */}
                                <div className="space-y-3">
                                    <div className="flex gap-3">
                                        {/* Provider Selector */}
                                        <div className="flex-1">
                                            <label className="block text-xs font-medium mb-1.5" style={{ color: 'var(--text-secondary)' }}>
                                                Provider
                                            </label>
                                            <select
                                                value={selectedProvider}
                                                onChange={(e) => {
                                                    const newProvider = e.target.value;
                                                    setSelectedProvider(newProvider);
                                                    // Auto-select first model from new provider
                                                    const firstModel = models.find(m => m.id.startsWith(newProvider + '/'));
                                                    if (firstModel) {
                                                        setForm({
                                                            ...form,
                                                            openrouter: { ...form.openrouter!, model: firstModel.id },
                                                        });
                                                    }
                                                }}
                                                className="w-full px-3 py-2.5 rounded-lg text-sm font-medium outline-none transition-all cursor-pointer hover:border-[var(--accent-purple)]"
                                                style={{
                                                    background: 'var(--bg-primary)',
                                                    border: '1px solid var(--border-default)',
                                                    color: 'var(--text-primary)',
                                                    boxShadow: 'inset 0 1px 2px rgba(0,0,0,0.1)',
                                                }}
                                            >
                                                {providers.map((provider) => (
                                                    <option key={provider} value={provider}>
                                                        {formatProvider(provider)}
                                                    </option>
                                                ))}
                                            </select>
                                        </div>

                                        {/* Model Selector */}
                                        <div className="flex-[2]">
                                            <label className="block text-xs font-medium mb-1.5" style={{ color: 'var(--text-secondary)' }}>
                                                Model
                                            </label>
                                            <select
                                                value={form.openrouter?.model || ''}
                                                onChange={(e) => setForm({
                                                    ...form,
                                                    openrouter: { ...form.openrouter!, model: e.target.value },
                                                })}
                                                className="w-full px-3 py-2.5 rounded-lg text-sm font-medium outline-none transition-all cursor-pointer hover:border-[var(--accent-purple)]"
                                                style={{
                                                    background: 'var(--bg-primary)',
                                                    border: '1px solid var(--border-default)',
                                                    color: 'var(--text-primary)',
                                                    boxShadow: 'inset 0 1px 2px rgba(0,0,0,0.1)',
                                                }}
                                            >
                                                {filteredModels.map((model) => (
                                                    <option key={model.id} value={model.id}>
                                                        {model.name.replace(/^[^:]+:\s*/, '')}
                                                    </option>
                                                ))}
                                            </select>
                                        </div>
                                    </div>

                                    {/* Model info card */}
                                    {models.find(m => m.id === form.openrouter?.model) && (
                                        <div
                                            className="text-xs p-3 rounded-lg space-y-2"
                                            style={{
                                                background: 'var(--bg-primary)',
                                                border: '1px solid var(--border-subtle)',
                                            }}
                                        >
                                            <div className="flex items-center justify-between">
                                                <span className="font-mono text-[10px] px-1.5 py-0.5 rounded" style={{ background: 'var(--bg-tertiary)', color: 'var(--accent-cyan)' }}>
                                                    {form.openrouter?.model}
                                                </span>
                                                {models.find(m => m.id === form.openrouter?.model)?.pricing && (
                                                    <span style={{ color: 'var(--accent-green)' }}>
                                                        {models.find(m => m.id === form.openrouter?.model)?.pricing}
                                                    </span>
                                                )}
                                            </div>
                                            <p style={{ color: 'var(--text-muted)' }}>
                                                {models.find(m => m.id === form.openrouter?.model)?.description || 'No description available'}
                                            </p>
                                            {models.find(m => m.id === form.openrouter?.model)?.context_length && (
                                                <div className="flex items-center gap-2 pt-1" style={{ borderTop: '1px solid var(--border-subtle)' }}>
                                                    <span style={{ color: 'var(--text-muted)' }}>Context window:</span>
                                                    <span className="font-medium" style={{ color: 'var(--accent-purple)' }}>
                                                        {(models.find(m => m.id === form.openrouter?.model)?.context_length || 0).toLocaleString()} tokens
                                                    </span>
                                                </div>
                                            )}
                                        </div>
                                    )}
                                </div>

                                {/* API Key */}
                                <div>
                                    <label className="block text-xs font-medium mb-1.5" style={{ color: 'var(--text-secondary)' }}>
                                        OpenRouter API Key
                                    </label>
                                    <input
                                        type="password"
                                        value={form.openrouter?.api_key || ''}
                                        onChange={(e) => setForm({
                                            ...form,
                                            openrouter: { ...form.openrouter!, api_key: e.target.value },
                                        })}
                                        placeholder="sk-or-..."
                                        className="w-full px-3 py-2 rounded-lg text-sm font-mono outline-none transition-all focus:ring-2"
                                        style={{
                                            background: 'var(--bg-secondary)',
                                            border: '1px solid var(--border-subtle)',
                                            color: 'var(--text-primary)',
                                        }}
                                    />
                                    <p className="text-xs mt-1 flex items-center gap-1" style={{ color: 'var(--text-muted)' }}>
                                        Get your key from{' '}
                                        <a
                                            href="https://openrouter.ai/keys"
                                            target="_blank"
                                            rel="noopener noreferrer"
                                            className="flex items-center gap-0.5 hover:underline"
                                            style={{ color: 'var(--accent-cyan)' }}
                                        >
                                            openrouter.ai/keys <ExternalLink size={10} />
                                        </a>
                                    </p>
                                </div>

                                {/* Custom Base URL (optional) */}
                                <div>
                                    <label className="block text-xs font-medium mb-1.5" style={{ color: 'var(--text-secondary)' }}>
                                        Custom Base URL <span style={{ color: 'var(--text-muted)' }}>(optional)</span>
                                    </label>
                                    <input
                                        type="url"
                                        value={form.openrouter?.base_url || ''}
                                        onChange={(e) => setForm({
                                            ...form,
                                            openrouter: { ...form.openrouter!, base_url: e.target.value || undefined },
                                        })}
                                        placeholder="https://openrouter.ai/api"
                                        className="w-full px-3 py-2 rounded-lg text-sm font-mono outline-none transition-all focus:ring-2"
                                        style={{
                                            background: 'var(--bg-secondary)',
                                            border: '1px solid var(--border-subtle)',
                                            color: 'var(--text-primary)',
                                        }}
                                    />
                                </div>
                            </div>
                        )}

                        {/* Description */}
                        <div>
                            <label className="block text-xs font-medium mb-1.5" style={{ color: 'var(--text-secondary)' }}>
                                Description <span style={{ color: 'var(--text-muted)' }}>(optional)</span>
                            </label>
                            <textarea
                                value={form.description || ''}
                                onChange={(e) => setForm({ ...form, description: e.target.value || undefined })}
                                placeholder="My agent for..."
                                rows={2}
                                className="w-full px-3 py-2 rounded-lg text-sm outline-none transition-all focus:ring-2 resize-none"
                                style={{
                                    background: 'var(--bg-tertiary)',
                                    border: '1px solid var(--border-subtle)',
                                    color: 'var(--text-primary)',
                                }}
                            />
                        </div>

                        {/* Custom Binary Path (for non-OpenRouter) */}
                        {!isOpenRouter && (
                            <div>
                                <label className="block text-xs font-medium mb-1.5" style={{ color: 'var(--text-secondary)' }}>
                                    Custom Binary Path <span style={{ color: 'var(--text-muted)' }}>(optional)</span>
                                </label>
                                <input
                                    type="text"
                                    value={form.binary_path || ''}
                                    onChange={(e) => setForm({ ...form, binary_path: e.target.value || undefined })}
                                    placeholder="/usr/local/bin/claude"
                                    className="w-full px-3 py-2 rounded-lg text-sm font-mono outline-none transition-all focus:ring-2"
                                    style={{
                                        background: 'var(--bg-tertiary)',
                                        border: '1px solid var(--border-subtle)',
                                        color: 'var(--text-primary)',
                                    }}
                                />
                            </div>
                        )}

                        {/* Enabled Toggle */}
                        <label className="flex items-center gap-3 cursor-pointer">
                            <input
                                type="checkbox"
                                checked={form.enabled}
                                onChange={(e) => setForm({ ...form, enabled: e.target.checked })}
                                className="sr-only"
                            />
                            <div
                                className="w-10 h-5 rounded-full relative transition-colors"
                                style={{ background: form.enabled ? 'var(--accent-green)' : 'var(--bg-tertiary)' }}
                            >
                                <div
                                    className="absolute top-0.5 w-4 h-4 rounded-full transition-transform"
                                    style={{
                                        background: 'var(--text-primary)',
                                        left: form.enabled ? 'calc(100% - 18px)' : '2px',
                                    }}
                                />
                            </div>
                            <span className="text-sm" style={{ color: 'var(--text-secondary)' }}>
                                Agent enabled
                            </span>
                        </label>
                    </div>

                    {/* Footer */}
                    <div
                        className="px-6 py-4 flex items-center justify-end gap-3"
                        style={{ borderTop: '1px solid var(--border-subtle)', background: 'var(--bg-tertiary)' }}
                    >
                        <button
                            type="button"
                            onClick={onClose}
                            className="px-4 py-2 rounded-lg text-sm font-medium transition-colors"
                            style={{
                                background: 'transparent',
                                color: 'var(--text-secondary)',
                                border: '1px solid var(--border-subtle)',
                            }}
                        >
                            Cancel
                        </button>
                        <button
                            type="submit"
                            disabled={isLoading || !form.name}
                            className="px-4 py-2 rounded-lg text-sm font-medium transition-colors flex items-center gap-2 disabled:opacity-50"
                            style={{
                                background: 'var(--accent-purple)',
                                color: 'var(--bg-primary)',
                            }}
                        >
                            {isLoading && <Loader2 size={14} className="animate-spin" />}
                            {agent ? 'Save Changes' : 'Add Agent'}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    );
};

export default AgentManager;

