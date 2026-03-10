/**
 * @module TaskEditor
 * @description
 * Task management interface for viewing and editing implementation tasks generated
 * from specs. Displays tasks grouped by phase with filtering, status tracking, and
 * the ability to execute individual tasks via AI agents.
 *
 * @context
 * Rendered as the main content of the Tasks page in the dashboard. Users review
 * generated tasks, update statuses, and trigger plan generation here. Auto-selects
 * spec based on current git branch.
 *
 * @dependencies
 * - useAutoSelectedSpec: Auto-selects spec based on current git branch
 * - useQuery/useMutation: React Query for task operations
 * - TaskDetailModal: Modal for viewing and executing individual tasks
 * - shadcn/ui components: Card, Badge, Button, Collapsible for consistent UI
 *
 * @example
 * // Rendered directly as a page component
 * <TaskEditor />
 *
 * // Displays tasks grouped by phase with filtering and status tracking
 * // Supports executing tasks via integrated terminal
 */

// === IMPORTS ===
import React, { useState, useEffect, useMemo } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useAutoSelectedSpec } from '../hooks/useAutoSelectedSpec';
import {
    ChevronDown, ChevronRight, Play,
    CheckCircle2, Circle, AlertTriangle, GitBranch,
    Layers, Zap, Brain, Cpu,
    Link2, FileText, Loader2, ClipboardList
} from 'lucide-react';
import { TaskDetailModal } from './TaskDetailModal';
import { Card, CardContent } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible';
import { toast } from 'sonner';

// === TYPES ===

/**
 * A task from the spec's tasks.yaml file.
 * Represents a single unit of work to be implemented.
 * 
 * @example
 * const task: Task = {
 *   id: 'T001',
 *   phase: 'Phase 1 - Setup',
 *   title: 'Add configuration module',
 *   description: 'Create config loading and validation',
 *   file: 'src/config.ts',
 *   user_story: null,
 *   parallel: true,
 *   complexity: 3,
 *   model_tier: 'standard',
 *   estimated_tokens: 5000,
 *   risk: 'low',
 *   context_required: ['utils.ts'],
 *   status: 'pending'
 * };
 */
interface Task {
    /** Unique task identifier (e.g., T001, T002) */
    id: string;
    /** Phase grouping for the task */
    phase: string;
    /** Brief task title */
    title: string;
    /** Detailed task description */
    description: string;
    /** Target file path for this task */
    file: string;
    /** Associated user story, if any */
    user_story: string | null;
    /** Whether task can run in parallel with others */
    parallel: boolean;
    /** Complexity score (1-5) */
    complexity: number;
    /** Model tier: 'light', 'standard', or 'heavy' */
    model_tier: string;
    /** Estimated token usage */
    estimated_tokens: number;
    /** Risk level: 'low', 'medium', 'high', or 'critical' */
    risk: string;
    /** File paths needed for context */
    context_required: string[];
    /** Current status: 'pending', 'in_progress', 'done', 'blocked' */
    status: string;
}

/**
 * Metadata for a spec returned by the specs list API.
 * Used to populate the spec selection view when no spec is auto-selected.
 */
interface SpecListItem {
    /** Spec name (folder name in specs/ directory) */
    name: string;
    /** Full path to the spec folder */
    path: string;
    /** Whether tasks.yaml exists for this spec */
    has_tasks: boolean;
    /** Whether plan.yaml exists for this spec */
    has_plan: boolean;
    /** Whether implementation has been started */
    has_implementation: boolean;
}

// === API FUNCTIONS ===

const fetchSpecs = async (): Promise<{ specs: SpecListItem[], count: number }> => {
    const res = await fetch('/api/specs');
    return res.json();
};

const fetchTasksDetail = async (spec: string): Promise<{ success: boolean; tasks: Task[]; raw_yaml?: string; count: number; error?: string }> => {
    const res = await fetch(`/api/specs/${encodeURIComponent(spec)}/tasks/artifacts`);
    const data = await res.json();
    return {
        success: data.has_tasks ?? false,
        tasks: data.tasks ?? [],
        raw_yaml: data.raw ?? undefined,
        count: data.task_count ?? 0,
    };
};

const updateTaskStatus = async (spec: string, taskId: string, status: string): Promise<{ success: boolean; message?: string }> => {
    const res = await fetch('/api/tasks/status', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ spec, task_id: taskId, status }),
    });
    return res.json();
};

const generatePlan = async (spec?: string): Promise<{ success: boolean; message?: string }> => {
    const res = await fetch('/api/command/plan', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ spec: spec ?? null }),
    });
    const data = await res.json();
    if (!res.ok) {
        throw new Error(data.error || data.message || 'Plan generation failed');
    }
    return data;
};

// === SUB-COMPONENTS ===

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
/**
 * Props for TaskCard component.
 * Displays a single task with status, complexity, and expandable details.
 */
