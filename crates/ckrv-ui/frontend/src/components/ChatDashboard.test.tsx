/**
 * @module ChatDashboard.test
 * @description
 * Tests for the ChatDashboard component. Validates spec creation flow,
 * error feedback display, and disabled states.
 *
 * @context
 * Tests the error handling chain: when the backend returns HTTP 500 for
 * failed spec creation, the UI should display the error message from the
 * response body.
 *
 * @dependencies
 * - @/test/test-utils: Custom render with QueryClientProvider
 * - msw: API mocking for /api/status, /api/specs, /api/command/spec-new
 * - fixtures: createSystemStatus factory
 */

import { describe, it, expect } from 'vitest';
import { render, screen, waitFor, userEvent } from '@/test/test-utils';
import { http, HttpResponse } from 'msw';
import { server } from '@/test/mocks/server';
import { NavigationContext } from '../App';
import { ChatDashboard } from './ChatDashboard';

// ============================================================
// HELPERS
// ============================================================

/**
 * Wraps ChatDashboard with the NavigationContext provider needed by the component.
 */
function renderDashboard() {
  return render(
    <NavigationContext.Provider value={{ currentPage: 'dashboard', setCurrentPage: () => {} }}>
      <ChatDashboard />
    </NavigationContext.Provider>
  );
}

// ============================================================
// TESTS
// ============================================================

describe('ChatDashboard', () => {
  it('renders the main heading', async () => {
    // Return no specs so the creation UI shows
    server.use(
      http.get('/api/specs', () => {
        return HttpResponse.json({ specs: [], count: 0 });
      })
    );

    renderDashboard();

    await waitFor(() => {
      expect(screen.getByText('What would you like to build?')).toBeInTheDocument();
    });
  });

  it('shows textarea disabled when project not initialized', async () => {
    server.use(
      http.get('/api/status', () => {
        return HttpResponse.json({ is_ready: false });
      }),
      http.get('/api/specs', () => {
        return HttpResponse.json({ specs: [], count: 0 });
      })
    );

    renderDashboard();

    await waitFor(() => {
      expect(screen.getByText(/project not initialized/i)).toBeInTheDocument();
    });

    const textarea = screen.getByPlaceholderText(/initialize the project first/i);
    expect(textarea).toBeDisabled();
  });

  it('displays error feedback when spec creation fails with HTTP 500', async () => {
    server.use(
      http.get('/api/status', () => {
        return HttpResponse.json({ is_ready: true });
      }),
      http.get('/api/specs', () => {
        return HttpResponse.json({ specs: [], count: 0 });
      }),
      http.post('/api/command/spec-new', () => {
        return HttpResponse.json(
          { error: 'Project not initialized. Run: ckrv init' },
          { status: 500 }
        );
      })
    );

    renderDashboard();

    // Wait for the textarea to be enabled
    const textarea = await waitFor(() => {
      const el = screen.getByPlaceholderText(/describe your feature/i);
      expect(el).not.toBeDisabled();
      return el;
    });

    // Type a description and submit
    const user = userEvent.setup();
    await user.type(textarea, 'Build a REST API');
    await user.keyboard('{Enter}');

    // Error message from the backend should appear
    await waitFor(() => {
      expect(screen.getByText('Project not initialized. Run: ckrv init')).toBeInTheDocument();
    });
  });

  it('displays generic error when backend returns non-JSON error', async () => {
    server.use(
      http.get('/api/status', () => {
        return HttpResponse.json({ is_ready: true });
      }),
      http.get('/api/specs', () => {
        return HttpResponse.json({ specs: [], count: 0 });
      }),
      http.post('/api/command/spec-new', () => {
        return new HttpResponse('Internal Server Error', { status: 500 });
      })
    );

    renderDashboard();

    const textarea = await waitFor(() => {
      const el = screen.getByPlaceholderText(/describe your feature/i);
      expect(el).not.toBeDisabled();
      return el;
    });

    const user = userEvent.setup();
    await user.type(textarea, 'Build a web app');
    await user.keyboard('{Enter}');

    // Should show generic fallback error
    await waitFor(() => {
      expect(screen.getByText(/Spec creation failed \(HTTP 500\)/)).toBeInTheDocument();
    });
  });

  it('shows success feedback when spec creation succeeds', async () => {
    server.use(
      http.get('/api/status', () => {
        return HttpResponse.json({ is_ready: true });
      }),
      http.get('/api/specs', () => {
        return HttpResponse.json({ specs: [], count: 0 });
      }),
      http.post('/api/command/spec-new', () => {
        return HttpResponse.json({
          success: true,
          message: 'Command completed successfully',
          output: 'Spec created at .specs/001-rest-api',
        });
      })
    );

    renderDashboard();

    const textarea = await waitFor(() => {
      const el = screen.getByPlaceholderText(/describe your feature/i);
      expect(el).not.toBeDisabled();
      return el;
    });

    const user = userEvent.setup();
    await user.type(textarea, 'Build a REST API');
    await user.keyboard('{Enter}');

    await waitFor(() => {
      expect(screen.getByText(/specification created/i)).toBeInTheDocument();
    });
  });

  it('shows suggestion chips', async () => {
    server.use(
      http.get('/api/specs', () => {
        return HttpResponse.json({ specs: [], count: 0 });
      })
    );

    renderDashboard();

    await waitFor(() => {
      expect(screen.getByText('REST API')).toBeInTheDocument();
    });

    expect(screen.getByText('CLI Tool')).toBeInTheDocument();
    expect(screen.getByText('Web App')).toBeInTheDocument();
  });

  it('fills textarea when suggestion chip is clicked', async () => {
    server.use(
      http.get('/api/status', () => {
        return HttpResponse.json({ is_ready: true });
      }),
      http.get('/api/specs', () => {
        return HttpResponse.json({ specs: [], count: 0 });
      })
    );

    renderDashboard();

    const chip = await waitFor(() => screen.getByText('REST API'));
    const user = userEvent.setup();
    await user.click(chip);

    const textarea = screen.getByPlaceholderText(/describe your feature/i) as HTMLTextAreaElement;
    expect(textarea.value).toContain('REST API');
  });

  it('shows existing spec actions when specs exist', async () => {
    server.use(
      http.get('/api/status', () => {
        return HttpResponse.json({ is_ready: true });
      }),
      http.get('/api/specs', () => {
        return HttpResponse.json({
          specs: [{ name: '001-auth', path: '.specs/001-auth', has_tasks: false, has_plan: false, has_implementation: false }],
          count: 1,
        });
      })
    );

    renderDashboard();

    await waitFor(() => {
      expect(screen.getByText('Spec Ready')).toBeInTheDocument();
    });

    expect(screen.getByText(/go to code page/i)).toBeInTheDocument();
    expect(screen.getByText(/create new spec/i)).toBeInTheDocument();
  });
});
