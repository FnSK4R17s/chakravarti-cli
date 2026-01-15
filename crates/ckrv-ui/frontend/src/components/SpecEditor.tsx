import React, { useState, useEffect } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import {
    ChevronDown, ChevronRight, Code, LayoutGrid, List,
    FileText, ArrowLeft, Loader2, AlertCircle, CheckCircle2,
    Circle, Lightbulb, X, GitBranch, Calendar, Tag
} from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible';
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { SpecWorkflow } from './SpecWorkflow';
import { ClarifyModal } from './ClarifyModal';
import { useClarifications, useSubmitClarifications, type Clarification } from '../hooks/useSpec';

// Types matching the NEW spec.yaml format
interface AcceptanceScenario {
    given: string;
    when: string;
    then: string;
}

interface UserStory {
    id: string;
    title: string;
    priority: string;
    description: string;
    why_priority?: string;
    independent_test?: string;
    acceptance_scenarios?: AcceptanceScenario[];
}

interface FunctionalRequirement {
    id: string;
    description: string;
    category?: string;
}

interface Requirements {
    functional?: FunctionalRequirement[];
}

interface SuccessCriterion {
    id: string;
    metric: string;
    measurement?: string;
}

interface SpecClarification {
    topic: string;
    question: string;
    options: { label: string; answer: string; implications?: string }[];
    resolved: string | null;
}

interface SpecDetail {
    id: string;
    branch?: string;
    created?: string;
    status?: string;
    overview: string;
    user_stories: UserStory[];
    requirements: Requirements;
    success_criteria: SuccessCriterion[];
    edge_cases?: string[];
    assumptions?: string[];
    clarifications?: SpecClarification[];
}

interface SpecListItem {
    name: string;
    path: string;
    has_tasks: boolean;
    has_plan: boolean;
    has_design: boolean;
    has_implementation: boolean;
    implementation_branch: string | null;
}

// API functions
const fetchSpecs = async (): Promise<{ specs: SpecListItem[], count: number }> => {
    const res = await fetch('/api/specs');
    return res.json();
};

const fetchSpecDetail = async (name: string): Promise<{ success: boolean; spec?: SpecDetail; raw_yaml?: string; error?: string }> => {
    const res = await fetch(`/api/specs/detail?name=${encodeURIComponent(name)}`);
    return res.json();
};

// Collapsible Section Component
const Section: React.FC<{
    title: string;
    count?: number;
    children: React.ReactNode;
    defaultOpen?: boolean;
    color?: 'slate' | 'blue' | 'green' | 'amber' | 'purple' | 'cyan';
    icon?: React.ReactNode;
}> = ({ title, count, children, defaultOpen = true, color = 'slate', icon }) => {
    const [isOpen, setIsOpen] = useState(defaultOpen);
    const colorClasses = {
        slate: 'border-border bg-muted/50',
        blue: 'border-accent-cyan bg-accent-cyan-dim',
        green: 'border-accent-green bg-accent-green-dim',
        amber: 'border-accent-amber bg-accent-amber-dim',
        purple: 'border-accent-purple bg-accent-purple-dim',
        cyan: 'border-cyan-500/30 bg-cyan-500/5'
    };

    return (
        <Collapsible open={isOpen} onOpenChange={setIsOpen}>
            <Card className={`mb-3 ${colorClasses[color]}`}>
                <CollapsibleTrigger asChild>
                    <button className="w-full px-4 py-3 flex items-center justify-between text-left font-medium text-foreground hover:bg-accent/50 transition-colors rounded-t-lg">
                        <div className="flex items-center gap-2">
                            {isOpen ? <ChevronDown size={18} /> : <ChevronRight size={18} />}
                            {icon}
                            <span>{title}</span>
                            {count !== undefined && (
                                <Badge variant="secondary" className="text-xs">{count}</Badge>
                            )}
                        </div>
                    </button>
                </CollapsibleTrigger>
                <CollapsibleContent>
                    <CardContent className="pt-0 border-t border-border bg-background/50 rounded-b-lg">
                        {children}
                    </CardContent>
                </CollapsibleContent>
            </Card>
        </Collapsible>
    );
};

