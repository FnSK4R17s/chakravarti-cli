/**
 * @module AgentManager.test
 * @description
 * Comprehensive unit tests for the AgentManager component, covering initial
 * loading state, agent list rendering, empty/error states, badge display,
 * and user interactions like add, delete, and set-default flows.
 *
 * @context
 * AgentManager is the main configuration surface for AI agents in the
 * dashboard. These tests verify the full lifecycle: data fetching via
 * TanStack Query, MSW-intercepted API calls, and user interaction flows.
 *
 * @dependencies
 * - @/test/test-utils: Custom render with QueryClientProvider wrapper
 * - @testing-library/user-event: Simulates user interactions
 * - msw: Intercepts HTTP requests for isolated API mocking
 * - @/test/mocks/server: MSW Node server instance
 * - @/test/mocks/fixtures: createAgent factory (used for type reference)
 */

// ============================================================
// TERMINAL MOCKS
// ============================================================
// jsdom cannot render xterm.js canvas-based terminals, so we mock all
// terminal-related modules before any imports that might trigger them.

vi.mock('@xterm/xterm', () => ({
  Terminal: vi.fn().mockImplementation(() => ({
    open: vi.fn(),
    write: vi.fn(),
    dispose: vi.fn(),
    onData: vi.fn(),
    onResize: vi.fn(),
    loadAddon: vi.fn(),
  })),
}));

vi.mock('@xterm/addon-fit', () => ({
  FitAddon: vi.fn().mockImplementation(() => ({
    fit: vi.fn(),
    dispose: vi.fn(),
  })),
}));

vi.mock('xterm', () => ({
  Terminal: vi.fn().mockImplementation(() => ({
    open: vi.fn(),
    write: vi.fn(),
    dispose: vi.fn(),
    onData: vi.fn(),
    onResize: vi.fn(),
    loadAddon: vi.fn(),
  })),
}));

vi.mock('xterm-addon-fit', () => ({
  FitAddon: vi.fn().mockImplementation(() => ({
    fit: vi.fn(),
    dispose: vi.fn(),
  })),
}));

// tauri-pty is already aliased in vitest.config.ts to a no-op stub,
// so no vi.mock needed here.

// ============================================================
// IMPORTS
// ============================================================
import { describe, it, expect, vi } from 'vitest';
import { render, screen, waitFor } from '@/test/test-utils';
import userEvent from '@testing-library/user-event';
import { http, HttpResponse } from 'msw';
import { server } from '@/test/mocks/server';
import AgentManager from './AgentManager';

// ============================================================
// HELPERS
// ============================================================

/**
 * Builds a mock AgentConfig using the LOCAL interface shape that AgentManager
 * expects (not the generated type from api.generated.ts). The base MSW handler
 * uses the generated type via createAgent(), which omits `id` and `level`.
 * These overrides produce data the component can actually render.
 */
function buildLocalAgent(overrides?: Record<string, unknown>) {
  return {
    id: 'claude-default',
    name: 'Claude Code',
    agent_type: 'claude' as const,
    level: 5,
    is_default: true,
    enabled: true,
    description: 'Primary Claude Code agent',
    ...overrides,
  };
}

/** Returns a standard two-agent payload matching the component's AgentConfig interface. */
function twoAgentsPayload() {
  return {
    agents: [
      buildLocalAgent(),
      buildLocalAgent({
        id: 'codex-agent',
        name: 'OpenAI Codex',
        agent_type: 'codex',
        level: 3,
        is_default: false,
        description: 'OpenAI Codex agent',
      }),
    ],
  };
}

// ============================================================
// TESTS
// ============================================================

