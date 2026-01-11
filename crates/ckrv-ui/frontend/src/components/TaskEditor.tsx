import React, { useState, useEffect, useMemo } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
    ChevronDown, ChevronRight, Play,
    CheckCircle2, Circle, AlertTriangle, GitBranch,
    Layers, LayoutGrid, Code, Filter, Zap, Brain, Cpu,
    Link2, FileText, ArrowLeft, Save, Loader2, RotateCcw
} from 'lucide-react';
import { TaskDetailModal } from './TaskDetailModal';

// Types matching backend
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

interface SpecListItem {
    name: string;
    path: string;
    has_tasks: boolean;
    has_implementation: boolean;
}

// API functions
const fetchSpecs = async (): Promise<{ specs: SpecListItem[], count: number }> => {
    const res = await fetch('/api/specs');
    return res.json();
};

const fetchTasksDetail = async (spec: string): Promise<{ success: boolean; tasks: Task[]; raw_yaml?: string; count: number; error?: string }> => {
    const res = await fetch(`/api/tasks/detail?spec=${encodeURIComponent(spec)}`);
    return res.json();
};

const saveTasks = async (spec: string, tasks: Task[]): Promise<{ success: boolean; message?: string }> => {
    const res = await fetch('/api/tasks/save', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ spec, tasks }),
    });
    return res.json();
};

const updateTaskStatus = async (spec: string, taskId: string, status: string): Promise<{ success: boolean; message?: string }> => {
    const res = await fetch('/api/tasks/status', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ spec, task_id: taskId, status }),
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

const StatusBadge: React.FC<{ status: string; onClick?: () => void }> = ({ status, onClick }) => {
    const styles: Record<string, { bg: string; icon: React.ElementType }> = {
        pending: { bg: 'bg-gray-700 text-gray-300', icon: Circle },
        running: { bg: 'bg-blue-900/50 text-blue-300', icon: Play },
        completed: { bg: 'bg-green-900/50 text-green-300', icon: CheckCircle2 },
        failed: { bg: 'bg-red-900/50 text-red-300', icon: AlertTriangle }
    };
    const { bg, icon: Icon } = styles[status] || styles.pending;
    return (
        <button
            onClick={onClick}
            className={`text-xs font-medium px-2 py-1 rounded flex items-center gap-1 ${bg} hover:opacity-80 transition-opacity`}
        >
            <Icon size={12} />
            {status}
        </button>
    );
};

const ComplexityDots: React.FC<{ complexity: number }> = ({ complexity }) => (
    <div className="flex gap-0.5" title={`Complexity: ${complexity}/5`}>
        {[1, 2, 3, 4, 5].map(i => (
            <div
                key={i}
                className={`w-1.5 h-1.5 rounded-full ${i <= complexity ? 'bg-gray-400' : 'bg-gray-700'}`}
            />
        ))}
    </div>
);

