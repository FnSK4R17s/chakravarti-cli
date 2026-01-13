import React, { useState, useEffect, useMemo } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
    ChevronDown, ChevronRight, Play,
    CheckCircle2, Circle, AlertTriangle, GitBranch,
    Layers, LayoutGrid, Code, Filter, Zap, Brain, Cpu,
    Link2, FileText, ArrowLeft, Save, Loader2, RotateCcw
} from 'lucide-react';
import { TaskDetailModal } from './TaskDetailModal';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible';
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs';
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select';
import { Checkbox } from '@/components/ui/checkbox';

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

const StatusBadge: React.FC<{ status: string; onClick?: () => void }> = ({ status, onClick }) => {
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
        <Badge
            variant={variants[status] || 'secondary'}
            className="cursor-pointer hover:opacity-80 transition-opacity flex items-center gap-1"
            onClick={onClick}
        >
            <Icon size={12} />
            {status}
        </Badge>
    );
};

const ComplexityDots: React.FC<{ complexity: number }> = ({ complexity }) => (
    <div className="flex gap-0.5" title={`Complexity: ${complexity}/5`}>
        {[1, 2, 3, 4, 5].map(i => (
            <div
                key={i}
                className={`w-1.5 h-1.5 rounded-full ${i <= complexity ? 'bg-muted-foreground' : 'bg-muted'}`}
            />
        ))}
    </div>
);

// Task Card Component using shadcn Card
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
        <Card className={`transition-all ${task.status === 'completed' ? 'opacity-60' : ''}`}>
            <CardContent className="p-4">
                <div className="flex items-start justify-between gap-3">
                    <div className="flex items-center gap-2 flex-wrap">
                        <Badge variant="secondary" className="font-mono text-xs">{task.id}</Badge>
                        {task.parallel && (
                            <Badge variant="success" className="flex items-center gap-1">
                                <GitBranch size={12} /> parallel
                            </Badge>
                        )}
                        {task.user_story && (
                            <Badge variant="info">{task.user_story}</Badge>
                        )}
                    </div>
                    <StatusBadge status={task.status} onClick={cycleStatus} />
                </div>

                <h4 className="font-medium text-foreground mt-2 text-sm">{task.title}</h4>

                {task.file && (
                    <code className="text-xs text-accent-cyan bg-accent-cyan-dim px-2 py-0.5 rounded mt-2 inline-block">
                        {task.file}
                    </code>
                )}

                <div className="flex items-center gap-3 mt-3 flex-wrap">
                    <RiskBadge risk={task.risk} />
                    <ModelTierBadge tier={task.model_tier} />
                    <ComplexityDots complexity={task.complexity} />
                    <span className="text-xs text-muted-foreground">{task.estimated_tokens} tokens</span>
                </div>

                {task.context_required.length > 0 && (
                    <div className="mt-3 pt-3 border-t border-border">
                        <div className="text-xs text-muted-foreground mb-1 flex items-center gap-1">
                            <Link2 size={12} /> Dependencies:
                        </div>
                        <div className="flex flex-wrap gap-1">
                            {task.context_required.map((dep, i) => (
                                <code key={i} className="text-xs bg-muted text-muted-foreground px-1.5 py-0.5 rounded">{dep}</code>
                            ))}
                        </div>
                    </div>
                )}

                <Button
                    variant="ghost"
                    size="sm"
                    onClick={onToggleExpand}
                    className="mt-3 text-xs"
                >
                    {expanded ? <><ChevronDown size={14} /> Less</> : <><ChevronRight size={14} /> More</>}
                </Button>

                {expanded && task.description && (
                    <div className="mt-3 pt-3 border-t border-border">
                        <p className="text-sm text-muted-foreground whitespace-pre-wrap">{task.description}</p>
                    </div>
                )}
            </CardContent>
        </Card>
    );
};

