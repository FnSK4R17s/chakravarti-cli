/**
 * @module test/test-utils
 * @description
 * Custom render function that wraps components with required providers
 * (QueryClientProvider). Re-exports all @testing-library/react utilities
 * for convenient single-import usage in tests.
 *
 * @context
 * All component tests should import from '@/test/test-utils' instead of
 * '@testing-library/react' directly.
 *
 * @dependencies
 * - @testing-library/react: Core testing utilities
 * - @tanstack/react-query: QueryClient provider wrapper
 */

import type { ReactElement } from 'react';
import { render, type RenderOptions } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';

/**
 * Creates a fresh QueryClient configured for testing.
 * Disables retries and sets staleTime to 0 for deterministic behavior.
 */
function createTestQueryClient(): QueryClient {
  return new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
        gcTime: Infinity,
      },
      mutations: {
        retry: false,
      },
    },
  });
}

/**
 * Custom render that wraps the component under test with all required providers.
 * @param ui - The React element to render
 * @param options - Additional render options (can override wrapper)
 */
function customRender(
  ui: ReactElement,
  options?: Omit<RenderOptions, 'wrapper'>
) {
  const queryClient = createTestQueryClient();

  function Wrapper({ children }: { children: React.ReactNode }) {
    return (
      <QueryClientProvider client={queryClient}>
        {children}
      </QueryClientProvider>
    );
  }

  return { ...render(ui, { wrapper: Wrapper, ...options }), queryClient };
}

// Re-export everything from testing-library
export * from '@testing-library/react';
export { userEvent } from '@testing-library/user-event';

// Override render with the custom version
export { customRender as render, createTestQueryClient };