describe('AgentManager', () => {
  // ----------------------------------------------------------
  // 1. Loading State
  // ----------------------------------------------------------
  describe('loading state', () => {
    it('renders a loading spinner while agents are being fetched', () => {
      // Override handler to never resolve so we stay in loading state
      server.use(
        http.get('/api/agents', () => new Promise(() => {}))
      );

      render(<AgentManager />);

      // Loader2 renders with animate-spin class
      expect(document.querySelector('.animate-spin')).toBeInTheDocument();
    });
  });

  // ----------------------------------------------------------
  // 2. Agent List - Happy Path
  // ----------------------------------------------------------
  describe('agent list', () => {
    it('displays agent names after data loads', async () => {
      server.use(
        http.get('/api/agents', () => HttpResponse.json(twoAgentsPayload()))
      );

      render(<AgentManager />);

      // Agent names may appear multiple times (name + type label), use getAllByText
      await waitFor(() => {
        expect(screen.getAllByText('Claude Code').length).toBeGreaterThanOrEqual(1);
      });
      expect(screen.getAllByText('OpenAI Codex').length).toBeGreaterThanOrEqual(1);
    });

    it('renders the Agent Manager heading', async () => {
      server.use(
        http.get('/api/agents', () => HttpResponse.json(twoAgentsPayload()))
      );

      render(<AgentManager />);

      expect(await screen.findByText('Agent Manager')).toBeInTheDocument();
    });

    it('renders the Add Agent button', async () => {
      server.use(
        http.get('/api/agents', () => HttpResponse.json(twoAgentsPayload()))
      );

      render(<AgentManager />);

      expect(await screen.findByRole('button', { name: /add agent/i })).toBeInTheDocument();
    });
  });

  // ----------------------------------------------------------
  // 3. Empty State
  // ----------------------------------------------------------
  describe('empty state', () => {
    it('shows empty-state message when no agents are configured', async () => {
      server.use(
        http.get('/api/agents', () => HttpResponse.json({ agents: [] }))
      );

      render(<AgentManager />);

      expect(await screen.findByText('No agents configured')).toBeInTheDocument();
      expect(screen.getByText(/click "add agent" to get started/i)).toBeInTheDocument();
    });
  });

  // ----------------------------------------------------------
  // 4. Error State
  // ----------------------------------------------------------
  describe('error state', () => {
    it('shows no agent list content when API returns an error', async () => {
      server.use(
        http.get('/api/agents', () => HttpResponse.error())
      );

      render(<AgentManager />);

      // Component stays blank (no spinner, no list) – the query fails silently.
      // We wait until the loading spinner is gone and confirm no agent cards appeared.
      await waitFor(() => {
        expect(document.querySelector('.animate-spin')).not.toBeInTheDocument();
      });

      expect(screen.queryByText('Claude Code')).not.toBeInTheDocument();
    });
  });

  // ----------------------------------------------------------
  // 5. Agent Type Badges
  // ----------------------------------------------------------
  describe('agent type badges', () => {
    it('displays the Claude Code type label for a claude agent', async () => {
      server.use(
        http.get('/api/agents', () =>
          HttpResponse.json({
            agents: [buildLocalAgent({ agent_type: 'claude', name: 'My Claude Agent' })],
          })
        )
      );

      render(<AgentManager />);

      // Type label rendered in subtitle text
      expect(await screen.findByText('Claude Code')).toBeInTheDocument();
    });

    it('displays the OpenAI Codex type label for a codex agent', async () => {
      server.use(
        http.get('/api/agents', () =>
          HttpResponse.json({
            agents: [
              buildLocalAgent({
                id: 'codex-1',
                name: 'My Codex',
                agent_type: 'codex',
                is_default: false,
              }),
            ],
          })
        )
      );

      render(<AgentManager />);

      expect(await screen.findByText('OpenAI Codex')).toBeInTheDocument();
    });

    it('shows level badge with correct level number', async () => {
      server.use(
        http.get('/api/agents', () =>
          HttpResponse.json({
            agents: [buildLocalAgent({ level: 4, name: 'High Level Agent' })],
          })
        )
      );

      render(<AgentManager />);

      await screen.findByText('High Level Agent');
      // Level badge renders as L{level}
      expect(screen.getByText('L4')).toBeInTheDocument();
    });
  });

  // ----------------------------------------------------------
  // 6. Default Badge
  // ----------------------------------------------------------
  describe('default agent badge', () => {
    it('shows DEFAULT badge on the default agent', async () => {
      server.use(
        http.get('/api/agents', () => HttpResponse.json(twoAgentsPayload()))
      );

      render(<AgentManager />);

      // Wait for agents to load (may appear multiple times as name + type label)
      await waitFor(() => {
        expect(screen.getAllByText('Claude Code').length).toBeGreaterThanOrEqual(1);
      });
      // DEFAULT badge only on the default agent
      expect(screen.getByText('DEFAULT')).toBeInTheDocument();
    });

    it('does not show DEFAULT badge on non-default agents', async () => {
      server.use(
        http.get('/api/agents', () =>
          HttpResponse.json({
            agents: [buildLocalAgent({ is_default: false, name: 'Non-Default Agent' })],
          })
        )
      );

      render(<AgentManager />);

      await screen.findByText('Non-Default Agent');
      expect(screen.queryByText('DEFAULT')).not.toBeInTheDocument();
    });

    it('shows QA badge on agent with is_qa_agent flag', async () => {
      server.use(
        http.get('/api/agents', () =>
          HttpResponse.json({
            agents: [buildLocalAgent({ is_qa_agent: true, name: 'QA Agent' })],
          })
        )
      );

      render(<AgentManager />);

      await screen.findByText('QA Agent');
      expect(screen.getByText('QA')).toBeInTheDocument();
    });

    it('shows TESTS badge on agent with is_test_writer flag', async () => {
      server.use(
        http.get('/api/agents', () =>
          HttpResponse.json({
            agents: [buildLocalAgent({ is_test_writer: true, name: 'Test Writer Agent' })],
          })
        )
      );

      render(<AgentManager />);

      await screen.findByText('Test Writer Agent');
      expect(screen.getByText('TESTS')).toBeInTheDocument();
    });
  });

  // ----------------------------------------------------------
  // 7. Add Agent Dialog
  // ----------------------------------------------------------
  describe('add agent dialog', () => {
    it('opens the Add New Agent dialog when clicking Add Agent button', async () => {
      server.use(
        http.get('/api/agents', () => HttpResponse.json({ agents: [] }))
      );
      // Also stub model endpoints used inside the modal query
      server.use(
        http.get('/api/agents/models', () => HttpResponse.json({ models: [] })),
        http.get('/api/agents/kilo-models', () => HttpResponse.json({ models: [] })),
        http.get('/api/agents/glm-models', () => HttpResponse.json({ models: [] }))
      );

      const user = userEvent.setup();
      render(<AgentManager />);

      // Wait for the page to finish loading (empty state shows)
      const addButton = await screen.findByRole('button', { name: /add agent/i });
      await user.click(addButton);

      // Dialog title appears
      expect(await screen.findByText('Add New Agent')).toBeInTheDocument();
    });

    it('shows name input field in the add agent form', async () => {
      server.use(
        http.get('/api/agents', () => HttpResponse.json({ agents: [] })),
        http.get('/api/agents/models', () => HttpResponse.json({ models: [] })),
        http.get('/api/agents/kilo-models', () => HttpResponse.json({ models: [] })),
        http.get('/api/agents/glm-models', () => HttpResponse.json({ models: [] }))
      );

      const user = userEvent.setup();
      render(<AgentManager />);

      const addButton = await screen.findByRole('button', { name: /add agent/i });
      await user.click(addButton);

      expect(await screen.findByLabelText('Name')).toBeInTheDocument();
    });
  });

  // ----------------------------------------------------------
  // 8. Delete Agent Flow
  // ----------------------------------------------------------
  describe('delete agent flow', () => {
    it('delete button is disabled for the default agent', async () => {
      server.use(
        http.get('/api/agents', () =>
          HttpResponse.json({
            agents: [buildLocalAgent({ is_default: true, name: 'Default Agent' })],
          })
        )
      );

      render(<AgentManager />);

      await screen.findByText('Default Agent');

      // The delete button title is "Cannot delete default agent" when is_default=true
      const deleteButton = screen.getByTitle('Cannot delete default agent');
      expect(deleteButton).toBeDisabled();
    });

    it('delete button is enabled for non-default agents', async () => {
      server.use(
        http.get('/api/agents', () =>
          HttpResponse.json({
            agents: [
              buildLocalAgent({
                id: 'non-default',
                name: 'Non-Default Agent',
                is_default: false,
              }),
            ],
          })
        )
      );

      render(<AgentManager />);

      await screen.findByText('Non-Default Agent');

      const deleteButton = screen.getByTitle('Delete agent');
      expect(deleteButton).not.toBeDisabled();
    });

    it('calls the delete API endpoint when delete button is clicked', async () => {
      let deleteCalled = false;

      server.use(
        http.get('/api/agents', () =>
          HttpResponse.json({
            agents: [
              buildLocalAgent({
                id: 'non-default',
                name: 'Deletable Agent',
                is_default: false,
              }),
            ],
          })
        ),
        http.post('/api/agents/delete', () => {
          deleteCalled = true;
          return HttpResponse.json({ success: true });
        })
      );

      const user = userEvent.setup();
      render(<AgentManager />);

      await screen.findByText('Deletable Agent');

      const deleteButton = screen.getByTitle('Delete agent');
      await user.click(deleteButton);

      await waitFor(() => {
        expect(deleteCalled).toBe(true);
      });
    });
  });

  // ----------------------------------------------------------
  // 9. Set Default Agent Flow
  // ----------------------------------------------------------
  describe('set default agent flow', () => {
    it('calls the set-default API endpoint when star button is clicked on a non-default agent', async () => {
      let setDefaultCalled = false;

      server.use(
        http.get('/api/agents', () =>
          HttpResponse.json({
            agents: [
              buildLocalAgent({
                id: 'not-default',
                name: 'Promote Me',
                is_default: false,
              }),
            ],
          })
        ),
        http.post('/api/agents/set-default', () => {
          setDefaultCalled = true;
          return HttpResponse.json({ success: true });
        })
      );

      const user = userEvent.setup();
      render(<AgentManager />);

      await screen.findByText('Promote Me');

      // Non-default agent shows "Set as default" title on the star button
      const starButton = screen.getByTitle('Set as default');
      await user.click(starButton);

      await waitFor(() => {
        expect(setDefaultCalled).toBe(true);
      });
    });

    it('shows the filled star button on the default agent', async () => {
      server.use(
        http.get('/api/agents', () =>
          HttpResponse.json({
            agents: [buildLocalAgent({ is_default: true, name: 'Already Default' })],
          })
        )
      );

      render(<AgentManager />);

      await screen.findByText('Already Default');

      // Default agent has title "Default agent" on the star button
      expect(screen.getByTitle('Default agent')).toBeInTheDocument();
    });
  });
});
