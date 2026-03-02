/**
 * @module useSpec.test
 * @description
 * Comprehensive unit tests for all useSpec hooks. Validates data fetching,
 * mutation behavior, cache invalidation, disabled query handling, and
 * the composite useSpecWorkflow hook.
 *
 * @context
 * Tests run in jsdom via Vitest with MSW for HTTP mocking. Each test
 * uses a fresh QueryClient to prevent state leakage between cases.
 *
 * @dependencies
 * - @testing-library/react: renderHook, waitFor, act
 * - @tanstack/react-query: QueryClient, QueryClientProvider
 * - msw: http, HttpResponse for per-test handler overrides
 * - @/test/mocks/server: MSW server instance
 * - @/test/mocks/fixtures: createSpecDetail factory
 * - @/hooks/useSpec: hooks under test
 */

import type { ReactNode } from 'react';
import { describe, it, expect } from 'vitest';
import { renderHook, waitFor, act } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { http, HttpResponse } from 'msw';
import { server } from '@/test/mocks/server';
import { createSpecDetail } from '@/test/mocks/fixtures';
import {
  useSpecs,
  useSpecDetail,
  useCreateSpec,
  useValidateSpec,
  useGenerateDesign,
  useGenerateTasks,
  useClarifications,
  useSubmitClarifications,
  useSpecWorkflow,
} from './useSpec';

// ============================================================
// WRAPPER FACTORY
// ============================================================

/**
 * Creates a fresh QueryClient wrapper for each test to prevent state bleed.
 * Retries disabled for deterministic failure behaviour.
 */
function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });
  return function Wrapper({ children }: { children: ReactNode }) {
    return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
  };
}

// ============================================================
// CONSTANTS
// ============================================================

const SPEC_NAME = '042-add-auth';

// ============================================================
// useSpecs
// ============================================================

describe('useSpecs', () => {
  it('fetches the spec list from GET /api/specs', async () => {
    const { result } = renderHook(() => useSpecs(), { wrapper: createWrapper() });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toBeDefined();
    // Base handler returns 2 specs
    expect(result.current.data!.count).toBe(2);
    expect(result.current.data!.specs).toHaveLength(2);
    expect(result.current.data!.specs[0].name).toBe('042-add-auth');
  });

  it('exposes loading state before data arrives', () => {
    const { result } = renderHook(() => useSpecs(), { wrapper: createWrapper() });

    expect(result.current.isLoading).toBe(true);
    expect(result.current.data).toBeUndefined();
  });

  it('transitions to error state when the API fails', async () => {
    server.use(
      http.get('/api/specs', () => HttpResponse.error()),
    );

    const { result } = renderHook(() => useSpecs(), { wrapper: createWrapper() });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(result.current.error).toBeInstanceOf(Error);
  });
});

// ============================================================
// useSpecDetail
// ============================================================

describe('useSpecDetail', () => {
  it('fetches spec detail from GET /api/specs/detail?name=X', async () => {
    server.use(
      http.get('/api/specs/detail', ({ request }) => {
        const url = new URL(request.url);
        expect(url.searchParams.get('name')).toBe(SPEC_NAME);
        return HttpResponse.json({ success: true, spec: createSpecDetail() });
      }),
    );

    const { result } = renderHook(() => useSpecDetail(SPEC_NAME), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toBeDefined();
    expect(result.current.data!.id).toBe(SPEC_NAME);
    expect(result.current.data!.goal).toBe('Add user authentication to the application');
  });

  it('returns null data and stays disabled when name is null', () => {
    const { result } = renderHook(() => useSpecDetail(null), {
      wrapper: createWrapper(),
    });

    // enabled: false means the query never fires and isLoading is false
    expect(result.current.isLoading).toBe(false);
    expect(result.current.fetchStatus).toBe('idle');
    expect(result.current.data).toBeUndefined();
  });

  it('reflects error state when the API fails', async () => {
    server.use(
      http.get('/api/specs/detail', () => HttpResponse.error()),
    );

    const { result } = renderHook(() => useSpecDetail(SPEC_NAME), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isError).toBe(true));
  });
});

// ============================================================
// useCreateSpec
// ============================================================

