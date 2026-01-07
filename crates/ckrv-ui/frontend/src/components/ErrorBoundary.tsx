/**
 * ErrorBoundary Component (T007)
 * 
 * Catches JavaScript errors anywhere in the child component tree,
 * logs those errors, and displays a fallback UI.
 * 
 * Addresses BUG-006: Missing Error Boundary for Component Crashes
 */

import { Component, type ErrorInfo, type ReactNode } from 'react';
import { AlertTriangle, RotateCcw } from 'lucide-react';

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

            // Default error UI
            return (
                <div
                    className="min-h-[400px] flex items-center justify-center p-8"
                    style={{ background: 'var(--bg-secondary)' }}
                >
                    <div
                        className="max-w-md w-full rounded-lg p-6 text-center"
                        style={{
                            background: 'var(--bg-tertiary)',
                            border: '1px solid var(--accent-red-dim)',
                        }}
                    >
                        <div
                            className="w-12 h-12 mx-auto mb-4 rounded-full flex items-center justify-center"
                            style={{
                                background: 'var(--accent-red-dim)',
                                color: 'var(--accent-red)',
                            }}
                        >
                            <AlertTriangle size={24} />
                        </div>

                        <h2
                            className="text-lg font-semibold mb-2"
                            style={{ color: 'var(--text-primary)' }}
                        >
                            Something went wrong
                        </h2>

                        <p
                            className="text-sm mb-4"
                            style={{ color: 'var(--text-secondary)' }}
                        >
                            An unexpected error occurred. Please try again or refresh the page.
                        </p>

                        {this.state.error && (
                            <div
                                className="text-xs font-mono p-3 rounded mb-4 text-left overflow-auto max-h-32"
                                style={{
                                    background: 'var(--bg-surface)',
                                    color: 'var(--accent-red)',
                                }}
                            >
                                {this.state.error.message}
                            </div>
                        )}

                        <button
                            onClick={this.handleRetry}
                            className="inline-flex items-center gap-2 px-4 py-2 rounded-md text-sm font-medium transition-all"
                            style={{
                                background: 'var(--accent-cyan-dim)',
                                color: 'var(--accent-cyan)',
                                border: '1px solid var(--accent-cyan)',
                            }}
                            onMouseOver={(e) => {
                                e.currentTarget.style.background = 'var(--accent-cyan)';
                                e.currentTarget.style.color = 'var(--bg-primary)';
                            }}
                            onMouseOut={(e) => {
                                e.currentTarget.style.background = 'var(--accent-cyan-dim)';
                                e.currentTarget.style.color = 'var(--accent-cyan)';
                            }}
                        >
                            <RotateCcw size={16} />
                            Try Again
                        </button>
                    </div>
                </div>
            );
        }

        return this.props.children;
    }
}

export default ErrorBoundary;
