import React from 'react';
import { FileText, ListTodo, Workflow, Rocket, CheckCircle2 } from 'lucide-react';
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@/components/ui/tabs';
import { SpecEditor } from './SpecEditor';
import { TaskEditor } from './TaskEditor';
import PlanEditor from './PlanEditor';
import BarebonesExecutor from './BarebonesExecutor';
import { CODE_TABS, type CodeTabType } from '../types';
import { useCodeTab } from '../hooks/useCodeTab';
import { useWorkflowProgress } from '../hooks/useWorkflowProgress';

// Icon map for dynamic rendering
const ICON_MAP = {
    FileText,
    ListTodo,
    Workflow,
    Rocket,
} as const;

interface CodePageProps {
    /** Optional initial tab to display */
    initialTab?: CodeTabType;
    /** Optional spec name to track progress for */
    selectedSpec?: string;
}

/**
 * CodePage - Unified code workflow page
 * 
 * Consolidates Spec, Tasks, Plan, and Run pages into a single tabbed interface.
 * This reduces navigation complexity while keeping all functionality accessible.
 * 
 * Features:
 * - Tab-based navigation between Spec, Tasks, Plan, and Run views
 * - Session persistence of active tab (remembers tab when navigating away/back)
 * - Visual progress indicators showing workflow stage completion
 * - Keyboard accessibility (arrow keys navigate between tabs)
 */
const CodePage: React.FC<CodePageProps> = ({
    initialTab = 'spec',
    selectedSpec
}) => {
    // Use custom hook for tab state with session persistence
    const [activeTab, setActiveTab] = useCodeTab(initialTab);

    // Fetch workflow progress for visual indicators
    const workflowProgress = useWorkflowProgress(selectedSpec);

    // Get completion status for a tab
    const getTabStatus = (tabId: CodeTabType): 'pending' | 'complete' => {
        const stage = workflowProgress.find(s => s.id === tabId);
        return stage?.status ?? 'pending';
    };

    return (
        <div className="h-full flex flex-col overflow-hidden">
            {/* Tab Navigation */}
            <Tabs
                value={activeTab}
                onValueChange={(value) => setActiveTab(value as CodeTabType)}
                className="h-full flex flex-col"
            >
                <div className="shrink-0 px-4 pt-2 border-b border-border bg-muted/30">
                    <TabsList className="h-11 bg-transparent p-0 gap-1">
                        {CODE_TABS.map((tab) => {
                            const Icon = ICON_MAP[tab.icon];
                            const status = getTabStatus(tab.id);
                            const isComplete = status === 'complete';

                            return (
                                <TabsTrigger
                                    key={tab.id}
                                    value={tab.id}
                                    className={`relative gap-2 px-4 py-2 rounded-t-lg rounded-b-none border-b-2 transition-all
                                        data-[state=active]:bg-background data-[state=active]:shadow-sm data-[state=active]:border-primary
                                        ${isComplete
                                            ? 'text-foreground border-transparent data-[state=inactive]:text-accent-green'
                                            : 'text-muted-foreground border-transparent data-[state=inactive]:hover:text-foreground'
                                        }`}
                                    data-testid={`code-tab-${tab.id}`}
                                >
                                    <Icon size={16} />
                                    <span>{tab.label}</span>
                                    {isComplete && (
                                        <CheckCircle2
                                            size={14}
                                            className="text-accent-green ml-1"
                                            data-testid={`code-tab-${tab.id}-complete`}
                                        />
                                    )}
                                </TabsTrigger>
                            );
                        })}
                    </TabsList>
                </div>

                {/* Tab Content - Each tab renders its respective editor */}
                <TabsContent
                    value="spec"
                    className="flex-1 m-0 overflow-hidden data-[state=inactive]:hidden"
                    data-testid="code-content-spec"
                >
                    <SpecEditor />
                </TabsContent>

                <TabsContent
                    value="tasks"
                    className="flex-1 m-0 overflow-hidden data-[state=inactive]:hidden"
                    data-testid="code-content-tasks"
                >
                    <TaskEditor />
                </TabsContent>

                <TabsContent
                    value="plan"
                    className="flex-1 m-0 overflow-hidden data-[state=inactive]:hidden"
                    data-testid="code-content-plan"
                >
                    <PlanEditor />
                </TabsContent>

                <TabsContent
                    value="run"
                    className="flex-1 m-0 overflow-hidden data-[state=inactive]:hidden"
                    data-testid="code-content-run"
                >
                    <BarebonesExecutor />
                </TabsContent>
            </Tabs>
        </div>
    );
};

export default CodePage;
