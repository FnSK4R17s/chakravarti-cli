/**
 * @module PlanEditor
 * @description
 * Visual editor for viewing and editing execution plans. Displays batches of tasks
 * in DAG view, list view, or raw YAML form. Allows reassigning agents to batches
 * and visualizes task dependencies and cost estimates.
 *
 * @context
 * Rendered as the main content of the Plan page in the dashboard. Users review and
 * modify execution plans here before running them in ExecutionRunner. Auto-selects
 * spec based on current git branch.
 *
 * @dependencies
 * - useAutoSelectedSpec: Auto-selects spec based on current git branch
 * - useQuery: React Query for fetching specs, plans, models, agents
 * - shadcn/ui components: Card, Badge, Tabs, Dialog for consistent UI
 *
 * @example
 * // Rendered directly as a page component
 * <PlanEditor />
 *
 * // Shows DAG of task batches with model assignments
 * // Supports editing model assignments per batch
 */

// === IMPORTS ===
import React, { useState, useMemo, useEffect } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { useAutoSelectedSpec } from '../hooks/useAutoSelectedSpec';
import {
    ChevronRight,
    GitBranch, Layers,
    Zap, Brain, Cpu, ArrowRight,
    Workflow, Box,
    Save, Settings2
} from 'lucide-react';
import { Card, CardContent } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
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

// === TYPES ===

interface ModelAssignment {
    default: string;
    overrides: Record<string, string>;
}

/**
 * An execution batch containing tasks to run together.
 * Batches form a DAG based on dependencies.
 * 
 * @example
 * const batch: Batch = {
 *   id: 'batch-1',
 *   name: 'Phase 1 Setup',
 *   task_ids: ['T001', 'T002'],
 *   depends_on: [],
 *   model_assignment: { default: 'claude-sonnet-4-20250514', overrides: {} },
 *   execution_strategy: 'parallel',
 *   estimated_time: '5m',
 *   reasoning: 'Independent setup tasks'
 * };
 */
