/**
 * Log entry models for persistent execution logs.
 *
 * These types mirror the Rust models in crates/ckrv-ui/src/models/log.rs
 */

/**
 * Log level/type enumeration
 */
export type LogLevel =
    | 'info'
    | 'warning'
    | 'error'
    | 'log'
    | 'start'
    | 'batch_start'
    | 'batch_complete'
    | 'batch_error'
    | 'success'
    | 'status';

/**
 * A single log entry persisted to disk (T010)
 */
export interface LogEntry {
    /** Unique identifier for this log entry */
    id: string;

    /** ID of the execution run this log belongs to */
    execution_id: string;

    /** When this log was generated (ISO 8601 format) */
    timestamp: string;

    /** Log level/type */
    level: LogLevel;

    /** The log message content */
    message: string;

    /** Optional source identifier (batch name, component, etc.) */
    source?: string;
}

/**
 * Request to fetch historical logs
 */
export interface LogHistoryRequest {
    /** Start from this line offset (0-indexed) */
    offset?: number;

    /** Maximum number of lines to return */
    limit?: number;

    /** If provided, only return logs after this timestamp (ISO 8601) */
    since?: string;
}

/**
 * Response containing historical logs (T011)
 */
export interface LogHistoryResponse {
    /** The execution ID */
    execution_id: string;

    /** The log entries */
    logs: LogEntry[];

    /** Total number of logs in the file */
    total_count: number;

    /** Offset used for this request */
    offset: number;

    /** Whether there are more logs after this batch */
    has_more: boolean;
}

/**
 * Response for tail logs endpoint
 */
export interface LogTailResponse {
    /** The execution ID */
    execution_id: string;

    /** The log entries (most recent N) */
    logs: LogEntry[];

    /** Total number of logs in the file */
    total_count: number;
}

/**
 * Response for delete logs endpoint
 */
export interface LogDeleteResponse {
    /** Whether the operation succeeded */
    success: boolean;

    /** The execution ID */
    execution_id: string;

    /** Number of lines deleted */
    deleted_lines: number;
}

/**
 * WebSocket message for log history backfill
 */
export interface LogHistoryBackfillMessage {
    type: 'history_backfill';
    logs: LogEntry[];
    total_count: number;
}

/**
 * WebSocket message indicating history backfill is complete
 */
export interface LogHistoryCompleteMessage {
    type: 'history_complete';
    total_logs_sent: number;
}