// Phase Group Component using Collapsible
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
        'Setup': 'border-l-accent-green bg-accent-green-dim',
        'Foundation': 'border-l-accent-cyan bg-accent-cyan-dim',
        'User Story 1': 'border-l-accent-purple bg-accent-purple-dim',
        'User Story 2': 'border-l-accent-purple bg-accent-purple-dim',
        'User Story 3': 'border-l-accent-purple bg-accent-purple-dim',
        'User Story 4': 'border-l-accent-purple bg-accent-purple-dim',
        'User Story 5': 'border-l-accent-purple bg-accent-purple-dim',
        'User Story 6': 'border-l-accent-purple bg-accent-purple-dim',
        'Polish': 'border-l-accent-amber bg-accent-amber-dim',
    };

    return (
        <Collapsible open={!collapsed} onOpenChange={(open) => setCollapsed(!open)}>
            <Card className={`border-l-4 mb-4 ${phaseColors[phase] || 'border-l-border bg-muted/30'}`}>
                <CollapsibleTrigger asChild>
                    <button className="w-full px-4 py-3 flex items-center justify-between text-left hover:bg-accent/50 transition-colors">
                        <div className="flex items-center gap-3">
                            {collapsed ? <ChevronRight size={18} /> : <ChevronDown size={18} />}
                            <span className="font-semibold text-foreground">{phase}</span>
                            <Badge variant="secondary">
                                {completedCount}/{tasks.length}
                            </Badge>
                        </div>
                        <div className="flex items-center gap-4 text-xs text-muted-foreground">
                            <span>{totalTokens.toLocaleString()} tokens</span>
                            <div className="w-24 h-1.5 bg-muted rounded-full overflow-hidden">
                                <div
                                    className="h-full bg-accent-green transition-all"
                                    style={{ width: `${(completedCount / tasks.length) * 100}%` }}
                                />
                            </div>
                        </div>
                    </button>
                </CollapsibleTrigger>
                <CollapsibleContent>
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
                </CollapsibleContent>
            </Card>
        </Collapsible>
    );
};

// Summary Stats using Card
const SummaryStats: React.FC<{ tasks: Task[] }> = ({ tasks }) => {
    const stats = [
        { label: 'Total Tasks', value: tasks.length, color: 'bg-muted-foreground' },
        { label: 'Completed', value: tasks.filter(t => t.status === 'completed').length, color: 'bg-accent-green' },
        { label: 'Parallelizable', value: tasks.filter(t => t.parallel).length, color: 'bg-accent-cyan' },
        { label: 'Critical Risk', value: tasks.filter(t => t.risk === 'critical').length, color: 'bg-destructive' }
    ];

    return (
        <div className="grid grid-cols-4 gap-4 mb-6">
            {stats.map(({ label, value, color }) => (
                <Card key={label}>
                    <CardContent className="p-4">
                        <div className="text-2xl font-bold text-foreground">{value}</div>
                        <div className="text-sm text-muted-foreground flex items-center gap-2">
                            <div className={`w-2 h-2 rounded-full ${color}`} />
                            {label}
                        </div>
                    </CardContent>
                </Card>
            ))}
        </div>
    );
};

// Filter Bar using Select components
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
    <Card className="mb-4">
        <CardContent className="p-4">
            <div className="flex items-center gap-4 flex-wrap">
                <div className="flex items-center gap-2">
                    <Filter size={16} className="text-muted-foreground" />
                    <span className="text-sm font-medium text-muted-foreground">Filters:</span>
                </div>

                <Select value={filters.phase} onValueChange={(v) => setFilters({ ...filters, phase: v })}>
                    <SelectTrigger className="w-[140px]">
                        <SelectValue placeholder="All Phases" />
                    </SelectTrigger>
                    <SelectContent>
                        <SelectItem value="__all__">All Phases</SelectItem>
                        {phases.map(p => <SelectItem key={p} value={p}>{p}</SelectItem>)}
                    </SelectContent>
                </Select>

                <Select value={filters.status} onValueChange={(v) => setFilters({ ...filters, status: v })}>
                    <SelectTrigger className="w-[130px]">
                        <SelectValue placeholder="All Status" />
                    </SelectTrigger>
                    <SelectContent>
                        <SelectItem value="__all__">All Status</SelectItem>
                        <SelectItem value="pending">Pending</SelectItem>
                        <SelectItem value="running">Running</SelectItem>
                        <SelectItem value="completed">Completed</SelectItem>
                        <SelectItem value="failed">Failed</SelectItem>
                    </SelectContent>
                </Select>

                <Select value={filters.risk} onValueChange={(v) => setFilters({ ...filters, risk: v })}>
                    <SelectTrigger className="w-[120px]">
                        <SelectValue placeholder="All Risk" />
                    </SelectTrigger>
                    <SelectContent>
                        <SelectItem value="__all__">All Risk</SelectItem>
                        <SelectItem value="low">Low</SelectItem>
                        <SelectItem value="medium">Medium</SelectItem>
                        <SelectItem value="high">High</SelectItem>
                        <SelectItem value="critical">Critical</SelectItem>
                    </SelectContent>
                </Select>

                <Select value={filters.tier} onValueChange={(v) => setFilters({ ...filters, tier: v })}>
                    <SelectTrigger className="w-[120px]">
                        <SelectValue placeholder="All Tiers" />
                    </SelectTrigger>
                    <SelectContent>
                        <SelectItem value="__all__">All Tiers</SelectItem>
                        <SelectItem value="light">Light</SelectItem>
                        <SelectItem value="standard">Standard</SelectItem>
                        <SelectItem value="heavy">Heavy</SelectItem>
                    </SelectContent>
                </Select>

                <label className="flex items-center gap-2 text-sm text-muted-foreground cursor-pointer">
                    <Checkbox
                        checked={filters.parallelOnly}
                        onCheckedChange={(checked: boolean | "indeterminate") => setFilters({ ...filters, parallelOnly: checked === true })}
                    />
                    Parallel only
                </label>

                <div className="ml-auto text-sm text-muted-foreground">
                    {stats.filtered}/{stats.total} tasks • {stats.tokens.toLocaleString()} tokens
                </div>
            </div>
        </CardContent>
    </Card>
);

