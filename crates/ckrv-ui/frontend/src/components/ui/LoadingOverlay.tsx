/**
 * @module LoadingOverlay
 * @description
 * Full-container overlay with loading spinner for async operations.
 * Renders a centered spinner with optional message, supporting both
 * full-screen and container-relative positioning with transparency options.
 *
 * @context
 * Used to block UI during async operations like data fetching or
 * form submission. Provides consistent loading indicator pattern.
 *
 * @dependencies
 * - Loader2 from lucide-react
 *
 * @example
 * <LoadingOverlay visible={isLoading} message="Fetching data..." />
 *
 * @example
 * <LoadingOverlay fullScreen transparent message="Processing..." />
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
                ${transparent ? 'bg-black/30 backdrop-blur-sm' : 'bg-background'}
            `}
            role="status"
            aria-live="polite"
            aria-busy="true"
        >
            <div
                className={`flex flex-col items-center gap-3 p-6 rounded-lg ${transparent ? 'bg-card' : ''}`}
            >
                <Loader2
                    size={32}
                    className="animate-spin text-primary"
                />
                {message && (
                    <span className="text-sm text-muted-foreground">
                        {message}
                    </span>
                )}
            </div>
        </div>
    );
};

export default LoadingOverlay;