describe('useCreateSpec', () => {
  it('calls POST /api/specs/create and returns the spec_id', async () => {
    server.use(
      http.post('/api/specs/create', () =>
        HttpResponse.json({ success: true, spec_id: SPEC_NAME }),
      ),
    );

    const { result } = renderHook(() => useCreateSpec(), { wrapper: createWrapper() });

    await act(async () => {
      result.current.mutate({ description: 'Add authentication' });
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data!.spec_id).toBe(SPEC_NAME);
    expect(result.current.data!.success).toBe(true);
  });

  it('transitions to error state when POST fails', async () => {
    server.use(
      http.post('/api/specs/create', () => HttpResponse.error()),
    );

    const { result } = renderHook(() => useCreateSpec(), { wrapper: createWrapper() });

    await act(async () => {
      result.current.mutate({ description: 'Add authentication' });
    });

    await waitFor(() => expect(result.current.isError).toBe(true));
  });
});

// ============================================================
// useValidateSpec
// ============================================================

describe('useValidateSpec', () => {
  it('calls GET /api/specs/X/validate and returns validation result', async () => {
    server.use(
      http.get(`/api/specs/${SPEC_NAME}/validate`, () =>
        HttpResponse.json({ success: true, valid: true, errors: [], warnings: [] }),
      ),
    );

    const { result } = renderHook(() => useValidateSpec(), { wrapper: createWrapper() });

    await act(async () => {
      result.current.mutate(SPEC_NAME);
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data!.valid).toBe(true);
    expect(result.current.data!.errors).toHaveLength(0);
    expect(result.current.data!.warnings).toHaveLength(0);
  });

  it('surfaces validation errors returned by the API', async () => {
    server.use(
      http.get(`/api/specs/${SPEC_NAME}/validate`, () =>
        HttpResponse.json({
          success: true,
          valid: false,
          errors: [{ field: 'goal', message: 'Goal is required' }],
          warnings: ['Consider adding acceptance criteria'],
        }),
      ),
    );

    const { result } = renderHook(() => useValidateSpec(), { wrapper: createWrapper() });

    await act(async () => {
      result.current.mutate(SPEC_NAME);
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data!.valid).toBe(false);
    expect(result.current.data!.errors).toHaveLength(1);
    expect(result.current.data!.errors[0].field).toBe('goal');
    expect(result.current.data!.warnings).toHaveLength(1);
  });

  it('transitions to error state when validation request fails', async () => {
    server.use(
      http.get(`/api/specs/${SPEC_NAME}/validate`, () => HttpResponse.error()),
    );

    const { result } = renderHook(() => useValidateSpec(), { wrapper: createWrapper() });

    await act(async () => {
      result.current.mutate(SPEC_NAME);
    });

    await waitFor(() => expect(result.current.isError).toBe(true));
  });
});

// ============================================================
// useGenerateDesign
// ============================================================

describe('useGenerateDesign', () => {
  it('calls POST /api/specs/X/design and returns success', async () => {
    server.use(
      http.post(`/api/specs/${SPEC_NAME}/design`, () =>
        HttpResponse.json({ success: true }),
      ),
    );

    const { result } = renderHook(() => useGenerateDesign(), { wrapper: createWrapper() });

    await act(async () => {
      result.current.mutate(SPEC_NAME);
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toMatchObject({ success: true });
  });

  it('transitions to error state when design generation fails', async () => {
    server.use(
      http.post(`/api/specs/${SPEC_NAME}/design`, () => HttpResponse.error()),
    );

    const { result } = renderHook(() => useGenerateDesign(), { wrapper: createWrapper() });

    await act(async () => {
      result.current.mutate(SPEC_NAME);
    });

    await waitFor(() => expect(result.current.isError).toBe(true));
  });
});

// ============================================================
// useGenerateTasks
// ============================================================

describe('useGenerateTasks', () => {
  it('calls POST /api/specs/X/tasks and returns success', async () => {
    server.use(
      http.post(`/api/specs/${SPEC_NAME}/tasks`, () =>
        HttpResponse.json({ success: true }),
      ),
    );

    const { result } = renderHook(() => useGenerateTasks(), { wrapper: createWrapper() });

    await act(async () => {
      result.current.mutate(SPEC_NAME);
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toMatchObject({ success: true });
  });

  it('transitions to error state when task generation fails', async () => {
    server.use(
      http.post(`/api/specs/${SPEC_NAME}/tasks`, () => HttpResponse.error()),
    );

    const { result } = renderHook(() => useGenerateTasks(), { wrapper: createWrapper() });

    await act(async () => {
      result.current.mutate(SPEC_NAME);
    });

    await waitFor(() => expect(result.current.isError).toBe(true));
  });
});

// ============================================================
// useClarifications
// ============================================================

describe('useClarifications', () => {
  it('fetches clarifications from GET /api/specs/X/clarifications', async () => {
    server.use(
      http.get(`/api/specs/${SPEC_NAME}/clarifications`, () =>
        HttpResponse.json({
          clarifications: [
            {
              topic: 'auth-method',
              question: 'Which authentication method should be used?',
              options: [
                { label: 'JWT', answer: 'jwt', implications: 'Stateless' },
                { label: 'Session', answer: 'session', implications: 'Requires session store' },
              ],
              resolved: null,
            },
          ],
          unresolved_count: 1,
        }),
      ),
    );

    const { result } = renderHook(() => useClarifications(SPEC_NAME), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data!.clarifications).toHaveLength(1);
    expect(result.current.data!.unresolved_count).toBe(1);
    expect(result.current.data!.clarifications[0].topic).toBe('auth-method');
  });

  it('stays disabled when name is null', () => {
    const { result } = renderHook(() => useClarifications(null), {
      wrapper: createWrapper(),
    });

    expect(result.current.isLoading).toBe(false);
    expect(result.current.fetchStatus).toBe('idle');
    expect(result.current.data).toBeUndefined();
  });

  it('transitions to error state when API fails', async () => {
    server.use(
      http.get(`/api/specs/${SPEC_NAME}/clarifications`, () => HttpResponse.error()),
    );

    const { result } = renderHook(() => useClarifications(SPEC_NAME), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isError).toBe(true));
  });
});

// ============================================================
// useSubmitClarifications
// ============================================================

describe('useSubmitClarifications', () => {
  it('calls POST /api/specs/X/clarify with answers payload', async () => {
    let capturedBody: unknown;

    server.use(
      http.post(`/api/specs/${SPEC_NAME}/clarify`, async ({ request }) => {
        capturedBody = await request.json();
        return HttpResponse.json({ success: true });
      }),
    );

    const { result } = renderHook(() => useSubmitClarifications(), {
      wrapper: createWrapper(),
    });

    const answers = [{ topic: 'auth-method', answer: 'jwt' }];

    await act(async () => {
      result.current.mutate({ name: SPEC_NAME, answers });
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(capturedBody).toEqual({ answers });
  });

  it('transitions to error state when submission fails', async () => {
    server.use(
      http.post(`/api/specs/${SPEC_NAME}/clarify`, () => HttpResponse.error()),
    );

    const { result } = renderHook(() => useSubmitClarifications(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      result.current.mutate({ name: SPEC_NAME, answers: [] });
    });

    await waitFor(() => expect(result.current.isError).toBe(true));
  });
});

// ============================================================
// useSpecWorkflow (composite hook)
// ============================================================

describe('useSpecWorkflow', () => {
  it('is in loading state while spec and clarifications resolve', () => {
    server.use(
      http.get('/api/specs/detail', () =>
        HttpResponse.json({ success: true, spec: createSpecDetail() }),
      ),
      http.get(`/api/specs/${SPEC_NAME}/clarifications`, () =>
        HttpResponse.json({ clarifications: [], unresolved_count: 0 }),
      ),
    );

    const { result } = renderHook(() => useSpecWorkflow(SPEC_NAME), {
      wrapper: createWrapper(),
    });

    expect(result.current.isLoading).toBe(true);
  });

  it('exposes spec data after loading completes', async () => {
    server.use(
      http.get('/api/specs/detail', () =>
        HttpResponse.json({ success: true, spec: createSpecDetail() }),
      ),
      http.get(`/api/specs/${SPEC_NAME}/clarifications`, () =>
        HttpResponse.json({ clarifications: [], unresolved_count: 0 }),
      ),
    );

    const { result } = renderHook(() => useSpecWorkflow(SPEC_NAME), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(result.current.spec).toBeDefined();
    expect(result.current.spec!.id).toBe(SPEC_NAME);
  });

  it('exposes clarifications and unresolvedCount from the API', async () => {
    server.use(
      http.get('/api/specs/detail', () =>
        HttpResponse.json({ success: true, spec: createSpecDetail() }),
      ),
      http.get(`/api/specs/${SPEC_NAME}/clarifications`, () =>
        HttpResponse.json({
          clarifications: [
            {
              topic: 'auth-method',
              question: 'Which method?',
              options: [],
              resolved: null,
            },
          ],
          unresolved_count: 1,
        }),
      ),
    );

    const { result } = renderHook(() => useSpecWorkflow(SPEC_NAME), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(result.current.clarifications).toHaveLength(1);
    expect(result.current.unresolvedCount).toBe(1);
  });

  it('returns empty clarifications and zero unresolvedCount when name is null', () => {
    const { result } = renderHook(() => useSpecWorkflow(null), {
      wrapper: createWrapper(),
    });

    expect(result.current.clarifications).toEqual([]);
    expect(result.current.unresolvedCount).toBe(0);
  });

  it('validate() calls the validate endpoint and returns the result', async () => {
    server.use(
      http.get('/api/specs/detail', () =>
        HttpResponse.json({ success: true, spec: createSpecDetail() }),
      ),
      http.get(`/api/specs/${SPEC_NAME}/clarifications`, () =>
        HttpResponse.json({ clarifications: [], unresolved_count: 0 }),
      ),
      http.get(`/api/specs/${SPEC_NAME}/validate`, () =>
        HttpResponse.json({ success: true, valid: true, errors: [], warnings: [] }),
      ),
    );

    const { result } = renderHook(() => useSpecWorkflow(SPEC_NAME), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    let validationResult: unknown;
    await act(async () => {
      validationResult = await result.current.validate();
    });

    expect(validationResult).toMatchObject({ success: true, valid: true });
    expect(result.current.isProcessing).toBe(false);
    expect(result.current.error).toBeNull();
  });

  it('validate() sets error state when the endpoint fails', async () => {
    server.use(
      http.get('/api/specs/detail', () =>
        HttpResponse.json({ success: true, spec: createSpecDetail() }),
      ),
      http.get(`/api/specs/${SPEC_NAME}/clarifications`, () =>
        HttpResponse.json({ clarifications: [], unresolved_count: 0 }),
      ),
      http.get(`/api/specs/${SPEC_NAME}/validate`, () => HttpResponse.error()),
    );

    const { result } = renderHook(() => useSpecWorkflow(SPEC_NAME), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    await act(async () => {
      await result.current.validate();
    });

    expect(result.current.error).not.toBeNull();
    expect(result.current.isProcessing).toBe(false);
  });

  it('validate() returns null when specName is null', async () => {
    const { result } = renderHook(() => useSpecWorkflow(null), {
      wrapper: createWrapper(),
    });

    let validationResult: unknown;
    await act(async () => {
      validationResult = await result.current.validate();
    });

    expect(validationResult).toBeNull();
  });

  it('generateDesignDoc() calls POST /api/specs/X/design', async () => {
    server.use(
      http.get('/api/specs/detail', () =>
        HttpResponse.json({ success: true, spec: createSpecDetail() }),
      ),
      http.get(`/api/specs/${SPEC_NAME}/clarifications`, () =>
        HttpResponse.json({ clarifications: [], unresolved_count: 0 }),
      ),
      http.post(`/api/specs/${SPEC_NAME}/design`, () =>
        HttpResponse.json({ success: true }),
      ),
    );

    const { result } = renderHook(() => useSpecWorkflow(SPEC_NAME), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    let designResult: unknown;
    await act(async () => {
      designResult = await result.current.generateDesignDoc();
    });

    expect(designResult).toMatchObject({ success: true });
    expect(result.current.isProcessing).toBe(false);
  });

  it('generateTasksDoc() calls POST /api/specs/X/tasks', async () => {
    server.use(
      http.get('/api/specs/detail', () =>
        HttpResponse.json({ success: true, spec: createSpecDetail() }),
      ),
      http.get(`/api/specs/${SPEC_NAME}/clarifications`, () =>
        HttpResponse.json({ clarifications: [], unresolved_count: 0 }),
      ),
      http.post(`/api/specs/${SPEC_NAME}/tasks`, () =>
        HttpResponse.json({ success: true }),
      ),
    );

    const { result } = renderHook(() => useSpecWorkflow(SPEC_NAME), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    let tasksResult: unknown;
    await act(async () => {
      tasksResult = await result.current.generateTasksDoc();
    });

    expect(tasksResult).toMatchObject({ success: true });
    expect(result.current.isProcessing).toBe(false);
  });

  it('submitAnswers() calls POST /api/specs/X/clarify with answers', async () => {
    let capturedBody: unknown;

    server.use(
      http.get('/api/specs/detail', () =>
        HttpResponse.json({ success: true, spec: createSpecDetail() }),
      ),
      http.get(`/api/specs/${SPEC_NAME}/clarifications`, () =>
        HttpResponse.json({ clarifications: [], unresolved_count: 0 }),
      ),
      http.post(`/api/specs/${SPEC_NAME}/clarify`, async ({ request }) => {
        capturedBody = await request.json();
        return HttpResponse.json({ success: true });
      }),
    );

    const { result } = renderHook(() => useSpecWorkflow(SPEC_NAME), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    const answers = [{ topic: 'auth-method', answer: 'jwt' }];
    let clarifyResult: unknown;
    await act(async () => {
      clarifyResult = await result.current.submitAnswers(answers);
    });

    expect(clarifyResult).toMatchObject({ success: true });
    expect(capturedBody).toEqual({ answers });
    expect(result.current.isProcessing).toBe(false);
  });

  it('submitAnswers() returns null when specName is null', async () => {
    const { result } = renderHook(() => useSpecWorkflow(null), {
      wrapper: createWrapper(),
    });

    let clarifyResult: unknown;
    await act(async () => {
      clarifyResult = await result.current.submitAnswers([]);
    });

    expect(clarifyResult).toBeNull();
  });
});
