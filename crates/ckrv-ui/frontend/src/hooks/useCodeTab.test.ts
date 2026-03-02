/**
 * @module useCodeTab.test
 * @description
 * Tests for the useCodeTab hook. Validates session storage persistence,
 * default tab selection, and invalid value handling.
 *
 * @context
 * Uses renderHook from @testing-library/react. sessionStorage is cleared
 * before each test for isolation.
 *
 * @dependencies
 * - @testing-library/react: renderHook, act
 * - vitest: describe, it, expect, beforeEach
 */

import { renderHook, act } from '@testing-library/react';
import { describe, it, expect, beforeEach } from 'vitest';
import { useCodeTab } from './useCodeTab';

const STORAGE_KEY = 'chakravarti-code-tab';

describe('useCodeTab', () => {
    beforeEach(() => {
        sessionStorage.clear();
    });

    it('defaults to "spec" when no sessionStorage value', () => {
        const { result } = renderHook(() => useCodeTab());
        expect(result.current[0]).toBe('spec');
    });

    it('uses custom default tab when provided', () => {
        const { result } = renderHook(() => useCodeTab('tasks'));
        expect(result.current[0]).toBe('tasks');
    });

    it('restores tab from sessionStorage', () => {
        sessionStorage.setItem(STORAGE_KEY, 'plan');
        const { result } = renderHook(() => useCodeTab());
        expect(result.current[0]).toBe('plan');
    });

    it('ignores invalid sessionStorage value, falls back to default', () => {
        sessionStorage.setItem(STORAGE_KEY, 'invalid-tab');
        const { result } = renderHook(() => useCodeTab());
        expect(result.current[0]).toBe('spec');
    });

    it('setTab() persists to sessionStorage', () => {
        const { result } = renderHook(() => useCodeTab());
        act(() => {
            result.current[1]('tasks');
        });
        expect(sessionStorage.getItem(STORAGE_KEY)).toBe('tasks');
        expect(result.current[0]).toBe('tasks');
    });
});