// Task Card Component
const TaskCard: React.FC<{
    task: Task;
    onStatusChange: (status: string) => void;
    expanded: boolean;
    onToggleExpand: () => void;
}> = ({ task, onStatusChange, expanded, onToggleExpand }) => {
    const cycleStatus = () => {
        const order = ['pending', 'running', 'completed', 'failed'];
        const next = order[(order.indexOf(task.status) + 1) % order.length];
        onStatusChange(next);
    };

    return (
        <div className={`border border-gray-700 rounded-lg bg-gray-800/50 hover:bg-gray-800 transition-all ${task.status === 'completed' ? 'opacity-60' : ''}`}>
            <div className="p-4">
                <div className="flex items-start justify-between gap-3">
                    <div className="flex items-center gap-2 flex-wrap">
                        <span className="font-mono text-xs bg-gray-700 px-2 py-0.5 rounded text-gray-300">{task.id}</span>
                        {task.parallel && (
                            <span className="text-xs bg-emerald-900/50 text-emerald-300 px-2 py-0.5 rounded flex items-center gap-1">
                                <GitBranch size={12} /> parallel
                            </span>
                        )}
                        {task.user_story && (
                            <span className="text-xs bg-purple-900/50 text-purple-300 px-2 py-0.5 rounded">{task.user_story}</span>
                        )}
                    </div>
                    <StatusBadge status={task.status} onClick={cycleStatus} />
                </div>

                <h4 className="font-medium text-gray-200 mt-2 text-sm">{task.title}</h4>

                {task.file && (
                    <code className="text-xs text-cyan-400 bg-cyan-900/30 px-2 py-0.5 rounded mt-2 inline-block">
                        {task.file}
                    </code>
                )}

                <div className="flex items-center gap-3 mt-3 flex-wrap">
                    <RiskBadge risk={task.risk} />
                    <ModelTierBadge tier={task.model_tier} />
                    <ComplexityDots complexity={task.complexity} />
                    <span className="text-xs text-gray-500">{task.estimated_tokens} tokens</span>
                </div>

                {task.context_required.length > 0 && (
                    <div className="mt-3 pt-3 border-t border-gray-700">
                        <div className="text-xs text-gray-500 mb-1 flex items-center gap-1">
                            <Link2 size={12} /> Dependencies:
                        </div>
                        <div className="flex flex-wrap gap-1">
                            {task.context_required.map((dep, i) => (
                                <code key={i} className="text-xs bg-gray-700 text-gray-400 px-1.5 py-0.5 rounded">{dep}</code>
                            ))}
                        </div>
                    </div>
                )}

                <button
                    onClick={onToggleExpand}
                    className="text-xs text-gray-500 hover:text-gray-300 mt-3 flex items-center gap-1"
                >
                    {expanded ? <><ChevronDown size={14} /> Less</> : <><ChevronRight size={14} /> More</>}
                </button>

                {expanded && task.description && (
                    <div className="mt-3 pt-3 border-t border-gray-700">
                        <p className="text-sm text-gray-400 whitespace-pre-wrap">{task.description}</p>
                    </div>
                )}
            </div>
        </div>
    );
};

// Phase Group Component
const PhaseGroup: React.FC<{
    phase: string;
    tasks: Task[];
    onStatusChange: (taskId: string, status: string) => void;
    expandedTasks: Set<string>;
    toggleExpand: (id: string) => void;
}> = ({ phase, tasks, onStatusChange, expandedTasks, toggleExpand }) => {
    const [collapsed, setCollapsed] = useState(false);
    const completedCount = tasks.filter(t => t.status === 'completed').length;
    const totalTokens = tasks.reduce((sum, t) => sum + t.estimated_tokens, 0);

    const phaseColors: Record<string, string> = {
        'Setup': 'border-l-emerald-500 bg-emerald-900/10',
        'Foundation': 'border-l-blue-500 bg-blue-900/10',
        'User Story 1': 'border-l-violet-500 bg-violet-900/10',
        'User Story 2': 'border-l-purple-500 bg-purple-900/10',
        'User Story 3': 'border-l-fuchsia-500 bg-fuchsia-900/10',
        'User Story 4': 'border-l-pink-500 bg-pink-900/10',
        'User Story 5': 'border-l-rose-500 bg-rose-900/10',
        'User Story 6': 'border-l-orange-500 bg-orange-900/10',
        'Polish': 'border-l-amber-500 bg-amber-900/10',
    };

    return (
        <div className={`border-l-4 rounded-r-lg mb-4 ${phaseColors[phase] || 'border-l-gray-500 bg-gray-800/30'}`}>
            <button
                onClick={() => setCollapsed(!collapsed)}
                className="w-full px-4 py-3 flex items-center justify-between text-left hover:bg-white/5 transition-colors"
            >
                <div className="flex items-center gap-3">
                    {collapsed ? <ChevronRight size={18} /> : <ChevronDown size={18} />}
                    <span className="font-semibold text-gray-200">{phase}</span>
                    <span className="text-xs bg-gray-700 px-2 py-0.5 rounded-full text-gray-400">
                        {completedCount}/{tasks.length}
                    </span>
                </div>
                <div className="flex items-center gap-4 text-xs text-gray-500">
                    <span>{totalTokens.toLocaleString()} tokens</span>
                    <div className="w-24 h-1.5 bg-gray-700 rounded-full overflow-hidden">
                        <div
                            className="h-full bg-emerald-500 transition-all"
                            style={{ width: `${(completedCount / tasks.length) * 100}%` }}
                        />
                    </div>
                </div>
            </button>

            {!collapsed && (
                <div className="px-4 pb-4 grid gap-3">
                    {tasks.map(task => (
                        <TaskCard
                            key={task.id}
                            task={task}
                            onStatusChange={(status) => onStatusChange(task.id, status)}
                            expanded={expandedTasks.has(task.id)}
                            onToggleExpand={() => toggleExpand(task.id)}
                        />
                    ))}
                </div>
            )}
        </div>
    );
};

