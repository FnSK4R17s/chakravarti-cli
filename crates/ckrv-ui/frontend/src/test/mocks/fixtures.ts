/**
 * @module test/mocks/fixtures
 * @description
 * Factory functions for creating test data matching API response types.
 * Each factory returns a sensible default that can be overridden with
 * partial arguments.
 *
 * @context
 * Used across all test files for consistent mock data generation.
 *
 * @dependencies
 * - Types from api.generated.ts and types.ts
 */

import type { AgentConfig, DockerStatus, SpecSummary } from '@/types/api.generated';
import type { SystemStatus } from '@/types';

// ============================================================
// SYSTEM FIXTURES
// ============================================================

export function createSystemStatus(overrides?: Partial<SystemStatus>): SystemStatus {
  return {
    active_branch: 'feature/test-branch',
    feature_number: '042',
    is_ready: true,
    mode: 'idle',
    project_root: '/home/user/project',
    ...overrides,
  };
}

// ============================================================
// AGENT FIXTURES
// ============================================================

export function createAgent(overrides?: Partial<AgentConfig>): AgentConfig {
  return {
    name: 'claude-default',
    display_name: 'Claude Code',
    agent_type: 'claude',
    model: 'claude-sonnet-4-20250514',
    is_default: true,
    enabled: true,
    ...overrides,
  } as AgentConfig;
}

// ============================================================
// SPEC FIXTURES
// ============================================================

export function createSpec(overrides?: Partial<SpecSummary>): SpecSummary {
  return {
    name: '042-add-auth',
    path: '.specs/042-add-auth',
    title: 'Add Authentication',
    status: 'draft',
    has_plan: false,
    has_tasks: false,
    has_design: false,
    has_implementation: false,
    implementation_branch: null,
    task_count: 0,
    created_at: '2026-01-15T10:00:00Z',
    updated_at: '2026-01-15T10:00:00Z',
    ...overrides,
  };
}

export function createSpecDetail(overrides?: Record<string, unknown>) {
  return {
    id: '042-add-auth',
    goal: 'Add user authentication to the application',
    overview: 'Implement login/signup with JWT tokens',
    status: 'draft',
    user_stories: [
      {
        id: 'US-1',
        title: 'User can log in',
        priority: 'high',
        description: 'As a user, I want to log in with my credentials',
      },
    ],
    requirements: [
      { id: 'REQ-1', description: 'Support email/password login' },
    ],
    success_criteria: [
      { id: 'SC-1', metric: 'Users can authenticate within 2 seconds' },
    ],
    assumptions: ['Backend API supports JWT tokens'],
    edge_cases: ['Expired tokens should redirect to login'],
    clarifications: [],
    ...overrides,
  };
}

// ============================================================
// DOCKER FIXTURES
// ============================================================

export function createDockerStatus(overrides?: Partial<DockerStatus>): DockerStatus {
  return {
    available: true,
    version: '24.0.7',
    error: null,
    ...overrides,
  };
}

// ============================================================
// TASK FIXTURES
// ============================================================

export function createTask(overrides?: Record<string, unknown>) {
  return {
    id: 'task-001',
    phase: 'Phase 1',
    title: 'Set up project structure',
    description: 'Create the initial project scaffold',
    file: 'src/index.ts',
    user_story: 'US-1',
    parallel: false,
    complexity: 2,
    model_tier: 'standard',
    estimated_tokens: 5000,
    risk: 'low',
    context_required: ['package.json'],
    status: 'pending',
    ...overrides,
  };
}
