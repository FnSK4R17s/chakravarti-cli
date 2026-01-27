import { useState } from 'react';
import type { CodeTabType } from '../types';

/** Session storage key for persisting Code page tab state */
const CODE_TAB_STORAGE_KEY = 'chakravarti-code-tab';

/**
 * Custom hook for managing Code page tab state with session persistence.
 * 
 * This hook stores the active tab in sessionStorage so that:
 * - Tab state persists when navigating away and back to the Code page
 * - Tab state resets when the browser session ends
 * 
 * @param defaultTab - The default tab to use if no persisted value exists
 * @returns A tuple of [activeTab, setActiveTab]
 */
export function useCodeTab(defaultTab: CodeTabType = 'spec'): [CodeTabType, (tab: CodeTabType) => void] {
    // Initialize state from sessionStorage or use default
    const [activeTab, setActiveTabState] = useState<CodeTabType>(() => {
        if (typeof window === 'undefined') return defaultTab;

        const stored = sessionStorage.getItem(CODE_TAB_STORAGE_KEY);
        if (stored && isValidCodeTab(stored)) {
            return stored as CodeTabType;
        }
        return defaultTab;
    });

    // Persist to sessionStorage when tab changes
    const setActiveTab = (tab: CodeTabType) => {
        setActiveTabState(tab);
        if (typeof window !== 'undefined') {
            sessionStorage.setItem(CODE_TAB_STORAGE_KEY, tab);
        }
    };

    return [activeTab, setActiveTab];
}

/**
 * Type guard to validate a string is a valid CodeTabType
 */
function isValidCodeTab(value: string): value is CodeTabType {
    return ['spec', 'tasks', 'plan', 'run'].includes(value);
}

export default useCodeTab;
