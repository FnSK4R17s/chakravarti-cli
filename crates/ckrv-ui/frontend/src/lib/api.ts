/**
 * @module api
 * @description
 * Unified API layer that automatically routes requests through Tauri IPC
 * when running as a desktop app. This module patches the global `fetch`
 * function so NO other frontend files need to be modified.
 *
 * @usage
 * Import this module once in main.tsx to activate the fetch interceptor:
 * ```
 * import '@/lib/api';
 * ```
 */

// === TAURI DETECTION ===

/** Check if running inside Tauri desktop app (Tauri v2 uses __TAURI_INTERNALS__) */
export const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

// === ENDPOINT MAPPING ===

/**
 * Map API endpoints to Tauri command names.
 * Supports both exact matches and pattern matching.
 */
const endpointToCommand: Record<string, string> = {
    // Status endpoints
    '/api/status': 'get_status',
    '/api/docker': 'check_docker',
    '/api/cloud': 'get_cloud_status',

    // Agent endpoints
    '/api/agents': 'list_agents',
    '/api/agents/models': 'get_openrouter_models',
    '/api/agents/kilo-models': 'get_kilo_models',
    '/api/agents/glm-models': 'get_glm_models',
    '/api/agents/upsert': 'upsert_agent',
    '/api/agents/delete': 'delete_agent',
    '/api/agents/set-default': 'set_default_agent',
    '/api/agents/set-qa': 'set_qa_agent',
    '/api/agents/set-test-writer': 'set_test_writer_agent',
    '/api/agents/test': 'test_agent',

    // Spec endpoints
    '/api/specs': 'list_specs',
    '/api/specs/create': 'create_spec',

    // Plan endpoints
    '/api/plans': 'list_plans',
    '/api/plans/save': 'save_plan',

    // Diff endpoints
    '/api/diff/branches': 'get_branches',
    '/api/diff': 'get_diff',

    // Git endpoints
    '/api/git/default-branch': 'get_default_branch',

    // QA endpoints
    '/api/qa/agent': 'get_qa_agent',
    '/api/qa/review': 'run_review',
    '/api/qa/bugs': 'run_bugs',
    '/api/qa/report': 'run_report',

    // Test endpoints
    '/api/test/agent': 'get_test_agent',
    '/api/test/run': 'run_tests',
    '/api/test/plan': 'plan_tests',
    '/api/test/plan-status': 'get_plan_status',
    '/api/test/write': 'write_tests',
    '/api/test/write-status': 'get_write_status',
    '/api/test/coverage': 'get_coverage',
    '/api/test/generate': 'generate_tests',
    '/api/test/fix': 'fix_tests',

    // CLI command endpoints
    '/api/command/init': 'run_init',
    '/api/command/git-init': 'run_git_init',
    '/api/command/spec-new': 'run_spec_new',
    '/api/command/spec-tasks': 'run_spec_tasks',
    '/api/command/plan': 'run_plan',
    '/api/command/execute': 'run_execute',
    '/api/command/diff': 'run_diff',
    '/api/command/verify': 'run_verify',
    '/api/command/promote': 'run_promote',
    '/api/command/fix': 'run_fix',

    // Terminal endpoints
    '/api/terminal/start': 'terminal_start',
    '/api/terminal/stop': 'terminal_stop',
    '/api/terminal/write': 'terminal_write',
    '/api/terminal/read': 'terminal_read',
    '/api/terminal/list': 'terminal_list',
};

/**
 * Pattern-based endpoint matching for dynamic routes like /api/specs/:name
 */
function getCommandForUrl(url: string): string | null {
    // Check exact match first
    if (endpointToCommand[url]) {
        return endpointToCommand[url];
    }

    // Pattern matching for dynamic routes
    const patterns: [RegExp, string][] = [
        [/^\/api\/specs\/detail/, 'get_spec'],
        [/^\/api\/specs\/([^/]+)\/validate$/, 'validate_spec'],
        [/^\/api\/specs\/([^/]+)\/design$/, 'generate_design'],
        [/^\/api\/specs\/([^/]+)\/tasks$/, 'generate_tasks'],
        [/^\/api\/specs\/([^/]+)\/clarifications$/, 'get_clarifications'],
        [/^\/api\/specs\/([^/]+)\/clarify$/, 'submit_clarification'],
        [/^\/api\/plans\/detail/, 'get_plan'],
        [/^\/api\/plans\/models$/, 'get_openrouter_models'],
        [/^\/api\/history\/([^/]+)$/, 'list_history'],
        [/^\/api\/history\/([^/]+)\/([^/]+)$/, 'get_run'],
    ];

    for (const [pattern, command] of patterns) {
        if (pattern.test(url)) {
            return command;
        }
    }

    return null;
}

