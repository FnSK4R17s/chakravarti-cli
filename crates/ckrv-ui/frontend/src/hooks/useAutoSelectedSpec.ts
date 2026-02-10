/**
 * @module useAutoSelectedSpec
 * @description
 * Hook for automatically determining the current spec based on the active git branch.
 * Matches branch names to spec names (e.g., "015-feature-name" on branch matches
 * spec "015-feature-name") with fallback to partial matching.
 *
 * @context
 * Used in SpecEditor, TaskEditor, and other components to auto-select the relevant
 * spec without manual user selection. Makes branch-per-feature workflows seamless.
 *
 * @dependencies
 * - useQuery: React Query for fetching status and specs
 * - SystemStatus, SpecListItem: Types for git status and spec info
 *
 * @example
 * const { selectedSpec, availableSpecs, activeBranch } = useAutoSelectedSpec();
 * // selectedSpec will be the spec matching the current branch, or null
 */

// === IMPORTS ===
import { useQuery } from '@tanstack/react-query';
import { useMemo } from 'react';
import type { SystemStatus } from '../types';

interface SpecListItem {
    name: string;
    path: string;
    has_tasks: boolean;
    has_plan: boolean;
    has_implementation: boolean;
}

interface SpecsResponse {
    specs: SpecListItem[];
    count: number;
}

const fetchStatus = async (): Promise<SystemStatus> => {
    const res = await fetch('/api/status');
    if (!res.ok) throw new Error('Failed to fetch status');
    return res.json();
};

const fetchSpecs = async (): Promise<SpecsResponse> => {
    const res = await fetch('/api/specs');
    return res.json();
};

/**
 * Hook to automatically determine the current spec based on the active git branch.
 *
 * The spec name is derived from the branch name (e.g., branch "015-unified-code-page"
 * maps to spec "015-unified-code-page").
 *
 * @returns Object containing:
 *   - selectedSpec: The auto-selected spec name if found, otherwise null
 *   - availableSpecs: Array of all available specs
 *   - isLoading: Whether status or specs are still loading
 *   - activeBranch: The current git branch name or null
 */
export function useAutoSelectedSpec(): {
    selectedSpec: string | null;
    availableSpecs: SpecListItem[];
    isLoading: boolean;
    activeBranch: string | null;
} {
    // Fetch current system status (includes active branch)
    const { data: status, isLoading: isStatusLoading } = useQuery({
        queryKey: ['status'],
        queryFn: fetchStatus,
        refetchInterval: 10000, // Refresh every 10 seconds
    });

    // Fetch available specs
    const { data: specsData, isLoading: isSpecsLoading } = useQuery({
        queryKey: ['specs'],
        queryFn: fetchSpecs,
    });

    const activeBranch = status?.active_branch ?? null;
    const specs = specsData?.specs ?? [];

    // Auto-select spec based on branch name
    const selectedSpec = useMemo(() => {
        if (!activeBranch || activeBranch === 'none' || activeBranch === '' || activeBranch === 'main' || activeBranch === 'master') {
            return null;
        }

        // Look for a spec that matches the branch name
        const matchingSpec = specs.find(s => s.name === activeBranch);
        if (matchingSpec) {
            return matchingSpec.name;
        }

        // Try partial match (branch might have prefix like "feature/")
        const branchParts = activeBranch.split('/');
        const branchSuffix = branchParts[branchParts.length - 1];
        const partialMatch = specs.find(s => s.name === branchSuffix);
        if (partialMatch) {
            return partialMatch.name;
        }

        return null;
    }, [activeBranch, specs]);

    return {
        selectedSpec,
        availableSpecs: specs,
        isLoading: isStatusLoading || isSpecsLoading,
        activeBranch,
    };
}

export default useAutoSelectedSpec;