// Priority Badge
const PriorityBadge: React.FC<{ priority: string }> = ({ priority }) => {
    const variants: Record<string, "destructive" | "warning" | "success"> = {
        P1: 'destructive',
        P2: 'warning',
        P3: 'success'
    };
    return (
        <Badge variant={variants[priority] || 'secondary'}>
            {priority}
        </Badge>
    );
};

// User Story Card
const UserStoryCard: React.FC<{ story: UserStory }> = ({ story }) => {
    const [expanded, setExpanded] = useState(false);

    return (
        <Card className="hover:bg-accent/50 transition-colors">
            <CardContent className="p-4">
                <div className="flex items-start justify-between gap-3">
                    <div className="flex items-center gap-2">
                        <span className="text-xs font-mono text-muted-foreground">{story.id}</span>
                        <PriorityBadge priority={story.priority} />
                    </div>
                    <Button
                        variant="ghost"
                        size="icon"
                        className="h-6 w-6"
                        onClick={() => setExpanded(!expanded)}
                    >
                        {expanded ? <ChevronDown size={18} /> : <ChevronRight size={18} />}
                    </Button>
                </div>
                <h4 className="font-medium text-foreground mt-2">{story.title}</h4>
                {expanded && (
                    <div className="mt-3 pt-3 border-t border-border space-y-3">
                        <div>
                            <label className="text-xs text-muted-foreground uppercase tracking-wide">Description</label>
                            <p className="text-sm text-secondary-foreground mt-1 whitespace-pre-wrap">{story.description}</p>
                        </div>
                        {story.why_priority && (
                            <div>
                                <label className="text-xs text-muted-foreground uppercase tracking-wide">Why Priority</label>
                                <p className="text-sm text-secondary-foreground mt-1">{story.why_priority}</p>
                            </div>
                        )}
                        {story.independent_test && (
                            <div>
                                <label className="text-xs text-muted-foreground uppercase tracking-wide">Independent Test</label>
                                <p className="text-sm text-secondary-foreground mt-1">{story.independent_test}</p>
                            </div>
                        )}
                        {story.acceptance_scenarios && story.acceptance_scenarios.length > 0 && (
                            <div>
                                <label className="text-xs text-muted-foreground uppercase tracking-wide">Acceptance Scenarios</label>
                                {story.acceptance_scenarios.map((ac, i) => (
                                    <div key={i} className="ml-2 mt-2 text-sm text-muted-foreground space-y-1 p-2 bg-muted/30 rounded">
                                        <div><span className="text-accent-purple font-medium">Given:</span> {ac.given}</div>
                                        <div><span className="text-accent-cyan font-medium">When:</span> {ac.when}</div>
                                        <div><span className="text-accent-green font-medium">Then:</span> {ac.then}</div>
                                    </div>
                                ))}
                            </div>
                        )}
                    </div>
                )}
            </CardContent>
        </Card>
    );
};

// View Mode Toggle
const ViewToggle: React.FC<{
    view: string;
    setView: (v: string) => void;
}> = ({ view, setView }) => (
    <Tabs value={view} onValueChange={setView}>
        <TabsList>
            <TabsTrigger value="visual" className="gap-1.5">
                <LayoutGrid size={16} />
                Visual
            </TabsTrigger>
            <TabsTrigger value="outline" className="gap-1.5">
                <List size={16} />
                Outline
            </TabsTrigger>
            <TabsTrigger value="code" className="gap-1.5">
                <Code size={16} />
                YAML
            </TabsTrigger>
        </TabsList>
    </Tabs>
);