interface TaskCardProps {
    /** Task data to display */
    task: Task;
    /** Callback to update the task status */
    onStatusChange: (status: string) => void;
    /** Whether the task details are expanded */
    expanded: boolean;
    /** Callback to toggle the expanded state */
    onToggleExpand: () => void;
}

const TaskCard: React.FC<TaskCardProps> = ({ task, onStatusChange, expanded, onToggleExpand }) => {
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
                    <code className="text-xs text-info bg-info/20 px-2 py-0.5 rounded mt-2 inline-block">
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
/**
 * Props for PhaseGroup component.
 * Collapsible section containing all tasks for a single phase.
 */
interface PhaseGroupProps {
    /** Phase name to display as the section header */
    phase: string;
    /** All tasks in this phase */
    tasks: Task[];
    /** Callback to update a task's status */
    onStatusChange: (taskId: string, status: string) => void;
    /** Set of task IDs with expanded details */
    expandedTasks: Set<string>;
    /** Callback to toggle a task's expanded state */
    toggleExpand: (id: string) => void;
    /** Force collapse/expand from parent */
    forceCollapsed?: boolean;
}

const PhaseGroup: React.FC<PhaseGroupProps> = ({ phase, tasks, onStatusChange, expandedTasks, toggleExpand, forceCollapsed }) => {
    /** Whether this phase group is collapsed */
    const [localCollapsed, setLocalCollapsed] = useState(false);
    const collapsed = forceCollapsed ?? localCollapsed;
    const setCollapsed = (v: boolean) => setLocalCollapsed(v);
    const completedCount = tasks.filter(t => t.status === 'completed').length;

    const phaseColors: Record<string, string> = {
        'Setup': 'border-l-success bg-success/20',
        'Foundation': 'border-l-info bg-info/20',
        'User Story 1': 'border-l-primary bg-primary/20',
        'User Story 2': 'border-l-primary bg-primary/20',
        'User Story 3': 'border-l-primary bg-primary/20',
        'User Story 4': 'border-l-primary bg-primary/20',
        'User Story 5': 'border-l-primary bg-primary/20',
        'User Story 6': 'border-l-primary bg-primary/20',
        'Polish': 'border-l-warning bg-warning/20',
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
                        <div className="w-24 h-1.5 bg-muted rounded-full overflow-hidden">
                            <div
                                className="h-full bg-success transition-all"
                                style={{ width: `${(completedCount / tasks.length) * 100}%` }}
                            />
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

// Spec List View using Card
/**
 * Props for SpecListView component.
 * Displays a list of specs with tasks for selection.
 */
interface SpecListViewProps {
    /** List of specs to display */
    specs: SpecListItem[];
    /** Callback when a spec is selected */
    onSelect: (name: string) => void;
    /** Whether the spec list is currently loading */
    isLoading: boolean;
}

const SpecListView: React.FC<SpecListViewProps> = ({ specs, onSelect, isLoading }) => {
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

// === MAIN COMPONENT ===

export const TaskEditor: React.FC = () => {
    const queryClient = useQueryClient();

    // === STATE ===

    // --- Spec Selection ---
    // Auto-select spec based on current branch
    const { selectedSpec: autoSelectedSpec, isLoading: isLoadingAutoSpec } = useAutoSelectedSpec();

    /** Manual spec override when user explicitly selects a different spec */
    const [manualSpecOverride, setManualSpecOverride] = useState<string | null>(null);
    const selectedSpecName = manualSpecOverride ?? autoSelectedSpec;

    // --- Task Data ---
    /** All tasks for the selected spec */
    const [tasks, setTasks] = useState<Task[]>([]);
    /** Set of task IDs with expanded details visible */
    const [expandedTasks, setExpandedTasks] = useState<Set<string>>(new Set());
    /** Currently selected task for the detail modal */
    const [selectedTask, setSelectedTask] = useState<Task | null>(null);
    /** Whether all phase groups are collapsed */
    const [allCollapsed, setAllCollapsed] = useState(false);

    // === QUERIES ===

    // Fetch specs list (for manual selection fallback)
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

    // === EFFECTS ===

    /**
     * Syncs local task state with fetched data from the server.
     * Runs whenever the tasks detail query completes successfully.
     * Resets the hasChanges flag since we're loading fresh data.
     */
    useEffect(() => {
        if (tasksDetailData?.success && tasksDetailData.tasks) {
            // eslint-disable-next-line react-hooks/set-state-in-effect
            setTasks(tasksDetailData.tasks);
        }
    }, [tasksDetailData]);

    // === MUTATIONS ===

    // Status update mutation (for quick updates)
    const statusMutation = useMutation({
        mutationFn: ({ taskId, status }: { taskId: string; status: string }) =>
            updateTaskStatus(selectedSpecName!, taskId, status),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['tasks', selectedSpecName] });
        },
    });

    // Plan generation mutation
    const planMutation = useMutation({
        mutationFn: () => generatePlan(selectedSpecName ?? undefined),
        onSuccess: (data) => {
            if (data.success) {
                toast.success('Plan Generated', {
                    description: 'Execution plan has been created successfully',
                });
                queryClient.invalidateQueries({ queryKey: ['specs'] });
            } else {
                toast.error('Plan Generation Failed', {
                    description: data.message || 'Unknown error',
                });
            }
        },
        onError: (error) => {
            toast.error('Plan Generation Failed', {
                description: error instanceof Error ? error.message : 'Unknown error',
            });
        },
    });

    // === HANDLERS ===

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

    const phases = useMemo(() => [...new Set(tasks.map(t => t.phase).filter(p => p && p.trim() !== ''))], [tasks]);

    const tasksByPhase = useMemo(() => {
        const grouped: Record<string, Task[]> = {};
        tasks.forEach(t => {
            if (!grouped[t.phase]) grouped[t.phase] = [];
            grouped[t.phase].push(t);
        });
        return grouped;
    }, [tasks]);

    // === RENDER HELPERS ===

    // Show spec list if nothing selected (neither auto nor manual)
    if (!selectedSpecName) {
        // If still loading auto-selection, show spinner
        if (isLoadingAutoSpec) {
            return (
                <div className="flex items-center justify-center h-full">
                    <Loader2 className="animate-spin text-muted-foreground" size={32} />
                </div>
            );
        }

        return (
            <div className="h-full overflow-auto p-4">
                <div className="mb-6">
                    <h1 className="text-2xl font-bold text-foreground">Task Orchestration</h1>
                    <p className="text-muted-foreground mt-1">No spec matches the current branch. Select a spec to view and manage tasks.</p>
                </div>
                <SpecListView
                    specs={specsData?.specs || []}
                    onSelect={setManualSpecOverride}
                    isLoading={isLoadingSpecs}
                />
            </div>
        );
    }

    if (isLoadingTasks) {
        return (
            <div className="flex items-center justify-center h-full">
                <Loader2 className="animate-spin text-muted-foreground" size={32} />
            </div>
        );
    }

    if (tasks.length === 0) {
        return (
            <div className="h-full overflow-auto p-4">
                <div className="text-center py-12 text-muted-foreground">
                    <FileText size={48} className="mx-auto mb-4 opacity-50" />
                    <h2 className="text-xl font-semibold text-foreground mb-2">No Tasks Found</h2>
                    <p>No tasks found for spec <code className="bg-muted px-2 py-0.5 rounded">{selectedSpecName}</code></p>
                    <p className="text-sm mt-2">Run <code className="bg-muted px-2 py-0.5 rounded">ckrv spec tasks</code> to generate tasks</p>
                </div>
            </div>
        );
    }

    return (
        <div className="h-full flex flex-col overflow-hidden">
            {/* Header */}
            <Card className="shrink-0 rounded-none border-x-0 border-t-0">
                <CardContent className="px-4 py-3 flex items-center justify-between">
                    <div className="flex items-center gap-4">
                        <div>
                            <span className="text-sm text-muted-foreground font-mono">tasks.yaml</span>
                            <h1 className="text-lg font-semibold text-foreground">{selectedSpecName}</h1>
                        </div>
                    </div>
                    <div className="flex items-center gap-2">
                        <Button
                            variant="outline"
                            size="sm"
                            onClick={() => setAllCollapsed(prev => !prev)}
                            className="gap-1.5"
                        >
                            {allCollapsed ? <ChevronRight size={14} /> : <ChevronDown size={14} />}
                            {allCollapsed ? 'Expand All' : 'Collapse All'}
                        </Button>

                        <div className="w-px h-5 bg-border" />

                        {specsData?.specs.find(s => s.name === selectedSpecName)?.has_plan ? (
                            <Button variant="ghost" size="sm" disabled className="text-success gap-1.5">
                                <CheckCircle2 size={14} />
                                Plan Generated
                            </Button>
                        ) : (
                            <Button
                                onClick={() => planMutation.mutate()}
                                disabled={planMutation.isPending}
                                className="gap-2"
                            >
                                {planMutation.isPending ? (
                                    <Loader2 size={16} className="animate-spin" />
                                ) : (
                                    <ClipboardList size={16} />
                                )}
                                Generate Plan
                            </Button>
                        )}
                    </div>
                </CardContent>
            </Card>

            {/* Content */}
            <div className="flex-1 overflow-auto p-4">
                {phases.filter(p => tasksByPhase[p]?.length > 0).map(phase => (
                    <PhaseGroup
                        key={phase}
                        phase={phase}
                        tasks={tasksByPhase[phase]}
                        onStatusChange={handleStatusChange}
                        expandedTasks={expandedTasks}
                        toggleExpand={toggleExpand}
                        forceCollapsed={allCollapsed ? true : undefined}
                    />
                ))}
            </div>

            {/* Status Bar */}
            <div className="shrink-0 px-4 py-2 border-t border-border flex items-center justify-between text-sm text-muted-foreground bg-muted/50">
                <div className="flex items-center gap-4">
                    <span>{tasks.length} tasks</span>
                    <span>{tasks.filter(t => t.status === 'completed').length} completed</span>
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
