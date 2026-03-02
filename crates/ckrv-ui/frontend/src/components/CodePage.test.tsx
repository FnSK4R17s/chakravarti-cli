/**
 * @module CodePage.test
 * @description
 * Unit tests for the CodePage component. Validates that all four tab triggers
 * are rendered with the correct labels, data-testid attributes, and tab content
 * areas. Also verifies default tab selection and tab switching behaviour.
 *
 * @context
 * Child components (SpecEditor, TaskEditor, PlanEditor, BarebonesExecutor) are
 * mocked to isolate CodePage logic. The hooks useCodeTab and useWorkflowProgress
 * are also mocked so tests do not depend on session storage or network requests.
 *
 * @dependencies
 * - @/test/test-utils: Custom render with QueryClientProvider
 * - @testing-library/user-event: User interaction simulation
 * - vitest: Test framework (describe, it, expect, vi)
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@/test/test-utils';
import { userEvent } from '@/test/test-utils';
import CodePage from './CodePage';

// ============================================================
// MODULE MOCKS
// ============================================================

// Mock child components so CodePage tests are pure unit tests
vi.mock('./SpecEditor', () => ({
  SpecEditor: () => <div data-testid="mock-spec-editor">SpecEditor</div>,
}));

vi.mock('./TaskEditor', () => ({
  TaskEditor: () => <div data-testid="mock-task-editor">TaskEditor</div>,
}));

vi.mock('./PlanEditor', () => ({
  default: () => <div data-testid="mock-plan-editor">PlanEditor</div>,
}));

vi.mock('./BarebonesExecutor', () => ({
  default: () => <div data-testid="mock-executor">Executor</div>,
}));

// Mock useCodeTab: use a simple useState-based implementation.
// vi.mock factories are hoisted, so we must import React inside the factory.
vi.mock('../hooks/useCodeTab', async () => {
  const React = await import('react');
  const useTabState = (initialTab: string) => React.useState(initialTab);
  return { useCodeTab: useTabState, default: useTabState };
});

// Mock useWorkflowProgress: return empty stages so no completion indicators show
vi.mock('../hooks/useWorkflowProgress', () => ({
  useWorkflowProgress: () => [],
  default: () => [],
}));

// ============================================================
// TESTS
// ============================================================

describe('CodePage', () => {
  it('renders all four tab triggers', () => {
    render(<CodePage />);

    expect(screen.getByRole('tab', { name: /spec/i })).toBeInTheDocument();
    expect(screen.getByRole('tab', { name: /tasks/i })).toBeInTheDocument();
    expect(screen.getByRole('tab', { name: /plan/i })).toBeInTheDocument();
    expect(screen.getByRole('tab', { name: /run/i })).toBeInTheDocument();
  });

  it('defaults to the spec tab', () => {
    render(<CodePage />);

    const specTab = screen.getByRole('tab', { name: /spec/i });
    expect(specTab).toHaveAttribute('data-state', 'active');
  });

  it('tab triggers have correct data-testid attributes', () => {
    render(<CodePage />);

    expect(screen.getByTestId('code-tab-spec')).toBeInTheDocument();
    expect(screen.getByTestId('code-tab-tasks')).toBeInTheDocument();
    expect(screen.getByTestId('code-tab-plan')).toBeInTheDocument();
    expect(screen.getByTestId('code-tab-run')).toBeInTheDocument();
  });

  it('tab content areas have correct data-testid attributes', () => {
    render(<CodePage />);

    expect(screen.getByTestId('code-content-spec')).toBeInTheDocument();
    expect(screen.getByTestId('code-content-tasks')).toBeInTheDocument();
    expect(screen.getByTestId('code-content-plan')).toBeInTheDocument();
    expect(screen.getByTestId('code-content-run')).toBeInTheDocument();
  });

  it('can switch to the Tasks tab', async () => {
    const user = userEvent.setup();
    render(<CodePage />);

    const tasksTab = screen.getByTestId('code-tab-tasks');
    await user.click(tasksTab);

    expect(tasksTab).toHaveAttribute('data-state', 'active');

    // Spec tab should no longer be active
    const specTab = screen.getByTestId('code-tab-spec');
    expect(specTab).toHaveAttribute('data-state', 'inactive');
  });
});
