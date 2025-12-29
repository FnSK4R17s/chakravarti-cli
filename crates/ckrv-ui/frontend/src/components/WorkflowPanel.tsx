import React from 'react';
import { useQuery } from '@tanstack/react-query';
import { FileText, ListChecks, Play, ArrowRight, CheckCircle2, Circle, Loader2, GitBranch, CheckCheck, GitPullRequest, ShieldCheck, GitCompare } from 'lucide-react';

interface Spec {
    name: string;
    path: string;
    has_tasks: boolean;
    has_implementation: boolean;
    implementation_branch: string | null;
}

interface Task {
    id: string;
    phase: string;
    title: string;
    status: 'pending' | 'in_progress' | 'completed';
}

interface SpecsResponse {
    specs: Spec[];
    count: number;
}

interface TasksResponse {
    tasks: Task[];
    spec_id: string;
}

const fetchSpecs = async (): Promise<SpecsResponse> => {
    const res = await fetch('/api/specs');
    if (!res.ok) return { specs: [], count: 0 };
    return res.json();
};

const fetchTasks = async (): Promise<TasksResponse> => {
    const res = await fetch('/api/tasks');
    if (!res.ok) return { tasks: [], spec_id: '' };
    return res.json();
};

export const WorkflowPanel: React.FC = () => {
    const { data: specsData, isLoading: specsLoading } = useQuery({
        queryKey: ['specs'],
        queryFn: fetchSpecs,
        refetchInterval: 10000,
    });

    const { data: tasksData, isLoading: tasksLoading } = useQuery({
        queryKey: ['tasks'],
        queryFn: fetchTasks,
        refetchInterval: 5000,
    });

    const specs = specsData?.specs || [];
    const tasks = tasksData?.tasks || [];
    const completedTasks = tasks.filter(t => t.status === 'completed').length;
    const inProgressTasks = tasks.filter(t => t.status === 'in_progress').length;
    
    // Check for completed implementation
    const implementedSpec = specs.find(s => s.has_implementation);
    const hasImplementation = !!implementedSpec;

    return (
        <div 
            className="rounded-lg"
            style={{ 
                background: 'var(--bg-secondary)',
                border: '1px solid var(--border-subtle)'
            }}
        >
            {/* Header */}
            <div 
                className="px-4 py-3 flex items-center justify-between"
                style={{ borderBottom: '1px solid var(--border-subtle)' }}
            >
                <h3 
                    className="font-semibold text-sm"
                    style={{ color: 'var(--text-primary)' }}
                >
                    Workflow Pipeline
                </h3>
                <span 
                    className="text-xs font-mono"
                    style={{ color: 'var(--text-muted)' }}
                >
                    spec → tasks → run → review
                </span>
            </div>

            {/* Pipeline Visualization */}
            <div className="p-4 overflow-x-auto">
                <div className="flex items-stretch gap-3 min-w-fit">
                    {/* Specs Stage */}
                    <PipelineStage
                        icon={<FileText size={18} />}
                        title="Specifications"
                        count={specs.length}
                        status={specs.length > 0 ? 'complete' : 'empty'}
                        color="cyan"
                        loading={specsLoading}
                    >
                        {specs.length === 0 ? (
                            <EmptyState text="No specs yet" hint="ckrv spec new" />
                        ) : (
                            <div className="space-y-1.5">
                                {specs.map((spec) => (
                                    <SpecItem key={spec.name} spec={spec} />
                                ))}
                            </div>
                        )}
                    </PipelineStage>

                    {/* Arrow */}
                    <div className="flex items-center">
                        <ArrowRight size={16} style={{ color: 'var(--text-muted)' }} />
                    </div>

                    {/* Tasks Stage */}
                    <PipelineStage
                        icon={<ListChecks size={18} />}
                        title="Tasks"
                        count={tasks.length}
                        status={inProgressTasks > 0 ? 'running' : completedTasks > 0 ? 'complete' : 'empty'}
                        color="green"
                        loading={tasksLoading}
                        subtitle={tasks.length > 0 ? `${completedTasks}/${tasks.length} done` : undefined}
                    >
                        {tasks.length === 0 ? (
                            <EmptyState text="No tasks" hint="ckrv spec tasks" />
                        ) : (
                            <div className="space-y-1.5">
                                {tasks.map((task) => (
                                    <TaskItem key={task.id} task={task} />
                                ))}
                            </div>
                        )}
                    </PipelineStage>

                    {/* Arrow */}
                    <div className="flex items-center">
                        <ArrowRight size={16} style={{ color: 'var(--text-muted)' }} />
                    </div>

                    {/* Jobs Stage */}
                    <PipelineStage
                        icon={hasImplementation ? <CheckCircle2 size={18} /> : <Play size={18} />}
                        title="Execution"
                        count={hasImplementation ? completedTasks : inProgressTasks}
                        status={hasImplementation ? 'complete' : inProgressTasks > 0 ? 'running' : 'idle'}
                        color={hasImplementation ? 'green' : 'purple'}
                        subtitle={hasImplementation ? 'Complete' : undefined}
                    >
                        {hasImplementation ? (
                            <ImplementationDetails 
                                branch={implementedSpec?.implementation_branch ?? ''} 
                                tasksCompleted={completedTasks}
                            />
                        ) : inProgressTasks === 0 ? (
                            <EmptyState text="Ready to run" hint="ckrv run" />
                        ) : (
                            <div 
                                className="flex items-center gap-2 text-sm"
                                style={{ color: 'var(--accent-purple)' }}
                            >
                                <Loader2 size={14} className="animate-spin" />
                                <span>{inProgressTasks} running</span>
                            </div>
                        )}
                    </PipelineStage>

                    {/* Arrow */}
                    <div className="flex items-center">
                        <ArrowRight size={16} style={{ color: 'var(--text-muted)' }} />
                    </div>

                    {/* Review Stage */}
                    <PipelineStage
                        icon={<GitPullRequest size={18} />}
                        title="Review"
                        count={0}
                        status={hasImplementation ? 'idle' : 'empty'}
                        color="amber"
                        subtitle={hasImplementation ? 'Ready' : undefined}
                    >
                        {hasImplementation ? (
                            <ReviewSteps />
                        ) : (
                            <EmptyState text="Awaiting code" hint="ckrv diff" />
                        )}
                    </PipelineStage>
                </div>
            </div>
        </div>
    );
};

