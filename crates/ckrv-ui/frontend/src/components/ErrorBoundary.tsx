/**
 * ErrorBoundary Component
 * 
 * Catches JavaScript errors anywhere in the child component tree,
 * logs those errors, and displays a fallback UI.
 * 
 * Migrated to use shadcn Alert and Button components.
 */

import { Component, type ErrorInfo, type ReactNode } from 'react';
import { AlertTriangle, RotateCcw } from 'lucide-react';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';

interface Props {
    children: ReactNode;
    fallback?: ReactNode;
}

interface State {
    hasError: boolean;
    error: Error | null;
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
