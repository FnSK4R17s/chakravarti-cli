/**
 * @module AgentManager
 * @description
 * Configuration and management interface for AI coding agents. Allows users
 * to add, edit, delete, and configure multiple agent types (Claude, OpenRouter,
 * GLM, Codex) with different capability levels for task assignment.
 *
 * @context
 * Rendered as the main content of the Agents page in the dashboard. Users
 * configure agents here which are then available for task execution in the
 * ExecutionRunner. Supports role assignments (QA, Test Writer) and default selection.
 *
 * @dependencies
 * - AgentCliModal: Opens interactive CLI for testing agent commands
 * - useQuery/useMutation: React Query for CRUD operations on agents
 * - shadcn/ui components: Card, Badge, Dialog, Select for consistent UI
 *
 * @example
 * // Rendered directly as a page component
 * <AgentManager />
 *
 * // Agents are stored in the backend and fetched on mount
 * // Users can add new agents via the "Add Agent" button
 */

// ============================================================
// IMPORTS
// ============================================================
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
    ChevronDown,
    ChevronRight,
    Sparkles,
    TestTube,
    AlertCircle,
    Terminal,
    Shield,
    FileCode
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

// ============================================================
// TYPES
// ============================================================

/** Supported agent types for task execution. */
type AgentType = 'claude' | 'claude_open_router' | 'claude_glm' | 'codex' | 'kilo_code' | 'gemini' | 'cursor' | 'amp' | 'qwen' | 'opencode' | 'factory_droid' | 'github_copilot' | 'mistral_vibe';

interface OpenRouterConfig {
    api_key?: string;
    model: string;
    base_url?: string;
    max_tokens?: number;
    temperature?: number;
}

interface GLMConfig {
    api_key?: string;
    model: string;
    timeout_ms?: number;
}

interface KiloCodeConfig {
    /** Model ID in kilo format (e.g., "kilo/google/gemma-3-27b-it:free") */
    model: string;
}

interface KiloCodeModel {
    id: string;
    provider: string;
    name: string;
    free: boolean;
}

interface GlmModel {
    id: string;
    name: string;
    context_length?: number;
}

/**
 * Configuration for an AI agent that can execute tasks.
 * 
 * @example
 * const claudeAgent: AgentConfig = {
 *   id: 'claude-default',
 *   name: 'Claude Code',
 *   agent_type: 'claude',
 *   level: 5,
 *   is_default: true,
 *   enabled: true
 * };
 */
export interface AgentConfig {
    /** Unique identifier for the agent */
    id: string;
    /** Display name shown in the UI */
    name: string;
    /** Type of agent: claude, claude_open_router, claude_glm, or codex */
    agent_type: AgentType;
    /** Capability level 1-5 (5 = strongest, for complex tasks) */
    level: number;
    /** Whether this agent is the default for task execution */
    is_default: boolean;
    /** Whether this agent is designated for QA reviews */
    is_qa_agent?: boolean;
    /** Whether this agent is designated for test writing */
    is_test_writer?: boolean;
    /** Whether this agent is enabled and available for use */
    enabled: boolean;
    /** Optional description of the agent's capabilities */
    description?: string;
    /** OpenRouter configuration (for claude_open_router type) */
    openrouter?: OpenRouterConfig;
    /** GLM configuration (for claude_glm type) */
    glm?: GLMConfig;
    /** Kilo Code configuration (for kilo_code type) */
    kilo?: KiloCodeConfig;
    /** Path to the agent binary (for codex type) */
    binary_path?: string;
    /** Additional command-line arguments for the agent */
    extra_args?: string[];
    /** Environment variables to set when running the agent */
    env_vars?: Record<string, string>;
}

interface OpenRouterModel {
    id: string;
    name: string;
    description: string;
    context_length?: number;
    pricing?: string;
}

// ============================================================
// API FUNCTIONS
// ============================================================

