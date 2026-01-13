/**
 * LoadingButton Component
 * 
 * A button that displays a loading spinner when in loading state.
 * This is a thin wrapper around shadcn/ui Button with loading support.
 * 
 * Migration: Now uses shadcn/ui Button component internally.
 */

import React, { type ReactNode } from 'react';
import { Loader2 } from 'lucide-react';
import { Button, type ButtonProps } from '@/components/ui/button';
import { cn } from '@/lib/utils';

interface LoadingButtonProps extends ButtonProps {
    loading?: boolean;
    loadingText?: string;
    icon?: ReactNode;
}

export const LoadingButton: React.FC<LoadingButtonProps> = ({
    loading = false,
    loadingText,
    icon,
    children,
    disabled,
    className,
    ...rest
}) => {
    return (
        <Button
            disabled={loading || disabled}
            className={cn(className)}
            {...rest}
        >
            {loading ? (
                <>
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                    <span>{loadingText || children}</span>
                </>
            ) : (
                <>
                    {icon && <span className="mr-2">{icon}</span>}
                    <span>{children}</span>
                </>
            )}
        </Button>
    );
};

export default LoadingButton;