// View Toggle using Tabs
const ViewToggle: React.FC<{ view: string; setView: (v: string) => void }> = ({ view, setView }) => (
    <Tabs value={view} onValueChange={setView}>
        <TabsList>
            <TabsTrigger value="phase" className="gap-1.5">
                <Layers size={16} />
                By Phase
            </TabsTrigger>
            <TabsTrigger value="kanban" className="gap-1.5">
                <LayoutGrid size={16} />
                Kanban
            </TabsTrigger>
            <TabsTrigger value="code" className="gap-1.5">
                <Code size={16} />
                YAML
            </TabsTrigger>
        </TabsList>
    </Tabs>
);

const KanbanColumn: React.FC<{
    status: string;
    tasks: Task[];
    onTaskClick: (task: Task) => void;
}> = ({ status, tasks, onTaskClick }) => {
    const statusConfig: Record<string, { label: string; variant: "secondary" | "info" | "success" | "destructive" }> = {
        pending: { label: 'Pending', variant: 'secondary' },
        running: { label: 'Running', variant: 'info' },
        completed: { label: 'Completed', variant: 'success' },
        failed: { label: 'Failed', variant: 'destructive' }
    };
    const config = statusConfig[status] || statusConfig.pending;

    return (
        <Card className="min-w-[280px] flex-1">
            <CardHeader className="py-2 px-4 flex flex-row items-center justify-between">
                <CardTitle className="text-sm font-medium">{config.label}</CardTitle>
                <Badge variant={config.variant}>{tasks.length}</Badge>
            </CardHeader>
            <CardContent className="p-3 space-y-2 max-h-[500px] overflow-y-auto">
                {tasks.map(task => (
                    <Card
                        key={task.id}
                        className="cursor-pointer hover:border-primary transition-all"
                        onClick={() => onTaskClick(task)}
                    >
                        <CardContent className="p-3">
                            <div className="flex items-center gap-2 mb-1">
                                <span className="font-mono text-xs text-muted-foreground">{task.id}</span>
                                {task.parallel && <GitBranch size={12} className="text-accent-green" />}
                            </div>
                            <p className="text-sm font-medium text-foreground line-clamp-2">{task.title}</p>
                            <div className="flex items-center gap-2 mt-2">
                                <ModelTierBadge tier={task.model_tier} />
                                <RiskBadge risk={task.risk} />
                            </div>
                        </CardContent>
                    </Card>
                ))}
            </CardContent>
        </Card>
    );
};

// YAML View
const YamlView: React.FC<{ rawYaml?: string }> = ({ rawYaml }) => (
    <pre className="font-mono text-sm bg-muted text-foreground p-4 rounded-lg overflow-auto max-h-[60vh]">
        <code>{rawYaml || '# No YAML content'}</code>
    </pre>
);

