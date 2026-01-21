/**
 * T020: Log service for fetching historical execution logs
 *
 * Provides functions to interact with the log history REST API endpoints.
 */

import type {
    LogHistoryRequest,
    LogHistoryResponse,
    LogTailResponse,
    LogDeleteResponse,
} from '../types/log';

/**
 * Fetch paginated historical logs for an execution
 *
 * @param executionId - The execution run ID
 * @param options - Optional pagination parameters
 * @returns Promise<LogHistoryResponse>
 */
export async function fetchLogs(
    executionId: string,
    options: Partial<LogHistoryRequest> = {}
): Promise<LogHistoryResponse> {
    const params = new URLSearchParams();

    if (options.offset !== undefined) {
        params.set('offset', options.offset.toString());
    }
    if (options.limit !== undefined) {
        params.set('limit', options.limit.toString());
    }
    if (options.since) {
        params.set('since', options.since);
    }

    const queryString = params.toString();
    const url = `/api/execution/${encodeURIComponent(executionId)}/logs${queryString ? `?${queryString}` : ''}`;

    const response = await fetch(url);
    if (!response.ok) {
        throw new Error(`Failed to fetch logs: ${response.status} ${response.statusText}`);
    }

    return response.json();
}

/**
 * Fetch the last N log entries for an execution
 *
 * @param executionId - The execution run ID
 * @param count - Number of recent logs to fetch (default: 10)
 * @returns Promise<LogTailResponse>
 */
export async function fetchTailLogs(
    executionId: string,
    count: number = 10
): Promise<LogTailResponse> {
    const url = `/api/execution/${encodeURIComponent(executionId)}/logs/tail?count=${count}`;

    const response = await fetch(url);
    if (!response.ok) {
        throw new Error(`Failed to fetch tail logs: ${response.status} ${response.statusText}`);
    }

    return response.json();
}

/**
 * Delete all logs for an execution
 *
 * @param executionId - The execution run ID
 * @returns Promise<LogDeleteResponse>
 */
export async function deleteLogs(executionId: string): Promise<LogDeleteResponse> {
    const url = `/api/execution/${encodeURIComponent(executionId)}/logs`;

    const response = await fetch(url, { method: 'DELETE' });
    if (!response.ok) {
        throw new Error(`Failed to delete logs: ${response.status} ${response.statusText}`);
    }

    return response.json();
}

/**
 * Fetch logs since a specific timestamp (for reconnection)
 *
 * @param executionId - The execution run ID
 * @param since - ISO 8601 timestamp to fetch logs after
 * @returns Promise<LogHistoryResponse>
 */
export async function fetchLogsSince(
    executionId: string,
    since: string
): Promise<LogHistoryResponse> {
    return fetchLogs(executionId, { since });
}
