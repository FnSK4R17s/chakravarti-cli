/**
 * @module ErrorBoundary
 * @description
 * React error boundary that catches JavaScript errors in the component tree,
 * logs errors, and displays a user-friendly fallback UI with retry option.
 *
 * @context
 * Should wrap major sections of the application to prevent entire UI crashes.
 * Provides a consistent error display using shadcn Alert and Button components.
 *
 * @dependencies
 * - shadcn/ui components: Alert, Button, Card for consistent UI
 * - lucide-react: Icons for error display
 *
 * @example
 * <ErrorBoundary>
 *   <MyComponent />
 * </ErrorBoundary>
 *
 * // With custom fallback
 * <ErrorBoundary fallback={<CustomError />}>
 *   <MyComponent />
 * </ErrorBoundary>
 */

// === IMPORTS ===
import { Component, type ErrorInfo, type ReactNode } from 'react';
import { AlertTriangle, RotateCcw } from 'lucide-react';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';

/**
 * Props for the ErrorBoundary component.
 */
interface Props {
    /** Child components to render when no error has occurred */
    children: ReactNode;
    /** Optional custom fallback UI to show instead of default error display */
    fallback?: ReactNode;
}

/**
 * Internal state for the ErrorBoundary component.
 */
interface State {
    /** Whether an error has been caught */
    hasError: boolean;
    /** The caught error object, if any */
    error: Error | null;
    /** React error info containing component stack trace */
    errorInfo: ErrorInfo | null;
}

export class ErrorBoundary extends Component<Props, State> {
    constructor(props: Props) {
        super(props);
        this.state = {
            hasError: false,
            error: null,
            errorInfo: null,
        };
    }

    static getDerivedStateFromError(error: Error): Partial<State> {
        return { hasError: true, error };
    }

    componentDidCatch(error: Error, errorInfo: ErrorInfo): void {
        // Log error to console for debugging
        console.error('ErrorBoundary caught an error:', error);
        console.error('Component stack:', errorInfo.componentStack);

        this.setState({ errorInfo });
    }

    handleRetry = (): void => {
        this.setState({
            hasError: false,
            error: null,
            errorInfo: null,
        });
    };

    render(): ReactNode {
        if (this.state.hasError) {
            // Custom fallback provided
            if (this.props.fallback) {
                return this.props.fallback;
            }

            // Default error UI using shadcn components
            return (
                <div className="min-h-[400px] flex items-center justify-center p-8 bg-background">
                    <Card className="max-w-md w-full border-destructive">
                        <CardContent className="p-6 text-center">
                            <div className="w-12 h-12 mx-auto mb-4 rounded-full flex items-center justify-center bg-destructive/10 text-destructive">
                                <AlertTriangle size={24} />
                            </div>

                            <h2 className="text-lg font-semibold mb-2 text-foreground">
                                Something went wrong
                            </h2>

                            <p className="text-sm mb-4 text-muted-foreground">
                                An unexpected error occurred. Please try again or refresh the page.
                            </p>

                            {this.state.error && (
                                <Alert variant="destructive" className="mb-4 text-left">
                                    <AlertTriangle className="h-4 w-4" />
                                    <AlertTitle>Error</AlertTitle>
                                    <AlertDescription className="font-mono text-xs overflow-auto max-h-32">
                                        {this.state.error.message}
                                    </AlertDescription>
                                </Alert>
                            )}

                            <Button onClick={this.handleRetry} variant="outline">
                                <RotateCcw size={16} className="mr-2" />
                                Try Again
                            </Button>
                        </CardContent>
                    </Card>
                </div>
            );
        }

        return this.props.children;
    }
}

export default ErrorBoundary;
