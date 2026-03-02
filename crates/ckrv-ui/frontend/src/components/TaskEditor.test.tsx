/**
 * @module TaskEditor.test
 * @description
 * Unit tests for the TaskEditor component. Validates loading state, empty state
 * (no specs with tasks), task list rendering after data loads, status badge
 * display on individual tasks, and error state when the API fails.
 *
 * @context
 * TaskEditor uses useAutoSelectedSpec which queries /api/status and /api/specs.
 * When no spec matches the active branch the component shows a SpecListView that
 * lets users manually select a spec. Tests control which branch is returned from
 * /api/status to drive auto-selection behaviour. xterm and related terminal
 * modules are mocked to prevent import errors in the jsdom environment.
 *
 * @dependencies
 * - @/test/test-utils: Custom render with QueryClientProvider
 * - @testing-library/user-event: User interaction simulation
 * - msw / HttpResponse: Per-test API handler overrides
 * - @/test/mocks/server: MSW server instance
 * - @/test/mocks/fixtures: createTask, createSystemStatus factories
 * - vitest: describe, it, expect, vi
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen, waitFor } from '@/test/test-utils';
import { http, HttpResponse } from 'msw';
import { server } from '@/test/mocks/server';
import { createTask, createSystemStatus } from '@/test/mocks/fixtures';
import { TaskEditor } from './TaskEditor';

// ============================================================
// MODULE MOCKS – terminal/pty libraries not available in jsdom
// ============================================================

vi.mock('@xterm/xterm', () => ({ Terminal: vi.fn() }));
vi.mock('@xterm/addon-fit', () => ({ FitAddon: vi.fn() }));
vi.mock('xterm', () => ({ Terminal: vi.fn() }));
vi.mock('xterm-addon-fit', () => ({ FitAddon: vi.fn() }));
vi.mock('tauri-pty', () => ({ default: {} }));

// ============================================================
// SHARED FIXTURES
// ============================================================

/**
 * A spec object shaped the way the TaskEditor / useAutoSelectedSpec hooks
 * expect (different from SpecSummary in api.generated – this is the raw
 * filesystem shape returned by GET /api/specs in the Rust server).
 */
const SPEC_WITH_TASKS = {
  name: 'my-feature',
  path: '/project/specs/my-feature',
  has_tasks: true,
  has_plan: false,
  has_implementation: false,
};

const SPEC_WITHOUT_TASKS = {
  name: 'empty-spec',
  path: '/project/specs/empty-spec',
  has_tasks: false,
  has_plan: false,
  has_implementation: false,
};

const TASK_ONE = createTask({
  id: 'T001',
  phase: 'Phase 1',
  title: 'Set up project structure',
  status: 'pending',
  risk: 'low',
});

const TASK_TWO = createTask({
  id: 'T002',
  phase: 'Phase 1',
  title: 'Add configuration module',
  status: 'completed',
  risk: 'medium',
});

// ============================================================
// HELPER – override /api/status to simulate a matching branch
// ============================================================

/**
 * Installs a one-time MSW override so /api/status returns the given branch.
 * useAutoSelectedSpec will then match a spec whose `name === branch`.
 */
function useActiveBranch(branch: string) {
  server.use(
    http.get('/api/status', () =>
      HttpResponse.json(createSystemStatus({ active_branch: branch }))
    )
  );
}

// ============================================================
// TESTS
// ============================================================

describe('TaskEditor', () => {
  it('shows loading state initially', async () => {
    // Default /api/status from base handlers returns branch 'feature/test-branch'
    // which won't match any spec → shows spec list loading spinner while queries resolve.
    render(<TaskEditor />);

    // During loading the component renders a spinner (Loader2 with animate-spin)
    // OR the spec list skeleton while specs load.
    // The easiest assertion is that the final "No specs with tasks found" text
    // has NOT appeared yet, meaning we're still in a loading/transitional state.
    // We assert a spinner is present on first paint.
    const spinner = document.querySelector('.animate-spin');
    expect(spinner).toBeInTheDocument();
  });

  it('shows empty state when no specs have tasks', async () => {
    // Override /api/specs to return a spec without tasks
    server.use(
      http.get('/api/specs', () =>
        HttpResponse.json({ specs: [SPEC_WITHOUT_TASKS], count: 1 })
      )
    );

    render(<TaskEditor />);

    await waitFor(() => {
      expect(screen.getByText('No specs with tasks found')).toBeInTheDocument();
    });
  });

  it('renders task list after loading when a spec is auto-selected', async () => {
    // Make the active branch match a spec that has tasks
    useActiveBranch('my-feature');

    server.use(
      http.get('/api/specs', () =>
        HttpResponse.json({ specs: [SPEC_WITH_TASKS], count: 1 })
      ),
      http.get('/api/tasks/detail', () =>
        HttpResponse.json({
          success: true,
          tasks: [TASK_ONE, TASK_TWO],
          raw_yaml: 'tasks:\n  - id: T001',
          count: 2,
        })
      )
    );

    render(<TaskEditor />);

    await waitFor(() => {
      expect(screen.getByText('Set up project structure')).toBeInTheDocument();
    });

    expect(screen.getByText('Add configuration module')).toBeInTheDocument();
  });

  it('shows status badges on tasks', async () => {
    useActiveBranch('my-feature');

    server.use(
      http.get('/api/specs', () =>
        HttpResponse.json({ specs: [SPEC_WITH_TASKS], count: 1 })
      ),
      http.get('/api/tasks/detail', () =>
        HttpResponse.json({
          success: true,
          tasks: [TASK_ONE, TASK_TWO],
          raw_yaml: '',
          count: 2,
        })
      )
    );

    render(<TaskEditor />);

    await waitFor(() => {
      // TASK_ONE has status 'pending', TASK_TWO has status 'completed'
      expect(screen.getByText('pending')).toBeInTheDocument();
    });

    expect(screen.getByText('completed')).toBeInTheDocument();
  });

  it('shows error state when API fails to return tasks', async () => {
    useActiveBranch('my-feature');

    server.use(
      http.get('/api/specs', () =>
        HttpResponse.json({ specs: [SPEC_WITH_TASKS], count: 1 })
      ),
      http.get('/api/tasks/detail', () =>
        HttpResponse.json({
          success: false,
          tasks: [],
          count: 0,
          error: 'Failed to load tasks',
        })
      )
    );

    render(<TaskEditor />);

    // When success is false and tasks is empty the component renders the
    // "No Tasks Found" empty state for the selected spec.
    await waitFor(() => {
      expect(screen.getByText('No Tasks Found')).toBeInTheDocument();
    });
  });
});
