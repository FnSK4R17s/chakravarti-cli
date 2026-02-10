import { useState, createContext, useContext, useEffect } from 'react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { TooltipProvider } from '@/components/ui/tooltip';
import { Toaster } from '@/components/ui/sonner';
import { DashboardLayout } from './layouts/Dashboard';
import { StatusWidget } from './components/StatusWidget';
import { CommandPalette, CommandResultContext } from './components/CommandPalette';
import { ChatDashboard } from './components/ChatDashboard';
import AgentManager from './components/AgentManager';
import CodePage from './components/CodePage';
import TestRunner from './components/TestRunner';
import QAReviewer from './components/QAReviewer';
import ProjectSelector from './components/ProjectSelector';
import { ErrorBoundary } from './components/ErrorBoundary';
import './index.css';

const queryClient = new QueryClient({
    defaultOptions: {
        queries: {
            refetchOnWindowFocus: false,
            retry: 1,
        },
    },
});

interface CommandResult {
    command: string;
    result: { success: boolean; message?: string };
}

// Navigation context - Updated to include settings
export type PageType = 'dashboard' | 'agents' | 'code' | 'test' | 'qa' | 'settings';
interface NavigationContextType {
    currentPage: PageType;
    setCurrentPage: (page: PageType) => void;
}
export const NavigationContext = createContext<NavigationContextType>({
    currentPage: 'dashboard',
    setCurrentPage: () => { },
});
export const useNavigation = () => useContext(NavigationContext);

// Check if running in Tauri environment
const isTauri = (): boolean => {
    return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
};

// Dashboard Page - Chat-inspired UI
const DashboardPage = () => (
    <div className="h-full max-h-full overflow-auto">
        <ChatDashboard />
    </div>
);

// Settings Page - Contains initialization commands
const SettingsPage = () => (
    <div className="h-full max-h-full overflow-auto">
        <div className="max-w-2xl mx-auto space-y-6 py-4">
            <StatusWidget />
            <CommandPalette />
        </div>
    </div>
);

// Agents Page
const AgentsPage = () => (
    <div className="h-full max-h-full overflow-hidden">
        <AgentManager />
    </div>
);

// Test Page
const TestPage = () => (
    <div className="h-full max-h-full overflow-hidden">
        <TestRunner />
    </div>
);

// QA Page
const QAPage = () => (
    <div className="h-full max-h-full overflow-hidden">
        <QAReviewer />
    </div>
);

function App() {
    const [lastResult, setLastResult] = useState<CommandResult | null>(null);
    const [currentPage, setCurrentPage] = useState<PageType>('dashboard');

    // Project selection state for Tauri
    const [needsProjectSelection, setNeedsProjectSelection] = useState<boolean | null>(null);

    // Initialize theme from localStorage on mount
    useEffect(() => {
        const savedTheme = localStorage.getItem('theme');
        const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
        const isDark = savedTheme === 'dark' || (!savedTheme && prefersDark);

        const root = document.documentElement;
        if (isDark) {
            root.classList.add('dark');
            root.classList.remove('light');
        } else {
            root.classList.remove('dark');
            root.classList.add('light');
        }
    }, []);

    // Check if we need project selection (Tauri only)
    useEffect(() => {
        const checkProjectRoot = async () => {
            if (!isTauri()) {
                // Web mode - no project selection needed
                setNeedsProjectSelection(false);
                return;
            }

            try {
                const { invoke } = await import('@tauri-apps/api/core');
                const projectRoot = await invoke<string | null>('get_project_root');

                // Need selection if no project is configured
                setNeedsProjectSelection(projectRoot === null);
            } catch (e) {
                console.error('Failed to check project root:', e);
                // On error, proceed with app (fallback to cwd)
                setNeedsProjectSelection(false);
            }
        };

        checkProjectRoot();
    }, []);

    // Show loading state while checking project root
    if (needsProjectSelection === null) {
        return (
            <div className="min-h-screen flex items-center justify-center bg-background">
                <div className="animate-pulse text-muted-foreground">Loading...</div>
            </div>
        );
    }

    // Show project selector if needed (Tauri only)
    if (needsProjectSelection) {
        return (
            <ErrorBoundary>
                <QueryClientProvider client={queryClient}>
                    <ProjectSelector onProjectSelected={() => window.location.reload()} />
                    <Toaster />
                </QueryClientProvider>
            </ErrorBoundary>
        );
    }

    return (
        <ErrorBoundary>
            <QueryClientProvider client={queryClient}>
                <TooltipProvider delayDuration={200}>
                    <NavigationContext.Provider value={{ currentPage, setCurrentPage }}>
                        <CommandResultContext.Provider value={{ lastResult, setLastResult }}>
                            <DashboardLayout>
                                <ErrorBoundary>
                                    {currentPage === 'dashboard' && <DashboardPage />}
                                    {currentPage === 'agents' && <AgentsPage />}
                                    {currentPage === 'code' && <CodePage />}
                                    {currentPage === 'test' && <TestPage />}
                                    {currentPage === 'qa' && <QAPage />}
                                    {currentPage === 'settings' && <SettingsPage />}
                                </ErrorBoundary>
                            </DashboardLayout>
                        </CommandResultContext.Provider>
                    </NavigationContext.Provider>
                </TooltipProvider>
                <Toaster />
            </QueryClientProvider>
        </ErrorBoundary>
    );
}

export default App;