interface PipelineStageProps {
    icon: React.ReactNode;
    title: string;
    count: number;
    status: 'empty' | 'complete' | 'running' | 'idle';
    color: 'cyan' | 'green' | 'purple' | 'amber';
    loading?: boolean;
    subtitle?: string;
    children: React.ReactNode;
}

const PipelineStage: React.FC<PipelineStageProps> = ({ 
    icon, title, count, status, color, loading, subtitle, children 
}) => {
    const colorMap = {
        cyan: 'var(--accent-cyan)',
        green: 'var(--accent-green)',
        purple: 'var(--accent-purple)',
        amber: 'var(--accent-amber)',
    };

    const dimColorMap = {
        cyan: 'var(--accent-cyan-dim)',
        green: 'var(--accent-green-dim)',
        purple: 'var(--accent-purple-dim)',
        amber: 'var(--accent-amber-dim)',
    };

    const accentColor = colorMap[color];
    const dimColor = dimColorMap[color];

    return (
        <div 
            className="flex-1 rounded-lg p-3"
            style={{ 
                background: 'var(--bg-tertiary)',
                border: status === 'running' ? `1px solid ${accentColor}` : '1px solid var(--border-subtle)',
                boxShadow: status === 'running' ? `0 0 20px ${dimColor}` : 'none',
                minWidth: '180px',
            }}
        >
            {/* Stage Header */}
            <div className="flex items-center justify-between mb-3">
                <div className="flex items-center gap-2">
                    <div 
                        className="p-1.5 rounded"
                        style={{ 
                            background: dimColor,
                            color: accentColor
                        }}
                    >
                        {icon}
                    </div>
                    <div>
                        <div 
                            className="text-sm font-medium"
                            style={{ color: 'var(--text-primary)' }}
                        >
                            {title}
                        </div>
                        {subtitle && (
                            <div 
                                className="text-xs"
                                style={{ color: 'var(--text-muted)' }}
                            >
                                {subtitle}
                            </div>
                        )}
                    </div>
                </div>
                <div 
                    className="text-lg font-mono font-bold"
                    style={{ color: count > 0 ? accentColor : 'var(--text-muted)' }}
                >
                    {loading ? <Loader2 size={18} className="animate-spin" /> : count}
                </div>
            </div>

            {/* Stage Content */}
            <div className="min-h-[80px] max-h-[140px] overflow-y-auto custom-scrollbar">
                {children}
            </div>
        </div>
    );
};

const SpecItem: React.FC<{ spec: Spec }> = ({ spec }) => {
    // Determine status: implemented > has_tasks > pending
    const getStatusColor = () => {
        if (spec.has_implementation) return 'var(--accent-cyan)';
        if (spec.has_tasks) return 'var(--accent-green)';
        return 'var(--accent-amber)';
    };

    return (
        <div 
            className="flex items-center gap-2 text-xs py-1.5 px-2 rounded"
            style={{ background: 'var(--bg-surface)' }}
        >
            <div 
                className="w-1.5 h-1.5 rounded-full"
                style={{ background: getStatusColor() }}
            />
            <span 
                className="font-mono truncate flex-1"
                style={{ color: 'var(--text-secondary)' }}
            >
                {spec.name}
            </span>
            {spec.has_implementation ? (
                <span 
                    className="text-[10px] px-1 rounded"
                    style={{ 
                        background: 'var(--accent-cyan-dim)', 
                        color: 'var(--accent-cyan)' 
                    }}
                >
                    merged
                </span>
            ) : spec.has_tasks ? (
                <CheckCircle2 size={12} style={{ color: 'var(--accent-green)' }} />
            ) : null}
        </div>
    );
};

