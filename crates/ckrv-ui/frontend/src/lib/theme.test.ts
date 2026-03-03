/**
 * @module theme.test
 * @description
 * Unit tests for theme utility functions. Validates status/log/agent class
 * mappings, terminal theme detection, and legacy alias referential equality.
 *
 * @context
 * Pure function tests running in jsdom via Vitest. No MSW, no QueryClient,
 * no renderHook needed.
 *
 * @dependencies
 * - vitest: describe, it, expect, afterEach
 * - @/lib/theme: Functions and constants under test
 */

import { afterEach, describe, expect, it } from 'vitest';
import {
    AGENT_CLASSES,
    STATUS_CLASSES,
    STATUS_COLORS,
    TERMINAL_THEMES,
    getAgentClasses,
    getLogClass,
    getStatusBgClass,
    getStatusBorderClass,
    getStatusClass,
    getStatusColor,
    getTerminalTheme,
    isDarkMode,
} from '@/lib/theme';

// ============================================================
// STATUS CLASS TESTS
// ============================================================

describe('getStatusClass', () => {
    it('returns correct class for known status "completed"', () => {
        expect(getStatusClass('completed')).toBe('text-success');
    });

    it('returns pending fallback for unknown status', () => {
        expect(getStatusClass('unknown-status')).toBe('text-muted-foreground');
    });
});

describe('getStatusBgClass', () => {
    it('returns correct bg class for "running"', () => {
        expect(getStatusBgClass('running')).toBe('bg-primary/10');
    });

    it('returns pending fallback for unknown status', () => {
        expect(getStatusBgClass('unknown-status')).toBe('bg-muted');
    });
});

describe('getStatusBorderClass', () => {
    it('returns correct border class for "failed"', () => {
        expect(getStatusBorderClass('failed')).toBe('border-error');
    });

    it('returns pending fallback for unknown status', () => {
        expect(getStatusBorderClass('unknown-status')).toBe('border-border');
    });
});

// ============================================================
// LOG CLASS TESTS
// ============================================================

describe('getLogClass', () => {
    it('returns correct class for "error"', () => {
        expect(getLogClass('error')).toBe('text-error');
    });

    it('returns info fallback for unknown log type', () => {
        expect(getLogClass('unknown-log-type')).toBe('text-muted-foreground');
    });
});

// ============================================================
// AGENT CLASS TESTS
// ============================================================

describe('getAgentClasses', () => {
    it('returns anthropic classes for exact "anthropic" match', () => {
        expect(getAgentClasses('anthropic')).toEqual(AGENT_CLASSES.anthropic);
    });

    it('returns anthropic classes for substring "claude"', () => {
        expect(getAgentClasses('claude-opus')).toEqual(AGENT_CLASSES.anthropic);
    });

    it('returns openai classes for substring "gpt"', () => {
        expect(getAgentClasses('gpt-4o')).toEqual(AGENT_CLASSES.openai);
    });

    it('returns default classes for unknown provider', () => {
        expect(getAgentClasses('some-unknown-model')).toEqual(AGENT_CLASSES.default);
    });
});

// ============================================================
// TERMINAL THEME TESTS
// ============================================================

describe('getTerminalTheme', () => {
    afterEach(() => {
        document.documentElement.classList.remove('dark');
    });

    it('returns dark theme when "dark" class is present on documentElement', () => {
        document.documentElement.classList.add('dark');
        expect(getTerminalTheme()).toEqual(TERMINAL_THEMES.dark);
    });

    it('returns light theme when "dark" class is absent', () => {
        document.documentElement.classList.remove('dark');
        expect(getTerminalTheme()).toEqual(TERMINAL_THEMES.light);
    });
});

describe('isDarkMode', () => {
    afterEach(() => {
        document.documentElement.classList.remove('dark');
    });

    it('returns true when "dark" class is present', () => {
        document.documentElement.classList.add('dark');
        expect(isDarkMode()).toBe(true);
    });

    it('returns false when "dark" class is absent', () => {
        document.documentElement.classList.remove('dark');
        expect(isDarkMode()).toBe(false);
    });
});

// ============================================================
// LEGACY ALIAS TESTS
// ============================================================

describe('legacy aliases', () => {
    it('STATUS_COLORS is the same reference as STATUS_CLASSES', () => {
        expect(STATUS_COLORS).toBe(STATUS_CLASSES);
    });

    it('getStatusColor is the same reference as getStatusClass', () => {
        expect(getStatusColor).toBe(getStatusClass);
    });
});
