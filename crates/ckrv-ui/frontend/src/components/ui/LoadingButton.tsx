/**
 * LoadingButton Component (T008)
 * 
 * A button that displays a loading spinner when in loading state.
 * Provides consistent loading indicator pattern across the app.
 * 
 * Addresses BUG-004: Inconsistent Loading States Across Components
 */

import React, { type ButtonHTMLAttributes, type ReactNode } from 'react';
import { Loader2 } from 'lucide-react';

interface LoadingButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
    loading?: boolean;
    loadingText?: string;
    icon?: ReactNode;
    variant?: 'primary' | 'secondary' | 'danger' | 'ghost';
    size?: 'sm' | 'md' | 'lg';
    children: ReactNode;
}

const variantStyles = {
    primary: {
        background: 'var(--accent-cyan)',
        color: 'var(--bg-primary)',
        border: 'none',
        hoverBackground: 'var(--accent-cyan)',
        hoverOpacity: '0.9',
    },
    secondary: {
        background: 'var(--bg-surface)',
        color: 'var(--text-primary)',
        border: '1px solid var(--border-default)',
        hoverBackground: 'var(--bg-elevated)',
        hoverOpacity: '1',
    },
    danger: {
        background: 'var(--accent-red-dim)',
        color: 'var(--accent-red)',
        border: '1px solid var(--accent-red)',
        hoverBackground: 'var(--accent-red)',
        hoverOpacity: '1',
    },
    ghost: {
        background: 'transparent',
        color: 'var(--text-secondary)',
        border: 'none',
        hoverBackground: 'var(--bg-surface)',
        hoverOpacity: '1',
    },
};

const sizeStyles = {
    sm: {
        padding: '0.375rem 0.75rem',
        fontSize: '0.75rem',
        gap: '0.375rem',
        iconSize: 14,
    },
    md: {
        padding: '0.5rem 1rem',
        fontSize: '0.875rem',
        gap: '0.5rem',
        iconSize: 16,
    },
    lg: {
        padding: '0.75rem 1.5rem',
        fontSize: '1rem',
        gap: '0.625rem',
        iconSize: 18,
    },
};

export const LoadingButton: React.FC<LoadingButtonProps> = ({
    loading = false,
    loadingText,
    icon,
    variant = 'primary',
    size = 'md',
    children,
    disabled,
    style,
    className,
    ...rest
}) => {
    const variantStyle = variantStyles[variant];
    const sizeStyle = sizeStyles[size];

    return (
        <button
            disabled={loading || disabled}
            className={`inline-flex items-center justify-center rounded-md font-medium transition-all ${className || ''}`}
            style={{
                padding: sizeStyle.padding,
                fontSize: sizeStyle.fontSize,
                gap: sizeStyle.gap,
                background: variantStyle.background,
                color: variantStyle.color,
                border: variantStyle.border,
                opacity: loading || disabled ? 0.6 : 1,
                cursor: loading || disabled ? 'not-allowed' : 'pointer',
                transitionDuration: 'var(--duration-normal, 200ms)',
                ...style,
            }}
            {...rest}
        >
            {loading ? (
                <>
                    <Loader2
                        size={sizeStyle.iconSize}
                        className="animate-spin"
                    />
                    <span>{loadingText || children}</span>
                </>
            ) : (
                <>
                    {icon}
                    <span>{children}</span>
                </>
            )}
        </button>
    );
};

export default LoadingButton;