/** Fetches all configured agents from the backend. */
const fetchAgents = async (): Promise<{ agents: AgentConfig[] }> => {
    const res = await fetch('/api/agents');
    return res.json();
};

const fetchModels = async (): Promise<{ models: OpenRouterModel[] }> => {
    const res = await fetch('/api/agents/models');
    return res.json();
};

/** Fetches available Kilo Code models from the backend. */
const fetchKiloModels = async (): Promise<{ models: KiloCodeModel[] }> => {
    const res = await fetch('/api/agents/kilo-models');
    return res.json();
};

/** Fetches available GLM Coding Plan models from the backend. */
const fetchGlmModels = async (): Promise<{ models: GlmModel[] }> => {
    const res = await fetch('/api/agents/glm-models');
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
        body: JSON.stringify({ name: id }),
    });
    return res.json();
};

const setDefaultAgent = async (id: string) => {
    const res = await fetch('/api/agents/set-default', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name: id }),
    });
    return res.json();
};

const setQaAgent = async (id: string) => {
    const res = await fetch('/api/agents/set-qa', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name: id }),
    });
    return res.json();
};

const setTestWriterAgent = async (id: string) => {
    const res = await fetch('/api/agents/set-test-writer', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name: id }),
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


// Agent type display info - Only supported types
const AGENT_TYPE_INFO: Record<AgentType, { label: string; icon: React.ReactNode; color: string }> = {
    claude: { label: 'Claude Code', icon: <Bot size={16} />, color: 'hsl(var(--warning))' },
    claude_open_router: { label: 'Claude + OpenRouter', icon: <Sparkles size={16} />, color: 'hsl(var(--primary))' },
    claude_glm: { label: 'GLM Coding Plan', icon: <Zap size={16} />, color: 'hsl(var(--info))' },
    codex: { label: 'OpenAI Codex', icon: <Zap size={16} />, color: 'hsl(var(--success))' },
    kilo_code: { label: 'Kilo Code', icon: <Sparkles size={16} />, color: 'hsl(var(--chart-4))' },
    gemini: { label: 'Gemini CLI', icon: <Zap size={16} />, color: 'hsl(var(--chart-5))' },
    cursor: { label: 'Cursor', icon: <Terminal size={16} />, color: 'hsl(var(--chart-5))' },
    amp: { label: 'Amp', icon: <Zap size={16} />, color: 'hsl(var(--chart-5))' },
    qwen: { label: 'Qwen Code', icon: <Zap size={16} />, color: 'hsl(var(--warning))' },
    opencode: { label: 'Opencode', icon: <Zap size={16} />, color: 'hsl(var(--chart-5))' },
    factory_droid: { label: 'Factory Droid', icon: <Zap size={16} />, color: 'hsl(var(--warning))' },
    github_copilot: { label: 'GitHub Copilot', icon: <Zap size={16} />, color: 'hsl(var(--chart-5))' },
    mistral_vibe: { label: 'Mistral Vibe', icon: <Zap size={16} />, color: 'hsl(var(--chart-5))' },
};

