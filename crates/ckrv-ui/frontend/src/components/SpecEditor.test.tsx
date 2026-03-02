/**
 * @module SpecEditor.test
 * @description
 * Unit tests for the SpecEditor component. Validates loading state,
 * empty state, spec list rendering, spec detail view, view toggles,
 * clarification badge, and error state.
 *
 * @context
 * SpecEditor is the main content panel for the Spec page. It auto-selects
 * a spec based on the git branch, or shows a selectable list as fallback.
 * These tests use MSW to intercept /api/status, /api/specs, /api/specs/detail,
 * and /api/specs/{name}/clarifications.
 *
 * @dependencies
 * - @/test/test-utils: Custom render with QueryClientProvider
 * - msw: API mocking for spec endpoints
 * - fixtures: createSpecDetail factory and inline spec list overrides
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@/test/test-utils';
import { http, HttpResponse } from 'msw';
import { server } from '@/test/mocks/server';
import { createSpecDetail } from '@/test/mocks/fixtures';
import { SpecEditor } from './SpecEditor';

// ============================================================
// HELPERS
// ============================================================

/**
 * A minimal spec list item that matches what SpecEditor's SpecListItem type expects.
 * The base handler uses SpecSummary from the generated types, which lacks has_design,
 * so we override /api/specs in every test that cares about spec-detail flow.
 */
function makeSpecListItem(overrides?: Record<string, unknown>) {
  return {
    name: '042-add-auth',
    path: 'specs/042-add-auth',
    has_tasks: false,
    has_plan: false,
    has_design: false,
    has_implementation: false,
    implementation_branch: null,
    ...overrides,
  };
}

/**
 * Override /api/status so that the branch matches a spec name, causing
 * useAutoSelectedSpec to auto-select that spec and SpecEditor to show detail view.
 */
function useAutoSelectSpec(specName: string) {
  server.use(
    http.get('/api/status', () =>
      HttpResponse.json({
        active_branch: specName,
        feature_number: '042',
        is_ready: true,
        mode: 'idle',
        project_root: '/home/user/project',
      })
    ),
    http.get('/api/specs', () =>
      HttpResponse.json({
        specs: [makeSpecListItem({ name: specName })],
        count: 1,
      })
    )
  );
}

/**
 * Override /api/status to return a branch that does NOT match any spec, so
 * SpecEditor falls through to the manual spec list view.
 */
function useNoAutoSelect() {
  server.use(
    http.get('/api/status', () =>
      HttpResponse.json({
        active_branch: 'main',
        feature_number: null,
        is_ready: true,
        mode: 'idle',
        project_root: '/home/user/project',
      })
    )
  );
}

// ============================================================
// TESTS
// ============================================================

