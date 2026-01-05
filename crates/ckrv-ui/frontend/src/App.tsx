import { useState, createContext, useContext } from 'react';
import { Theme } from '@radix-ui/themes';
import '@radix-ui/themes/styles.css';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { DashboardLayout } from './layouts/Dashboard';
import { StatusWidget } from './components/StatusWidget';
import { LogViewer } from './components/LogViewer';
import { CommandPalette, CommandResultContext } from './components/CommandPalette';
import { WorkflowPanel } from './components/WorkflowPanel';
import { AgentManager } from './components/AgentManager';
import { SpecEditor } from './components/SpecEditor';
import { TaskEditor } from './components/TaskEditor';
import PlanEditor from './components/PlanEditor';
import ExecutionRunner from './components/ExecutionRunner';
import DiffViewer from './components/DiffViewer';
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

// Navigation context
export type PageType = 'dashboard' | 'agents' | 'specs' | 'tasks' | 'plan' | 'runner' | 'diff';
interface NavigationContextType {
    currentPage: PageType;
    setCurrentPage: (page: PageType) => void;
}
export const NavigationContext = createContext<NavigationContextType>({
    currentPage: 'dashboard',
    setCurrentPage: () => { },
});
export const useNavigation = () => useContext(NavigationContext);

// Dashboard Page
const DashboardPage = () => (
    <div className="grid grid-cols-12 gap-4 h-full max-h-full overflow-hidden">

        {/* Left Column: Status & Commands */}
        <div className="col-span-12 lg:col-span-3 flex flex-col gap-4 overflow-y-auto pr-2 max-h-full">
            <StatusWidget />
            <CommandPalette />
        </div>

        {/* Right Column: Workflow Pipeline & Logs */}
        <div className="col-span-12 lg:col-span-9 flex flex-col gap-4 h-full max-h-full overflow-hidden">
            {/* Workflow Pipeline - Fixed height, scrollable */}
            <div className="shrink-0 overflow-x-auto">
                <WorkflowPanel />
            </div>

            {/* Logs - Takes remaining space */}
            <div className="flex-1 min-h-0 overflow-hidden">
                <LogViewer />
            </div>
        </div>

    </div>
);

// Agents Page
const AgentsPage = () => (
    <div className="h-full max-h-full overflow-hidden">
        <AgentManager />
    </div>
);

// Specs Page
const SpecsPage = () => (
    <div className="h-full max-h-full overflow-hidden">
        <SpecEditor />
    </div>
);

// Tasks Page
const TasksPage = () => (
    <div className="h-full max-h-full overflow-hidden">
        <TaskEditor />
    </div>
);

// Plan Page
const PlanPage = () => (
    <div className="h-full max-h-full overflow-hidden">
        <PlanEditor />
    </div>
);

// Runner Page
const RunnerPage = () => (
    <div className="h-full max-h-full overflow-hidden">
        <ExecutionRunner />
    </div>
);

// Diff Page
const DiffPage = () => (
    <div className="h-full max-h-full overflow-hidden">
        <DiffViewer />
    </div>
);

function App() {
    const [lastResult, setLastResult] = useState<CommandResult | null>(null);
    const [currentPage, setCurrentPage] = useState<PageType>('dashboard');

    return (
        <QueryClientProvider client={queryClient}>
            <Theme appearance="dark" accentColor="cyan" grayColor="slate" radius="medium" scaling="100%">
                <NavigationContext.Provider value={{ currentPage, setCurrentPage }}>
                    <CommandResultContext.Provider value={{ lastResult, setLastResult }}>
                        <DashboardLayout>
                            {currentPage === 'dashboard' && <DashboardPage />}
                            {currentPage === 'agents' && <AgentsPage />}
                            {currentPage === 'specs' && <SpecsPage />}
                            {currentPage === 'plan' && <PlanPage />}
                            {currentPage === 'tasks' && <TasksPage />}
                            {currentPage === 'runner' && <RunnerPage />}
                            {currentPage === 'diff' && <DiffPage />}
                        </DashboardLayout>
                    </CommandResultContext.Provider>
                </NavigationContext.Provider>
            </Theme>
        </QueryClientProvider>
    );
}

export default App;