const AgentManager: React.FC = () => {
    const queryClient = useQueryClient();

    // ============================================================
    // STATE
    // ============================================================

    /** Agent currently being edited in the form modal */
    const [editingAgent, setEditingAgent] = useState<AgentConfig | null>(null);
    /** Controls visibility of the add new agent modal */
    const [showAddModal, setShowAddModal] = useState(false);
    /** Agent config for CLI launch modal - when set, shows the interactive CLI */
    const [cliAgent, setCliAgent] = useState<AgentConfig | null>(null);
    /** Set of agent IDs with expanded detail views in the card list */
    const [expandedAgents, setExpandedAgents] = useState<Set<string>>(new Set());
    /** Results from agent connectivity tests, keyed by agent ID - auto-clears after 5 seconds */
    const [testResults, setTestResults] = useState<Record<string, { success: boolean; message: string }>>({});

    // ============================================================
    // QUERIES
    // ============================================================

    /** Fetches all configured agents from the backend */
    const { data: agentsData, isLoading: isLoadingAgents } = useQuery({
        queryKey: ['agents'],
        queryFn: fetchAgents,
    });

    /** Fetches available OpenRouter models for the model selector dropdown */
    const { data: modelsData } = useQuery({
        queryKey: ['openrouter-models'],
        queryFn: fetchModels,
    });

    /** Fetches available Kilo Code models for the model selector dropdown */
    const { data: kiloModelsData } = useQuery({
        queryKey: ['kilo-models'],
        queryFn: fetchKiloModels,
    });

    /** Fetches available GLM Coding Plan models for the model selector dropdown */
    const { data: glmModelsData } = useQuery({
        queryKey: ['glm-models'],
        queryFn: fetchGlmModels,
    });

    // ============================================================
    // MUTATIONS
    // ============================================================

    /** Creates or updates an agent configuration */
    const upsertMutation = useMutation({
        mutationFn: upsertAgent,
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['agents'] });
            setEditingAgent(null);
            setShowAddModal(false);
        },
    });

    /** Deletes an agent configuration by ID */
    const deleteMutation = useMutation({
        mutationFn: deleteAgent,
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['agents'] });
        },
    });

    /** Sets an agent as the default for task execution */
    const setDefaultMutation = useMutation({
        mutationFn: setDefaultAgent,
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['agents'] });
        },
    });

    /** Designates an agent as the QA reviewer */
    const setQaMutation = useMutation({
        mutationFn: setQaAgent,
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['agents'] });
        },
    });

    /** Designates an agent as the test writer */
    const setTestWriterMutation = useMutation({
        mutationFn: setTestWriterAgent,
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['agents'] });
        },
    });

    /** Tests agent connectivity and stores results with auto-clear after 5 seconds */
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
                    // eslint-disable-next-line @typescript-eslint/no-unused-vars
                    const { [agent.id]: _removed, ...rest } = prev;
                    return rest;
                });
            }, 5000);
        },
    });

    /** Derived list of agents from query response */
    const agents = agentsData?.agents || [];
    /** Derived list of OpenRouter models from query response */
    const models = modelsData?.models || [];
    /** Derived list of Kilo Code models from query response */
    const kiloModels = kiloModelsData?.models || [];
    /** Derived list of GLM models from query response */
    const glmModels = glmModelsData?.models || [];

    // ============================================================
    // HANDLERS
    // ============================================================

    /** Toggles the expanded state of an agent card in the list */
    const toggleExpanded = (id: string) => {
        setExpandedAgents((prev) => {
            const next = new Set(prev);
            if (next.has(id)) next.delete(id);
            else next.add(id);
            return next;
        });
    };

    // ============================================================
    // MAIN RENDER
    // ============================================================

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
                            onSetQa={() => setQaMutation.mutate(agent.id)}
                            onSetTestWriter={() => setTestWriterMutation.mutate(agent.id)}
                            onTest={() => testMutation.mutate(agent)}
                            onCli={() => setCliAgent(agent)}
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
                    kiloModels={kiloModels}
                    glmModels={glmModels}
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

// ============================================================
// RENDER HELPERS
// ============================================================

/**
 * Props for AgentCard component.
 * Displays a single agent with expandable details and action buttons.
 */
interface AgentCardProps {
    /** Agent configuration to display */
    agent: AgentConfig;
    /** Whether the card's details section is expanded */
    expanded: boolean;
    /** Callback to toggle the expanded state */
    onToggleExpand: () => void;
    /** Callback to open the edit modal for this agent */
    onEdit: () => void;
    /** Callback to delete this agent */
    onDelete: () => void;
    /** Callback to set this agent as the default */
    onSetDefault: () => void;
    /** Callback to designate this agent as QA reviewer */
    onSetQa: () => void;
    /** Callback to designate this agent as test writer */
    onSetTestWriter: () => void;
    /** Callback to run a connectivity test on this agent */
    onTest: () => void;
    /** Callback to open the interactive CLI modal for this agent */
    onCli: () => void;
    /** Whether a delete operation is in progress */
    isDeleting: boolean;
    /** Whether a test operation is in progress for this agent */
    isTesting: boolean;
    /** Result of the last connectivity test, or null if not tested */
    testResult: { success: boolean; message: string } | null;
}