// Summary Stats
const SummaryStats: React.FC<{ tasks: Task[] }> = ({ tasks }) => {
    const stats = [
        { label: 'Total Tasks', value: tasks.length, color: 'bg-gray-500' },
        { label: 'Completed', value: tasks.filter(t => t.status === 'completed').length, color: 'bg-green-500' },
        { label: 'Parallelizable', value: tasks.filter(t => t.parallel).length, color: 'bg-emerald-500' },
        { label: 'Critical Risk', value: tasks.filter(t => t.risk === 'critical').length, color: 'bg-red-500' }
    ];

    return (
        <div className="grid grid-cols-4 gap-4 mb-6">
            {stats.map(({ label, value, color }) => (
                <div key={label} className="bg-gray-800/50 rounded-lg border border-gray-700 p-4">
                    <div className="text-2xl font-bold text-gray-200">{value}</div>
                    <div className="text-sm text-gray-500 flex items-center gap-2">
                        <div className={`w-2 h-2 rounded-full ${color}`} />
                        {label}
                    </div>
                </div>
            ))}
        </div>
    );
};

// Filter Bar
interface FilterState {
    phase: string;
    status: string;
    risk: string;
    tier: string;
    parallelOnly: boolean;
}

const FilterBar: React.FC<{
    filters: FilterState;
    setFilters: (f: FilterState) => void;
    phases: string[];
    stats: { filtered: number; total: number; tokens: number };
}> = ({ filters, setFilters, phases, stats }) => (
    <div className="bg-gray-800/50 border border-gray-700 rounded-lg p-4 mb-4">
        <div className="flex items-center gap-4 flex-wrap">
            <div className="flex items-center gap-2">
                <Filter size={16} className="text-gray-500" />
                <span className="text-sm font-medium text-gray-400">Filters:</span>
            </div>

            <select
                value={filters.phase}
                onChange={(e) => setFilters({ ...filters, phase: e.target.value })}
                className="text-sm border border-gray-600 rounded px-2 py-1 bg-gray-800 text-gray-300"
            >
                <option value="">All Phases</option>
                {phases.map(p => <option key={p} value={p}>{p}</option>)}
            </select>

            <select
                value={filters.status}
                onChange={(e) => setFilters({ ...filters, status: e.target.value })}
                className="text-sm border border-gray-600 rounded px-2 py-1 bg-gray-800 text-gray-300"
            >
                <option value="">All Status</option>
                <option value="pending">Pending</option>
                <option value="running">Running</option>
                <option value="completed">Completed</option>
                <option value="failed">Failed</option>
            </select>

            <select
                value={filters.risk}
                onChange={(e) => setFilters({ ...filters, risk: e.target.value })}
                className="text-sm border border-gray-600 rounded px-2 py-1 bg-gray-800 text-gray-300"
            >
                <option value="">All Risk</option>
                <option value="low">Low</option>
                <option value="medium">Medium</option>
                <option value="high">High</option>
                <option value="critical">Critical</option>
            </select>

            <select
                value={filters.tier}
                onChange={(e) => setFilters({ ...filters, tier: e.target.value })}
                className="text-sm border border-gray-600 rounded px-2 py-1 bg-gray-800 text-gray-300"
            >
                <option value="">All Tiers</option>
                <option value="light">Light</option>
                <option value="standard">Standard</option>
                <option value="heavy">Heavy</option>
            </select>

            <label className="flex items-center gap-2 text-sm text-gray-400 cursor-pointer">
                <input
                    type="checkbox"
                    checked={filters.parallelOnly}
                    onChange={(e) => setFilters({ ...filters, parallelOnly: e.target.checked })}
                    className="rounded bg-gray-700 border-gray-600"
                />
                Parallel only
            </label>

            <div className="ml-auto text-sm text-gray-500">
                {stats.filtered}/{stats.total} tasks • {stats.tokens.toLocaleString()} tokens
            </div>
        </div>
    </div>
);