describe('SpecEditor', () => {
  // ----------------------------------------------------------
  // 1. Loading / skeleton state
  // ----------------------------------------------------------
  describe('loading state', () => {
    it('shows a spinner while auto-spec selection is loading', () => {
      // Status and specs queries are pending — component should show spinner
      // We slow down /api/status so the loading branch is exercised
      server.use(
        http.get('/api/status', async () => {
          await new Promise(() => {}); // never resolves in this render
        })
      );

      render(<SpecEditor />);

      // The Loader2 spinner is rendered with animate-spin class
      expect(document.querySelector('.animate-spin')).toBeInTheDocument();
    });

    it('shows a spinner while spec detail is loading after auto-select', async () => {
      const specName = '042-add-auth';
      useAutoSelectSpec(specName);

      // Delay the detail response so we see the loading state
      server.use(
        http.get('/api/specs/detail', async () => {
          await new Promise(() => {}); // never resolves in this render
        }),
        http.get(`/api/specs/${specName}/clarifications`, () =>
          HttpResponse.json({ clarifications: [], unresolved_count: 0 })
        )
      );

      render(<SpecEditor />);

      // After status+specs resolve but before detail resolves, spinner shows
      await waitFor(() => {
        expect(document.querySelector('.animate-spin')).toBeInTheDocument();
      });
    });
  });

  // ----------------------------------------------------------
  // 2. Empty state — no specs exist
  // ----------------------------------------------------------
  describe('empty state', () => {
    it('shows "No specifications found" when spec list is empty', async () => {
      useNoAutoSelect();
      server.use(
        http.get('/api/specs', () =>
          HttpResponse.json({ specs: [], count: 0 })
        )
      );

      render(<SpecEditor />);

      await waitFor(() => {
        expect(screen.getByText('No specifications found')).toBeInTheDocument();
      });

      expect(screen.getByText(/ckrv spec new/)).toBeInTheDocument();
    });
  });

  // ----------------------------------------------------------
  // 3. Spec list renders after loading
  // ----------------------------------------------------------
  describe('spec list view', () => {
    it('renders spec names when no spec is auto-selected', async () => {
      useNoAutoSelect();
      server.use(
        http.get('/api/specs', () =>
          HttpResponse.json({
            specs: [
              makeSpecListItem({ name: '042-add-auth' }),
              makeSpecListItem({ name: '043-dashboard', has_tasks: true, has_plan: true }),
            ],
            count: 2,
          })
        )
      );

      render(<SpecEditor />);

      await waitFor(() => {
        expect(screen.getByText('042-add-auth')).toBeInTheDocument();
      });

      expect(screen.getByText('043-dashboard')).toBeInTheDocument();
    });

    it('shows "has tasks" badge on specs that have tasks', async () => {
      useNoAutoSelect();
      server.use(
        http.get('/api/specs', () =>
          HttpResponse.json({
            specs: [makeSpecListItem({ name: '042-add-auth', has_tasks: true })],
            count: 1,
          })
        )
      );

      render(<SpecEditor />);

      await waitFor(() => {
        expect(screen.getByText('has tasks')).toBeInTheDocument();
      });
    });

    it('shows "Specifications" heading in list view', async () => {
      useNoAutoSelect();
      server.use(
        http.get('/api/specs', () =>
          HttpResponse.json({ specs: [], count: 0 })
        )
      );

      render(<SpecEditor />);

      await waitFor(() => {
        expect(screen.getByText('Specifications')).toBeInTheDocument();
      });
    });
  });

  // ----------------------------------------------------------
  // 4. Spec detail view when a spec is auto-selected
  // ----------------------------------------------------------
  describe('spec detail view', () => {
    const SPEC_NAME = '042-add-auth';

    beforeEach(() => {
      useAutoSelectSpec(SPEC_NAME);

      server.use(
        http.get('/api/specs/detail', () =>
          HttpResponse.json({
            success: true,
            spec: createSpecDetail({
              id: SPEC_NAME,
              overview: 'Implement login/signup with JWT tokens',
              status: 'draft',
              user_stories: [
                {
                  id: 'US-1',
                  title: 'User can log in',
                  priority: 'P1',
                  description: 'As a user, I want to log in',
                },
              ],
              requirements: {
                functional: [
                  { id: 'REQ-1', description: 'Support email/password login' },
                ],
              },
              success_criteria: [
                { id: 'SC-1', metric: 'Users authenticate within 2 seconds' },
              ],
            }),
            raw_yaml: 'id: 042-add-auth\noverview: Implement login/signup',
          })
        ),
        http.get(`/api/specs/${SPEC_NAME}/clarifications`, () =>
          HttpResponse.json({ clarifications: [], unresolved_count: 0 })
        )
      );
    });

    it('displays the spec id badge in the header', async () => {
      render(<SpecEditor />);

      await waitFor(() => {
        expect(screen.getByText(SPEC_NAME)).toBeInTheDocument();
      });
    });

    it('displays spec overview text in the visual view', async () => {
      render(<SpecEditor />);

      await waitFor(() => {
        expect(
          screen.getByText('Implement login/signup with JWT tokens')
        ).toBeInTheDocument();
      });
    });

    it('renders the Overview section heading', async () => {
      render(<SpecEditor />);

      await waitFor(() => {
        expect(screen.getByText('Overview')).toBeInTheDocument();
      });
    });

    it('renders User Stories section with count badge', async () => {
      render(<SpecEditor />);

      await waitFor(() => {
        expect(screen.getByText('User Stories')).toBeInTheDocument();
      });

      // Count badges show the number of items — use getAllByText since multiple
      // sections may have a count of 1 (stories, requirements, criteria)
      const countBadges = screen.getAllByText('1');
      expect(countBadges.length).toBeGreaterThanOrEqual(1);
    });

    it('renders Functional Requirements section', async () => {
      render(<SpecEditor />);

      await waitFor(() => {
        expect(screen.getByText('Functional Requirements')).toBeInTheDocument();
      });
    });

    it('renders Success Criteria section', async () => {
      render(<SpecEditor />);

      await waitFor(() => {
        expect(screen.getByText('Success Criteria')).toBeInTheDocument();
      });
    });

    it('displays the status bar with story/requirement counts', async () => {
      render(<SpecEditor />);

      await waitFor(() => {
        expect(screen.getByText('1 stories')).toBeInTheDocument();
      });

      expect(screen.getByText('1 requirements')).toBeInTheDocument();
      expect(screen.getByText('1 success criteria')).toBeInTheDocument();
    });

    it('shows "Read-only view" in the status bar', async () => {
      render(<SpecEditor />);

      await waitFor(() => {
        expect(screen.getByText('Read-only view')).toBeInTheDocument();
      });
    });
  });

  // ----------------------------------------------------------
  // 5. View toggle tabs (visual / outline / code)
  // ----------------------------------------------------------
  describe('view toggle tabs', () => {
    const SPEC_NAME = '042-add-auth';

    beforeEach(() => {
      useAutoSelectSpec(SPEC_NAME);

      server.use(
        http.get('/api/specs/detail', () =>
          HttpResponse.json({
            success: true,
            spec: createSpecDetail({ id: SPEC_NAME }),
            raw_yaml: 'id: 042-add-auth\noverview: Implement login/signup',
          })
        ),
        http.get(`/api/specs/${SPEC_NAME}/clarifications`, () =>
          HttpResponse.json({ clarifications: [], unresolved_count: 0 })
        )
      );
    });

    it('shows Visual, Outline, and YAML tab triggers', async () => {
      render(<SpecEditor />);

      await waitFor(() => {
        expect(screen.getByRole('tab', { name: /visual/i })).toBeInTheDocument();
      });

      expect(screen.getByRole('tab', { name: /outline/i })).toBeInTheDocument();
      expect(screen.getByRole('tab', { name: /yaml/i })).toBeInTheDocument();
    });

    it('defaults to the Visual tab', async () => {
      render(<SpecEditor />);

      await waitFor(() => {
        const visualTab = screen.getByRole('tab', { name: /visual/i });
        expect(visualTab).toHaveAttribute('data-state', 'active');
      });
    });

    it('switches to outline view when the Outline tab is clicked', async () => {
      const { userEvent: user } = await import('@testing-library/user-event');
      const ue = user.setup();

      render(<SpecEditor />);

      const outlineTab = await screen.findByRole('tab', { name: /outline/i });
      await ue.click(outlineTab);

      // The outline view renders 'spec:' in font-mono
      await waitFor(() => {
        expect(screen.getByText('spec:')).toBeInTheDocument();
      });
    });

    it('switches to YAML view when the YAML tab is clicked', async () => {
      const { userEvent: user } = await import('@testing-library/user-event');
      const ue = user.setup();

      render(<SpecEditor />);

      const yamlTab = await screen.findByRole('tab', { name: /yaml/i });
      await ue.click(yamlTab);

      // The YAML view renders the raw_yaml in a <pre><code> block
      await waitFor(() => {
        expect(screen.getByText(/id: 042-add-auth/)).toBeInTheDocument();
      });
    });
  });

  // ----------------------------------------------------------
  // 6. Clarification badge when unresolved clarifications exist
  // ----------------------------------------------------------
  describe('clarification badge', () => {
    const SPEC_NAME = '042-add-auth';

    beforeEach(() => {
      useAutoSelectSpec(SPEC_NAME);

      server.use(
        http.get('/api/specs/detail', () =>
          HttpResponse.json({
            success: true,
            spec: createSpecDetail({ id: SPEC_NAME }),
            raw_yaml: '',
          })
        )
      );
    });

    it('shows clarification alert when unresolved clarifications exist', async () => {
      server.use(
        http.get(`/api/specs/${SPEC_NAME}/clarifications`, () =>
          HttpResponse.json({
            clarifications: [
              {
                topic: 'auth-method',
                question: 'Which auth method?',
                options: [{ label: 'JWT', answer: 'jwt' }],
                resolved: null,
              },
            ],
            unresolved_count: 1,
          })
        )
      );

      render(<SpecEditor />);

      await waitFor(() => {
        expect(
          screen.getByText(/1 clarification needed/)
        ).toBeInTheDocument();
      });

      expect(screen.getByRole('button', { name: /resolve now/i })).toBeInTheDocument();
    });

    it('shows plural form for multiple unresolved clarifications', async () => {
      server.use(
        http.get(`/api/specs/${SPEC_NAME}/clarifications`, () =>
          HttpResponse.json({
            clarifications: [
              {
                topic: 'auth-method',
                question: 'Which auth method?',
                options: [],
                resolved: null,
              },
              {
                topic: 'session-length',
                question: 'How long should sessions last?',
                options: [],
                resolved: null,
              },
            ],
            unresolved_count: 2,
          })
        )
      );

      render(<SpecEditor />);

      await waitFor(() => {
        expect(
          screen.getByText(/2 clarifications needed/)
        ).toBeInTheDocument();
      });
    });

    it('does NOT show the clarification alert banner when all are resolved', async () => {
      server.use(
        http.get(`/api/specs/${SPEC_NAME}/clarifications`, () =>
          HttpResponse.json({
            clarifications: [
              {
                topic: 'auth-method',
                question: 'Which auth method?',
                options: [],
                resolved: 'jwt',
              },
            ],
            unresolved_count: 0,
          })
        )
      );

      render(<SpecEditor />);

      // Wait for spec detail to load
      await waitFor(() => {
        expect(screen.getByText('Overview')).toBeInTheDocument();
      });

      // The clarification alert banner says "X clarification(s) needed" where X is
      // a positive integer. When unresolvedCount is 0 this banner must not appear.
      // Note: SpecWorkflow renders "No clarifications needed" (starts with "No"),
      // so we match only the numeric form: e.g. "1 clarification needed".
      expect(screen.queryByText(/\d+ clarifications? needed/)).not.toBeInTheDocument();
    });
  });

  // ----------------------------------------------------------
  // 7. Error state when APIs fail
  // ----------------------------------------------------------
  describe('error state', () => {
    it('shows empty spec list when /api/specs returns an error', async () => {
      useNoAutoSelect();
      server.use(
        http.get('/api/specs', () => HttpResponse.error())
      );

      render(<SpecEditor />);

      // With failed specs query the list view still renders (empty or with no specs)
      // The component renders SpecListView which shows the empty state when specs=[]
      await waitFor(() => {
        expect(screen.getByText('Specifications')).toBeInTheDocument();
      });
    });

    it('keeps showing spinner when spec detail request fails and spec is null', async () => {
      const SPEC_NAME = '042-add-auth';
      useAutoSelectSpec(SPEC_NAME);

      server.use(
        http.get('/api/specs/detail', () =>
          HttpResponse.json({ success: false, error: 'Spec not found' })
        ),
        http.get(`/api/specs/${SPEC_NAME}/clarifications`, () =>
          HttpResponse.json({ clarifications: [], unresolved_count: 0 })
        )
      );

      render(<SpecEditor />);

      // When specDetailData.success is false, setSpec is never called, so spec stays null
      // and the component renders the loading spinner indefinitely
      await waitFor(() => {
        expect(document.querySelector('.animate-spin')).toBeInTheDocument();
      });
    });
  });
});
