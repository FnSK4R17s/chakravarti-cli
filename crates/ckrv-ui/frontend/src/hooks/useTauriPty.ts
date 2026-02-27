/**
 * @module useTauriPty
 * @description
 * Custom hook for managing PTY (Pseudo-TTY) terminals in Tauri desktop mode.
 * Provides a unified interface for spawning and managing interactive docker exec
 * sessions that works with xterm.js.
 *
 * @context
 * Used by AgentCliModal, TestFixModal, TaskDetailModal for interactive terminal
 * sessions within Tauri desktop app. Not available in web mode (falls back to WebSocket).
 *
 * @dependencies
 * - tauri-pty: NPM package for frontend PTY API
 * - tauri-plugin-pty: Rust crate (initialized in main.rs)
 * - Docker: Container must be running with project mounted at /workspace
 *
 * ## Architecture Overview
 *
 * In Tauri mode, we use `tauri-plugin-pty` instead of WebSocket for interactive terminals:
 *
 * ```
 * ┌─────────────────────────────────────────────────────────────────┐
 * │                      Frontend (xterm.js)                        │
 * │  term.onData(data => pty.write(data))  // User types           │
 * │  pty.onData(data => term.write(data))  // Shell output         │
 * └─────────────────────────────────────────────────────────────────┘
 *                              │
 *                              ▼
 * ┌─────────────────────────────────────────────────────────────────┐
 * │                    tauri-plugin-pty (Rust)                      │
 * │  Uses portable-pty to create pseudo-terminal                   │
 * │  Runs: docker exec -it <container_id> /bin/bash                │
 * └─────────────────────────────────────────────────────────────────┘
 *                              │
 *                              ▼
 * ┌─────────────────────────────────────────────────────────────────┐
 * │                    Docker Container                             │
 * │  Running ckrv-claude:latest, ckrv-codex:latest, ckrv-kilo:latest, or ckrv-factory:latest  │
 * │  With project mounted at /workspace                            │
 * └─────────────────────────────────────────────────────────────────┘
 * ```
 *
 * ## Web vs Tauri Mode
 *
 * | Feature          | Web (Axum)           | Tauri (Desktop)      |
 * |------------------|----------------------|----------------------|
 * | Container create | Same (DockerClient)  | Same (DockerClient)  |
 * | Interactive shell| WebSocket + bollard  | PTY + docker exec    |
 * | Data transport   | WS binary frames     | Tauri IPC events     |
 *
 * ## Usage
 *
 * ```typescript
 * const { spawn, isAvailable } = useTauriPty();
 *
 * if (isAvailable && containerId) {
 *   const pty = await spawn(containerId, term.cols, term.rows);
 *   pty.onData(data => term.write(data));
 *   term.onData(data => pty.write(data));
 * }
 * ```
 *
 * ## Maintenance Notes
 *
 * - **Version coupling**: tauri-plugin-pty (Rust) and tauri-pty (npm) versions must match
 * - **Current versions**: 0.2.x for both
 * - **Upgrade path**: When upgrading Tauri, check tauri-plugin-pty compatibility
 */

import { useCallback, useRef } from 'react';

// === TYPES ===

/** PTY spawn options */
export interface PtySpawnOptions {
    /** Number of terminal columns */
    cols: number;
    /** Number of terminal rows */
    rows: number;
    /** Working directory inside container (default: /workspace) */
    cwd?: string;
    /** Additional environment variables */
    env?: Record<string, string>;
}

/** 
 * PTY instance returned by spawn.
 * Data comes as an array-like that should be wrapped in Uint8Array.
 */
export interface PtyInstance {
    /** Write data to PTY stdin */
    write: (data: string) => void;
    /** 
     * Subscribe to PTY stdout/stderr data.
     * Data comes as array-like, wrap in Uint8Array:
     * ```typescript
     * pty.onData((data) => term.write(new Uint8Array(data)));
     * ```
     */
    onData: (callback: (data: ArrayLike<number>) => void) => { dispose: () => void };
    /** Handle PTY exit */
    onExit: (callback: (e: { exitCode: number; signal?: number }) => void) => { dispose: () => void };
    /** Resize PTY dimensions */
    resize: (cols: number, rows: number) => void;
    /** Kill PTY process */
    kill: (signal?: string) => void;
    /** PTY process ID */
    pid: number;
}

/** Hook return type */
export interface UseTauriPtyReturn {
    /** Whether PTY is available (running in Tauri) */
    isAvailable: boolean;
    /**
     * Spawn a PTY process running docker exec in the given container.
     * @param containerId Docker container ID from terminal_start
     * @param options Spawn options (cols, rows, cwd)
     * @returns PTY instance for bidirectional communication
     */
    spawn: (containerId: string, options: PtySpawnOptions) => Promise<PtyInstance | null>;
    /**
     * Spawn a PTY process with custom command (for non-docker use cases)
     * @param command Command to run
     * @param args Command arguments
     * @param options Spawn options
     */
    spawnCommand: (command: string, args: string[], options: PtySpawnOptions) => Promise<PtyInstance | null>;
}

// === DETECTION ===

/**
 * Check if running in Tauri environment
 */
export function isTauriEnvironment(): boolean {
    return typeof window !== 'undefined' && !!(window as any).__TAURI__;
}

// === HOOK ===

/**
 * Hook for managing PTY terminals in Tauri mode.
 *
 * Provides a clean abstraction over tauri-pty plugin for spawning
 * interactive docker exec sessions.
 */
export function useTauriPty(): UseTauriPtyReturn {
    const ptyRef = useRef<any>(null);
    const isAvailable = isTauriEnvironment();

    /**
     * Spawn docker exec -it in container through PTY
     */
    const spawn = useCallback(async (
        containerId: string,
        options: PtySpawnOptions
    ): Promise<PtyInstance | null> => {
        if (!isAvailable) {
            console.warn('[useTauriPty] Not in Tauri environment');
            return null;
        }

        try {
            // Dynamic import to avoid loading tauri-pty in web mode
            const { spawn: ptySpawn } = await import('tauri-pty');

            // Spawn docker exec with interactive TTY
            const pty = await ptySpawn('docker', [
                'exec',
                '-it',
                containerId,
                '/bin/bash',
                '-l'  // Login shell for proper environment
            ], {
                cols: options.cols,
                rows: options.rows,
                cwd: options.cwd,
                env: options.env,
            });

            ptyRef.current = pty;

            console.log('[useTauriPty] Spawned PTY for container:', containerId);

            return pty as PtyInstance;
        } catch (error) {
            console.error('[useTauriPty] Failed to spawn PTY:', error);
            return null;
        }
    }, [isAvailable]);

    /**
     * Spawn custom command through PTY
     */
    const spawnCommand = useCallback(async (
        command: string,
        args: string[],
        options: PtySpawnOptions
    ): Promise<PtyInstance | null> => {
        if (!isAvailable) {
            console.warn('[useTauriPty] Not in Tauri environment');
            return null;
        }

        try {
            const { spawn: ptySpawn } = await import('tauri-pty');

            const pty = await ptySpawn(command, args, {
                cols: options.cols,
                rows: options.rows,
                cwd: options.cwd,
                env: options.env,
            });

            ptyRef.current = pty;

            console.log('[useTauriPty] Spawned PTY command:', command, args);

            return pty as PtyInstance;
        } catch (error) {
            console.error('[useTauriPty] Failed to spawn command:', error);
            return null;
        }
    }, [isAvailable]);

    return {
        isAvailable,
        spawn,
        spawnCommand,
    };
}

export default useTauriPty;
