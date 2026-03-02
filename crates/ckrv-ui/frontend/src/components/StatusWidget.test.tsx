/**
 * @module StatusWidget.test
 * @description
 * Unit tests for the StatusWidget component. Validates status display,
 * loading/error states, git init flow, and mode badge rendering.
 *
 * @context
 * Smoke test verifying the test infrastructure works end-to-end:
 * Vitest + React Testing Library + MSW.
 *
 * @dependencies
 * - @/test/test-utils: Custom render with QueryClientProvider
 * - msw: API mocking for /api/status
 * - fixtures: createSystemStatus factory
 */

import { describe, it, expect } from 'vitest';
import { render, screen, waitFor } from '@/test/test-utils';
import { http, HttpResponse } from 'msw';
import { server } from '@/test/mocks/server';
import { createSystemStatus } from '@/test/mocks/fixtures';
import { StatusWidget } from './StatusWidget';

// ============================================================
// TESTS
// ============================================================

describe('StatusWidget', () => {
  it('renders loading skeleton initially', () => {
    render(<StatusWidget />);
    // Skeletons are rendered during loading - the card structure is present
    expect(document.querySelector('.animate-pulse')).toBeInTheDocument();
  });

  it('displays repository status when data loads', async () => {
    render(<StatusWidget />);

    await waitFor(() => {
      expect(screen.getByText('Repository Status')).toBeInTheDocument();
    });

    expect(screen.getByText('feature/test-branch')).toBeInTheDocument();
    expect(screen.getByText('Ready')).toBeInTheDocument();
  });

  it('shows branch name and initialized status for a git repo', async () => {
    render(<StatusWidget />);

    await waitFor(() => {
      expect(screen.getByText('Branch')).toBeInTheDocument();
    });

    expect(screen.getByText('feature/test-branch')).toBeInTheDocument();
    expect(screen.getByText('Initialized')).toBeInTheDocument();
    expect(screen.getByText('Yes')).toBeInTheDocument();
  });

  it('shows feature number when present', async () => {
    render(<StatusWidget />);

    await waitFor(() => {
      expect(screen.getByText('Spec')).toBeInTheDocument();
    });

    expect(screen.getByText('042')).toBeInTheDocument();
  });

  it('shows git init button when repo is not initialized', async () => {
    server.use(
      http.get('/api/status', () => {
        return HttpResponse.json(
          createSystemStatus({ active_branch: 'none', is_ready: false })
        );
      })
    );

    render(<StatusWidget />);

    await waitFor(() => {
      expect(screen.getByText('Not initialized')).toBeInTheDocument();
    });

    expect(screen.getByRole('button', { name: /initialize git repository/i })).toBeInTheDocument();
  });

  it('displays mode badge correctly for running state', async () => {
    server.use(
      http.get('/api/status', () => {
        return HttpResponse.json(createSystemStatus({ mode: 'running' }));
      })
    );

    render(<StatusWidget />);

    await waitFor(() => {
      expect(screen.getByText('Running')).toBeInTheDocument();
    });
  });

  it('shows error state when API fails', async () => {
    server.use(
      http.get('/api/status', () => {
        return HttpResponse.error();
      })
    );

    render(<StatusWidget />);

    await waitFor(() => {
      expect(screen.getByText('Connection Error')).toBeInTheDocument();
    });
  });

  it('shows project path when available', async () => {
    render(<StatusWidget />);

    await waitFor(() => {
      expect(screen.getByText('Project')).toBeInTheDocument();
    });

    expect(screen.getByText('/home/user/project')).toBeInTheDocument();
  });

  it('shows hint when not ready', async () => {
    server.use(
      http.get('/api/status', () => {
        return HttpResponse.json(
          createSystemStatus({ is_ready: false })
        );
      })
    );

    render(<StatusWidget />);

    await waitFor(() => {
      expect(screen.getByText('No')).toBeInTheDocument();
    });

    expect(screen.getByText('Run: ckrv init')).toBeInTheDocument();
  });
});