// View Toggle
const ViewToggle: React.FC<{ view: string; setView: (v: string) => void }> = ({ view, setView }) => (
    <div className="flex bg-gray-800 rounded-lg p-1 gap-1">
        {[
            { id: 'phase', icon: Layers, label: 'By Phase' },
            { id: 'kanban', icon: LayoutGrid, label: 'Kanban' },
            { id: 'code', icon: Code, label: 'YAML' }
        ].map(({ id, icon: Icon, label }) => (
            <button
                key={id}
                onClick={() => setView(id)}
                className={`flex items-center gap-1.5 px-3 py-1.5 rounded text-sm font-medium transition-all ${view === id ? 'bg-gray-700 text-white shadow-sm' : 'text-gray-400 hover:text-gray-200'}`}
            >
                <Icon size={16} />
                {label}
            </button>
        ))}
    </div>
);

const KanbanColumn: React.FC<{
    status: string;
    tasks: Task[];
    onTaskClick: (task: Task) => void;
}> = ({ status, tasks, onTaskClick }) => {
    const statusConfig: Record<string, { label: string; color: string; headerColor: string; borderColor: string }> = {
        pending: {
            label: 'Pending',
            color: 'bg-slate-950/30',
            headerColor: 'bg-slate-900/50',
            borderColor: 'border-slate-800/50'
        },
        running: {
            label: 'Running',
            color: 'bg-indigo-950/30',
            headerColor: 'bg-indigo-900/40',
            borderColor: 'border-indigo-800/40'
        },
        completed: {
            label: 'Completed',
            color: 'bg-emerald-950/20',
            headerColor: 'bg-emerald-900/30',
            borderColor: 'border-emerald-800/30'
        },
        failed: {
            label: 'Failed',
            color: 'bg-rose-950/20',
            headerColor: 'bg-rose-900/30',
            borderColor: 'border-rose-800/30'
        }
    };
    const config = statusConfig[status] || statusConfig.pending;

    return (
        <div className={`rounded-lg ${config.color} min-w-[280px] flex-1 border ${config.borderColor} transition-colors duration-300`}>
            <div className={`${config.headerColor} px-4 py-2 rounded-t-lg flex items-center justify-between border-b ${config.borderColor}`}>
                <span className="font-medium text-gray-200">{config.label}</span>
                <span className="text-xs bg-gray-900/50 px-2 py-0.5 rounded-full text-gray-300">{tasks.length}</span>
            </div>
            <div className="p-3 space-y-2 max-h-[500px] overflow-y-auto">
                {tasks.map(task => (
                    <div
                        key={task.id}
                        className="bg-gray-800/80 rounded-lg p-3 border border-gray-700/50 hover:border-cyan-600/50 hover:shadow-lg hover:shadow-cyan-900/10 transition-all cursor-pointer backdrop-blur-sm"
                        onClick={() => onTaskClick(task)}
                    >
                        <div className="flex items-center gap-2 mb-1">
                            <span className="font-mono text-xs text-gray-500">{task.id}</span>
                            {task.parallel && <GitBranch size={12} className="text-emerald-400" />}
                        </div>
                        <p className="text-sm font-medium text-gray-300 line-clamp-2">{task.title}</p>
                        <div className="flex items-center gap-2 mt-2">
                            <ModelTierBadge tier={task.model_tier} />
                            <RiskBadge risk={task.risk} />
                        </div>
                    </div>
                ))}
            </div>
        </div>
    );
};

