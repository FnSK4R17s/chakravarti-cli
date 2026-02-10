/**
 * @module LoadingButton
 * @description
 * Button component with integrated loading spinner.
 * Wraps shadcn/ui Button to show a spinner during async operations,
 * automatically disabling interaction when loading.
 *
 * @context
 * Used throughout the app for form submissions and async actions
 * where visual feedback is needed during processing.
 *
 * @dependencies
 * - Button from @/components/ui/button
 * - Loader2 from lucide-react
 *
 * @example
 * <LoadingButton loading={isSubmitting} onClick={handleSubmit}>
 *   Submit
 * </LoadingButton>
 *
 * @example
 * <LoadingButton loading={true} loadingText="Saving...">
 *   Save Changes
 * </LoadingButton>
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
