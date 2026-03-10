/**
 * @module CodePage
 * @description
 * Unified code workflow page consolidating Spec, Tasks, Plan, and Run views into
 * a single tabbed interface. Reduces navigation complexity while keeping all
 * functionality accessible with visual progress indicators.
 *
 * @context
 * Rendered as the main content of the Code page in the dashboard. Tab state persists
 * in session storage so users return to their last active tab.
 *
 * @dependencies
 * - useCodeTab: Custom hook for tab state with session persistence
 * - useWorkflowProgress: Hook for fetching workflow stage completion status
 * - SpecEditor, TaskEditor, PlanEditor, BarebonesExecutor: Tab content components
 * - shadcn/ui Tabs: Tab navigation component
 *
 * @example
 * // Rendered directly as a page component
 * <CodePage />
 *
 * // With initial tab selection
 * <CodePage initialTab="tasks" selectedSpec="my-feature" />
 */

// === IMPORTS ===
import React from 'react';
import { useQuery } from '@tanstack/react-query';
import { FileText, ListTodo, Workflow, Rocket, CheckCircle2, Code, Lock } from 'lucide-react';
import { Card, CardHeader, CardTitle } from '@/components/ui/card';
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@/components/ui/tabs';
import { SpecEditor } from './SpecEditor';
import { TaskEditor } from './TaskEditor';
import PlanEditor from './PlanEditor';
import BarebonesExecutor from './BarebonesExecutor';
import { CODE_TABS, type CodeTabType } from '../types';
import { useCodeTab } from '../hooks/useCodeTab';
import { useWorkflowProgress } from '../hooks/useWorkflowProgress';
import { useAutoSelectedSpec } from '../hooks/useAutoSelectedSpec';

// Icon map for dynamic rendering
const ICON_MAP = {
    FileText,
    ListTodo,
    Workflow,
    Rocket,
} as const;

/**
 * Props for the CodePage component.
 */
interface CodePageProps {
    /**
     * Initial tab to display when the component mounts.
     * @default 'spec'
     */
    initialTab?: CodeTabType;
    /** Spec name to track workflow progress for; enables completion indicators */
    selectedSpec?: string;
}

const CodePage: React.FC<CodePageProps> = ({
    initialTab = 'spec',
    selectedSpec
}) => {
    // Use custom hook for tab state with session persistence
    const [activeTab, setActiveTab] = useCodeTab(initialTab);

    // Fetch workflow progress for visual indicators
    const workflowProgress = useWorkflowProgress(selectedSpec);

    // Auto-detect current spec for status check
    const { selectedSpec: autoSpec } = useAutoSelectedSpec();
    const specName = selectedSpec ?? autoSpec;

    // Fetch specs list to check artifact existence for tab locking
    const { data: specsData } = useQuery({
        queryKey: ['specs'],
        queryFn: async () => {
            const res = await fetch('/api/specs');
            return res.json();
        },
        staleTime: 5000,
    });

    // Lock tabs based on artifact existence (progressive unlocking)
    const currentSpec = specsData?.specs?.find((s: { name: string }) => s.name === specName);
    const lockedTabs = new Set<string>();
    if (!currentSpec?.has_design && !currentSpec?.has_tasks) lockedTabs.add('tasks');
    if (!currentSpec?.has_tasks) lockedTabs.add('plan');
    if (!currentSpec?.has_plan) lockedTabs.add('run');

    // Get completion status for a tab
    const getTabStatus = (tabId: CodeTabType): 'pending' | 'complete' => {
        const stage = workflowProgress.find(s => s.id === tabId);
        return stage?.status ?? 'pending';
    };

    return (
        <div className="h-full flex flex-col overflow-hidden bg-background text-foreground">
            {/* Header */}
            <Card className="shrink-0 rounded-none border-x-0 border-t-0">
                <CardHeader className="pb-4">
                    <div>
                        <CardTitle className="text-xl flex items-center gap-3">
                            <Code className="text-primary" size={24} />
                            Code
                        </CardTitle>
                        <p className="text-muted-foreground text-sm mt-1">Create specs, generate tasks, plan execution, and run AI agents</p>
                    </div>
                </CardHeader>
            </Card>

            {/* Tab Navigation */}
            <Tabs
                value={activeTab}
                onValueChange={(value) => setActiveTab(value as CodeTabType)}
                className="flex-1 min-h-0 flex flex-col"
            >
                <div className="shrink-0 px-4 pt-2 border-b border-border bg-muted/30">
                    <TabsList className="h-11 bg-transparent p-0 gap-1">
                        {CODE_TABS.map((tab) => {
                            const Icon = ICON_MAP[tab.icon];
                            const status = getTabStatus(tab.id);
                            const isComplete = status === 'complete';
                            const isLocked = lockedTabs.has(tab.id);

                            return (
                                <TabsTrigger
                                    key={tab.id}
                                    value={tab.id}
                                    disabled={isLocked}
                                    className={`relative gap-2 px-4 py-2 rounded-t-lg rounded-b-none border-b-2 transition-all
                                        data-[state=active]:bg-background data-[state=active]:shadow-sm data-[state=active]:border-primary
                                        ${isLocked
                                            ? 'opacity-40 cursor-not-allowed border-transparent'
                                            : isComplete
                                                ? 'text-foreground border-transparent data-[state=inactive]:text-success'
                                                : 'text-muted-foreground border-transparent data-[state=inactive]:hover:text-foreground'
                                        }`}
                                    data-testid={`code-tab-${tab.id}`}
                                >
                                    <Icon size={16} />
                                    <span>{tab.label}</span>
                                    {isLocked && (
                                        <Lock size={12} className="ml-1 text-muted-foreground" />
                                    )}
                                    {!isLocked && isComplete && (
                                        <CheckCircle2
                                            size={14}
                                            className="text-success ml-1"
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
