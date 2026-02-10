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

/**
 * @returns {Object} Object containing command execution functions and state
 * @returns {Function} runInit - Mutation function to initialize the repository
 * @returns {boolean} isInitPending - Whether init command is currently executing
 * @returns {Error|null} initError - Error from init command if any
 * @returns {Function} runSpecNew - Mutation function to create a new spec
 * @returns {boolean} isSpecNewPending - Whether spec new command is executing
 * @returns {Error|null} specNewError - Error from spec new command if any
 * @returns {Function} runSpecTasks - Mutation function to generate spec tasks
 * @returns {boolean} isSpecTasksPending - Whether spec tasks command is executing
 * @returns {Error|null} specTasksError - Error from spec tasks command if any
 * @returns {Function} runExec - Mutation function to execute orchestration
 * @returns {boolean} isExecPending - Whether run command is executing
 * @returns {Error|null} execError - Error from run command if any
 */
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
