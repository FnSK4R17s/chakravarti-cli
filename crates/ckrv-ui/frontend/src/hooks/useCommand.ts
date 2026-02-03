/**
 * @module useCommand
 * @description
 * Hook for executing CLI commands via the API. Provides mutations for init,
 * spec new, spec tasks, and run commands with automatic cache invalidation
 * after successful execution.
 *
 * @context
 * Used by CommandPalette and other components to trigger CLI operations.
 * Wraps API calls with React Query mutations for consistent loading/error states.
 *
 * @dependencies
 * - useMutation, useQueryClient: React Query for command execution
 *
 * @example
 * const { runInit, isInitPending, runExec } = useCommand();
 * await runInit();
 */

// === IMPORTS ===
import { useMutation, useQueryClient } from '@tanstack/react-query';

interface CommandResult {
    success: boolean;
    message?: string;
}

const runCommand = async (endpoint: string): Promise<CommandResult> => {
    const res = await fetch(`/api/command/${endpoint}`, { method: 'POST' });
    if (!res.ok) {
        const error = await res.text();
        throw new Error(error || 'Command failed');
    }
    return res.json();
};

export const useCommand = () => {
    const queryClient = useQueryClient();

    const init = useMutation({
        mutationFn: () => runCommand('init'),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['status'] });
            queryClient.invalidateQueries({ queryKey: ['specs'] });
        }
    });

    const specNew = useMutation({
        mutationFn: (description: string) =>
            fetch('/api/command/spec/new', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ description })
            }).then(res => res.json()),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['specs'] });
        }
    });

    const specTasks = useMutation({
        mutationFn: () => runCommand('spec/tasks'),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['tasks'] });
        }
    });

    const run = useMutation({
        mutationFn: () => runCommand('run'),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['status'] });
            queryClient.invalidateQueries({ queryKey: ['tasks'] });
        }
    });

    return {
        runInit: init.mutate,
        isInitPending: init.isPending,
        initError: init.error,

        runSpecNew: specNew.mutate,
        isSpecNewPending: specNew.isPending,
        specNewError: specNew.error,

        runSpecTasks: specTasks.mutate,
        isSpecTasksPending: specTasks.isPending,
        specTasksError: specTasks.error,

        runExec: run.mutate,
        isExecPending: run.isPending,
        execError: run.error,
    };
};