const AgentCard: React.FC<AgentCardProps> = ({
    agent,
    expanded,
    onToggleExpand,
    onEdit,
    onDelete,
    onSetDefault,
    onSetQa,
    onSetTestWriter,
    onTest,
    onCli,
    isDeleting,
    isTesting,
    testResult,
}) => {
    const typeInfo = AGENT_TYPE_INFO[agent.agent_type] || AGENT_TYPE_INFO.claude;

    return (
        <Card>
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
                                {agent.is_qa_agent && (
                                    <Badge variant="success" className="text-[10px]">
                                        QA
                                    </Badge>
                                )}
                                {agent.is_test_writer && (
                                    <Badge variant="info" className="text-[10px]">
                                        TESTS
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
                                {agent.agent_type === 'claude_glm' && agent.glm && (
                                    <> • {agent.glm.model}</>
                                )}
                                {agent.agent_type === 'kilo_code' && agent.kilo && (
                                    <> • {agent.kilo.model.replace('kilo/', '')}</>
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
                            <Button variant="ghost" size="icon" className="h-7 w-7" onClick={onSetQa} title={agent.is_qa_agent ? 'QA Agent' : 'Set as QA agent'}>
                                <Shield size={14} className={agent.is_qa_agent ? 'text-success' : 'text-muted-foreground'} />
                            </Button>
                            <Button variant="ghost" size="icon" className="h-7 w-7" onClick={onSetTestWriter} title={agent.is_test_writer ? 'Test Writer' : 'Set as Test Writer'}>
                                <FileCode size={14} className={agent.is_test_writer ? 'text-primary' : 'text-muted-foreground'} />
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
                                    <code className="px-1.5 py-0.5 rounded bg-muted text-info">
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

                        {agent.agent_type === 'claude_glm' && agent.glm && (
                            <div className="space-y-1">
                                <div className="flex items-center gap-2">
                                    <span className="text-muted-foreground">Model:</span>
                                    <code className="px-1.5 py-0.5 rounded bg-muted text-info">
                                        {agent.glm.model}
                                    </code>
                                </div>
                                {agent.glm.api_key && (
                                    <div className="flex items-center gap-2">
                                        <span className="text-muted-foreground">Z.AI API Key:</span>
                                        <code className="px-1.5 py-0.5 rounded bg-muted text-muted-foreground">
                                            ••••••••{agent.glm.api_key.slice(-4)}
                                        </code>
                                    </div>
                                )}
                                <div className="flex items-center gap-2">
                                    <span className="text-muted-foreground">Base URL:</span>
                                    <code className="px-1.5 py-0.5 rounded bg-muted text-muted-foreground">
                                        https://api.z.ai/api/anthropic
                                    </code>
                                </div>
                            </div>
                        )}

                        {agent.agent_type === 'kilo_code' && agent.kilo && (
                            <div className="space-y-1">
                                <div className="flex items-center gap-2">
                                    <span className="text-muted-foreground">Model:</span>
                                    <code className="px-1.5 py-0.5 rounded bg-muted text-info">
                                        {agent.kilo.model}
                                    </code>
                                </div>
                                <div className="flex items-center gap-2">
                                    <span className="text-muted-foreground">Provider:</span>
                                    <code className="px-1.5 py-0.5 rounded bg-muted text-muted-foreground">
                                        {agent.kilo.model.split('/')[1] || 'unknown'}
                                    </code>
                                </div>
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
/**
 * Props for AgentModal component.
 * Dialog for creating or editing an agent configuration.
 */
interface AgentModalProps {
    /** Existing agent to edit, or null to create a new one */
    agent: AgentConfig | null;
    /** List of available OpenRouter models for selection */
    models: OpenRouterModel[];
    /** List of available Kilo Code models for selection */
    kiloModels: KiloCodeModel[];
    /** List of available GLM Coding Plan models for selection */
    glmModels: GlmModel[];
    /** Callback to close the modal without saving */
    onClose: () => void;
    /** Callback to save the agent configuration */
    onSave: (agent: AgentConfig) => void;
    /** Whether a save operation is in progress */
    isLoading: boolean;
}

const AgentModal: React.FC<AgentModalProps> = ({ agent, models, kiloModels, glmModels, onClose, onSave, isLoading }) => {
    // ============================================================
    // STATE
    // ============================================================

    /** Form state for editing/creating an agent, initialized from props or defaults */
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

    /** Currently selected AI provider for OpenRouter model filtering */
    const [selectedProvider, setSelectedProvider] = useState(() =>
        form.openrouter?.model ? form.openrouter.model.split('/')[0] || 'anthropic' : 'anthropic'
    );

    /** Currently selected provider for Kilo Code model filtering */
    const [selectedKiloProvider, setSelectedKiloProvider] = useState(() => {
        if (form.kilo?.model) {
            // kilo model format: kilo/provider/model-name
            const parts = form.kilo.model.split('/');
            return parts.length >= 2 ? parts[1] : 'deepseek';
        }
        return 'deepseek';
    });

    // ============================================================
    // COMPUTED VALUES
    // ============================================================

    /** Extracts provider name from model ID (e.g., "anthropic/claude-sonnet-4" -> "anthropic") */
    const getProvider = (modelId: string) => modelId.split('/')[0] || 'unknown';

    /** Unique list of providers sorted by priority (Anthropic first, then OpenAI, etc.) */
    const providers = [...new Set(models.map(m => getProvider(m.id)))].sort((a, b) => {
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

    /** Models filtered to only show those from the currently selected provider */
    const filteredModels = models.filter(m => getProvider(m.id) === selectedProvider);

    /** Whether the current form is for an OpenRouter agent type */
    const isOpenRouter = form.agent_type === 'claude_open_router';

    /** Whether the current form is for a Kilo Code agent type */
    const isKiloCode = form.agent_type === 'kilo_code';

    /** Unique list of Kilo Code providers, sorted alphabetically */
    const kiloProviders = [...new Set(kiloModels.map(m => m.provider))].sort((a, b) => {
        const priority = (p: string) => {
            if (p === 'deepseek') return 0;
            if (p === 'google') return 1;
            if (p === 'qwen') return 2;
            if (p === 'meta-llama') return 3;
            if (p === 'openai') return 4;
            if (p === 'mistralai') return 5;
            if (p === 'openrouter') return 6;
            return 10;
        };
        return priority(a) - priority(b);
    });

    /** Kilo models filtered to only show those from the currently selected provider */
    const filteredKiloModels = kiloModels.filter(m => m.provider === selectedKiloProvider);

    // ============================================================
    // EFFECTS
    // ============================================================

    /**
     * Syncs the selected model when provider changes.
     * When user switches providers, auto-select the first available model from that provider.
     */
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
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [selectedProvider, filteredModels.length, isOpenRouter]);

    /**
     * Syncs the selected kilo model when kilo provider changes.
     * Auto-select the first available model from the new provider.
     */
    React.useEffect(() => {
        if (isKiloCode && filteredKiloModels.length > 0) {
            const currentModelInProvider = filteredKiloModels.some(m => m.id === form.kilo?.model);
            if (!currentModelInProvider) {
                setForm(f => ({
                    ...f,
                    kilo: { model: filteredKiloModels[0].id },
                }));
            }
        }
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [selectedKiloProvider, filteredKiloModels.length, isKiloCode]);

    // ============================================================
    // HANDLERS
    // ============================================================

    /** Formats a provider ID into a human-readable display name */
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
            'z-ai': 'Z.AI',
            'arcee-ai': 'Arcee AI',
            'cognitivecomputations': 'Cognitive Computations',
            'nousresearch': 'Nous Research',
            'nvidia': 'NVIDIA',
            'liquid': 'Liquid AI',
            'stepfun': 'StepFun',
            'tngtech': 'TNG Technology',
            'upstage': 'Upstage',
            'openrouter': 'OpenRouter',
            'kilo': 'Kilo (Native)',
        };
        return names[provider] || provider.charAt(0).toUpperCase() + provider.slice(1);
    };

    /** Handles form submission, ensuring a valid model is selected before saving */
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
        if (finalForm.agent_type === 'kilo_code' && finalForm.kilo) {
            // If current kilo model is not in the list, pick the first available
            const currentModelExists = kiloModels.some(m => m.id === finalForm.kilo?.model);
            if (!currentModelExists && filteredKiloModels.length > 0) {
                finalForm.kilo = { model: filteredKiloModels[0].id };
            } else if (!currentModelExists && kiloModels.length > 0) {
                finalForm.kilo = { model: kiloModels[0].id };
            }
        }
        if (finalForm.agent_type === 'claude_glm' && finalForm.glm) {
            // If current GLM model is not in the list, pick the first available
            const currentModelExists = glmModels.some(m => m.id === finalForm.glm?.model);
            if (!currentModelExists && glmModels.length > 0) {
                finalForm.glm = { ...finalForm.glm, model: glmModels[0].id };
            }
        }
        onSave(finalForm);
    };

    // ============================================================
    // MAIN RENDER
    // ============================================================

    return (
        <Dialog open={true} onOpenChange={(open) => !open && onClose()}>
            <DialogContent className="max-w-lg max-h-[85vh] flex flex-col p-0 gap-0 overflow-hidden">
                <form onSubmit={handleSubmit} className="flex flex-col min-h-0 h-full" autoComplete="off">
                    {/* Header */}
                    <DialogHeader className="px-6 py-4 border-b border-border shrink-0">
                        <DialogTitle>{agent ? 'Edit Agent' : 'Add New Agent'}</DialogTitle>
                    </DialogHeader>

                    {/* Body - scrollable */}
                    <div className="flex-1 min-h-0 overflow-y-auto px-6 py-4 space-y-4">
                        {/* Name */}
                        <div className="space-y-2">
                            <Label htmlFor="agent-name">Name</Label>
                            <Input
                                id="agent-name"
                                name="agent-display-name"
                                value={form.name}
                                onChange={(e) => setForm({ ...form, name: e.target.value })}
                                placeholder="My Custom Agent"
                                autoComplete="off"
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
                                    glm: value === 'claude_glm' ? form.glm || { model: 'glm-4.7' } : undefined,
                                    kilo: value === 'kilo_code' ? form.kilo || { model: kiloModels[0]?.id || 'kilo/deepseek/deepseek-r1-0528:free' } : undefined,
                                })}
                            >
                                <SelectTrigger>
                                    <SelectValue />
                                </SelectTrigger>
                                <SelectContent>
                                    <SelectItem value="claude">Claude Code (Default CLI)</SelectItem>
                                    <SelectItem value="claude_open_router">Claude + OpenRouter</SelectItem>
                                    <SelectItem value="claude_glm">GLM Coding Plan (Z.AI)</SelectItem>
                                    <SelectItem value="codex">OpenAI Codex</SelectItem>
                                    <SelectItem value="kilo_code">Kilo Code (Multi-Provider)</SelectItem>
                                    <SelectItem value="gemini">Gemini CLI</SelectItem>
                                    <SelectItem value="cursor">Cursor</SelectItem>
                                    <SelectItem value="amp">Amp</SelectItem>
                                    <SelectItem value="qwen">Qwen Code</SelectItem>
                                    <SelectItem value="opencode">Opencode</SelectItem>
                                    <SelectItem value="factory_droid">Factory Droid</SelectItem>
                                    <SelectItem value="github_copilot">GitHub Copilot</SelectItem>
                                    <SelectItem value="mistral_vibe">Mistral Vibe</SelectItem>
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
                                                <SelectContent className="max-h-60 overflow-y-auto">
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
                                                <SelectContent className="max-h-60 overflow-y-auto">
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
                                                <code className="text-[10px] px-1.5 py-0.5 rounded bg-muted text-info">
                                                    {form.openrouter?.model}
                                                </code>
                                                {models.find(m => m.id === form.openrouter?.model)?.pricing && (
                                                    <span className="text-success">
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
                                    <Label htmlFor="openrouter-api-key">OpenRouter API Key</Label>
                                    <Input
                                        id="openrouter-api-key"
                                        name="openrouter-api-key"
                                        type="password"
                                        value={form.openrouter?.api_key || ''}
                                        onChange={(e) => setForm({
                                            ...form,
                                            openrouter: { ...form.openrouter!, api_key: e.target.value },
                                        })}
                                        placeholder="sk-or-..."
                                        className="font-mono"
                                        autoComplete="new-password"
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

                        {/* GLM Coding Plan Config */}
                        {form.agent_type === 'claude_glm' && (
                            <Card className="p-4 space-y-4">
                                <div className="flex items-center gap-2">
                                    <Zap size={14} className="text-info" />
                                    <span className="text-xs font-medium text-info">GLM Coding Plan Configuration</span>
                                </div>

                                {/* Model Selection */}
                                <div className="space-y-3">
                                    <div className="space-y-2">
                                        <Label>Model</Label>
                                        <Select
                                            value={form.glm?.model || 'glm-4.7'}
                                            onValueChange={(value) => setForm({
                                                ...form,
                                                glm: { ...form.glm!, model: value },
                                            })}
                                        >
                                            <SelectTrigger>
                                                <SelectValue />
                                            </SelectTrigger>
                                            <SelectContent className="max-h-60 overflow-y-auto">
                                                {glmModels.length > 0 ? (
                                                    glmModels.map((model) => (
                                                        <SelectItem key={model.id} value={model.id}>
                                                            {model.name}
                                                        </SelectItem>
                                                    ))
                                                ) : (
                                                    <>
                                                        <SelectItem value="glm-4.7">GLM-4.7 (Recommended)</SelectItem>
                                                        <SelectItem value="glm-4.5-air">GLM-4.5 Air</SelectItem>
                                                    </>
                                                )}
                                            </SelectContent>
                                        </Select>
                                    </div>

                                    {/* Selected model info card */}
                                    {(() => {
                                        const selectedModel = glmModels.find(m => m.id === form.glm?.model);
                                        if (!selectedModel) return null;
                                        return (
                                            <Card className="p-3 text-xs space-y-1">
                                                <code className="text-[10px] px-1.5 py-0.5 rounded bg-muted text-info">
                                                    {selectedModel.id}
                                                </code>
                                                {selectedModel.context_length && (
                                                    <div className="flex items-center gap-2 pt-1 border-t border-border">
                                                        <span className="text-muted-foreground">Context window:</span>
                                                        <span className="font-medium text-info">
                                                            {selectedModel.context_length.toLocaleString()} tokens
                                                        </span>
                                                    </div>
                                                )}
                                            </Card>
                                        );
                                    })()}
                                </div>

                                {/* API Key */}
                                <div className="space-y-2">
                                    <Label htmlFor="glm-api-key">Z.AI API Key</Label>
                                    <Input
                                        id="glm-api-key"
                                        name="glm-api-key"
                                        type="password"
                                        value={form.glm?.api_key || ''}
                                        onChange={(e) => setForm({
                                            ...form,
                                            glm: { ...form.glm!, api_key: e.target.value },
                                        })}
                                        placeholder="Your Z.AI API key"
                                        className="font-mono"
                                        autoComplete="new-password"
                                    />
                                    <p className="text-xs text-muted-foreground flex items-center gap-1">
                                        Get your key from{' '}
                                        <a
                                            href="https://z.ai/manage-apikey/apikey-list"
                                            target="_blank"
                                            rel="noopener noreferrer"
                                            className="flex items-center gap-0.5 text-primary hover:underline"
                                        >
                                            z.ai/manage-apikey <ExternalLink size={10} />
                                        </a>
                                    </p>
                                </div>
                            </Card>
                        )}

                        {/* Kilo Code Config */}
                        {isKiloCode && (
                            <Card className="p-4 space-y-4">
                                <div className="flex items-center gap-2">
                                    <Sparkles size={14} style={{ color: 'hsl(var(--chart-4))' }} />
                                    <span className="text-xs font-medium" style={{ color: 'hsl(var(--chart-4))' }}>Kilo Code Configuration</span>
                                </div>

                                {/* Provider + Model Selection */}
                                <div className="space-y-3">
                                    <div className="grid grid-cols-3 gap-3">
                                        {/* Provider Selector */}
                                        <div className="space-y-2">
                                            <Label>Provider</Label>
                                            <Select
                                                value={selectedKiloProvider}
                                                onValueChange={(value) => {
                                                    setSelectedKiloProvider(value);
                                                    // Auto-select first model from new provider
                                                    const firstModel = kiloModels.find(m => m.provider === value);
                                                    if (firstModel) {
                                                        setForm({
                                                            ...form,
                                                            kilo: { model: firstModel.id },
                                                        });
                                                    }
                                                }}
                                            >
                                                <SelectTrigger>
                                                    <SelectValue />
                                                </SelectTrigger>
                                                <SelectContent className="max-h-60 overflow-y-auto">
                                                    {kiloProviders.map((provider) => (
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
                                                value={form.kilo?.model || ''}
                                                onValueChange={(value) => setForm({
                                                    ...form,
                                                    kilo: { model: value },
                                                })}
                                            >
                                                <SelectTrigger>
                                                    <SelectValue />
                                                </SelectTrigger>
                                                <SelectContent className="max-h-60 overflow-y-auto">
                                                    {filteredKiloModels.map((model) => (
                                                        <SelectItem key={model.id} value={model.id}>
                                                            <div className="flex items-center gap-2">
                                                                <span>{model.name}</span>
                                                                {model.free && (
                                                                    <span className="text-[10px] text-success font-medium">FREE</span>
                                                                )}
                                                            </div>
                                                        </SelectItem>
                                                    ))}
                                                </SelectContent>
                                            </Select>
                                        </div>
                                    </div>

                                    {/* Selected model info */}
                                    {form.kilo?.model && (
                                        <Card className="p-3 text-xs space-y-1">
                                            <div className="flex items-center justify-between">
                                                <code className="text-[10px] px-1.5 py-0.5 rounded bg-muted text-info">
                                                    {form.kilo.model}
                                                </code>
                                                {kiloModels.find(m => m.id === form.kilo?.model)?.free && (
                                                    <span className="text-success font-medium">Free</span>
                                                )}
                                            </div>
                                            <p className="text-muted-foreground">
                                                Passed to <code className="text-[10px]">kilo run --model</code> for execution
                                            </p>
                                        </Card>
                                    )}
                                </div>

                                <p className="text-xs text-muted-foreground">
                                    Kilo Code uses file-based auth. Run <code className="px-1 py-0.5 rounded bg-muted">kilo auth</code> to configure credentials.
                                </p>
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
                    <DialogFooter className="px-6 py-4 border-t border-border bg-muted/50 shrink-0">
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
