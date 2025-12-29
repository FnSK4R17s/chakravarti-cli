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
    metadata?: Record<string, any>;
    step_name?: string;
    status?: string;
}
