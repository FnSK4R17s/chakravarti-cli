/**
 * @module test/mocks/tauri-pty.stub
 * @description
 * Stub module for tauri-pty in the Vitest environment. The real tauri-pty
 * package ships without a CJS/ESM exports field that Vite can resolve in
 * the jsdom test environment, so this stub is aliased via vitest.config.ts.
 *
 * @context
 * Aliased in vitest.config.ts resolve.alias so any import of 'tauri-pty'
 * resolves here instead of the broken node_modules package.
 *
 * @dependencies
 * None – pure stub with no runtime behaviour.
 */

export const spawn = () => Promise.resolve(null);

export default { spawn };