// Outline View
const OutlineView: React.FC<{ spec: SpecDetail }> = ({ spec }) => (
    <div className="font-mono text-sm space-y-1">
        <div className="text-muted-foreground">spec:</div>
        <div className="pl-4">
            <div><span className="text-accent-purple">id:</span> <span className="text-foreground">{spec.id}</span></div>
            {spec.branch && <div><span className="text-accent-purple">branch:</span> <span className="text-muted-foreground">{spec.branch}</span></div>}
            {spec.status && <div><span className="text-accent-purple">status:</span> <span className="text-muted-foreground">{spec.status}</span></div>}
            <div><span className="text-accent-purple">overview:</span> <span className="text-muted-foreground truncate inline-block max-w-md">{spec.overview?.slice(0, 80)}...</span></div>
            <div className="mt-2">
                <span className="text-accent-purple">user_stories:</span> <span className="text-muted-foreground">({spec.user_stories?.length || 0})</span>
                {spec.user_stories?.map(s => (
                    <div key={s.id} className="pl-4 text-muted-foreground">- {s.id}: {s.title}</div>
                ))}
            </div>
            <div className="mt-2">
                <span className="text-accent-purple">requirements.functional:</span> <span className="text-muted-foreground">({spec.requirements?.functional?.length || 0})</span>
            </div>
            <div className="mt-2">
                <span className="text-accent-purple">success_criteria:</span> <span className="text-muted-foreground">({spec.success_criteria?.length || 0})</span>
            </div>
            {spec.clarifications && spec.clarifications.length > 0 && (
                <div className="mt-2">
                    <span className="text-accent-purple">clarifications:</span> <span className="text-yellow-400">({spec.clarifications.filter(c => !c.resolved).length} unresolved)</span>
                </div>
            )}
        </div>
    </div>
);

// YAML View
const YamlView: React.FC<{ rawYaml?: string }> = ({ rawYaml }) => (
    <pre className="font-mono text-sm bg-muted text-foreground p-4 rounded-lg overflow-auto max-h-[60vh]">
        <code>{rawYaml || '# No YAML content'}</code>
    </pre>
);

// Spec List View
const SpecListView: React.FC<{
    specs: SpecListItem[];
    onSelect: (name: string) => void;
    isLoading: boolean;
}> = ({ specs, onSelect, isLoading }) => {
    if (isLoading) {
        return (
            <div className="flex items-center justify-center h-64">
                <Loader2 className="animate-spin text-muted-foreground" size={24} />
            </div>
        );
    }

    if (specs.length === 0) {
        return (
            <div className="text-center py-12 text-muted-foreground">
                <FileText size={48} className="mx-auto mb-4 opacity-50" />
                <p>No specifications found</p>
                <p className="text-sm mt-2">Run <code className="bg-muted px-2 py-0.5 rounded">ckrv spec new</code> to create one</p>
            </div>
        );
    }

    return (
        <div className="space-y-2">
            {specs.map((spec) => (
                <Card
                    key={spec.name}
                    className="cursor-pointer hover:bg-accent/50 transition-colors"
                    onClick={() => onSelect(spec.name)}
                >
                    <CardContent className="p-4">
                        <div className="flex items-center justify-between">
                            <div className="flex items-center gap-3">
                                <FileText size={20} className="text-primary" />
                                <div>
                                    <h3 className="font-medium text-foreground">{spec.name}</h3>
                                    <div className="flex items-center gap-2 mt-1">
                                        {spec.has_tasks && (
                                            <Badge variant="success">has tasks</Badge>
                                        )}
                                        {spec.has_plan && (
                                            <Badge variant="info">has plan</Badge>
                                        )}
                                        {spec.has_implementation && (
                                            <Badge variant="secondary">
                                                implemented: {spec.implementation_branch}
                                            </Badge>
                                        )}
                                    </div>
                                </div>
                            </div>
                            <ChevronRight size={20} className="text-muted-foreground" />
                        </div>
                    </CardContent>
                </Card>
            ))}
        </div>
    );
};