const TaskItem: React.FC<{ task: Task }> = ({ task }) => {
    const getStatusIcon = () => {
        switch (task.status) {
            case 'completed':
                return <CheckCircle2 size={12} style={{ color: 'var(--accent-green)' }} />;
            case 'in_progress':
                return <Loader2 size={12} className="animate-spin" style={{ color: 'var(--accent-cyan)' }} />;
            default:
                return <Circle size={12} style={{ color: 'var(--text-muted)' }} />;
        }
    };

    return (
        <div 
            className="flex items-center gap-2 text-xs py-1.5 px-2 rounded"
            style={{ background: 'var(--bg-surface)' }}
        >
            {getStatusIcon()}
            <span 
                className="font-mono"
                style={{ color: 'var(--text-muted)' }}
            >
                {task.id}
            </span>
            <span 
                className="truncate flex-1"
                style={{ 
                    color: task.status === 'completed' ? 'var(--text-muted)' : 'var(--text-secondary)',
                    textDecoration: task.status === 'completed' ? 'line-through' : 'none'
                }}
            >
                {task.title}
            </span>
        </div>
    );
};

const EmptyState: React.FC<{ text: string; hint: string }> = ({ text, hint }) => (
    <div className="text-center py-2">
        <div 
            className="text-xs mb-1"
            style={{ color: 'var(--text-muted)' }}
        >
            {text}
        </div>
        <code 
            className="text-xs font-mono px-1.5 py-0.5 rounded"
            style={{ 
                background: 'var(--bg-surface)',
                color: 'var(--text-muted)'
            }}
        >
            {hint}
        </code>
    </div>
);

interface ImplementationDetailsProps {
    branch: string;
    tasksCompleted: number;
}

const ImplementationDetails: React.FC<ImplementationDetailsProps> = ({ branch, tasksCompleted }) => (
    <div className="space-y-2">
        {/* Success indicator */}
        <div 
            className="flex items-center gap-2 text-xs py-1.5 px-2 rounded"
            style={{ background: 'var(--accent-green-dim)' }}
        >
            <CheckCheck size={14} style={{ color: 'var(--accent-green)' }} />
            <span style={{ color: 'var(--accent-green)' }}>
                All code merged
            </span>
        </div>
        
        {/* Branch info */}
        <div 
            className="flex items-center gap-2 text-xs py-1.5 px-2 rounded"
            style={{ background: 'var(--bg-surface)' }}
        >
            <GitBranch size={12} style={{ color: 'var(--accent-cyan)' }} />
            <span 
                className="font-mono truncate"
                style={{ color: 'var(--text-secondary)' }}
                title={branch}
            >
                {branch}
            </span>
        </div>
        
        {/* Tasks count */}
        <div 
            className="flex items-center gap-2 text-xs py-1.5 px-2 rounded"
            style={{ background: 'var(--bg-surface)' }}
        >
            <CheckCircle2 size={12} style={{ color: 'var(--accent-green)' }} />
            <span style={{ color: 'var(--text-secondary)' }}>
                {tasksCompleted} tasks completed
            </span>
        </div>
        
        {/* Ready for review hint */}
        <div 
            className="text-[10px] text-center pt-1"
            style={{ color: 'var(--text-muted)' }}
        >
            Ready for code review
        </div>
    </div>
);

const ReviewSteps: React.FC = () => (
    <div className="space-y-2">
        {/* Diff */}
        <div 
            className="flex items-center gap-2 text-xs py-1.5 px-2 rounded"
            style={{ background: 'var(--bg-surface)' }}
        >
            <GitCompare size={12} style={{ color: 'var(--accent-cyan)' }} />
            <span style={{ color: 'var(--text-secondary)' }}>
                View Diff
            </span>
            <code 
                className="ml-auto text-[10px] font-mono"
                style={{ color: 'var(--text-muted)' }}
            >
                ckrv diff
            </code>
        </div>
        
        {/* Verify */}
        <div 
            className="flex items-center gap-2 text-xs py-1.5 px-2 rounded"
            style={{ background: 'var(--bg-surface)' }}
        >
            <ShieldCheck size={12} style={{ color: 'var(--accent-amber)' }} />
            <span style={{ color: 'var(--text-secondary)' }}>
                Verify
            </span>
            <code 
                className="ml-auto text-[10px] font-mono"
                style={{ color: 'var(--text-muted)' }}
            >
                ckrv verify
            </code>
        </div>
        
        {/* Promote */}
        <div 
            className="flex items-center gap-2 text-xs py-1.5 px-2 rounded"
            style={{ background: 'var(--bg-surface)' }}
        >
            <GitPullRequest size={12} style={{ color: 'var(--accent-green)' }} />
            <span style={{ color: 'var(--text-secondary)' }}>
                Create PR
            </span>
            <code 
                className="ml-auto text-[10px] font-mono"
                style={{ color: 'var(--text-muted)' }}
            >
                ckrv promote
            </code>
        </div>
        
        {/* Hint */}
        <div 
            className="text-[10px] text-center pt-1"
            style={{ color: 'var(--text-muted)' }}
        >
            Use commands panel →
        </div>
    </div>
);

