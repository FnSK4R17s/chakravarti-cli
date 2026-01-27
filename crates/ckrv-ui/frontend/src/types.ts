// ===== Code Page Types =====

/** Tab identifiers for the unified Code page */
export type CodeTabType = 'spec' | 'tasks' | 'plan' | 'run';

/** Tab metadata for rendering the Code page tabs */
export interface CodeTab {
    id: CodeTabType;
    label: string;
    icon: 'FileText' | 'ListTodo' | 'Workflow' | 'Rocket';
}

/** Constant array of all Code page tabs with metadata */
export const CODE_TABS: CodeTab[] = [
    { id: 'spec', label: 'Spec', icon: 'FileText' },
    { id: 'tasks', label: 'Tasks', icon: 'ListTodo' },
    { id: 'plan', label: 'Plan', icon: 'Workflow' },
    { id: 'run', label: 'Run', icon: 'Rocket' },
];

/** Workflow stage status for progress indicators */
export type WorkflowStageStatus = 'pending' | 'complete';

/** Workflow stage interface for tracking progress */
export interface WorkflowStage {
    id: CodeTabType;
    status: WorkflowStageStatus;
}

// ===== System Types =====

export interface SystemStatus {
    active_branch: string;
    feature_number: string | null;
    is_ready: boolean;
    mode: 'idle' | 'planning' | 'running' | 'promoting';
}

export interface OrchestrationEvent {
    type: 'log' | 'step_start' | 'step_end' | 'error' | 'success';
    message: string;
    timestamp: string;
    metadata?: Record<string, unknown>;
    step_name?: string;
    status?: string;
}
