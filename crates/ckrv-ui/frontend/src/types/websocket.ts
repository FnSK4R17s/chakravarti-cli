/**
 * WebSocket Message Types for Execution Runner
 * 
 * These types define the contract between the Rust backend and React frontend
 * for execution state synchronization.
 */

// ============================================================================
// Execution Status Types
// ============================================================================

/**
 * Overall execution status
 */
export type ExecutionStatus =
    | 'idle'         // No execution in progress
    | 'starting'     // API request sent, waiting for WS
    | 'running'      // Execution in progress
    | 'completed'    // All batches completed
    | 'failed'       // Execution failed
    | 'aborted'      // User aborted
    | 'reconnecting'; // WS disconnected, reconnecting

/**
 * Individual batch status
 */
export type BatchStatus =
    | 'pending'      // Waiting for dependencies
    | 'running'      // In progress
    | 'completed'    // Successfully completed
    | 'failed';      // Failed with error

// ============================================================================
// WebSocket Message Types
// ============================================================================

/**
 * Base message interface - all messages have these fields
 */
interface BaseMessage {
    type: string;
    message?: string;
    timestamp?: string;
}

/**
 * Status change message - sent when overall execution state changes
 * 
 * @example
 * { type: "status", status: "running" }
 * { type: "status", status: "completed", message: "All batches done" }
 */
export interface StatusMessage extends BaseMessage {
    type: 'status';
    status: ExecutionStatus;
}

/**
 * Batch status message - sent when a batch state changes
 * 
 * @example
 * { type: "batch_status", batch_id: "batch-001", batch_name: "Auth", status: "running" }
 * { type: "batch_status", batch_id: "batch-001", status: "completed", branch: "feat-auth" }
 */
export interface BatchStatusMessage extends BaseMessage {
    type: 'batch_status';
    batch_id: string;
    batch_name?: string;
    status: BatchStatus;
    branch?: string;
    error?: string;
}

/**
 * Log message - general output for terminal display
 * 
 * @example
 * { type: "info", message: "Processing..." }
 * { type: "error", message: "Failed to connect" }
 */
export interface LogMessage extends BaseMessage {
    type: 'info' | 'error' | 'success' | 'log' | 'start' | 'batch_start' | 'batch_complete' | 'batch_error';
    message: string;
    stream?: 'stdout' | 'stderr';
}

/**
 * Union type for all possible WebSocket messages
 */
export type WsMessage = StatusMessage | BatchStatusMessage | LogMessage;

// ============================================================================
// Type Guards
// ============================================================================

/**
 * Check if message is a status message
 */
export function isStatusMessage(msg: WsMessage): msg is StatusMessage {
    return msg.type === 'status';
}

/**
 * Check if message is a batch status message
 */
export function isBatchStatusMessage(msg: WsMessage): msg is BatchStatusMessage {
    return msg.type === 'batch_status';
}

/**
 * Check if message is a log message
 */
export function isLogMessage(msg: WsMessage): msg is LogMessage {
    return ['info', 'error', 'success', 'log', 'start', 'batch_start', 'batch_complete', 'batch_error'].includes(msg.type);
}

// ============================================================================
// Helper Functions
// ============================================================================

/**
 * Parse execution status from various message formats
 * Provides fallback handling for legacy message types
 */
export function parseExecutionStatus(msg: WsMessage): ExecutionStatus | null {
    // Explicit status message
    if (isStatusMessage(msg)) {
        return msg.status as ExecutionStatus;
    }

    // Fallback: "start" type indicates running
    if (msg.type === 'start') {
        return 'running';
    }

    // Fallback: "success" type indicates completed
    if (msg.type === 'success') {
        return 'completed';
    }

    // Fallback: "error" type at top level indicates failed
    if (msg.type === 'error' && !msg.message?.includes('batch')) {
        return 'failed';
    }

    return null;
}

/**
 * Parse batch name from log messages using multiple patterns
 */
export function parseBatchFromLog(message: string): { name: string | null; status: BatchStatus | null } {
    // Pattern 1: "Spawning batch: <name>"
    const spawnMatch = message.match(/Spawning batch:\s*(.+)/i);
    if (spawnMatch) {
        return { name: spawnMatch[1].trim(), status: 'running' };
    }

    // Pattern 2: "Batch <id> completed on branch <branch>"
    const batchCompleteMatch = message.match(/Batch\s+(\S+)\s+completed on branch\s+(\S+)/i);
    if (batchCompleteMatch) {
        return { name: batchCompleteMatch[1].trim(), status: 'completed' };
    }

    // Pattern 3: "Mission completed: <name>" (legacy)
    const missionMatch = message.match(/Mission completed:\s*(.+)/i);
    if (missionMatch) {
        return { name: missionMatch[1].trim(), status: 'completed' };
    }

    // Pattern 4: "Successfully merged batch '<name>'" (legacy)
    const mergeMatch = message.match(/Successfully merged batch\s*'?([^']+)'?/i);
    if (mergeMatch) {
        return { name: mergeMatch[1].trim(), status: 'completed' };
    }

    return { name: null, status: null };
}
