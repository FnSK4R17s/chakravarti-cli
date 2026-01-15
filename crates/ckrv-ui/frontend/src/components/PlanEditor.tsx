import React, { useState, useMemo, useEffect } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import {
    ChevronDown, ChevronRight,
    GitBranch, Layers, List, Code,
    Zap, Brain, Cpu, ArrowRight, Link2, DollarSign, Timer,
    Network, Workflow, Box, Sparkles,
    Save, Settings2
} from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs';
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog';
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select';
import { toast } from 'sonner';

// Types
interface ModelAssignment {
    default: string;
    overrides: Record<string, string>;
}

interface Batch {
    id: string;
    name: string;
    task_ids: string[];
    depends_on: string[];
    model_assignment: ModelAssignment;
    execution_strategy: string;
    estimated_cost: number;
    estimated_time: string;
    reasoning: string;
}

interface PlanResponse {
    success: boolean;
    batches: Batch[];
    raw_yaml?: string;
    error?: string;
}

interface ModelInfo {
    id: string;
    name: string;
    cost_per_1k_prompt: number;
    cost_per_1k_completion: number;
    context_length: number;
}

interface Spec {
    name: string;
    path: string;
    task_count: number;
    has_plan: boolean;
    has_implementation: boolean;
}

interface AgentConfig {
    id: string;
    name: string;
    agent_type: string;
    is_default: boolean;
    level: number;
    enabled: boolean;
    openrouter?: {
        model: string;
    };
}

// API Functions
const fetchSpecs = async (): Promise<{ specs: Spec[] }> => {
    const res = await fetch('/api/specs');
    return res.json();
};

const fetchAgents = async (): Promise<{ agents: AgentConfig[] }> => {
    const res = await fetch('/api/agents');
    return res.json();
};

const fetchPlan = async (spec: string): Promise<PlanResponse> => {
    const res = await fetch(`/api/plans/detail?spec=${spec}`);
    return res.json();
};

const savePlan = async (spec: string, batches: Batch[]) => {
    const res = await fetch('/api/plans/save', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ spec, batches }),
    });
    return res.json();
};

const fetchModels = async (): Promise<{ models: ModelInfo[] }> => {
    const res = await fetch('/api/plans/models');
    return res.json();
};

// Helper: Get model config with color and icon based on ID
const getModelConfig = (modelId: string, modelInfoList: ModelInfo[]) => {
    const info = modelInfoList.find(m => m.id === modelId);
    let tier: 'light' | 'standard' | 'heavy' = 'standard';
    let icon = Cpu;

    if (modelId.includes('claude') || modelId.includes('gpt-4')) {
        icon = Brain;
        tier = 'heavy';
    } else if (modelId.includes('minimax') || modelId.includes('haiku') || modelId.includes('flash')) {
        icon = Zap;
        tier = 'light';
    }

    // Default costs if not found (fallback)
    const costPer1k = info ? (info.cost_per_1k_prompt + info.cost_per_1k_completion) / 2 : 0.001;

    return {
        name: info?.name || modelId.split('/').pop() || modelId,
        tier,
        icon,
        costPer1k,
        contextWindow: info?.context_length
    };
};

// Components using shadcn Badge
const ModelBadge: React.FC<{ model: string; size?: 'sm' | 'md' | 'lg'; models: ModelInfo[] }> = ({ model, size = 'md', models }) => {
    const config = getModelConfig(model, models);
    const Icon = config.icon;
    const variants: Record<string, "info" | "warning" | "secondary"> = {
        light: 'info',
        standard: 'secondary',
        heavy: 'warning'
    };
    const sizeClasses = {
        sm: 'text-xs max-w-[140px]',
        md: 'text-xs max-w-[180px]',
        lg: 'text-sm max-w-[220px]'
    };

    return (
        <Badge
            variant={variants[config.tier]}
            className={`inline-flex items-center gap-1 ${sizeClasses[size]}`}
            title={config.name}
        >
            <Icon size={size === 'sm' ? 10 : 12} className="shrink-0" />
            <span className="truncate">{config.name}</span>
        </Badge>
    );
};