interface Batch {
    /** Unique batch identifier */
    id: string;
    /** Human-readable batch name */
    name: string;
    /** IDs of tasks included in this batch */
    task_ids: string[];
    /** IDs of batches that must complete before this one */
    depends_on: string[];
    /** Model selection for this batch */
    model_assignment: ModelAssignment;
    /** Execution strategy: 'parallel' or 'sequential' */
    execution_strategy: string;
    /** Estimated execution time (e.g., '5m', '1h') */
    estimated_time: string;
    /** Reasoning for batch grouping */
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

// === API FUNCTIONS ===

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
    const data = await res.json();
    // Normalize batches to ensure arrays are never undefined
    if (data.batches) {
        data.batches = data.batches.map((b: Batch) => ({
            ...b,
            task_ids: b.task_ids ?? [],
            depends_on: b.depends_on ?? [],
            model_assignment: b.model_assignment ?? { default: 'unknown', overrides: {} },
            estimated_time: b.estimated_time ?? '',
            reasoning: b.reasoning ?? '',
        }));
    }
    return data;
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

// === HELPER FUNCTIONS ===

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

// === SUB-COMPONENTS ===

// Components using shadcn Badge
/**
 * Props for ModelBadge component.
 * Displays a model name with tier-based styling and icon.
 */
interface ModelBadgeProps {
    /** Model ID to display */
    model: string;
    /** Size variant: sm, md, or lg */
    size?: 'sm' | 'md' | 'lg';
    /** List of model info for looking up display names and costs */
    models: ModelInfo[];
}

const ModelBadge: React.FC<ModelBadgeProps> = ({ model, size = 'md', models }) => {
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

/**
 * Props for StrategyBadge component.
 * Displays execution strategy (parallel/sequential) with icon.
 */
interface StrategyBadgeProps {
    /** Execution strategy: 'parallel' or 'sequential' */
    strategy: string;
}

const StrategyBadge: React.FC<StrategyBadgeProps> = ({ strategy }) => {
    const isParallel = strategy === 'parallel';
    return (
        <Badge variant={isParallel ? 'success' : 'secondary'} className="flex items-center gap-1">
            {isParallel ? <GitBranch size={12} /> : <ArrowRight size={12} />}
            {strategy}
        </Badge>
    );
};

// Batch Edit Modal
/**
 * Props for BatchEditModal component.
 * Modal dialog for editing a batch's model assignment.
 */
interface BatchEditModalProps {
    /** Batch to edit, or null to hide the modal */
    batch: Batch | null;
    /** Whether the modal is visible */
    isOpen: boolean;
    /** Callback to close the modal */
    onClose: () => void;
    /** Callback to save the updated model assignment */
    onSave: (batchId: string, modelAssignment: ModelAssignment) => void;
    /** List of available agents for selection */
    agents: AgentConfig[];
}

const BatchEditModal: React.FC<BatchEditModalProps> = ({ batch, isOpen, onClose, onSave, agents }) => {
    /** Currently selected agent ID for model assignment */
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
                            <span className="text-xs text-muted-foreground">Strategy</span>
                            <div className="font-medium capitalize">{batch.execution_strategy}</div>
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

// DAG View using Card
/**
 * Props for DagView component.
 * Visualizes batches as a directed acyclic graph by dependency levels.
 */
interface DagViewProps {
    /** All batches to display in the DAG */
    batches: Batch[];
    /** ID of the currently selected batch */
    selectedBatch: string | null;
    /** Callback when a batch is selected */
    onSelectBatch: (id: string) => void;
    /** Optional callback to open batch edit modal */
    onEditBatch?: (batch: Batch) => void;
    /** List of model info for tier-based styling */
    models: ModelInfo[];
}

const DagView: React.FC<DagViewProps> = ({ batches, selectedBatch, onSelectBatch, onEditBatch, models }) => {
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
                                light: 'border-info bg-info/20',
                                standard: 'border-primary bg-primary/20',
                                heavy: 'border-warning bg-warning/20'
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
                    <span className="w-3 h-3 rounded bg-info/20 border border-info"></span>
                    <span className="text-xs text-muted-foreground">Light</span>
                </div>
                <div className="flex items-center gap-2">
                    <span className="w-3 h-3 rounded bg-primary/20 border border-primary"></span>
                    <span className="text-xs text-muted-foreground">Standard</span>
                </div>
                <div className="flex items-center gap-2">
                    <span className="w-3 h-3 rounded bg-warning/20 border border-warning"></span>
                    <span className="text-xs text-muted-foreground">Heavy</span>
                </div>
            </div>
        </Card>
    );
};

// Spec List View using Card
/**
 * Props for SpecListView component.
 * Displays a list of specs with plans for selection.
 */
interface SpecListViewProps {
    /** List of specs to display */
    specs: Spec[];
    /** Callback when a spec is selected */
    onSelect: (name: string) => void;
    /** Whether the spec list is currently loading */
    isLoading: boolean;
}

const SpecListView: React.FC<SpecListViewProps> = ({ specs, onSelect, isLoading }) => {
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

// === MAIN COMPONENT ===

// Main Plan Editor
export default function PlanEditor() {
    const queryClient = useQueryClient();

    // === STATE ===

    // --- Spec Selection ---
    // Auto-select spec based on current branch
    const { selectedSpec: autoSelectedSpec, isLoading: isLoadingAutoSpec } = useAutoSelectedSpec();

    /** Manual spec override when user explicitly selects a different spec */
    const [manualSpecOverride, setManualSpecOverride] = useState<string | null>(null);
    const selectedSpecName = manualSpecOverride ?? autoSelectedSpec;

    // --- UI State ---
    /** Currently selected batch ID for detail panel */
    const [selectedBatch, setSelectedBatch] = useState<string | null>(null);
    /** Timestamp of last successful save */
    const [lastSaved, setLastSaved] = useState<Date | null>(null);
    /** Batch currently being edited in modal */
    const [editingBatch, setEditingBatch] = useState<Batch | null>(null);

    // --- Plan Data ---
    /** Local copy of batches for editing */
    const [editableBatches, setEditableBatches] = useState<Batch[]>([]);
    /** Track unsaved changes for save prompt */
    const [hasChanges, setHasChanges] = useState(false);

    // === QUERIES ===
    // Data Fetching (for manual selection fallback)
    const { data: specsData, isLoading: isLoadingSpecs } = useQuery({ queryKey: ['specs'], queryFn: fetchSpecs });
    const { data: modelsData } = useQuery({ queryKey: ['openrouter-models'], queryFn: fetchModels });
    const { data: agentsData } = useQuery({ queryKey: ['agents'], queryFn: fetchAgents });
    const { data: planData, isError, error } = useQuery({
        queryKey: ['plan', selectedSpecName],
        queryFn: () => fetchPlan(selectedSpecName!),
        enabled: !!selectedSpecName
    });

    // === EFFECTS ===

    // Initialize editable batches when plan data loads
    useEffect(() => {
        if (planData?.batches) {
            setEditableBatches(planData.batches);
            setHasChanges(false);
        }
    }, [planData]);

    // eslint-disable-next-line react-hooks/exhaustive-deps
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
        sequentialBatches: batches.filter(b => b.execution_strategy === 'sequential').length,
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

    // Show spec list if nothing selected (neither auto nor manual)
    if (!selectedSpecName) {
        // If still loading auto-selection, show spinner
        if (isLoadingAutoSpec) {
            return (
                <div className="flex items-center justify-center h-full">
                    <Workflow className="animate-spin text-muted-foreground" size={32} />
                </div>
            );
        }

        return (
            <div className="h-full overflow-auto p-4">
                <div className="mb-6">
                    <h1 className="text-2xl font-bold text-foreground">Execution Plan</h1>
                    <p className="text-muted-foreground mt-1">No spec matches the current branch. Select a spec to view its execution plan.</p>
                </div>
                <SpecListView
                    specs={specsData?.specs || []}
                    onSelect={setManualSpecOverride}
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
                        <div>
                            <div className="text-sm text-muted-foreground font-mono">plan.yaml</div>
                            <h1 className="text-lg font-semibold text-foreground">{selectedSpecName}</h1>
                        </div>
                        {lastSaved && <span className="text-xs text-muted-foreground">• Saved {lastSaved.toLocaleTimeString()}</span>}
                    </div>

                    <div className="flex items-center gap-2">
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
                            <Box className="text-info" size={20} />
                        </CardContent>
                    </Card>
                    <Card>
                        <CardContent className="p-4 flex items-center justify-between">
                            <div>
                                <div className="text-2xl font-bold text-foreground">{stats.parallelBatches}</div>
                                <div className="text-xs text-muted-foreground">Parallel Batches</div>
                            </div>
                            <GitBranch className="text-warning" size={20} />
                        </CardContent>
                    </Card>
                    <Card>
                        <CardContent className="p-4 flex items-center justify-between">
                            <div>
                                <div className="text-2xl font-bold text-foreground">{stats.heavyTasks}</div>
                                <div className="text-xs text-muted-foreground">Heavy Model Tasks</div>
                            </div>
                            <Brain className="text-primary" size={20} />
                        </CardContent>
                    </Card>
                </div>

                {batches.length === 0 ? (
                    <Card className="flex items-center justify-center h-64 text-muted-foreground border-dashed">
                        No plan available for this spec. Run 'ckrv plan' to generate one.
                    </Card>
                ) : (
                    <DagView
                        batches={batches}
                        selectedBatch={selectedBatch}
                        onSelectBatch={setSelectedBatch}
                        onEditBatch={setEditingBatch}
                        models={models}
                    />
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
                                <span className="text-warning">${(model.cost_per_1k_prompt + model.cost_per_1k_completion).toFixed(4)}/1k</span>
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