// YAML View
const YamlView: React.FC<{ rawYaml?: string }> = ({ rawYaml }) => (
    <pre className="font-mono text-sm bg-gray-900 text-gray-100 p-4 rounded-lg overflow-auto max-h-[60vh]">
        <code>{rawYaml || '# No YAML content'}</code>
    </pre>
);

// Spec List View
const SpecListView: React.FC<{
    specs: SpecListItem[];
    onSelect: (name: string) => void;
    isLoading: boolean;
}> = ({ specs, onSelect, isLoading }) => {
    const specsWithTasks = specs.filter(s => s.has_tasks);

    if (isLoading) {
        return (
            <div className="flex items-center justify-center h-64">
                <Loader2 className="animate-spin text-gray-500" size={24} />
            </div>
        );
    }

    if (specsWithTasks.length === 0) {
        return (
            <div className="text-center py-12 text-gray-500">
                <FileText size={48} className="mx-auto mb-4 opacity-50" />
                <p>No specs with tasks found</p>
                <p className="text-sm mt-2">Run <code className="bg-gray-800 px-2 py-0.5 rounded">ckrv spec tasks</code> to generate tasks</p>
            </div>
        );
    }

    return (
        <div className="space-y-2">
            {specsWithTasks.map((spec) => (
                <button
                    key={spec.name}
                    onClick={() => onSelect(spec.name)}
                    className="w-full text-left p-4 bg-gray-800/50 hover:bg-gray-800 rounded-lg border border-gray-700 transition-colors"
                >
                    <div className="flex items-center justify-between">
                        <div className="flex items-center gap-3">
                            <Layers size={20} className="text-cyan-400" />
                            <div>
                                <h3 className="font-medium text-gray-200">{spec.name}</h3>
                                <div className="flex items-center gap-2 mt-1">
                                    <span className="text-xs bg-green-900/50 text-green-300 px-2 py-0.5 rounded">has tasks</span>
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

// Main Task Editor Component
export const TaskEditor: React.FC = () => {
    const queryClient = useQueryClient();
    const [selectedSpecName, setSelectedSpecName] = useState<string | null>(null);
    const [tasks, setTasks] = useState<Task[]>([]);
    const [rawYaml, setRawYaml] = useState<string | undefined>();
    const [view, setView] = useState<'phase' | 'kanban' | 'code'>('phase');
    const [hasChanges, setHasChanges] = useState(false);
    const [expandedTasks, setExpandedTasks] = useState<Set<string>>(new Set());
    const [filters, setFilters] = useState({ phase: '', status: '', risk: '', tier: '', parallelOnly: false });
    const [selectedTask, setSelectedTask] = useState<Task | null>(null);

    // Fetch specs list
    const { data: specsData, isLoading: isLoadingSpecs } = useQuery({
        queryKey: ['specs'],
        queryFn: fetchSpecs,
    });

    // Fetch tasks detail when spec selected
    const { data: tasksDetailData, isLoading: isLoadingTasks } = useQuery({
        queryKey: ['tasks', selectedSpecName],
        queryFn: () => fetchTasksDetail(selectedSpecName!),
        enabled: !!selectedSpecName,
    });

    // Update local state when tasks are fetched
    useEffect(() => {
        if (tasksDetailData?.success && tasksDetailData.tasks) {
            setTasks(tasksDetailData.tasks);
            setRawYaml(tasksDetailData.raw_yaml);
            setHasChanges(false);
        }
    }, [tasksDetailData]);

    // Save mutation
    const saveMutation = useMutation({
        mutationFn: () => saveTasks(selectedSpecName!, tasks),
        onSuccess: (data: { success: boolean }) => {
            if (data.success) {
                setHasChanges(false);
                queryClient.invalidateQueries({ queryKey: ['tasks', selectedSpecName] });
            }
        },
    });

    // Status update mutation (for quick updates)
    const statusMutation = useMutation({
        mutationFn: ({ taskId, status }: { taskId: string; status: string }) =>
            updateTaskStatus(selectedSpecName!, taskId, status),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['tasks', selectedSpecName] });
        },
    });

    const handleStatusChange = (taskId: string, status: string) => {
        // Update local state immediately
        setTasks(prev => prev.map(t => t.id === taskId ? { ...t, status } : t));
        // Also persist to server
        statusMutation.mutate({ taskId, status });
    };

    const toggleExpand = (id: string) => {
        const newExpanded = new Set(expandedTasks);
        if (newExpanded.has(id)) newExpanded.delete(id);
        else newExpanded.add(id);
        setExpandedTasks(newExpanded);
    };

    const phases = useMemo(() => [...new Set(tasks.map(t => t.phase))], [tasks]);

    const filteredTasks = useMemo(() => {
        return tasks.filter(t => {
            if (filters.phase && t.phase !== filters.phase) return false;
            if (filters.status && t.status !== filters.status) return false;
            if (filters.risk && t.risk !== filters.risk) return false;
            if (filters.tier && t.model_tier !== filters.tier) return false;
            if (filters.parallelOnly && !t.parallel) return false;
            return true;
        });
    }, [tasks, filters]);

    const stats = {
        total: tasks.length,
        filtered: filteredTasks.length,
        tokens: filteredTasks.reduce((sum, t) => sum + t.estimated_tokens, 0)
    };

    const tasksByPhase = useMemo(() => {
        const grouped: Record<string, Task[]> = {};
        filteredTasks.forEach(t => {
            if (!grouped[t.phase]) grouped[t.phase] = [];
            grouped[t.phase].push(t);
        });
        return grouped;
    }, [filteredTasks]);

    const tasksByStatus = useMemo(() => {
        const grouped: Record<string, Task[]> = { pending: [], running: [], completed: [], failed: [] };
        filteredTasks.forEach(t => grouped[t.status]?.push(t));
        return grouped;
    }, [filteredTasks]);

    // Show spec list if nothing selected
    if (!selectedSpecName) {
        return (
            <div className="h-full overflow-auto p-4">
                <div className="mb-6">
                    <h1 className="text-2xl font-bold text-gray-100">Task Orchestration</h1>
                    <p className="text-gray-500 mt-1">Select a spec to view and manage tasks</p>
                </div>
                <SpecListView
                    specs={specsData?.specs || []}
                    onSelect={setSelectedSpecName}
                    isLoading={isLoadingSpecs}
                />
            </div>
        );
    }

    if (isLoadingTasks || tasks.length === 0) {
        return (
            <div className="flex items-center justify-center h-full">
                <Loader2 className="animate-spin text-gray-500" size={32} />
            </div>
        );
    }

    return (
        <div className="h-full flex flex-col overflow-hidden">
            {/* Header */}
            <div className="shrink-0 px-4 py-3 border-b border-gray-700 flex items-center justify-between bg-gray-900/50">
                <div className="flex items-center gap-4">
                    <button
                        onClick={() => setSelectedSpecName(null)}
                        className="p-2 hover:bg-gray-800 rounded-lg transition-colors"
                    >
                        <ArrowLeft size={20} className="text-gray-400" />
                    </button>
                    <div>
                        <span className="text-sm text-gray-500 font-mono">tasks.yaml</span>
                        <h1 className="text-lg font-semibold text-gray-200">{selectedSpecName}</h1>
                    </div>
                </div>
                <div className="flex items-center gap-3">
                    <ViewToggle view={view} setView={(v) => setView(v as typeof view)} />
                    {hasChanges && (
                        <button
                            onClick={() => {
                                if (tasksDetailData?.tasks) {
                                    setTasks(tasksDetailData.tasks);
                                    setRawYaml(tasksDetailData.raw_yaml);
                                    setHasChanges(false);
                                }
                            }}
                            className="flex items-center gap-2 px-4 py-2 rounded-lg font-medium transition-colors bg-gray-700 hover:bg-gray-600 text-gray-200"
                        >
                            <RotateCcw size={16} />
                            Discard
                        </button>
                    )}
                    <button
                        onClick={() => saveMutation.mutate()}
                        disabled={!hasChanges || saveMutation.isPending}
                        className={`flex items-center gap-2 px-4 py-2 rounded-lg font-medium transition-colors ${hasChanges
                            ? 'bg-cyan-600 hover:bg-cyan-500 text-white'
                            : 'bg-gray-800 text-gray-500 cursor-not-allowed'
                            }`}
                    >
                        {saveMutation.isPending ? (
                            <Loader2 size={16} className="animate-spin" />
                        ) : (
                            <Save size={16} />
                        )}
                        Save
                    </button>
                </div>
            </div>

            {/* Content */}
            <div className="flex-1 overflow-auto p-4">
                {view !== 'code' && (
                    <>
                        <SummaryStats tasks={tasks} />
                        <FilterBar filters={filters} setFilters={setFilters} phases={phases} stats={stats} />
                    </>
                )}

                {view === 'phase' && (
                    <div>
                        {phases.filter(p => tasksByPhase[p]?.length > 0).map(phase => (
                            <PhaseGroup
                                key={phase}
                                phase={phase}
                                tasks={tasksByPhase[phase]}
                                onStatusChange={handleStatusChange}
                                expandedTasks={expandedTasks}
                                toggleExpand={toggleExpand}
                            />
                        ))}
                    </div>
                )}

                {view === 'kanban' && (
                    <div className="flex gap-4 overflow-x-auto pb-4">
                        {(['pending', 'running', 'completed', 'failed'] as const).map(status => (
                            <KanbanColumn
                                key={status}
                                status={status}
                                tasks={tasksByStatus[status]}
                                onTaskClick={setSelectedTask}
                            />
                        ))}
                    </div>
                )}

                {view === 'code' && (
                    <div className="rounded-lg overflow-hidden border border-gray-700">
                        <div className="bg-gray-800 px-4 py-2 flex items-center justify-between border-b border-gray-700">
                            <span className="text-gray-400 text-sm">tasks.yaml</span>
                            <button className="text-xs text-gray-400 hover:text-white">Copy</button>
                        </div>
                        <YamlView rawYaml={rawYaml} />
                    </div>
                )}
            </div>

            {/* Status Bar */}
            <div className="shrink-0 px-4 py-2 border-t border-gray-700 flex items-center justify-between text-sm text-gray-500 bg-gray-900/50">
                <div className="flex items-center gap-4">
                    <span>{tasks.length} tasks</span>
                    <span>{tasks.filter(t => t.status === 'completed').length} completed</span>
                    <span>{tasks.reduce((s, t) => s + t.estimated_tokens, 0).toLocaleString()} tokens</span>
                </div>
                <div className="flex items-center gap-2">
                    <span className={`w-2 h-2 rounded-full ${hasChanges ? 'bg-amber-500' : 'bg-green-500'}`}></span>
                    <span>{hasChanges ? 'Unsaved changes' : 'All changes saved'}</span>
                </div>
            </div>

            {/* Task Detail Modal */}
            {selectedTask && selectedSpecName && (
                <TaskDetailModal
                    task={selectedTask}
                    specName={selectedSpecName}
                    onClose={() => setSelectedTask(null)}
                    onStatusChange={(taskId, status) => {
                        handleStatusChange(taskId, status);
                        // Update the selected task if it's still selected
                        setSelectedTask(prev => prev?.id === taskId ? { ...prev, status } : prev);
                    }}
                />
            )}
        </div>
    );
};
