/**
 * Run History Types for Persistent Execution Tracking
 * 
 * These types mirror the Rust backend models for YAML persistence.
 */

// ============================================================================
// Status Types
// ============================================================================

/**
 * Overall run status
 */
export type RunStatus = 'pending' | 'running' | 'completed' | 'failed' | 'aborted';

/**
 * Individual batch status within a run
 */
export type HistoryBatchStatus = 'pending' | 'running' | 'completed' | 'failed';

// ============================================================================
// Entity Types
// ============================================================================

/**
 * Summary statistics for a run (for quick display)
 */
export interface RunSummary {
    total_batches: number;
    completed_batches: number;
    failed_batches: number;
    pending_batches: number;
    tasks_completed: number;
    branches_merged: number;
}

/**
 * Result of a single batch within a run
 */
export interface BatchResult {
    id: string;
    name: string;
    status: HistoryBatchStatus;
    started_at: string | null;     // ISO 8601
    ended_at: string | null;       // ISO 8601
    branch: string | null;
    merged: boolean;
    error: string | null;
}

/**
 * A single execution run for a specification
 */
export interface Run {
    id: string;
    spec_name: string;
    started_at: string;            // ISO 8601
    ended_at: string | null;       // ISO 8601
    status: RunStatus;
    dry_run: boolean;
    elapsed_seconds: number | null;
    batches: BatchResult[];
    summary: RunSummary;
    error: string | null;
}

/**
 * Collection of runs for a specification (root of runs.yaml)
 */
export interface RunHistory {
    version: string;
    spec_name: string;
    runs: Run[];
}

// ============================================================================
// API Response Types
// ============================================================================

/**
 * Response for GET /api/history/{spec}
 */
export interface HistoryListResponse {
    success: boolean;
    spec_name: string;
    total_count: number;
    runs: Run[];
    error?: string;
}

/**
 * Response for GET /api/history/{spec}/{run_id}
 */
export interface HistoryDetailResponse {
    success: boolean;
    run?: Run;
    error?: string;
}

/**
 * Response for POST /api/history/{spec}
 */
export interface CreateRunResponse {
    success: boolean;
    run_id?: string;
    started_at?: string;
    error?: string;
    existing_run_id?: string;    // If 409 conflict
    existing_started_at?: string;
}

/**
 * Response for PATCH /api/history/{spec}/{run_id}
 */
export interface UpdateRunResponse {
    success: boolean;
    updated_at?: string;
    error?: string;
}

/**
 * Response for DELETE /api/history/{spec}/{run_id}
 */
export interface DeleteRunResponse {
    success: boolean;
    deleted_run_id?: string;
    error?: string;
}

// ============================================================================
// Helper Functions
// ============================================================================

/**
 * Format elapsed time in human-readable format
 */
export function formatElapsedTime(seconds: number | null): string {
    if (seconds === null) return '--:--';

    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;

    if (hours > 0) {
        return `${hours}h ${minutes}m ${secs}s`;
    } else if (minutes > 0) {
        return `${minutes}m ${secs}s`;
    } else {
        return `${secs}s`;
    }
}

/**
 * Get status color for display
 */
export function getRunStatusColor(status: RunStatus): string {
    switch (status) {
        case 'completed':
            return 'text-green-500';
        case 'failed':
            return 'text-red-500';
        case 'running':
            return 'text-blue-500';
        case 'aborted':
            return 'text-orange-500';
        case 'pending':
        default:
            return 'text-gray-400';
    }
}

/**
 * Get status icon name for display
 */
export function getRunStatusIcon(status: RunStatus): string {
    switch (status) {
        case 'completed':
            return 'check-circle';
        case 'failed':
            return 'x-circle';
        case 'running':
            return 'loader';
        case 'aborted':
            return 'slash';
        case 'pending':
        default:
            return 'circle';
    }
}

/**
 * Format timestamp for display
 */
export function formatTimestamp(iso: string | null): string {
    if (!iso) return '--';
    const date = new Date(iso);
    return date.toLocaleString();
}

/**
 * Format timestamp relative (e.g., "2 hours ago")
 */
export function formatRelativeTime(iso: string): string {
    const date = new Date(iso);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMins / 60);
    const diffDays = Math.floor(diffHours / 24);

    if (diffMins < 1) return 'just now';
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 7) return `${diffDays}d ago`;
    return date.toLocaleDateString();
}
