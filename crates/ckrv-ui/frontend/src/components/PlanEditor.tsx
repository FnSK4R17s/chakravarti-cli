import React, { useState, useMemo } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import {
    ChevronDown, ChevronRight,
    GitBranch, Layers, List, Code,
    Zap, Brain, Cpu, ArrowRight, Link2, DollarSign, Timer,
    Network, Workflow, Box, Sparkles,
    Save
} from 'lucide-react';

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
    openrouter?: {
        model: string;
    };
}

// API Functions
const fetchSpecs = async (): Promise<{ specs: Spec[] }> => {
    const res = await fetch('/api/specs');
    return res.json();
};

const fetchAgents = async (): Promise<{ agents: Record<string, AgentConfig> }> => {
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
    let color = 'slate';
    let icon = Cpu;
    let tier = 'standard';

    if (modelId.includes('claude') || modelId.includes('gpt-4')) {
        color = 'amber';
        icon = Brain;
        tier = 'heavy';
    } else if (modelId.includes('minimax') || modelId.includes('haiku') || modelId.includes('flash')) {
        color = 'sky';
        icon = Zap;
        tier = 'light';
    } else if (modelId.includes('glm') || modelId.includes('llama')) {
        color = 'violet';
        tier = 'standard';
    }

    // Default costs if not found (fallback)
    const costPer1k = info ? (info.cost_per_1k_prompt + info.cost_per_1k_completion) / 2 : 0.001;

    return {
        name: info?.name || modelId.split('/').pop() || modelId,
        tier,
        color,
        icon,
        costPer1k,
        contextWindow: info?.context_length
    };
};

// Components

const ModelBadge: React.FC<{ model: string; size?: 'sm' | 'md' | 'lg'; models: ModelInfo[] }> = ({ model, size = 'md', models }) => {
    const config = getModelConfig(model, models);
    const Icon = config.icon;
    const sizeClasses = {
        sm: 'text-xs px-1.5 py-0.5 max-w-[140px]',
        md: 'text-xs px-2 py-1 max-w-[180px]',
        lg: 'text-sm px-3 py-1.5 max-w-[220px]'
    };
    const colorClasses: Record<string, string> = {
        sky: 'bg-sky-900/40 text-sky-300 border-sky-800',
        violet: 'bg-violet-900/40 text-violet-300 border-violet-800',
        amber: 'bg-amber-900/40 text-amber-300 border-amber-800',
        slate: 'bg-slate-800 text-slate-300 border-slate-700'
    };

    return (
        <span
            className={`font-medium rounded border inline-flex items-center gap-1 ${sizeClasses[size]} ${colorClasses[config.color] || colorClasses.slate}`}
            title={config.name}
        >
            <Icon size={size === 'sm' ? 10 : 12} className="shrink-0" />
            <span className="truncate">{config.name}</span>
        </span>
    );
};

const StrategyBadge: React.FC<{ strategy: string }> = ({ strategy }) => {
    const isParallel = strategy === 'parallel';
    return (
        <span className={`text-xs px-2 py-0.5 rounded flex items-center gap-1 ${isParallel ? 'bg-emerald-900/40 text-emerald-400 border border-emerald-800' : 'bg-slate-800 text-slate-400 border border-slate-700'
            }`}>
            {isParallel ? <GitBranch size={12} /> : <ArrowRight size={12} />}
            {strategy}
        </span>
    );
};

