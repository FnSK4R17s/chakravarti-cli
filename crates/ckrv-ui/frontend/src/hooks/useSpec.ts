/**
 * useSpec - Hook for spec state management and API interactions
 */
import { useState, useCallback } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';

// API Types
interface Spec {
    name: string;
    path: string;
    has_tasks: boolean;
    has_plan: boolean;
    task_count: number;
    has_implementation: boolean;
    implementation_branch?: string;
}

interface SpecsResponse {
    specs: Spec[];
    count: number;
}

interface UserStoryAcceptance {
    given: string;
    when: string;
    then: string;
}

interface UserStory {
    id: string;
    title: string;
    priority?: string;
    description?: string;
    acceptance?: UserStoryAcceptance[];
}

interface Requirement {
    id: string;
    description: string;
}

interface SuccessCriterion {
    id: string;
    metric: string;
}

interface SpecDetail {
    id: string;
    goal?: string;
    overview?: string;
    constraints?: string[];
    acceptance?: string[];
    user_stories?: UserStory[];
    requirements?: Requirement[];
    success_criteria?: SuccessCriterion[];
    assumptions?: string[];
    clarifications?: Clarification[];
    status?: string;
}

interface Clarification {
    topic: string;
    question: string;
    options: ClarificationOption[];
    resolved?: string;
}

interface ClarificationOption {
    label: string;
    answer: string;
    implications?: string;
}

interface ValidationError {
    field: string;
    message: string;
}

interface ValidateResponse {
    success: boolean;
    valid: boolean;
    errors: ValidationError[];
    warnings: string[];
}

interface CreateSpecPayload {
    description: string;
    name?: string;
}

interface CreateSpecResponse {
    success: boolean;
    spec_id?: string;
    spec_path?: string;
    message?: string;
    error?: string;
}

// API Functions
const fetchSpecs = async (): Promise<SpecsResponse> => {
    const response = await fetch('/api/specs');
    if (!response.ok) throw new Error('Failed to fetch specs');
    return response.json();
};

const fetchSpecDetail = async (name: string): Promise<SpecDetail | null> => {
    const response = await fetch(`/api/specs/detail?name=${encodeURIComponent(name)}`);
    if (!response.ok) throw new Error('Failed to fetch spec detail');
    const data = await response.json();
    return data.success ? data.spec : null;
};

const createSpec = async (payload: CreateSpecPayload): Promise<CreateSpecResponse> => {
    const response = await fetch('/api/specs/create', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
    });
    if (!response.ok) throw new Error('Failed to create spec');
    return response.json();
};

const validateSpec = async (name: string): Promise<ValidateResponse> => {
    const response = await fetch(`/api/specs/${encodeURIComponent(name)}/validate`);
    if (!response.ok) throw new Error('Failed to validate spec');
    return response.json();
};

const generateDesign = async (name: string) => {
    const response = await fetch(`/api/specs/${encodeURIComponent(name)}/design`, {
        method: 'POST',
    });
    if (!response.ok) throw new Error('Failed to generate design');
    return response.json();
};

const generateTasks = async (name: string) => {
    const response = await fetch(`/api/specs/${encodeURIComponent(name)}/tasks`, {
        method: 'POST',
    });
    if (!response.ok) throw new Error('Failed to generate tasks');
    return response.json();
};

const fetchClarifications = async (name: string) => {
    const response = await fetch(`/api/specs/${encodeURIComponent(name)}/clarifications`);
    if (!response.ok) throw new Error('Failed to fetch clarifications');
    return response.json();
};

const submitClarifications = async (name: string, answers: { topic: string; answer: string }[]) => {
    const response = await fetch(`/api/specs/${encodeURIComponent(name)}/clarify`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ answers }),
    });
    if (!response.ok) throw new Error('Failed to submit clarifications');
    return response.json();
};

// Hooks
export function useSpecs() {
    return useQuery({
        queryKey: ['specs'],
        queryFn: fetchSpecs,
        staleTime: 5000,
    });
}

export function useSpecDetail(name: string | null) {
    return useQuery({
        queryKey: ['spec', name],
        queryFn: () => (name ? fetchSpecDetail(name) : null),
        enabled: !!name,
        staleTime: 5000,
    });
}