// === GLOBAL FETCH INTERCEPTOR ===

if (isTauri) {
    const originalFetch = window.fetch;

    window.fetch = async function (input: RequestInfo | URL, init?: RequestInit): Promise<Response> {
        // Get the URL string
        let url: string;
        if (typeof input === 'string') {
            url = input;
        } else if (input instanceof URL) {
            url = input.pathname + input.search;
        } else {
            url = input.url;
        }

        // Only intercept /api/* calls
        if (!url.startsWith('/api/')) {
            return originalFetch(input, init);
        }

        // Parse the URL for query parameters
        const urlObj = new URL(url, window.location.origin);
        const baseUrl = urlObj.pathname;

        // Find matching command
        const command = getCommandForUrl(baseUrl);
        if (!command) {
            console.warn(`[Tauri] No command mapping for: ${baseUrl}`);
            // For unmapped endpoints, create a mock error response
            return new Response(
                JSON.stringify({ error: `Endpoint not yet implemented in Tauri: ${baseUrl}` }),
                { status: 501, headers: { 'Content-Type': 'application/json' } }
            );
        }

        try {
            // Dynamic import Tauri API
            const { invoke } = await import('@tauri-apps/api/core');

            // Start with query parameters as args
            const args: Record<string, unknown> = {};

            // Add query parameters
            urlObj.searchParams.forEach((value, key) => {
                args[key] = value;
            });

            // Extract dynamic route parameters
            const specMatch = baseUrl.match(/^\/api\/specs\/([^/]+)(?:\/(validate|design|tasks|clarifications|clarify))?$/);
            if (specMatch && specMatch[1] !== 'detail' && specMatch[1] !== 'create') {
                args['name'] = decodeURIComponent(specMatch[1]);
            }

            const historyMatch = baseUrl.match(/^\/api\/history\/([^/]+)(?:\/([^/]+))?$/);
            if (historyMatch) {
                args['spec'] = decodeURIComponent(historyMatch[1]);
                if (historyMatch[2]) {
                    args['run_id'] = decodeURIComponent(historyMatch[2]);
                }
            }

            const planMatch = baseUrl.match(/^\/api\/plans\/detail/);
            if (planMatch && urlObj.searchParams.has('spec')) {
                args['spec'] = urlObj.searchParams.get('spec')!;
            }

            // Parse body if present and merge with args
            if (init?.body) {
                try {
                    const bodyArgs = JSON.parse(init.body as string);
                    Object.assign(args, bodyArgs);
                } catch {
                    console.warn('[Tauri] Could not parse request body as JSON');
                }
            }

            console.debug(`[Tauri] invoke(${command})`, args);
            const result = await invoke(command, args);

            // Wrap result in a Response object to match fetch API
            return new Response(JSON.stringify(result), {
                status: 200,
                headers: { 'Content-Type': 'application/json' },
            });
        } catch (error) {
            console.error(`[Tauri] Command failed: ${command}`, error);
            return new Response(
                JSON.stringify({ error: String(error) }),
                { status: 500, headers: { 'Content-Type': 'application/json' } }
            );
        }
    };

    console.log('[Tauri] Fetch interceptor installed - API calls will route through IPC');
}

// === LEGACY EXPORT (for gradual migration) ===

/**
 * @deprecated Use standard fetch() instead - it's automatically intercepted in Tauri.
 * This export is kept for backward compatibility.
 */
export async function apiRequest<T>(
    endpoint: string,
    options?: RequestInit
): Promise<T> {
    const res = await fetch(endpoint, options);
    return res.json();
}
