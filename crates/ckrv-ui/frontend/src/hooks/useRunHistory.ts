/**
 * Hook for fetching and managing run history data.
 */

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import type { HistoryListResponse, HistoryDetailResponse, CreateRunResponse } from '../types/history';

// ============================================================================
// API Functions
// ============================================================================

/**
 * Fetch run history for a specification
 */
export async function fetchRunHistory(spec: string, limit = 50, offset = 0): Promise<HistoryListResponse> {
    const params = new URLSearchParams({
        limit: limit.toString(),
        offset: offset.toString(),
    });

    const res = await fetch(`/api/history/${encodeURIComponent(spec)}?${params}`);
    return res.json();
}

/**
 * Fetch a single run by ID
 */
export async function fetchRun(spec: string, runId: string): Promise<HistoryDetailResponse> {
    const res = await fetch(`/api/history/${encodeURIComponent(spec)}/${encodeURIComponent(runId)}`);
    return res.json();
}

/**
 * Create a new run entry
 */
export async function createRun(
    spec: string,
    runId: string,
    batches: { id: string; name: string }[],
    dryRun: boolean
): Promise<CreateRunResponse> {
    const res = await fetch(`/api/history/${encodeURIComponent(spec)}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ run_id: runId, dry_run: dryRun, batches }),
    });
    return res.json();
}

/**
 * Update run status
 */
export async function updateRunStatus(
    spec: string,
    runId: string,
    status: string,
    error?: string
): Promise<{ success: boolean; error?: string }> {
    const res = await fetch(`/api/history/${encodeURIComponent(spec)}/${encodeURIComponent(runId)}`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ status, error }),
    });
    return res.json();
}

/**
 * Update batch status within a run
 */
export async function updateBatchStatus(
    spec: string,
    runId: string,
    batchId: string,
    status: string,
    branch?: string,
    error?: string
): Promise<{ success: boolean; error?: string }> {
    const res = await fetch(`/api/history/${encodeURIComponent(spec)}/${encodeURIComponent(runId)}`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            batch_update: { batch_id: batchId, status, branch, error },
        }),
    });
    return res.json();
}

/**
 * Delete a run from history
 */
export async function deleteRun(
    spec: string,
    runId: string
): Promise<{ success: boolean; error?: string }> {
    const res = await fetch(`/api/history/${encodeURIComponent(spec)}/${encodeURIComponent(runId)}`, {
        method: 'DELETE',
    });
    return res.json();
}

// ============================================================================
// Hooks
// ============================================================================

/**
 * Hook for fetching run history
 */
export function useRunHistory(spec: string | null, limit = 50, offset = 0) {
    return useQuery({
        queryKey: ['runHistory', spec, limit, offset],
        queryFn: () => fetchRunHistory(spec!, limit, offset),
        enabled: !!spec,
        staleTime: 10000, // 10 seconds
        refetchInterval: 30000, // Refetch every 30 seconds for live updates
    });
}

/**
 * Hook for fetching a single run
 */
export function useRun(spec: string | null, runId: string | null) {
    return useQuery({
        queryKey: ['run', spec, runId],
        queryFn: () => fetchRun(spec!, runId!),
        enabled: !!spec && !!runId,
        staleTime: 5000,
    });
}

/**
 * Hook for creating a new run
 */
export function useCreateRun() {
    const queryClient = useQueryClient();

    return useMutation({
        mutationFn: ({ spec, runId, batches, dryRun }: {
            spec: string;
            runId: string;
            batches: { id: string; name: string }[];
            dryRun: boolean;
        }) => createRun(spec, runId, batches, dryRun),
        onSuccess: (_, variables) => {
            // Invalidate history query to show new run
            queryClient.invalidateQueries({ queryKey: ['runHistory', variables.spec] });
        },
    });
}

/**
 * Hook for updating run status
 */
export function useUpdateRunStatus() {
    const queryClient = useQueryClient();

    return useMutation({
        mutationFn: ({ spec, runId, status, error }: {
            spec: string;
            runId: string;
            status: string;
            error?: string;
        }) => updateRunStatus(spec, runId, status, error),
        onSuccess: (_, variables) => {
            queryClient.invalidateQueries({ queryKey: ['runHistory', variables.spec] });
            queryClient.invalidateQueries({ queryKey: ['run', variables.spec, variables.runId] });
        },
    });
}

/**
 * Hook for deleting a run
 */
export function useDeleteRun() {
    const queryClient = useQueryClient();

    return useMutation({
        mutationFn: ({ spec, runId }: { spec: string; runId: string }) =>
            deleteRun(spec, runId),
        onSuccess: (_, variables) => {
            queryClient.invalidateQueries({ queryKey: ['runHistory', variables.spec] });
        },
    });
}