export function useCreateSpec() {
    const queryClient = useQueryClient();

    return useMutation({
        mutationFn: createSpec,
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['specs'] });
        },
    });
}

export function useValidateSpec() {
    return useMutation({
        mutationFn: validateSpec,
    });
}

export function useGenerateDesign() {
    const queryClient = useQueryClient();

    return useMutation({
        mutationFn: generateDesign,
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['specs'] });
        },
    });
}

export function useGenerateTasks() {
    const queryClient = useQueryClient();

    return useMutation({
        mutationFn: generateTasks,
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['specs'] });
        },
    });
}

export function useClarifications(name: string | null) {
    return useQuery({
        queryKey: ['clarifications', name],
        queryFn: () => (name ? fetchClarifications(name) : null),
        enabled: !!name,
        staleTime: 5000,
    });
}

export function useSubmitClarifications() {
    const queryClient = useQueryClient();

    return useMutation({
        mutationFn: ({ name, answers }: { name: string; answers: { topic: string; answer: string }[] }) =>
            submitClarifications(name, answers),
        onSuccess: (_, variables) => {
            queryClient.invalidateQueries({ queryKey: ['spec', variables.name] });
            queryClient.invalidateQueries({ queryKey: ['clarifications', variables.name] });
        },
    });
}

// Composite hook for full spec workflow
export function useSpecWorkflow(specName: string | null) {
    const [isProcessing, setIsProcessing] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const { data: spec, isLoading: specLoading, refetch: refetchSpec } = useSpecDetail(specName);
    const { data: clarifications, isLoading: clarificationsLoading } = useClarifications(specName);

    const validateMutation = useValidateSpec();
    const designMutation = useGenerateDesign();
    const tasksMutation = useGenerateTasks();
    const clarifyMutation = useSubmitClarifications();

    const validate = useCallback(async () => {
        if (!specName) return null;
        setIsProcessing(true);
        setError(null);
        try {
            const result = await validateMutation.mutateAsync(specName);
            return result;
        } catch (e) {
            setError(e instanceof Error ? e.message : 'Validation failed');
            return null;
        } finally {
            setIsProcessing(false);
        }
    }, [specName, validateMutation]);

    const generateDesignDoc = useCallback(async () => {
        if (!specName) return null;
        setIsProcessing(true);
        setError(null);
        try {
            const result = await designMutation.mutateAsync(specName);
            await refetchSpec();
            return result;
        } catch (e) {
            setError(e instanceof Error ? e.message : 'Design generation failed');
            return null;
        } finally {
            setIsProcessing(false);
        }
    }, [specName, designMutation, refetchSpec]);

    const generateTasksDoc = useCallback(async () => {
        if (!specName) return null;
        setIsProcessing(true);
        setError(null);
        try {
            const result = await tasksMutation.mutateAsync(specName);
            await refetchSpec();
            return result;
        } catch (e) {
            setError(e instanceof Error ? e.message : 'Task generation failed');
            return null;
        } finally {
            setIsProcessing(false);
        }
    }, [specName, tasksMutation, refetchSpec]);

    const submitAnswers = useCallback(async (answers: { topic: string; answer: string }[]) => {
        if (!specName) return null;
        setIsProcessing(true);
        setError(null);
        try {
            const result = await clarifyMutation.mutateAsync({ name: specName, answers });
            await refetchSpec();
            return result;
        } catch (e) {
            setError(e instanceof Error ? e.message : 'Clarification submission failed');
            return null;
        } finally {
            setIsProcessing(false);
        }
    }, [specName, clarifyMutation, refetchSpec]);

    return {
        spec,
        clarifications: clarifications?.clarifications ?? [],
        unresolvedCount: clarifications?.unresolved_count ?? 0,
        isLoading: specLoading || clarificationsLoading,
        isProcessing,
        error,
        validate,
        generateDesignDoc,
        generateTasksDoc,
        submitAnswers,
        refetchSpec,
    };
}

export type { Spec, SpecDetail, Clarification, ClarificationOption, ValidationError };
