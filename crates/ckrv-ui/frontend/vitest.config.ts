import { defineConfig, mergeConfig } from 'vitest/config';
import path from 'path';
import viteConfig from './vite.config';

export default mergeConfig(
  viteConfig,
  defineConfig({
    resolve: {
      alias: {
        // tauri-pty ships without a resolvable CJS/ESM entry in jsdom tests;
        // alias to a no-op stub so dynamic imports inside components don't
        // break module graph resolution.
        'tauri-pty': path.resolve(__dirname, './src/test/mocks/tauri-pty.stub.ts'),
      },
    },
    test: {
      environment: 'jsdom',
      globals: true,
      setupFiles: ['./src/test/setup.ts'],
      include: ['src/**/*.test.{ts,tsx}'],
      exclude: ['tests/e2e/**', 'src/components/ui/**'],
      css: false,
      coverage: {
        provider: 'v8',
        include: ['src/**/*.{ts,tsx}'],
        exclude: [
          'src/test/**',
          'src/components/ui/**',
          'src/types/**',
          'src/main.tsx',
          'src/vite-env.d.ts',
        ],
        thresholds: {
          lines: 20,
          functions: 18,
          branches: 15,
        },
      },
    },
  })
);
