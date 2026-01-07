/**
 * useFocusTrap Hook (T012)
 * 
 * Custom hook for trapping focus within a container element.
 * Used for modal dialogs to prevent focus from escaping.
 * 
 * Addresses BUG-010: Focus Trap Missing in Modals
 */

import { useRef, useEffect, useCallback, type RefObject } from 'react';

const FOCUSABLE_SELECTORS = [
    'button:not([disabled])',
    'input:not([disabled])',
    'select:not([disabled])',
    'textarea:not([disabled])',
    'a[href]',
    '[tabindex]:not([tabindex="-1"])',
].join(', ');

export interface UseFocusTrapOptions {
    /** Whether the focus trap is active (default: true) */
    enabled?: boolean;
    /** Element to return focus to on unmount */
    returnFocusOnUnmount?: boolean;
    /** Initial element to focus when trap activates */
    initialFocusRef?: RefObject<HTMLElement>;
}

export interface UseFocusTrapReturn<T extends HTMLElement> {
    /** Ref to attach to the container element */
    containerRef: RefObject<T>;
    /** Manually activate the focus trap */
    activate: () => void;
    /** Manually deactivate the focus trap */
    deactivate: () => void;
}

export function useFocusTrap<T extends HTMLElement = HTMLDivElement>(
    options: UseFocusTrapOptions = {}
): UseFocusTrapReturn<T> {
    const {
        enabled = true,
        returnFocusOnUnmount = true,
        initialFocusRef,
    } = options;

    const containerRef = useRef<T>(null);
    const previousFocusRef = useRef<HTMLElement | null>(null);

    // Get all focusable elements within container
    const getFocusableElements = useCallback((): HTMLElement[] => {
        if (!containerRef.current) return [];
        return Array.from(
            containerRef.current.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTORS)
        );
    }, []);

    // Handle keydown for focus trapping
    const handleKeyDown = useCallback((event: KeyboardEvent) => {
        if (event.key !== 'Tab' || !containerRef.current) return;

        const focusableElements = getFocusableElements();
        if (focusableElements.length === 0) return;

        const firstElement = focusableElements[0];
        const lastElement = focusableElements[focusableElements.length - 1];
        const activeElement = document.activeElement as HTMLElement;

        // Shift + Tab: move to previous element
        if (event.shiftKey) {
            if (activeElement === firstElement || !containerRef.current.contains(activeElement)) {
                event.preventDefault();
                lastElement.focus();
            }
        }
        // Tab: move to next element
        else {
            if (activeElement === lastElement || !containerRef.current.contains(activeElement)) {
                event.preventDefault();
                firstElement.focus();
            }
        }
    }, [getFocusableElements]);

    // Activate focus trap
    const activate = useCallback(() => {
        // Store current focus
        previousFocusRef.current = document.activeElement as HTMLElement;

        // Focus initial element or first focusable
        if (initialFocusRef?.current) {
            initialFocusRef.current.focus();
        } else {
            const focusableElements = getFocusableElements();
            if (focusableElements.length > 0) {
                focusableElements[0].focus();
            }
        }

        // Add keydown listener
        document.addEventListener('keydown', handleKeyDown);
    }, [getFocusableElements, handleKeyDown, initialFocusRef]);

    // Deactivate focus trap
    const deactivate = useCallback(() => {
        document.removeEventListener('keydown', handleKeyDown);

        // Return focus to previous element
        if (returnFocusOnUnmount && previousFocusRef.current) {
            previousFocusRef.current.focus();
        }
    }, [handleKeyDown, returnFocusOnUnmount]);

    // Effect to manage focus trap lifecycle
    useEffect(() => {
        if (enabled) {
            // Small delay to ensure DOM is ready
            const timeoutId = setTimeout(activate, 10);
            return () => {
                clearTimeout(timeoutId);
                deactivate();
            };
        }
    }, [enabled, activate, deactivate]);

    return {
        containerRef: containerRef as RefObject<T>,
        activate,
        deactivate,
    };
}

export default useFocusTrap;
