import { useQuery } from '@tanstack/react-query';
import type { WorkflowStage, CodeTabType } from '../types';

interface SystemStatus {
    is_ready: boolean;
    active_branch?: string;
}

interface SpecInfo {
    name: string;
    has_tasks: boolean;
    has_plan: boolean;
    has_implementation: boolean;
}

interface SpecsResponse {
    specs: SpecInfo[];
    count: number;
}

/**
 * Custom hook for tracking workflow progress across all stages.
 * 
 * This hook automatically selects the spec based on the current git branch
 * and determines which workflow stages are complete based on file existence.
 * 
 * @param overrideSpec - Optional spec name to override auto-selection
 * @returns An array of WorkflowStage objects with status
 */
export function useWorkflowProgress(overrideSpec?: string): WorkflowStage[] {
    // Fetch system status to get active branch
    const { data: status } = useQuery<SystemStatus>({
        queryKey: ['status'],
        queryFn: async () => {
            const res = await fetch('/api/status');
            return res.json();
        },
        staleTime: 5000,
    });

    // Fetch specs list to check stage completion
    const { data: specsData } = useQuery<SpecsResponse>({
        queryKey: ['specs'],
        queryFn: async () => {
            const res = await fetch('/api/specs');
            return res.json();
        },
        staleTime: 5000,
    });

    const activeBranch = status?.active_branch ?? '';
    const specs = specsData?.specs ?? [];

    // Auto-select spec based on branch name (matching useAutoSelectedSpec logic)
    const getAutoSelectedSpec = (): SpecInfo | undefined => {
        if (overrideSpec) {
            return specs.find(s => s.name === overrideSpec);
        }

        if (!activeBranch || activeBranch === 'none' || activeBranch === '' || activeBranch === 'main' || activeBranch === 'master') {
            return specs[0]; // Fall back to first spec
        }

        // Look for a spec that matches the branch name
        const matchingSpec = specs.find(s => s.name === activeBranch);
        if (matchingSpec) {
            return matchingSpec;
        }

        // Try partial match (branch might have prefix like "feature/")
        const branchParts = activeBranch.split('/');
        const branchSuffix = branchParts[branchParts.length - 1];
        const partialMatch = specs.find(s => s.name === branchSuffix);
        if (partialMatch) {
            return partialMatch;
        }

        return specs[0]; // Fall back to first spec
    };

    const currentSpec = getAutoSelectedSpec();
    const hasMatchedSpec = currentSpec !== undefined;
    const hasTasks = currentSpec?.has_tasks ?? false;
    const hasPlan = currentSpec?.has_plan ?? false;
    const hasImplementation = currentSpec?.has_implementation ?? false;

    // Build workflow stages array - only turn green when files exist for the matched spec
    const stages: WorkflowStage[] = [
        { id: 'spec' as CodeTabType, status: hasMatchedSpec ? 'complete' : 'pending' },
        { id: 'tasks' as CodeTabType, status: hasTasks ? 'complete' : 'pending' },
        { id: 'plan' as CodeTabType, status: hasPlan ? 'complete' : 'pending' },
        { id: 'run' as CodeTabType, status: hasImplementation ? 'complete' : 'pending' },
    ];

    return stages;
}

export default useWorkflowProgress;