// Spec List View using Card
const SpecListView: React.FC<{
    specs: SpecListItem[];
    onSelect: (name: string) => void;
    isLoading: boolean;
}> = ({ specs, onSelect, isLoading }) => {
    const specsWithTasks = specs.filter(s => s.has_tasks);

    if (isLoading) {
        return (
            <div className="flex items-center justify-center h-64">
                <Loader2 className="animate-spin text-muted-foreground" size={24} />
            </div>
        );
    }

    if (specsWithTasks.length === 0) {
        return (
            <div className="text-center py-12 text-muted-foreground">
                <FileText size={48} className="mx-auto mb-4 opacity-50" />
                <p>No specs with tasks found</p>
                <p className="text-sm mt-2">Run <code className="bg-muted px-2 py-0.5 rounded">ckrv spec tasks</code> to generate tasks</p>
            </div>
        );
    }

    return (
        <div className="space-y-2">
            {specsWithTasks.map((spec) => (
                <Card
                    key={spec.name}
                    className="cursor-pointer hover:bg-accent/50 transition-colors"
                    onClick={() => onSelect(spec.name)}
                >
                    <CardContent className="p-4">
                        <div className="flex items-center justify-between">
                            <div className="flex items-center gap-3">
                                <Layers size={20} className="text-primary" />
                                <div>
                                    <h3 className="font-medium text-foreground">{spec.name}</h3>
                                    <div className="flex items-center gap-2 mt-1">
                                        <Badge variant="success">has tasks</Badge>
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

// Main Task Editor Component
export const TaskEditor: React.FC = () => {
    const queryClient = useQueryClient();
    const [selectedSpecName, setSelectedSpecName] = useState<string | null>(null);
    const [tasks, setTasks] = useState<Task[]>([]);
    const [rawYaml, setRawYaml] = useState<string | undefined>();
    const [view, setView] = useState<'phase' | 'kanban' | 'code'>('phase');
    const [hasChanges, setHasChanges] = useState(false);
    const [expandedTasks, setExpandedTasks] = useState<Set<string>>(new Set());
    const [filters, setFilters] = useState({ phase: '__all__', status: '__all__', risk: '__all__', tier: '__all__', parallelOnly: false });
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
            if (filters.phase && filters.phase !== '__all__' && t.phase !== filters.phase) return false;
            if (filters.status && filters.status !== '__all__' && t.status !== filters.status) return false;
            if (filters.risk && filters.risk !== '__all__' && t.risk !== filters.risk) return false;
            if (filters.tier && filters.tier !== '__all__' && t.model_tier !== filters.tier) return false;
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
                    <h1 className="text-2xl font-bold text-foreground">Task Orchestration</h1>
                    <p className="text-muted-foreground mt-1">Select a spec to view and manage tasks</p>
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
                <Loader2 className="animate-spin text-muted-foreground" size={32} />
            </div>
        );
    }

    return (
        <div className="h-full flex flex-col overflow-hidden">
            {/* Header */}
            <Card className="shrink-0 rounded-none border-x-0 border-t-0">
                <CardContent className="px-4 py-3 flex items-center justify-between">
                    <div className="flex items-center gap-4">
                        <Button
                            variant="ghost"
                            size="icon"
                            onClick={() => setSelectedSpecName(null)}
                        >
                            <ArrowLeft size={20} />
                        </Button>
                        <div>
                            <span className="text-sm text-muted-foreground font-mono">tasks.yaml</span>
                            <h1 className="text-lg font-semibold text-foreground">{selectedSpecName}</h1>
                        </div>
                    </div>
                    <div className="flex items-center gap-3">
                        <ViewToggle view={view} setView={(v) => setView(v as typeof view)} />
                        {hasChanges && (
                            <Button
                                variant="outline"
                                onClick={() => {
                                    if (tasksDetailData?.tasks) {
                                        setTasks(tasksDetailData.tasks);
                                        setRawYaml(tasksDetailData.raw_yaml);
                                        setHasChanges(false);
                                    }
                                }}
                            >
                                <RotateCcw size={16} className="mr-2" />
                                Discard
                            </Button>
                        )}
                        <Button
                            onClick={() => saveMutation.mutate()}
                            disabled={!hasChanges || saveMutation.isPending}
                        >
                            {saveMutation.isPending ? (
                                <Loader2 size={16} className="mr-2 animate-spin" />
                            ) : (
                                <Save size={16} className="mr-2" />
                            )}
                            Save
                        </Button>
                    </div>
                </CardContent>
            </Card>

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
                    <Card>
                        <CardHeader className="py-2 px-4 flex flex-row items-center justify-between border-b border-border">
                            <CardTitle className="text-sm">tasks.yaml</CardTitle>
                            <Button variant="ghost" size="sm">Copy</Button>
                        </CardHeader>
                        <CardContent className="p-0">
                            <YamlView rawYaml={rawYaml} />
                        </CardContent>
                    </Card>
                )}
            </div>

            {/* Status Bar */}
            <div className="shrink-0 px-4 py-2 border-t border-border flex items-center justify-between text-sm text-muted-foreground bg-muted/50">
                <div className="flex items-center gap-4">
                    <span>{tasks.length} tasks</span>
                    <span>{tasks.filter(t => t.status === 'completed').length} completed</span>
                    <span>{tasks.reduce((s, t) => s + t.estimated_tokens, 0).toLocaleString()} tokens</span>
                </div>
                <div className="flex items-center gap-2">
                    <span className={`w-2 h-2 rounded-full ${hasChanges ? 'bg-accent-amber' : 'bg-accent-green'}`}></span>
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
