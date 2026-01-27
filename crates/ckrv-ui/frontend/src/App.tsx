import { useState, createContext, useContext } from 'react';
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

    return (
        <ErrorBoundary>
            <QueryClientProvider client={queryClient}>
                <div className="dark">
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
                </div>
            </QueryClientProvider>
        </ErrorBoundary>
    );
}

export default App;