const StrategyBadge: React.FC<{ strategy: string }> = ({ strategy }) => {
    const isParallel = strategy === 'parallel';
    return (
        <Badge variant={isParallel ? 'success' : 'secondary'} className="flex items-center gap-1">
            {isParallel ? <GitBranch size={12} /> : <ArrowRight size={12} />}
            {strategy}
        </Badge>
    );
};

// Batch Edit Modal
const BatchEditModal: React.FC<{
    batch: Batch | null;
    isOpen: boolean;
    onClose: () => void;
    onSave: (batchId: string, modelAssignment: ModelAssignment) => void;
    agents: AgentConfig[];
}> = ({ batch, isOpen, onClose, onSave, agents }) => {
    const [selectedAgent, setSelectedAgent] = useState<string>('');

    // Initialize with batch's current model when modal opens
    React.useEffect(() => {
        if (batch) {
            // Find which agent matches the current model
            const currentModel = batch.model_assignment.default;
            const matchingAgent = agents.find(a =>
                a.openrouter?.model === currentModel ||
                (a.agent_type === 'claude' && currentModel === 'claude-code')
            );
            setSelectedAgent(matchingAgent?.id || agents.find(a => a.is_default)?.id || '');
        }
    }, [batch, agents]);

    if (!batch) return null;

    // Get the model ID for a given agent
    const getModelIdForAgent = (agentId: string): string => {
        const agent = agents.find(a => a.id === agentId);
        if (!agent) return batch.model_assignment.default;
        if (agent.agent_type === 'claude') return 'claude-code';
        return agent.openrouter?.model || batch.model_assignment.default;
    };

    const handleSave = () => {
        const modelId = getModelIdForAgent(selectedAgent);
        onSave(batch.id, {
            default: modelId,
            overrides: batch.model_assignment.overrides
        });
        onClose();
    };

    const getCurrentAgentName = () => {
        const currentModel = batch.model_assignment.default;
        const matchingAgent = agents.find(a =>
            a.openrouter?.model === currentModel ||
            (a.agent_type === 'claude' && currentModel === 'claude-code')
        );
        return matchingAgent?.name || currentModel;
    };

    return (
        <Dialog open={isOpen} onOpenChange={onClose}>
            <DialogContent className="max-w-md">
                <DialogHeader>
                    <DialogTitle className="flex items-center gap-2">
                        <Settings2 size={18} />
                        Edit Stage: {batch.name}
                    </DialogTitle>
                    <DialogDescription>
                        Configure the agent/model for this execution stage
                    </DialogDescription>
                </DialogHeader>

                <div className="space-y-4 py-4">
                    <div className="space-y-2">
                        <label className="text-sm font-medium text-foreground">
                            Select Agent
                        </label>
                        <Select value={selectedAgent} onValueChange={setSelectedAgent}>
                            <SelectTrigger>
                                <SelectValue placeholder="Select an agent" />
                            </SelectTrigger>
                            <SelectContent>
                                {agents
                                    .filter(a => a.enabled)
                                    .sort((a, b) => b.level - a.level)
                                    .map(agent => (
                                        <SelectItem key={agent.id} value={agent.id}>
                                            <div className="flex items-center gap-2">
                                                <span>{agent.name}</span>
                                                {agent.is_default && (
                                                    <span className="text-xs text-muted-foreground">(default)</span>
                                                )}
                                            </div>
                                        </SelectItem>
                                    ))}
                            </SelectContent>
                        </Select>
                        <p className="text-xs text-muted-foreground">
                            Current: {getCurrentAgentName()}
                        </p>
                    </div>

                    <div className="space-y-2">
                        <label className="text-sm font-medium text-foreground">
                            Tasks in this stage
                        </label>
                        <div className="flex flex-wrap gap-1">
                            {batch.task_ids.map(taskId => (
                                <Badge key={taskId} variant="secondary" className="font-mono text-xs">
                                    {taskId}
                                </Badge>
                            ))}
                        </div>
                    </div>

                    <div className="grid grid-cols-2 gap-4 pt-2 border-t border-border">
                        <div>
                            <span className="text-xs text-muted-foreground">Estimated Cost</span>
                            <div className="font-medium">${batch.estimated_cost.toFixed(2)}</div>
                        </div>
                        <div>
                            <span className="text-xs text-muted-foreground">Estimated Time</span>
                            <div className="font-medium">{batch.estimated_time}</div>
                        </div>
                    </div>
                </div>

                <DialogFooter>
                    <Button variant="outline" onClick={onClose}>
                        Cancel
                    </Button>
                    <Button onClick={handleSave}>
                        <Save size={16} className="mr-2" />
                        Save Changes
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
};

// BatchCard using shadcn Card
const BatchCard: React.FC<{
    batch: Batch;
    isSelected: boolean;
    onClick: () => void;
    onEdit?: () => void;
    models: ModelInfo[]
}> = ({ batch, isSelected, onClick, onEdit, models }) => {
    const [expanded, setExpanded] = useState(false);

    return (
        <Card
            className={`cursor-pointer transition-all ${isSelected ? 'ring-1 ring-primary border-primary/50' : 'hover:border-muted-foreground/50'}`}
            onClick={onClick}
        >
            <CardContent className="p-4">
                <div className="flex items-start justify-between gap-2">
                    <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2 mb-1">
                            <Badge variant="secondary" className="font-mono text-xs truncate">
                                {batch.id}
                            </Badge>
                            <StrategyBadge strategy={batch.execution_strategy} />
                        </div>
                        <h4 className="font-medium text-foreground truncate">{batch.name}</h4>
                    </div>
                    <div className="flex items-center gap-2">
                        <ModelBadge model={batch.model_assignment.default} size="sm" models={models} />
                        {onEdit && (
                            <Button
                                variant="ghost"
                                size="icon"
                                className="h-7 w-7"
                                onClick={(e) => { e.stopPropagation(); onEdit(); }}
                                title="Edit stage configuration"
                            >
                                <Settings2 size={14} />
                            </Button>
                        )}
                    </div>
                </div>

                {/* Tasks */}
                <div className="flex flex-wrap gap-1 mt-3">
                    {batch.task_ids.map(taskId => {
                        const override = batch.model_assignment.overrides[taskId];
                        return (
                            <Badge
                                key={taskId}
                                variant={override ? 'warning' : 'secondary'}
                                className="font-mono text-xs"
                                title={override ? `Override: ${override}` : undefined}
                            >
                                {taskId}
                                {override && <Sparkles size={10} className="inline ml-0.5" />}
                            </Badge>
                        );
                    })}
                </div>

                {/* Dependencies */}
                {batch.depends_on.length > 0 && (
                    <div className="mt-3 pt-3 border-t border-border">
                        <div className="text-xs text-muted-foreground flex items-center gap-1 mb-1">
                            <Link2 size={12} /> Depends on:
                        </div>
                        <div className="flex flex-wrap gap-1">
                            {batch.depends_on.map(dep => (
                                <Badge key={dep} variant="info" className="font-mono text-xs">
                                    {dep}
                                </Badge>
                            ))}
                        </div>
                    </div>
                )}

                {/* Stats */}
                <div className="flex items-center gap-4 mt-3 text-xs text-muted-foreground">
                    <span className="flex items-center gap-1">
                        <DollarSign size={12} />
                        ${batch.estimated_cost.toFixed(2)}
                    </span>
                    <span className="flex items-center gap-1">
                        <Timer size={12} />
                        {batch.estimated_time}
                    </span>
                </div>

                {/* Expand for reasoning */}
                <Button
                    variant="ghost"
                    size="sm"
                    onClick={(e) => { e.stopPropagation(); setExpanded(!expanded); }}
                    className="mt-2 text-xs"
                >
                    {expanded ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
                    Reasoning
                </Button>

                {expanded && (
                    <p className="text-sm text-muted-foreground mt-2 p-2 bg-muted rounded border border-border">
                        {batch.reasoning}
                    </p>
                )}
            </CardContent>
        </Card>
    );
};

// DAG View using Card
const DagView: React.FC<{
    batches: Batch[];
    selectedBatch: string | null;
    onSelectBatch: (id: string) => void;
    onEditBatch?: (batch: Batch) => void;
    models: ModelInfo[]
}> = ({ batches, selectedBatch, onSelectBatch, onEditBatch, models }) => {
    const levels = useMemo(() => {
        const batchMap = new Map(batches.map(b => [b.id, b]));
        const levelMap = new Map<string, number>();

        const getLevel = (batchId: string, visited = new Set<string>()): number => {
            if (visited.has(batchId)) return 0;
            if (levelMap.has(batchId)) return levelMap.get(batchId)!;

            visited.add(batchId);
            const batch = batchMap.get(batchId);
            if (!batch) {
                levelMap.set(batchId, 0);
                return 0;
            }

            const maxDepLevel = Math.max(...batch.depends_on.map(dep => getLevel(dep, visited)), -1);
            const level = maxDepLevel + 1;
            levelMap.set(batchId, level);
            return level;
        };

        batches.forEach(b => getLevel(b.id));
        return levelMap;
    }, [batches]);

    const maxLevel = Math.max(...Array.from(levels.values()), 0);

    const batchesByLevel = useMemo(() => {
        const grouped: Record<number, Batch[]> = {};
        for (let i = 0; i <= maxLevel; i++) grouped[i] = [];
        batches.forEach(b => {
            const level = levels.get(b.id) || 0;
            grouped[level].push(b);
        });
        return grouped;
    }, [batches, levels, maxLevel]);

    return (
        <Card className="p-6 overflow-x-auto">
            <div className="flex gap-8 min-w-max">
                {Object.entries(batchesByLevel).map(([level, levelBatches]) => (
                    <div key={level} className="flex flex-col gap-3">
                        <div className="text-xs font-medium text-muted-foreground text-center mb-2">
                            Stage {parseInt(level) + 1}
                        </div>
                        {levelBatches.map(batch => {
                            const config = getModelConfig(batch.model_assignment.default, models);
                            const tierColors = {
                                light: 'border-accent-cyan bg-accent-cyan-dim',
                                standard: 'border-accent-purple bg-accent-purple-dim',
                                heavy: 'border-accent-amber bg-accent-amber-dim'
                            };
                            return (
                                <Card
                                    key={batch.id}
                                    onClick={() => onSelectBatch(batch.id)}
                                    className={`
                                      w-56 p-3 cursor-pointer transition-all border-2
                                      ${tierColors[config.tier]}
                                      ${selectedBatch === batch.id ? 'ring-2 ring-primary ring-offset-2 ring-offset-background' : ''}
                                    `}
                                >
                                    <div className="flex items-start justify-between">
                                        <div className="font-medium text-sm text-foreground truncate flex-1">{batch.name}</div>
                                        {onEditBatch && (
                                            <Button
                                                variant="ghost"
                                                size="icon"
                                                className="h-6 w-6 -mt-1 -mr-1"
                                                onClick={(e) => { e.stopPropagation(); onEditBatch(batch); }}
                                                title="Edit stage"
                                            >
                                                <Settings2 size={12} />
                                            </Button>
                                        )}
                                    </div>
                                    <div className="flex items-center gap-2 mt-1">
                                        <StrategyBadge strategy={batch.execution_strategy} />
                                        <span className="text-xs text-muted-foreground">{batch.task_ids.length} tasks</span>
                                    </div>
                                    <div className="mt-2">
                                        <ModelBadge model={batch.model_assignment.default} size="sm" models={models} />
                                    </div>
                                    {batch.depends_on.length > 0 && (
                                        <div className="text-xs text-muted-foreground mt-2 flex items-center gap-1">
                                            <ArrowRight size={10} />
                                            {batch.depends_on.length} deps
                                        </div>
                                    )}
                                </Card>
                            );
                        })}
                    </div>
                ))}
            </div>

            {/* Legend */}
            <div className="flex items-center gap-4 mt-6 pt-4 border-t border-border">
                <span className="text-xs text-muted-foreground">Model Tiers:</span>
                <div className="flex items-center gap-2">
                    <span className="w-3 h-3 rounded bg-accent-cyan-dim border border-accent-cyan"></span>
                    <span className="text-xs text-muted-foreground">Light</span>
                </div>
                <div className="flex items-center gap-2">
                    <span className="w-3 h-3 rounded bg-accent-purple-dim border border-accent-purple"></span>
                    <span className="text-xs text-muted-foreground">Standard</span>
                </div>
                <div className="flex items-center gap-2">
                    <span className="w-3 h-3 rounded bg-accent-amber-dim border border-accent-amber"></span>
                    <span className="text-xs text-muted-foreground">Heavy</span>
                </div>
            </div>
        </Card>
    );
};

// Spec List View using Card
const SpecListView: React.FC<{
    specs: Spec[];
    onSelect: (name: string) => void;
    isLoading: boolean;
}> = ({ specs, onSelect, isLoading }) => {
    const specsWithPlan = specs.filter(s => s.has_plan);

    if (isLoading) {
        return (
            <div className="flex items-center justify-center h-64">
                <Workflow className="animate-spin text-muted-foreground" size={24} />
            </div>
        );
    }

    if (specsWithPlan.length === 0) {
        return (
            <div className="text-center py-12 text-muted-foreground">
                <Workflow size={48} className="mx-auto mb-4 opacity-50" />
                <p>No specs with execution plans found</p>
                <p className="text-sm mt-2">Run <code className="bg-muted px-2 py-0.5 rounded">ckrv plan</code> to generate an execution plan</p>
            </div>
        );
    }

    return (
        <div className="space-y-2">
            {specsWithPlan.map((spec) => (
                <Card
                    key={spec.name}
                    className="cursor-pointer hover:bg-accent/50 transition-colors"
                    onClick={() => onSelect(spec.name)}
                >
                    <CardContent className="p-4">
                        <div className="flex items-center justify-between">
                            <div className="flex items-center gap-3">
                                <Workflow size={20} className="text-primary" />
                                <div>
                                    <h3 className="font-medium text-foreground">{spec.name}</h3>
                                    <div className="flex items-center gap-2 mt-1">
                                        <Badge variant="success">has plan</Badge>
                                        <span className="text-xs text-muted-foreground">{spec.task_count} tasks</span>
                                        {spec.has_implementation && (
                                            <Badge variant="info">implemented</Badge>
                                        )}
                                    </div>
                                </div>
                            </div>
                            <ChevronRight size={20} className="text-muted-foreground" />
                        </div>
                    </CardContent>
                </Card>
            ))}
        </div>
    );
};

// Main Plan Editor
export default function PlanEditor() {
    const queryClient = useQueryClient();
    const [selectedSpecName, setSelectedSpecName] = useState<string | null>(null);
    const [view, setView] = useState<'dag' | 'list' | 'timeline' | 'cost' | 'code'>('dag');
    const [selectedBatch, setSelectedBatch] = useState<string | null>(null);
    const [lastSaved, setLastSaved] = useState<Date | null>(null);
    const [editingBatch, setEditingBatch] = useState<Batch | null>(null);
    const [editableBatches, setEditableBatches] = useState<Batch[]>([]);
    const [hasChanges, setHasChanges] = useState(false);

    // Data Fetching
    const { data: specsData, isLoading: isLoadingSpecs } = useQuery({ queryKey: ['specs'], queryFn: fetchSpecs });
    const { data: modelsData } = useQuery({ queryKey: ['openrouter-models'], queryFn: fetchModels });
    const { data: agentsData } = useQuery({ queryKey: ['agents'], queryFn: fetchAgents });
    const { data: planData, isError, error } = useQuery({
        queryKey: ['plan', selectedSpecName],
        queryFn: () => fetchPlan(selectedSpecName!),
        enabled: !!selectedSpecName
    });

    // Initialize editable batches when plan data loads
    useEffect(() => {
        if (planData?.batches) {
            setEditableBatches(planData.batches);
            setHasChanges(false);
        }
    }, [planData]);

    const batches = editableBatches.length > 0 ? editableBatches : (planData?.batches || []);
    const models = useMemo(() => modelsData?.models || [], [modelsData]);

    // Configured models from agents
    const configuredModels = useMemo(() => {
        const set = new Set<string>();
        if (agentsData?.agents) {
            Object.values(agentsData.agents).forEach(agent => {
                if (agent.openrouter?.model) {
                    set.add(agent.openrouter.model);
                }
            });
        }
        return set;
    }, [agentsData]);

    // Stats
    const stats = useMemo(() => ({
        totalBatches: batches.length,
        totalTasks: batches.reduce((sum, b) => sum + b.task_ids.length, 0),
        parallelBatches: batches.filter(b => b.execution_strategy === 'parallel').length,
        totalCost: batches.reduce((sum, b) => sum + b.estimated_cost, 0),
        heavyTasks: batches.filter(b => getModelConfig(b.model_assignment.default, models).tier === 'heavy').reduce((sum, b) => sum + b.task_ids.length, 0),
    }), [batches, models]);

    const handleSave = async () => {
        if (!selectedSpecName) return;
        try {
            await savePlan(selectedSpecName, batches);
            setLastSaved(new Date());
            setHasChanges(false);
            queryClient.invalidateQueries({ queryKey: ['plan', selectedSpecName] });
            toast.success('Plan Saved', {
                description: 'Execution plan has been saved successfully',
            });
        } catch (e) {
            console.error(e);
            toast.error('Save Failed', {
                description: e instanceof Error ? e.message : 'Unknown error',
            });
        }
    };

    // Handler for updating a batch's model assignment
    const handleBatchUpdate = (batchId: string, modelAssignment: ModelAssignment) => {
        setEditableBatches(prev => prev.map(b =>
            b.id === batchId
                ? { ...b, model_assignment: modelAssignment }
                : b
        ));
        setHasChanges(true);
        toast.success('Stage Updated', {
            description: `Model assignment for ${batchId} changed`,
        });
    };

    if (isError) {
        return <div className="p-8 text-destructive">Error loading plan: {(error as Error).message}</div>;
    }

    // Show spec list if nothing selected
    if (!selectedSpecName) {
        return (
            <div className="h-full overflow-auto p-4">
                <div className="mb-6">
                    <h1 className="text-2xl font-bold text-foreground">Execution Plan</h1>
                    <p className="text-muted-foreground mt-1">Select a spec to view its execution plan</p>
                </div>
                <SpecListView
                    specs={specsData?.specs || []}
                    onSelect={setSelectedSpecName}
                    isLoading={isLoadingSpecs}
                />
            </div>
        );
    }

    return (
        <div className="h-full flex flex-col bg-background text-foreground font-sans">
            {/* Header */}
            <Card className="shrink-0 rounded-none border-x-0 border-t-0">
                <CardContent className="px-6 py-4 flex items-center justify-between">
                    <div className="flex items-center gap-4">
                        <Button
                            variant="ghost"
                            size="icon"
                            onClick={() => setSelectedSpecName(null)}
                        >
                            <ArrowRight size={20} className="rotate-180" />
                        </Button>
                        <div>
                            <div className="text-sm text-muted-foreground font-mono">plan.yaml</div>
                            <h1 className="text-lg font-semibold text-foreground">{selectedSpecName}</h1>
                        </div>
                        {lastSaved && <span className="text-xs text-muted-foreground">• Saved {lastSaved.toLocaleTimeString()}</span>}
                    </div>

                    <div className="flex items-center gap-4">
                        {/* View Toggles using Tabs */}
                        <Tabs value={view} onValueChange={(v) => setView(v as typeof view)}>
                            <TabsList>
                                <TabsTrigger value="dag" className="gap-1.5">
                                    <Network size={16} />
                                    DAG
                                </TabsTrigger>
                                <TabsTrigger value="list" className="gap-1.5">
                                    <List size={16} />
                                    List
                                </TabsTrigger>
                                <TabsTrigger value="code" className="gap-1.5">
                                    <Code size={16} />
                                    YAML
                                </TabsTrigger>
                            </TabsList>
                        </Tabs>

                        <div className="w-px h-6 bg-border" />

                        <Button
                            onClick={handleSave}
                            size="icon"
                            title="Save Plan"
                            disabled={!hasChanges}
                            className={hasChanges ? '' : 'opacity-50'}
                        >
                            <Save size={20} />
                        </Button>
                    </div>
                </CardContent>
            </Card>

            {/* Content */}
            <div className="flex-1 overflow-auto p-6">
                {/* Stats Row */}
                <div className="grid grid-cols-4 gap-4 mb-6">
                    <Card>
                        <CardContent className="p-4 flex items-center justify-between">
                            <div>
                                <div className="text-2xl font-bold text-foreground">{stats.totalBatches}</div>
                                <div className="text-xs text-muted-foreground">Batches</div>
                            </div>
                            <Layers className="text-muted-foreground" size={20} />
                        </CardContent>
                    </Card>
                    <Card>
                        <CardContent className="p-4 flex items-center justify-between">
                            <div>
                                <div className="text-2xl font-bold text-foreground">{stats.totalTasks}</div>
                                <div className="text-xs text-muted-foreground">Total Tasks</div>
                            </div>
                            <Box className="text-accent-cyan" size={20} />
                        </CardContent>
                    </Card>
                    <Card>
                        <CardContent className="p-4 flex items-center justify-between">
                            <div>
                                <div className="text-2xl font-bold text-foreground">${stats.totalCost.toFixed(2)}</div>
                                <div className="text-xs text-muted-foreground">Est. Cost</div>
                            </div>
                            <DollarSign className="text-accent-amber" size={20} />
                        </CardContent>
                    </Card>
                    <Card>
                        <CardContent className="p-4 flex items-center justify-between">
                            <div>
                                <div className="text-2xl font-bold text-foreground">{stats.heavyTasks}</div>
                                <div className="text-xs text-muted-foreground">Heavy Model Tasks</div>
                            </div>
                            <Brain className="text-accent-purple" size={20} />
                        </CardContent>
                    </Card>
                </div>

                {batches.length === 0 ? (
                    <Card className="flex items-center justify-center h-64 text-muted-foreground border-dashed">
                        No plan available for this spec. Run 'ckrv plan' to generate one.
                    </Card>
                ) : (
                    <>
                        {view === 'dag' && (
                            <DagView
                                batches={batches}
                                selectedBatch={selectedBatch}
                                onSelectBatch={setSelectedBatch}
                                onEditBatch={setEditingBatch}
                                models={models}
                            />
                        )}

                        {view === 'list' && (
                            <div className="grid lg:grid-cols-2 gap-4">
                                {batches.map(batch => (
                                    <BatchCard
                                        key={batch.id}
                                        batch={batch}
                                        isSelected={selectedBatch === batch.id}
                                        onClick={() => setSelectedBatch(batch.id)}
                                        onEdit={() => setEditingBatch(batch)}
                                        models={models}
                                    />
                                ))}
                            </div>
                        )}

                        {view === 'code' && (
                            <Card>
                                <CardHeader className="py-2 px-4 flex flex-row items-center justify-between border-b border-border">
                                    <CardTitle className="text-sm font-mono">plan.yaml</CardTitle>
                                    <Button variant="ghost" size="sm">Copy</Button>
                                </CardHeader>
                                <CardContent className="p-0">
                                    <pre className="p-4 text-xs font-mono text-foreground overflow-auto max-h-[600px] bg-muted">
                                        {planData?.raw_yaml}
                                    </pre>
                                </CardContent>
                            </Card>
                        )}
                    </>
                )}
            </div>

            {/* Model Pricing Footer */}
            <div className="shrink-0 px-6 py-3 border-t border-border bg-muted/50 text-xs">
                <div className="flex items-center gap-6 overflow-x-auto">
                    <span className="text-muted-foreground font-medium whitespace-nowrap">Current Pricing (Configured Models):</span>
                    {models.filter(m => configuredModels.has(m.id))
                        .map(model => (
                            <div key={model.id} className="flex items-center gap-2 whitespace-nowrap">
                                <span className="text-muted-foreground">{model.name}:</span>
                                <span className="text-accent-amber">${(model.cost_per_1k_prompt + model.cost_per_1k_completion).toFixed(4)}/1k</span>
                            </div>
                        ))}
                    {configuredModels.size === 0 && <span className="text-muted-foreground italic">No configured models found</span>}
                </div>
            </div>

            {/* Batch Edit Modal */}
            <BatchEditModal
                batch={editingBatch}
                isOpen={!!editingBatch}
                onClose={() => setEditingBatch(null)}
                onSave={handleBatchUpdate}
                agents={agentsData?.agents || []}
            />
        </div>
    );
}