const BatchCard: React.FC<{
    batch: Batch;
    isSelected: boolean;
    onClick: () => void;
    models: ModelInfo[]
}> = ({ batch, isSelected, onClick, models }) => {
    const [expanded, setExpanded] = useState(false);

    return (
        <div
            className={`border rounded-lg bg-gray-900/40 shadow-sm hover:shadow-md transition-all cursor-pointer ${isSelected ? 'ring-1 ring-cyan-500 border-cyan-500/50' : 'border-gray-800 hover:border-gray-700'
                }`}
            onClick={onClick}
        >
            <div className="p-4">
                <div className="flex items-start justify-between gap-2">
                    <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2 mb-1">
                            <code className="text-xs bg-gray-800 px-1.5 py-0.5 rounded text-gray-400 truncate">
                                {batch.id}
                            </code>
                            <StrategyBadge strategy={batch.execution_strategy} />
                        </div>
                        <h4 className="font-medium text-gray-200 truncate">{batch.name}</h4>
                    </div>
                    <ModelBadge model={batch.model_assignment.default} size="sm" models={models} />
                </div>

                {/* Tasks */}
                <div className="flex flex-wrap gap-1 mt-3">
                    {batch.task_ids.map(taskId => {
                        const override = batch.model_assignment.overrides[taskId];
                        return (
                            <span
                                key={taskId}
                                className={`text-xs px-1.5 py-0.5 rounded font-mono ${override
                                    ? 'bg-amber-900/30 text-amber-400 border border-amber-800/50'
                                    : 'bg-gray-800 text-gray-400'
                                    }`}
                                title={override ? `Override: ${override}` : undefined}
                            >
                                {taskId}
                                {override && <Sparkles size={10} className="inline ml-0.5" />}
                            </span>
                        );
                    })}
                </div>

                {/* Dependencies */}
                {batch.depends_on.length > 0 && (
                    <div className="mt-3 pt-3 border-t border-gray-800">
                        <div className="text-xs text-gray-500 flex items-center gap-1 mb-1">
                            <Link2 size={12} /> Depends on:
                        </div>
                        <div className="flex flex-wrap gap-1">
                            {batch.depends_on.map(dep => (
                                <code key={dep} className="text-xs bg-blue-900/20 text-blue-400 px-1.5 py-0.5 rounded border border-blue-900/30">
                                    {dep}
                                </code>
                            ))}
                        </div>
                    </div>
                )}

                {/* Stats */}
                <div className="flex items-center gap-4 mt-3 text-xs text-gray-500">
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
                <button
                    onClick={(e) => { e.stopPropagation(); setExpanded(!expanded); }}
                    className="text-xs text-gray-500 hover:text-gray-300 mt-2 flex items-center gap-1"
                >
                    {expanded ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
                    Reasoning
                </button>

                {expanded && (
                    <p className="text-sm text-gray-400 mt-2 p-2 bg-gray-800/50 rounded border border-gray-800">
                        {batch.reasoning}
                    </p>
                )}
            </div>
        </div>
    );
};

// DAG View
const DagView: React.FC<{ batches: Batch[]; selectedBatch: string | null; onSelectBatch: (id: string) => void; models: ModelInfo[] }> = ({ batches, selectedBatch, onSelectBatch, models }) => {
    const levels = useMemo(() => {
        const batchMap = new Map(batches.map(b => [b.id, b]));
        const levelMap = new Map<string, number>();

        const getLevel = (batchId: string, visited = new Set<string>()): number => {
            if (visited.has(batchId)) return 0;
            if (levelMap.has(batchId)) return levelMap.get(batchId)!;

            visited.add(batchId);
            const batch = batchMap.get(batchId);
            if (!batch || batch.depends_on.length === 0) {
                levelMap.set(batchId, 0);
                return 0;
            }

            const maxDepLevel = Math.max(...batch.depends_on.map(dep => getLevel(dep, visited)), -1); // -1 ensures 0 if empty but caught above
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

    const modelColorClasses: Record<string, string> = {
        sky: 'bg-sky-900/20 border-sky-800 hover:bg-sky-900/30',
        violet: 'bg-violet-900/20 border-violet-800 hover:bg-violet-900/30',
        amber: 'bg-amber-900/20 border-amber-800 hover:bg-amber-900/30',
        slate: 'bg-gray-800/50 border-gray-700 hover:bg-gray-800'
    };

    return (
        <div className="bg-gray-900/30 rounded-lg border border-gray-800 p-6 overflow-x-auto">
            <div className="flex gap-8 min-w-max">
                {Object.entries(batchesByLevel).map(([level, levelBatches]) => (
                    <div key={level} className="flex flex-col gap-3">
                        <div className="text-xs font-medium text-gray-500 text-center mb-2">
                            Stage {parseInt(level) + 1}
                        </div>
                        {levelBatches.map(batch => {
                            const config = getModelConfig(batch.model_assignment.default, models);
                            return (
                                <div
                                    key={batch.id}
                                    onClick={() => onSelectBatch(batch.id)}
                                    className={`
                                      w-56 p-3 rounded-lg border-2 cursor-pointer transition-all
                                      ${modelColorClasses[config.color] || modelColorClasses.slate}
                                      ${selectedBatch === batch.id ? 'ring-2 ring-blue-500 ring-offset-2 ring-offset-gray-900' : ''}
                                    `}
                                >
                                    <div className="font-medium text-sm text-gray-200 truncate">{batch.name}</div>
                                    <div className="flex items-center gap-2 mt-1">
                                        <StrategyBadge strategy={batch.execution_strategy} />
                                        <span className="text-xs text-gray-500">{batch.task_ids.length} tasks</span>
                                    </div>
                                    <div className="mt-2">
                                        <ModelBadge model={batch.model_assignment.default} size="sm" models={models} />
                                    </div>
                                    {batch.depends_on.length > 0 && (
                                        <div className="text-xs text-gray-600 mt-2 flex items-center gap-1">
                                            <ArrowRight size={10} />
                                            {batch.depends_on.length} deps
                                        </div>
                                    )}
                                </div>
                            );
                        })}
                    </div>
                ))}
            </div>

            {/* Legend */}
            <div className="flex items-center gap-4 mt-6 pt-4 border-t border-gray-800">
                <span className="text-xs text-gray-500">Model Tiers:</span>
                <div className="flex items-center gap-2">
                    <span className="w-3 h-3 rounded bg-sky-900/40 border border-sky-800"></span>
                    <span className="text-xs text-gray-400">Light</span>
                </div>
                <div className="flex items-center gap-2">
                    <span className="w-3 h-3 rounded bg-violet-900/40 border border-violet-800"></span>
                    <span className="text-xs text-gray-400">Standard</span>
                </div>
                <div className="flex items-center gap-2">
                    <span className="w-3 h-3 rounded bg-amber-900/40 border border-amber-800"></span>
                    <span className="text-xs text-gray-400">Heavy</span>
                </div>
            </div>
        </div>
    );
};

// Spec List View
const SpecListView: React.FC<{
    specs: Spec[];
    onSelect: (name: string) => void;
    isLoading: boolean;
}> = ({ specs, onSelect, isLoading }) => {
    const specsWithPlan = specs.filter(s => s.has_plan);

    if (isLoading) {
        return (
            <div className="flex items-center justify-center h-64">
                <Workflow className="animate-spin text-gray-500" size={24} />
            </div>
        );
    }

    if (specsWithPlan.length === 0) {
        return (
            <div className="text-center py-12 text-gray-500">
                <Workflow size={48} className="mx-auto mb-4 opacity-50" />
                <p>No specs with execution plans found</p>
                <p className="text-sm mt-2">Run <code className="bg-gray-800 px-2 py-0.5 rounded">ckrv plan</code> to generate an execution plan</p>
            </div>
        );
    }

    return (
        <div className="space-y-2">
            {specsWithPlan.map((spec) => (
                <button
                    key={spec.name}
                    onClick={() => onSelect(spec.name)}
                    className="w-full text-left p-4 bg-gray-800/50 hover:bg-gray-800 rounded-lg border border-gray-700 transition-colors"
                >
                    <div className="flex items-center justify-between">
                        <div className="flex items-center gap-3">
                            <Workflow size={20} className="text-cyan-400" />
                            <div>
                                <h3 className="font-medium text-gray-200">{spec.name}</h3>
                                <div className="flex items-center gap-2 mt-1">
                                    <span className="text-xs bg-green-900/50 text-green-300 px-2 py-0.5 rounded">has plan</span>
                                    <span className="text-xs text-gray-500">{spec.task_count} tasks</span>
                                    {spec.has_implementation && (
                                        <span className="text-xs bg-purple-900/50 text-purple-300 px-2 py-0.5 rounded">
                                            implemented
                                        </span>
                                    )}
                                </div>
                            </div>
                        </div>
                        <ChevronRight size={20} className="text-gray-500" />
                    </div>
                </button>
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

    // Data Fetching
    const { data: specsData, isLoading: isLoadingSpecs } = useQuery({ queryKey: ['specs'], queryFn: fetchSpecs });
    const { data: modelsData } = useQuery({ queryKey: ['openrouter-models'], queryFn: fetchModels });
    const { data: agentsData } = useQuery({ queryKey: ['agents'], queryFn: fetchAgents });
    const { data: planData, isError, error } = useQuery({
        queryKey: ['plan', selectedSpecName],
        queryFn: () => fetchPlan(selectedSpecName!),
        enabled: !!selectedSpecName
    });

    const batches = useMemo(() => planData?.batches || [], [planData]);
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
            queryClient.invalidateQueries({ queryKey: ['plan', selectedSpecName] });
        } catch (e) {
            console.error(e);
        }
    };

    if (isError) {
        return <div className="p-8 text-red-400">Error loading plan: {(error as Error).message}</div>;
    }

    // Show spec list if nothing selected
    if (!selectedSpecName) {
        return (
            <div className="h-full overflow-auto p-4">
                <div className="mb-6">
                    <h1 className="text-2xl font-bold text-gray-100">Execution Plan</h1>
                    <p className="text-gray-500 mt-1">Select a spec to view its execution plan</p>
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
        <div className="h-full flex flex-col bg-gray-950 text-gray-100 font-sans">
            {/* Header */}
            <div className="shrink-0 px-6 py-4 border-b border-gray-800 flex items-center justify-between bg-gray-900/50">
                <div className="flex items-center gap-4">
                    <button
                        onClick={() => setSelectedSpecName(null)}
                        className="p-2 hover:bg-gray-800 rounded-lg transition-colors"
                    >
                        <ArrowRight size={20} className="text-gray-400 rotate-180" />
                    </button>
                    <div>
                        <div className="text-sm text-gray-500 font-mono">plan.yaml</div>
                        <h1 className="text-lg font-semibold text-gray-100">{selectedSpecName}</h1>
                    </div>
                    {lastSaved && <span className="text-xs text-gray-500">• Saved {lastSaved.toLocaleTimeString()}</span>}
                </div>

                <div className="flex items-center gap-4">
                    {/* View Toggles */}
                    <div className="flex bg-gray-800 rounded-lg p-1 gap-1">
                        {[
                            { id: 'dag', icon: Network, label: 'DAG' },
                            { id: 'list', icon: List, label: 'List' },
                            // { id: 'timeline', icon: BarChart3, label: 'Timeline' },
                            // { id: 'cost', icon: DollarSign, label: 'Cost' },
                            { id: 'code', icon: Code, label: 'YAML' }
                        ].map((item) => (
                            <button
                                key={item.id}
                                onClick={() => setView(item.id as any)}
                                className={`flex items-center gap-1.5 px-3 py-1.5 rounded text-sm font-medium transition-all ${view === item.id ? 'bg-gray-700 text-white shadow-sm' : 'text-gray-400 hover:text-gray-200'
                                    }`}
                            >
                                <item.icon size={16} />
                                {item.label}
                            </button>
                        ))}
                    </div>

                    <div className="w-px h-6 bg-gray-800" />

                    <button onClick={handleSave} className="p-2 bg-indigo-600 hover:bg-indigo-500 text-white rounded-lg transition-colors" title="Save Plan">
                        <Save size={20} />
                    </button>
                </div>
            </div>

            {/* Content */}
            <div className="flex-1 overflow-auto p-6">
                {/* Stats Row */}
                <div className="grid grid-cols-4 gap-4 mb-6">
                    <div className="bg-gray-900/50 border border-gray-800 p-4 rounded-lg">
                        <div className="flex items-center justify-between">
                            <div>
                                <div className="text-2xl font-bold text-gray-200">{stats.totalBatches}</div>
                                <div className="text-xs text-gray-500">Batches</div>
                            </div>
                            <Layers className="text-gray-600" size={20} />
                        </div>
                    </div>
                    <div className="bg-gray-900/50 border border-gray-800 p-4 rounded-lg">
                        <div className="flex items-center justify-between">
                            <div>
                                <div className="text-2xl font-bold text-gray-200">{stats.totalTasks}</div>
                                <div className="text-xs text-gray-500">Total Tasks</div>
                            </div>
                            <Box className="text-blue-500/50" size={20} />
                        </div>
                    </div>
                    <div className="bg-gray-900/50 border border-gray-800 p-4 rounded-lg">
                        <div className="flex items-center justify-between">
                            <div>
                                <div className="text-2xl font-bold text-gray-200">${stats.totalCost.toFixed(2)}</div>
                                <div className="text-xs text-gray-500">Est. Cost</div>
                            </div>
                            <DollarSign className="text-amber-500/50" size={20} />
                        </div>
                    </div>
                    <div className="bg-gray-900/50 border border-gray-800 p-4 rounded-lg">
                        <div className="flex items-center justify-between">
                            <div>
                                <div className="text-2xl font-bold text-gray-200">{stats.heavyTasks}</div>
                                <div className="text-xs text-gray-500">Heavy Model Tasks</div>
                            </div>
                            <Brain className="text-violet-500/50" size={20} />
                        </div>
                    </div>
                </div>

                {batches.length === 0 ? (
                    <div className="flex items-center justify-center h-64 text-gray-500 border-2 border-dashed border-gray-800 rounded-lg">
                        No plan available for this spec. Run 'ckrv plan' to generate one.
                    </div>
                ) : (
                    <>
                        {view === 'dag' && (
                            <DagView
                                batches={batches}
                                selectedBatch={selectedBatch}
                                onSelectBatch={setSelectedBatch}
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
                                        models={models}
                                    />
                                ))}
                            </div>
                        )}

                        {view === 'code' && (
                            <div className="bg-gray-900 rounded-lg border border-gray-800 overflow-hidden">
                                <div className="px-4 py-2 border-b border-gray-800 bg-gray-900/50 flex items-center justify-between">
                                    <span className="text-xs font-mono text-gray-500">plan.yaml</span>
                                    <button className="text-xs text-gray-400 hover:text-white">Copy</button>
                                </div>
                                <pre className="p-4 text-xs font-mono text-gray-300 overflow-auto max-h-[600px]">
                                    {planData?.raw_yaml}
                                </pre>
                            </div>
                        )}
                    </>
                )}
            </div>

            {/* Model Pricing Footer (Pulled from API) */}
            <div className="shrink-0 px-6 py-3 border-t border-gray-800 bg-gray-900/80 text-xs">
                <div className="flex items-center gap-6 overflow-x-auto">
                    <span className="text-gray-500 font-medium whitespace-nowrap">Current Pricing (Configured Models):</span>
                    {models.filter(m => configuredModels.has(m.id))
                        .map(model => (
                            <div key={model.id} className="flex items-center gap-2 whitespace-nowrap">
                                <span className="text-gray-400">{model.name}:</span>
                                <span className="text-amber-500">${(model.cost_per_1k_prompt + model.cost_per_1k_completion).toFixed(4)}/1k</span>
                            </div>
                        ))}
                    {configuredModels.size === 0 && <span className="text-gray-600 italic">No configured models found</span>}
                </div>
            </div>
        </div>
    );
}
