/**
 * LoadingOverlay Component (T009)
 * 
 * A full-container overlay with loading spinner for async operations.
 * Provides consistent loading indicator pattern across the app.
 * 
 * Addresses BUG-004: Inconsistent Loading States Across Components
 */

import React from 'react';
import { Loader2 } from 'lucide-react';

interface LoadingOverlayProps {
    visible?: boolean;
    message?: string;
    fullScreen?: boolean;
    transparent?: boolean;
}

export const LoadingOverlay: React.FC<LoadingOverlayProps> = ({
    visible = true,
    message = 'Loading...',
    fullScreen = false,
    transparent = false,
}) => {
    if (!visible) return null;

    return (
        <div
            className={`
                ${fullScreen ? 'fixed inset-0 z-50' : 'absolute inset-0 z-10'}
                flex items-center justify-center
            `}
            style={{
                background: transparent
                    ? 'rgba(0, 0, 0, 0.3)'
                    : 'var(--bg-primary)',
                backdropFilter: transparent ? 'blur(2px)' : 'none',
            }}
            role="status"
            aria-live="polite"
            aria-busy="true"
        >
            <div
                className="flex flex-col items-center gap-3 p-6 rounded-lg"
                style={{
                    background: transparent ? 'var(--bg-tertiary)' : 'transparent',
                }}
            >
                <Loader2
                    size={32}
                    className="animate-spin"
                    style={{ color: 'var(--accent-cyan)' }}
                />
                {message && (
                    <span
                        className="text-sm"
                        style={{ color: 'var(--text-secondary)' }}
                    >
                        {message}
                    </span>
                )}
            </div>
        </div>
    );
};

export default LoadingOverlay;