// Main Spec Editor Component
export const SpecEditor: React.FC = () => {
    const queryClient = useQueryClient();
    const [selectedSpecName, setSelectedSpecName] = useState<string | null>(null);
    const [spec, setSpec] = useState<SpecDetail | null>(null);
    const [rawYaml, setRawYaml] = useState<string | undefined>();
    const [view, setView] = useState<'visual' | 'outline' | 'code'>('visual');
    const [showClarifyModal, setShowClarifyModal] = useState(false);
    const [showWorkflowPanel, setShowWorkflowPanel] = useState(true);

    // Fetch specs list
    const { data: specsData, isLoading: isLoadingSpecs } = useQuery({
        queryKey: ['specs'],
        queryFn: fetchSpecs,
    });

    // Fetch spec detail when selected
    const { data: specDetailData, isLoading: isLoadingDetail } = useQuery({
        queryKey: ['spec', selectedSpecName],
        queryFn: () => fetchSpecDetail(selectedSpecName!),
        enabled: !!selectedSpecName,
    });

    // Update local state when spec detail is fetched
    useEffect(() => {
        if (specDetailData?.success && specDetailData.spec) {
            setSpec(specDetailData.spec);
            setRawYaml(specDetailData.raw_yaml);
        }
    }, [specDetailData]);

    // Fetch clarifications for selected spec
    const { data: clarificationsData } = useClarifications(selectedSpecName);
    const clarifications: Clarification[] = clarificationsData?.clarifications ?? [];
    const unresolvedCount = clarificationsData?.unresolved_count ?? 0;

    // Submit clarifications mutation
    const submitClarificationsMutation = useSubmitClarifications();

    const handleSubmitClarifications = async (answers: { topic: string; answer: string }[]) => {
        if (!selectedSpecName) return;
        await submitClarificationsMutation.mutateAsync({ name: selectedSpecName, answers });
        queryClient.invalidateQueries({ queryKey: ['spec', selectedSpecName] });
        queryClient.invalidateQueries({ queryKey: ['clarifications', selectedSpecName] });
    };

    // Show spec list if nothing selected
    if (!selectedSpecName) {
        return (
            <div className="h-full overflow-auto p-4">
                <div className="mb-6">
                    <h1 className="text-2xl font-bold text-foreground">Specifications</h1>
                    <p className="text-muted-foreground mt-1">Select a spec to view and edit</p>
                </div>
                <SpecListView
                    specs={specsData?.specs || []}
                    onSelect={setSelectedSpecName}
                    isLoading={isLoadingSpecs}
                />
            </div>
        );
    }

    if (isLoadingDetail || !spec) {
        return (
            <div className="flex items-center justify-center h-full">
                <Loader2 className="animate-spin text-muted-foreground" size={32} />
            </div>
        );
    }

    const functionalReqs = spec.requirements?.functional || [];

    return (
        <div className="h-full flex flex-col overflow-hidden">
            {/* Header */}
            <Card className="shrink-0 rounded-none border-x-0 border-t-0">
                <CardContent className="px-4 py-3 flex items-center justify-between">
                    <div className="flex items-center gap-4">
                        <Button
                            variant="ghost"
                            size="icon"
                            onClick={() => setSelectedSpecName(null)}
                        >
                            <ArrowLeft size={20} />
                        </Button>
                        <Badge variant="secondary" className="font-mono">{spec.id}</Badge>
                        {spec.status && (
                            <Badge variant={spec.status === 'draft' ? 'warning' : 'success'}>
                                <Tag size={12} className="mr-1" />
                                {spec.status}
                            </Badge>
                        )}
                        {spec.branch && (
                            <Badge variant="outline" className="text-muted-foreground">
                                <GitBranch size={12} className="mr-1" />
                                {spec.branch}
                            </Badge>
                        )}
                        {spec.created && (
                            <span className="text-xs text-muted-foreground flex items-center gap-1">
                                <Calendar size={12} />
                                {spec.created}
                            </span>
                        )}
                    </div>
                    <div className="flex items-center gap-3">
                        <ViewToggle view={view} setView={(v) => setView(v as typeof view)} />
                    </div>
                </CardContent>
            </Card>

            {/* Clarifications Alert */}
            {unresolvedCount > 0 && (
                <div className="shrink-0 mx-4 mt-2">
                    <Card className="border-yellow-500/50 bg-yellow-500/5">
                        <CardContent className="p-3 flex items-center justify-between">
                            <div className="flex items-center gap-2 text-yellow-400">
                                <Lightbulb size={18} />
                                <span className="text-sm font-medium">
                                    {unresolvedCount} clarification{unresolvedCount > 1 ? 's' : ''} needed
                                </span>
                            </div>
                            <Button
                                size="sm"
                                variant="outline"
                                className="border-yellow-500/50 text-yellow-400 hover:bg-yellow-500/10"
                                onClick={() => setShowClarifyModal(true)}
                            >
                                Resolve Now
                            </Button>
                        </CardContent>
                    </Card>
                </div>
            )}

            {/* Content with optional workflow panel */}
            <div className="flex-1 flex overflow-hidden">
                <div className="flex-1 overflow-auto p-4">
                    {view === 'visual' && (
                        <>
                            {/* Overview Section */}
                            <Section title="Overview" color="blue" defaultOpen={true}>
                                <p className="text-sm text-secondary-foreground whitespace-pre-wrap p-2">
                                    {spec.overview}
                                </p>
                            </Section>

                            {/* User Stories */}
                            <Section title="User Stories" count={spec.user_stories?.length || 0} color="purple">
                                <div className="space-y-2 mt-2">
                                    {spec.user_stories?.map((story) => (
                                        <UserStoryCard key={story.id} story={story} />
                                    ))}
                                </div>
                            </Section>

                            {/* Requirements */}
                            <Section title="Functional Requirements" count={functionalReqs.length} color="green">
                                <div className="space-y-2 mt-2">
                                    {functionalReqs.map((req) => (
                                        <div key={req.id} className="flex items-start gap-3 py-2 border-b border-border last:border-0">
                                            <Badge variant="info" className="font-mono text-xs flex-shrink-0">{req.id}</Badge>
                                            <span className="text-sm text-secondary-foreground">{req.description}</span>
                                        </div>
                                    ))}
                                </div>
                            </Section>

                            {/* Success Criteria */}
                            <Section title="Success Criteria" count={spec.success_criteria?.length || 0} color="cyan">
                                <div className="space-y-3 mt-2">
                                    {spec.success_criteria?.map((sc) => (
                                        <div key={sc.id} className="py-2 border-b border-border last:border-0">
                                            <div className="flex items-center gap-2">
                                                <Badge variant="secondary" className="font-mono text-xs">{sc.id}</Badge>
                                                <CheckCircle2 size={14} className="text-accent-green" />
                                            </div>
                                            <p className="text-sm text-secondary-foreground mt-1">{sc.metric}</p>
                                            {sc.measurement && (
                                                <p className="text-xs text-muted-foreground mt-1">
                                                    <span className="font-medium">Measurement:</span> {sc.measurement}
                                                </p>
                                            )}
                                        </div>
                                    ))}
                                </div>
                            </Section>

                            {/* Edge Cases */}
                            {spec.edge_cases && spec.edge_cases.length > 0 && (
                                <Section title="Edge Cases" count={spec.edge_cases.length} color="amber">
                                    <div className="space-y-1 mt-2">
                                        {spec.edge_cases.map((ec, i) => (
                                            <div key={i} className="flex items-start gap-2 py-1 text-sm text-muted-foreground">
                                                <AlertCircle size={14} className="mt-1 flex-shrink-0 text-accent-amber" />
                                                {ec}
                                            </div>
                                        ))}
                                    </div>
                                </Section>
                            )}

                            {/* Assumptions */}
                            {spec.assumptions && spec.assumptions.length > 0 && (
                                <Section title="Assumptions" count={spec.assumptions.length} color="slate">
                                    <div className="space-y-1 mt-2">
                                        {spec.assumptions.map((a, i) => (
                                            <div key={i} className="flex items-start gap-2 py-1 text-sm text-muted-foreground">
                                                <Circle size={8} className="mt-2 flex-shrink-0" />
                                                {a}
                                            </div>
                                        ))}
                                    </div>
                                </Section>
                            )}

                            {/* Clarifications */}
                            {spec.clarifications && spec.clarifications.length > 0 && (
                                <Section
                                    title="Clarifications"
                                    count={spec.clarifications.filter(c => !c.resolved).length}
                                    color="amber"
                                    icon={<Lightbulb size={16} className="text-yellow-400" />}
                                >
                                    <div className="space-y-3 mt-2">
                                        {spec.clarifications.map((cl, i) => (
                                            <div key={i} className={`p-3 rounded-lg border ${cl.resolved ? 'border-green-500/30 bg-green-500/5' : 'border-yellow-500/30 bg-yellow-500/5'}`}>
                                                <div className="flex items-center gap-2 mb-2">
                                                    <Badge variant={cl.resolved ? 'success' : 'warning'}>{cl.topic}</Badge>
                                                    {cl.resolved && <CheckCircle2 size={14} className="text-green-400" />}
                                                </div>
                                                <p className="text-sm text-foreground">{cl.question}</p>
                                                {cl.resolved && (
                                                    <p className="text-sm text-green-400 mt-2">
                                                        <span className="font-medium">Answer:</span> {cl.resolved}
                                                    </p>
                                                )}
                                            </div>
                                        ))}
                                    </div>
                                </Section>
                            )}
                        </>
                    )}

                    {view === 'outline' && (
                        <Card>
                            <CardContent className="p-6">
                                <OutlineView spec={spec} />
                            </CardContent>
                        </Card>
                    )}

                    {view === 'code' && (
                        <Card>
                            <CardHeader className="py-2 px-4 flex flex-row items-center justify-between border-b border-border">
                                <CardTitle className="text-sm">spec.yaml</CardTitle>
                            </CardHeader>
                            <CardContent className="p-0">
                                <YamlView rawYaml={rawYaml} />
                            </CardContent>
                        </Card>
                    )}
                </div>

                {/* Workflow Panel (collapsible sidebar) */}
                {showWorkflowPanel && selectedSpecName && (
                    <div className="w-80 shrink-0 border-l border-border overflow-auto p-4 bg-muted/20">
                        <div className="flex items-center justify-between mb-4">
                            <h3 className="font-semibold text-foreground">Workflow</h3>
                            <Button
                                variant="ghost"
                                size="icon"
                                className="h-6 w-6"
                                onClick={() => setShowWorkflowPanel(false)}
                            >
                                <X size={14} />
                            </Button>
                        </div>
                        <SpecWorkflow
                            specName={selectedSpecName}
                            unresolvedClarifications={unresolvedCount}
                            hasDesign={specsData?.specs.find(s => s.name === selectedSpecName)?.has_design ?? false}
                            hasTasks={specsData?.specs.find(s => s.name === selectedSpecName)?.has_tasks ?? false}
                            onClarifyClick={() => setShowClarifyModal(true)}
                            onDesignComplete={() => queryClient.invalidateQueries({ queryKey: ['specs'] })}
                            onTasksComplete={() => queryClient.invalidateQueries({ queryKey: ['specs'] })}
                        />
                    </div>
                )}
            </div>

            {/* Toggle workflow panel button when closed */}
            {!showWorkflowPanel && (
                <Button
                    variant="outline"
                    size="sm"
                    className="absolute right-4 top-20"
                    onClick={() => setShowWorkflowPanel(true)}
                >
                    <Lightbulb size={14} className="mr-1" />
                    Workflow
                </Button>
            )}

            {/* Clarify Modal */}
            <ClarifyModal
                open={showClarifyModal}
                onOpenChange={setShowClarifyModal}
                specName={selectedSpecName || ''}
                clarifications={clarifications}
                onSubmit={handleSubmitClarifications}
                isSubmitting={submitClarificationsMutation.isPending}
            />

            {/* Status Bar */}
            <div className="shrink-0 px-4 py-2 border-t border-border flex items-center justify-between text-sm text-muted-foreground bg-muted/50">
                <div className="flex items-center gap-4">
                    <span>{spec.user_stories?.length || 0} stories</span>
                    <span>{functionalReqs.length} requirements</span>
                    <span>{spec.success_criteria?.length || 0} success criteria</span>
                </div>
                <div className="flex items-center gap-2">
                    <span className="w-2 h-2 rounded-full bg-accent-green"></span>
                    <span>Read-only view</span>
                </div>
            </div>
        </div>
    );
};
