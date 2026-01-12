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
import { Card, CardContent } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Textarea } from '@/components/ui/textarea';
import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
    DialogFooter,
} from '@/components/ui/dialog';
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select';
import {
    Collapsible,
    CollapsibleContent,
    CollapsibleTrigger,
} from '@/components/ui/collapsible';
import { Switch } from '@/components/ui/switch';
import { Label } from '@/components/ui/label';

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

const AgentManager: React.FC = () => {
    const queryClient = useQueryClient();
    const [editingAgent, setEditingAgent] = useState<AgentConfig | null>(null);
    const [showAddModal, setShowAddModal] = useState(false);
    const [expandedAgents, setExpandedAgents] = useState<Set<string>>(new Set());
    const [testResults, setTestResults] = useState<Record<string, { success: boolean; message: string }>>({});
    const [cliAgent, setCliAgent] = useState<AgentConfig | null>(null);

    // Queries
    const { data: agentsData, isLoading: isLoadingAgents } = useQuery({
        queryKey: ['agents'],
        queryFn: fetchAgents,
    });

    const { data: modelsData } = useQuery({
        queryKey: ['openrouter-models'],
        queryFn: fetchModels,
    });

    // Mutations
    const upsertMutation = useMutation({
        mutationFn: upsertAgent,
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['agents'] });
            setEditingAgent(null);
            setShowAddModal(false);
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
            setTestResults((prev) => ({
                ...prev,
                [agent.id]: { success: data.success, message: data.message || (data.success ? 'OK' : 'Failed') },
            }));
            // Clear result after 5 seconds
            setTimeout(() => {
                setTestResults((prev) => {
                    const { [agent.id]: _, ...rest } = prev;
                    return rest;
                });
            }, 5000);
        },
    });

    const agents = agentsData?.agents || [];
    const models = modelsData?.models || [];

    const toggleExpanded = (id: string) => {
        setExpandedAgents((prev) => {
            const next = new Set(prev);
            if (next.has(id)) next.delete(id);
            else next.add(id);
            return next;
        });
    };

    if (isLoadingAgents) {
        return (
            <div className="h-full flex items-center justify-center">
                <Loader2 className="animate-spin text-muted-foreground" size={24} />
            </div>
        );
    }

    return (
        <div className="h-full flex flex-col bg-background text-foreground">
            {/* Header */}
            <Card className="shrink-0 rounded-none border-x-0 border-t-0">
                <CardContent className="px-6 py-4 flex items-center justify-between">
                    <div>
                        <h1 className="text-lg font-semibold text-foreground">Agent Manager</h1>
                        <p className="text-sm text-muted-foreground">
                            Configure AI coding agents for task execution
                        </p>
                    </div>
                    <Button onClick={() => setShowAddModal(true)}>
                        <Plus size={16} className="mr-2" />
                        Add Agent
                    </Button>
                </CardContent>
            </Card>

            {/* Agent List */}
            <div className="flex-1 overflow-auto p-4 space-y-3">
                {agents.length === 0 ? (
                    <div className="text-center py-12 text-muted-foreground">
                        <Bot size={48} className="mx-auto mb-4 opacity-50" />
                        <p>No agents configured</p>
                        <p className="text-sm mt-2">Click "Add Agent" to get started</p>
                    </div>
                ) : (
                    agents.map((agent) => (
                        <AgentCard
                            key={agent.id}
                            agent={agent}
                            expanded={expandedAgents.has(agent.id)}
                            onToggleExpand={() => toggleExpanded(agent.id)}
                            onEdit={() => setEditingAgent(agent)}
                            onDelete={() => deleteMutation.mutate(agent.id)}
                            onSetDefault={() => setDefaultMutation.mutate(agent.id)}
                            onTest={() => testMutation.mutate(agent)}
                            onCli={() => { console.log('[AgentManager] setCliAgent called with:', agent.name); setCliAgent(agent); }}
                            isDeleting={deleteMutation.isPending}
                            isTesting={testMutation.isPending && testMutation.variables?.id === agent.id}
                            testResult={testResults[agent.id] || null}
                        />
                    ))
                )}
            </div>

            {/* Add/Edit Modal */}
            {(showAddModal || editingAgent) && (
                <AgentModal
                    agent={editingAgent}
                    models={models}
                    onClose={() => {
                        setEditingAgent(null);
                        setShowAddModal(false);
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

// Agent Card Component using shadcn Card and Badge
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
        <Card className={agent.is_default ? 'border-primary' : ''}>
            <Collapsible open={expanded} onOpenChange={onToggleExpand}>
                {/* Main row */}
                <CardContent className="p-3">
                    <div className="flex items-center gap-3">
                        <CollapsibleTrigger asChild>
                            <Button variant="ghost" size="icon" className="h-6 w-6 shrink-0">
                                {expanded ? (
                                    <ChevronDown size={14} className="text-muted-foreground" />
                                ) : (
                                    <ChevronRight size={14} className="text-muted-foreground" />
                                )}
                            </Button>
                        </CollapsibleTrigger>

                        <div
                            className="w-8 h-8 rounded-lg flex items-center justify-center shrink-0"
                            style={{ background: `${typeInfo.color}20`, color: typeInfo.color }}
                        >
                            {typeInfo.icon}
                        </div>

                        <div className="flex-1 min-w-0">
                            <div className="flex items-center gap-2">
                                <span className="font-medium text-sm truncate text-foreground">
                                    {agent.name}
                                </span>
                                {/* Level Badge */}
                                <Badge
                                    variant={agent.level >= 4 ? 'info' : 'secondary'}
                                    className="text-[10px] font-bold"
                                >
                                    L{agent.level || 3}
                                </Badge>
                                {agent.is_default && (
                                    <Badge variant="warning" className="text-[10px]">
                                        DEFAULT
                                    </Badge>
                                )}
                                {!agent.enabled && (
                                    <Badge variant="destructive" className="text-[10px]">
                                        DISABLED
                                    </Badge>
                                )}
                            </div>
                            <p className="text-xs truncate text-muted-foreground">
                                {typeInfo.label}
                                {agent.agent_type === 'claude_open_router' && agent.openrouter && (
                                    <> • {agent.openrouter.model}</>
                                )}
                            </p>
                        </div>

                        {/* Test Result */}
                        {testResult && (
                            <Badge variant={testResult.success ? 'success' : 'destructive'} className="animate-fade-in">
                                {testResult.success ? <Check size={12} /> : <AlertCircle size={12} />}
                                <span className="truncate max-w-[100px] ml-1">{testResult.message}</span>
                            </Badge>
                        )}

                        {/* Actions */}
                        <div className="flex items-center gap-1 shrink-0">
                            <Button variant="ghost" size="icon" className="h-7 w-7" onClick={onCli} title="Open Interactive CLI">
                                <Terminal size={14} className="text-muted-foreground" />
                            </Button>
                            <Button variant="ghost" size="icon" className="h-7 w-7" onClick={onTest} disabled={isTesting} title="Test agent">
                                {isTesting ? (
                                    <Loader2 size={14} className="animate-spin text-muted-foreground" />
                                ) : (
                                    <TestTube size={14} className="text-muted-foreground" />
                                )}
                            </Button>
                            <Button variant="ghost" size="icon" className="h-7 w-7" onClick={onSetDefault} title={agent.is_default ? 'Default agent' : 'Set as default'}>
                                {agent.is_default ? (
                                    <Star size={14} style={{ color: typeInfo.color }} />
                                ) : (
                                    <StarOff size={14} className="text-muted-foreground" />
                                )}
                            </Button>
                            <Button variant="ghost" size="icon" className="h-7 w-7" onClick={onEdit} title="Edit agent">
                                <Settings2 size={14} className="text-muted-foreground" />
                            </Button>
                            <Button
                                variant="ghost"
                                size="icon"
                                className="h-7 w-7"
                                onClick={onDelete}
                                disabled={isDeleting || agent.is_default}
                                title={agent.is_default ? 'Cannot delete default agent' : 'Delete agent'}
                            >
                                <Trash2 size={14} className="text-destructive" />
                            </Button>
                        </div>
                    </div>
                </CardContent>

                {/* Expanded details */}
                <CollapsibleContent>
                    <div className="px-4 py-3 border-t border-border text-xs space-y-2 bg-muted/50">
                        {agent.description && (
                            <p className="text-muted-foreground">{agent.description}</p>
                        )}

                        {agent.agent_type === 'claude_open_router' && agent.openrouter && (
                            <div className="space-y-1">
                                <div className="flex items-center gap-2">
                                    <span className="text-muted-foreground">Model:</span>
                                    <code className="px-1.5 py-0.5 rounded bg-muted text-[var(--accent-cyan)]">
                                        {agent.openrouter.model}
                                    </code>
                                </div>
                                {agent.openrouter.api_key && (
                                    <div className="flex items-center gap-2">
                                        <span className="text-muted-foreground">API Key:</span>
                                        <code className="px-1.5 py-0.5 rounded bg-muted text-muted-foreground">
                                            ••••••••{agent.openrouter.api_key.slice(-4)}
                                        </code>
                                    </div>
                                )}
                                {agent.openrouter.base_url && (
                                    <div className="flex items-center gap-2">
                                        <span className="text-muted-foreground">Base URL:</span>
                                        <code className="px-1.5 py-0.5 rounded bg-muted text-muted-foreground truncate">
                                            {agent.openrouter.base_url}
                                        </code>
                                    </div>
                                )}
                            </div>
                        )}

                        {agent.binary_path && (
                            <div className="flex items-center gap-2">
                                <span className="text-muted-foreground">Binary:</span>
                                <code className="px-1.5 py-0.5 rounded bg-muted text-muted-foreground">
                                    {agent.binary_path}
                                </code>
                            </div>
                        )}
                    </div>
                </CollapsibleContent>
            </Collapsible>
        </Card>
    );
};

// Agent Modal Component using shadcn Dialog
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
        <Dialog open={true} onOpenChange={(open) => !open && onClose()}>
            <DialogContent className="max-w-lg max-h-[90vh] flex flex-col p-0 gap-0">
                <form onSubmit={handleSubmit} className="flex flex-col h-full">
                    {/* Header */}
                    <DialogHeader className="px-6 py-4 border-b border-border">
                        <DialogTitle>{agent ? 'Edit Agent' : 'Add New Agent'}</DialogTitle>
                    </DialogHeader>

                    {/* Body */}
                    <div className="flex-1 overflow-y-auto px-6 py-4 space-y-4">
                        {/* Name */}
                        <div className="space-y-2">
                            <Label htmlFor="name">Name</Label>
                            <Input
                                id="name"
                                value={form.name}
                                onChange={(e) => setForm({ ...form, name: e.target.value })}
                                placeholder="My Custom Agent"
                                required
                            />
                        </div>

                        {/* Agent Type */}
                        <div className="space-y-2">
                            <Label>Agent Type</Label>
                            <Select
                                value={form.agent_type}
                                onValueChange={(value) => setForm({
                                    ...form,
                                    agent_type: value as AgentType,
                                    openrouter: value === 'claude_open_router' ? form.openrouter || { model: 'anthropic/claude-sonnet-4' } : undefined,
                                })}
                            >
                                <SelectTrigger>
                                    <SelectValue />
                                </SelectTrigger>
                                <SelectContent>
                                    <SelectItem value="claude">Claude Code (Default CLI)</SelectItem>
                                    <SelectItem value="claude_open_router">Claude Code + OpenRouter</SelectItem>
                                    <SelectItem value="gemini">Gemini CLI</SelectItem>
                                    <SelectItem value="codex">OpenAI Codex</SelectItem>
                                    <SelectItem value="cursor">Cursor CLI</SelectItem>
                                    <SelectItem value="amp">Amp</SelectItem>
                                    <SelectItem value="qwen_code">Qwen Code</SelectItem>
                                    <SelectItem value="opencode">Opencode</SelectItem>
                                    <SelectItem value="copilot">GitHub Copilot</SelectItem>
                                </SelectContent>
                            </Select>
                        </div>

                        {/* Capability Level */}
                        <div className="space-y-2">
                            <Label>
                                Capability Level
                                <span className="ml-2 text-xs text-muted-foreground">(1=lightest, 5=strongest)</span>
                            </Label>
                            <div className="flex items-center gap-2">
                                {[1, 2, 3, 4, 5].map((level) => (
                                    <Button
                                        key={level}
                                        type="button"
                                        variant={form.level === level ? 'default' : 'outline'}
                                        className="flex-1"
                                        onClick={() => setForm({ ...form, level })}
                                    >
                                        {level}
                                    </Button>
                                ))}
                            </div>
                            <p className="text-xs text-muted-foreground">
                                {form.level === 1 && '📄 Simple files, boilerplate, configs'}
                                {form.level === 2 && '🔧 Basic implementations, CRUD'}
                                {form.level === 3 && '⚙️ Standard development tasks'}
                                {form.level === 4 && '🧠 Complex logic, architecture'}
                                {form.level === 5 && '🚀 Planning, reasoning, critical tasks'}
                            </p>
                        </div>

                        {/* OpenRouter Config */}
                        {isOpenRouter && (
                            <Card className="p-4 space-y-4">
                                <div className="flex items-center gap-2">
                                    <Key size={14} className="text-primary" />
                                    <span className="text-xs font-medium text-primary">OpenRouter Configuration</span>
                                </div>

                                {/* Model Selection - Provider + Model */}
                                <div className="space-y-3">
                                    <div className="grid grid-cols-3 gap-3">
                                        {/* Provider Selector */}
                                        <div className="space-y-2">
                                            <Label>Provider</Label>
                                            <Select
                                                value={selectedProvider}
                                                onValueChange={(value) => {
                                                    setSelectedProvider(value);
                                                    // Auto-select first model from new provider
                                                    const firstModel = models.find(m => m.id.startsWith(value + '/'));
                                                    if (firstModel) {
                                                        setForm({
                                                            ...form,
                                                            openrouter: { ...form.openrouter!, model: firstModel.id },
                                                        });
                                                    }
                                                }}
                                            >
                                                <SelectTrigger>
                                                    <SelectValue />
                                                </SelectTrigger>
                                                <SelectContent>
                                                    {providers.map((provider) => (
                                                        <SelectItem key={provider} value={provider}>
                                                            {formatProvider(provider)}
                                                        </SelectItem>
                                                    ))}
                                                </SelectContent>
                                            </Select>
                                        </div>

                                        {/* Model Selector */}
                                        <div className="col-span-2 space-y-2">
                                            <Label>Model</Label>
                                            <Select
                                                value={form.openrouter?.model || ''}
                                                onValueChange={(value) => setForm({
                                                    ...form,
                                                    openrouter: { ...form.openrouter!, model: value },
                                                })}
                                            >
                                                <SelectTrigger>
                                                    <SelectValue />
                                                </SelectTrigger>
                                                <SelectContent>
                                                    {filteredModels.map((model) => (
                                                        <SelectItem key={model.id} value={model.id}>
                                                            {model.name.replace(/^[^:]+:\s*/, '')}
                                                        </SelectItem>
                                                    ))}
                                                </SelectContent>
                                            </Select>
                                        </div>
                                    </div>

                                    {/* Model info card */}
                                    {models.find(m => m.id === form.openrouter?.model) && (
                                        <Card className="p-3 text-xs space-y-2">
                                            <div className="flex items-center justify-between">
                                                <code className="text-[10px] px-1.5 py-0.5 rounded bg-muted text-[var(--accent-cyan)]">
                                                    {form.openrouter?.model}
                                                </code>
                                                {models.find(m => m.id === form.openrouter?.model)?.pricing && (
                                                    <span className="text-[var(--accent-green)]">
                                                        {models.find(m => m.id === form.openrouter?.model)?.pricing}
                                                    </span>
                                                )}
                                            </div>
                                            <p className="text-muted-foreground">
                                                {models.find(m => m.id === form.openrouter?.model)?.description || 'No description available'}
                                            </p>
                                            {models.find(m => m.id === form.openrouter?.model)?.context_length && (
                                                <div className="flex items-center gap-2 pt-1 border-t border-border">
                                                    <span className="text-muted-foreground">Context window:</span>
                                                    <span className="font-medium text-primary">
                                                        {(models.find(m => m.id === form.openrouter?.model)?.context_length || 0).toLocaleString()} tokens
                                                    </span>
                                                </div>
                                            )}
                                        </Card>
                                    )}
                                </div>

                                {/* API Key */}
                                <div className="space-y-2">
                                    <Label htmlFor="api-key">OpenRouter API Key</Label>
                                    <Input
                                        id="api-key"
                                        type="password"
                                        value={form.openrouter?.api_key || ''}
                                        onChange={(e) => setForm({
                                            ...form,
                                            openrouter: { ...form.openrouter!, api_key: e.target.value },
                                        })}
                                        placeholder="sk-or-..."
                                        className="font-mono"
                                    />
                                    <p className="text-xs text-muted-foreground flex items-center gap-1">
                                        Get your key from{' '}
                                        <a
                                            href="https://openrouter.ai/keys"
                                            target="_blank"
                                            rel="noopener noreferrer"
                                            className="flex items-center gap-0.5 text-primary hover:underline"
                                        >
                                            openrouter.ai/keys <ExternalLink size={10} />
                                        </a>
                                    </p>
                                </div>

                                {/* Custom Base URL (optional) */}
                                <div className="space-y-2">
                                    <Label htmlFor="base-url">
                                        Custom Base URL <span className="text-muted-foreground">(optional)</span>
                                    </Label>
                                    <Input
                                        id="base-url"
                                        type="url"
                                        value={form.openrouter?.base_url || ''}
                                        onChange={(e) => setForm({
                                            ...form,
                                            openrouter: { ...form.openrouter!, base_url: e.target.value || undefined },
                                        })}
                                        placeholder="https://openrouter.ai/api"
                                        className="font-mono"
                                    />
                                </div>
                            </Card>
                        )}

                        {/* Description */}
                        <div className="space-y-2">
                            <Label htmlFor="description">
                                Description <span className="text-muted-foreground">(optional)</span>
                            </Label>
                            <Textarea
                                id="description"
                                value={form.description || ''}
                                onChange={(e: React.ChangeEvent<HTMLTextAreaElement>) => setForm({ ...form, description: e.target.value || undefined })}
                                placeholder="My agent for..."
                                rows={2}
                            />
                        </div>

                        {/* Custom Binary Path (for non-OpenRouter) */}
                        {!isOpenRouter && (
                            <div className="space-y-2">
                                <Label htmlFor="binary-path">
                                    Custom Binary Path <span className="text-muted-foreground">(optional)</span>
                                </Label>
                                <Input
                                    id="binary-path"
                                    value={form.binary_path || ''}
                                    onChange={(e) => setForm({ ...form, binary_path: e.target.value || undefined })}
                                    placeholder="/usr/local/bin/claude"
                                    className="font-mono"
                                />
                            </div>
                        )}

                        {/* Enabled Toggle */}
                        <div className="flex items-center space-x-2">
                            <Switch
                                id="enabled"
                                checked={form.enabled}
                                onCheckedChange={(checked: boolean) => setForm({ ...form, enabled: checked })}
                            />
                            <Label htmlFor="enabled">Agent enabled</Label>
                        </div>
                    </div>

                    {/* Footer */}
                    <DialogFooter className="px-6 py-4 border-t border-border bg-muted/50">
                        <Button type="button" variant="outline" onClick={onClose}>
                            Cancel
                        </Button>
                        <Button type="submit" disabled={isLoading || !form.name}>
                            {isLoading && <Loader2 size={14} className="animate-spin mr-2" />}
                            {agent ? 'Save Changes' : 'Add Agent'}
                        </Button>
                    </DialogFooter>
                </form>
            </DialogContent>
        </Dialog>
    );
};

export default AgentManager;
