/**
 * @module SpecEditor
 * @description
 * Read-only viewer for feature specifications. Displays spec content in visual,
 * outline, or YAML views. Shows user stories, requirements, success criteria,
 * and clarifications. Includes a workflow panel for spec-to-tasks progression.
 *
 * @context
 * Rendered as the main content of the Spec page in the dashboard. Users review
 * specs, resolve clarifications, and trigger design/task generation here.
 * Auto-selects spec based on current git branch.
 *
 * @dependencies
 * - useAutoSelectedSpec: Auto-selects spec based on current git branch
 * - useClarifications: Hook for fetching and submitting clarifications
 * - SpecWorkflow: Sidebar panel showing spec-to-implementation workflow
 * - ClarifyModal: Modal for resolving spec clarifications
 * - shadcn/ui components: Card, Badge, Tabs, Collapsible for consistent UI
 *
 * @example
 * // Rendered directly as a page component
 * <SpecEditor />
 *
 * // Displays spec in visual, outline, or YAML view
 * // Includes workflow sidebar for design and task generation
 */

// === IMPORTS ===
import React, { useState, useEffect } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { useAutoSelectedSpec } from '../hooks/useAutoSelectedSpec';
import {
    ChevronDown, ChevronRight,
    FileText, Loader2, AlertCircle, CheckCircle2,
    Circle, Lightbulb, GitBranch, Calendar, Tag, Sparkles
} from 'lucide-react';
import { Card, CardContent } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible';
import { ClarifyModal } from './ClarifyModal';
import { useClarifications, useSubmitClarifications, useValidateSpec, useGenerateDesign, useGenerateTasks, type Clarification } from '../hooks/useSpec';
import { toast } from 'sonner';

// === TYPES ===

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

/**
 * Detailed specification data from spec.yaml.
 * Contains the full spec content for display and editing.
 * 
 * @example
 * const spec: SpecDetail = {
 *   id: '018-frontend-docs',
 *   branch: '018-frontend-docs',
 *   overview: 'Add documentation to frontend components',
 *   user_stories: [...],
 *   requirements: { functional: [...] },
 *   success_criteria: [...]
 * };
 */
interface SpecDetail {
    /** Unique spec identifier (e.g., 018-frontend-docs) */
    id: string;
    /** Git branch for this spec */
    branch?: string;
    /** Creation timestamp */
    created?: string;
    /** Spec status: draft, ready, in_progress, done */
    status?: string;
    /** High-level spec overview (2-4 sentences) */
    overview: string;
    /** User stories describing the feature from user perspective */
    user_stories: UserStory[];
    /** Functional and non-functional requirements */
    requirements: Requirements;
    /** Measurable success criteria */
    success_criteria: SuccessCriterion[];
    /** Edge cases to consider */
    edge_cases?: string[];
    /** Assumptions made during spec creation */
    assumptions?: string[];
    /** Clarification questions for ambiguous areas */
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

// === API FUNCTIONS ===

const fetchSpecs = async (): Promise<{ specs: SpecListItem[], count: number }> => {
    const res = await fetch('/api/specs');
    return res.json();
};

const fetchSpecDetail = async (name: string): Promise<{ success: boolean; spec?: SpecDetail; raw_yaml?: string; error?: string }> => {
    const res = await fetch(`/api/specs/detail?name=${encodeURIComponent(name)}`);
    return res.json();
};

// === SUB-COMPONENTS ===

// Collapsible Section Component
/**
 * Props for Section component.
 * Collapsible card section with a title and optional count badge.
 */
interface SectionProps {
    /** Section title displayed in the header */
    title: string;
    /** Optional count to display as a badge */
    count?: number;
    /** Content to show when expanded */
    children: React.ReactNode;
    /** Whether the section starts expanded. @default true */
    defaultOpen?: boolean;
    /** Color theme for the section border and background. @default 'slate' */
    color?: 'slate' | 'blue' | 'green' | 'amber' | 'purple' | 'cyan';
    /** Optional icon to display before the title */
    icon?: React.ReactNode;
}

const Section: React.FC<SectionProps> = ({ title, count, children, defaultOpen = true, color = 'slate', icon }) => {
    /** Whether this collapsible section is expanded */
    const [isOpen, setIsOpen] = useState(defaultOpen);
    const colorClasses = {
        slate: 'border-border bg-muted/50',
        blue: 'border-border bg-muted/50',
        green: 'border-border bg-muted/50',
        amber: 'border-border bg-muted/50',
        purple: 'border-border bg-muted/50',
        cyan: 'border-border bg-muted/50'
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
/**
 * Props for PriorityBadge component.
 * Displays priority level with color-coded styling.
 */
interface PriorityBadgeProps {
    /** Priority level: P1 (critical), P2 (major), or P3 (minor) */
    priority: string;
}

const PriorityBadge: React.FC<PriorityBadgeProps> = ({ priority }) => {
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
/**
 * Props for UserStoryCard component.
 * Expandable card displaying a user story with details and acceptance criteria.
 */
interface UserStoryCardProps {
    /** User story data to display */
    story: UserStory;
}

const UserStoryCard: React.FC<UserStoryCardProps> = ({ story }) => {
    /** Whether this user story card is expanded to show details */
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
                            <p className="text-sm text-foreground mt-1 whitespace-pre-wrap">{story.description}</p>
                        </div>
                        {story.why_priority && (
                            <div>
                                <label className="text-xs text-muted-foreground uppercase tracking-wide">Why Priority</label>
                                <p className="text-sm text-foreground mt-1">{story.why_priority}</p>
                            </div>
                        )}
                        {story.independent_test && (
                            <div>
                                <label className="text-xs text-muted-foreground uppercase tracking-wide">Independent Test</label>
                                <p className="text-sm text-foreground mt-1">{story.independent_test}</p>
                            </div>
                        )}
                        {story.acceptance_scenarios && story.acceptance_scenarios.length > 0 && (
                            <div>
                                <label className="text-xs text-muted-foreground uppercase tracking-wide">Acceptance Scenarios</label>
                                {story.acceptance_scenarios.map((ac, i) => (
                                    <div key={i} className="ml-2 mt-2 text-sm text-muted-foreground space-y-1 p-2 bg-muted/30 rounded">
                                        <div><span className="text-primary font-medium">Given:</span> {ac.given}</div>
                                        <div><span className="text-info font-medium">When:</span> {ac.when}</div>
                                        <div><span className="text-success font-medium">Then:</span> {ac.then}</div>
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

// Spec List View
/**
 * Props for SpecListView component.
 * Displays a list of specs for selection when no spec is auto-selected.
 */
interface SpecListViewProps {
    /** List of specs to display */
    specs: SpecListItem[];
    /** Callback when a spec is selected */
    onSelect: (name: string) => void;
    /** Whether the spec list is currently loading */
    isLoading: boolean;
}

const SpecListView: React.FC<SpecListViewProps> = ({ specs, onSelect, isLoading }) => {
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

// === MAIN COMPONENT ===

export const SpecEditor: React.FC = () => {
    const queryClient = useQueryClient();

    // === STATE ===

    // --- Spec Selection ---
    // Auto-select spec based on current branch
    const { selectedSpec: autoSelectedSpec, isLoading: isLoadingAutoSpec } = useAutoSelectedSpec();

    /** Manual spec override when user explicitly selects a different spec */
    const [manualSpecOverride, setManualSpecOverride] = useState<string | null>(null);
    const selectedSpecName = manualSpecOverride ?? autoSelectedSpec;

    // --- Spec Data ---
    /** Loaded spec detail for display */
    const [spec, setSpec] = useState<SpecDetail | null>(null);

    // --- UI State ---
    /** Show the clarification modal for answering spec questions */
    const [showClarifyModal, setShowClarifyModal] = useState(false);

    // --- Workflow State ---
    /** Result from spec validation (valid flag and errors list) */
    const [validationResult, setValidationResult] = useState<{ valid: boolean; errors: string[] } | null>(null);
    /** Track if an async workflow operation is in progress */
    const [isWorkflowProcessing, setIsWorkflowProcessing] = useState(false);

    // === QUERIES ===

    // Fetch specs list (for manual selection fallback)
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

    // === EFFECTS ===

    /**
     * Updates local spec state when spec detail data is fetched.
     * Syncs the spec and rawYaml from the query result to local state.
     */
    useEffect(() => {
        if (specDetailData?.success && specDetailData.spec) {
            // eslint-disable-next-line react-hooks/set-state-in-effect
            setSpec(specDetailData.spec);
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

    // --- Workflow Mutations ---
    const validateMutation = useValidateSpec();
    const designMutation = useGenerateDesign();
    const tasksMutation = useGenerateTasks();

    const hasDesign = specsData?.specs.find(s => s.name === selectedSpecName)?.has_design ?? false;
    const hasTasks = specsData?.specs.find(s => s.name === selectedSpecName)?.has_tasks ?? false;
    const needsClarification = unresolvedCount > 0;
    const canDesign = !needsClarification && !hasDesign;
    const canGenerateTasks = hasDesign && !hasTasks;

    const handleValidate = async () => {
        if (!selectedSpecName) return;
        setIsWorkflowProcessing(true);
        try {
            const result = await validateMutation.mutateAsync(selectedSpecName);
            setValidationResult({ valid: result.valid, errors: result.errors.map((e: { field: string; message: string }) => `${e.field}: ${e.message}`) });
        } catch (e) {
            toast.error('Validation failed', { description: e instanceof Error ? e.message : 'Unknown error' });
        } finally {
            setIsWorkflowProcessing(false);
        }
    };

    const handleDesign = async () => {
        if (!selectedSpecName) return;
        setIsWorkflowProcessing(true);
        try {
            await designMutation.mutateAsync(selectedSpecName);
            toast.success('Design Generated', { description: 'design.md has been created successfully' });
            queryClient.invalidateQueries({ queryKey: ['specs'] });
        } catch (e) {
            toast.error('Design Generation Failed', { description: e instanceof Error ? e.message : 'Unknown error' });
        } finally {
            setIsWorkflowProcessing(false);
        }
    };

    const handleTasks = async () => {
        if (!selectedSpecName) return;
        setIsWorkflowProcessing(true);
        try {
            await tasksMutation.mutateAsync(selectedSpecName);
            toast.success('Tasks Generated', { description: 'tasks.yaml has been created successfully' });
            queryClient.invalidateQueries({ queryKey: ['specs'] });
        } catch (e) {
            toast.error('Tasks Generation Failed', { description: e instanceof Error ? e.message : 'Unknown error' });
        } finally {
            setIsWorkflowProcessing(false);
        }
    };

    // Show spec list if nothing selected (neither auto nor manual)
    if (!selectedSpecName) {
        // If still loading auto-selection, show spinner
        if (isLoadingAutoSpec) {
            return (
                <div className="flex items-center justify-center h-full">
                    <Loader2 className="animate-spin text-muted-foreground" size={32} />
                </div>
            );
        }

        return (
            <div className="h-full overflow-auto p-4">
                <div className="mb-6">
                    <h1 className="text-2xl font-bold text-foreground">Specifications</h1>
                    <p className="text-muted-foreground mt-1">No spec matches the current branch. Select a spec to view and edit.</p>
                </div>
                <SpecListView
                    specs={specsData?.specs || []}
                    onSelect={setManualSpecOverride}
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
                    {/* Workflow Actions */}
                    <div className="flex items-center gap-1.5">
                        <Button
                            variant="outline"
                            size="sm"
                            onClick={handleValidate}
                            disabled={isWorkflowProcessing}
                            className={validationResult?.valid ? 'border-success/50 text-success' : ''}
                        >
                            {isWorkflowProcessing && validateMutation.isPending ? (
                                <Loader2 size={14} className="mr-1.5 animate-spin" />
                            ) : validationResult?.valid ? (
                                <CheckCircle2 size={14} className="mr-1.5" />
                            ) : (
                                <CheckCircle2 size={14} className="mr-1.5" />
                            )}
                            Validate
                        </Button>

                        <div className="w-px h-5 bg-border" />

                        {needsClarification ? (
                            <Button
                                variant="outline"
                                size="sm"
                                onClick={() => setShowClarifyModal(true)}
                                className="border-warning/50 text-warning hover:bg-warning/10"
                            >
                                <Lightbulb size={14} className="mr-1.5" />
                                Clarify ({unresolvedCount})
                            </Button>
                        ) : (
                            <Button variant="ghost" size="sm" disabled className="text-success gap-1.5">
                                <CheckCircle2 size={14} />
                                Clarified
                            </Button>
                        )}

                        {hasDesign ? (
                            <Button variant="ghost" size="sm" disabled className="text-success gap-1.5">
                                <CheckCircle2 size={14} />
                                Design
                            </Button>
                        ) : canDesign ? (
                            <Button
                                size="sm"
                                onClick={handleDesign}
                                disabled={isWorkflowProcessing}
                            >
                                {isWorkflowProcessing && designMutation.isPending ? (
                                    <Loader2 size={14} className="mr-1.5 animate-spin" />
                                ) : (
                                    <Sparkles size={14} className="mr-1.5" />
                                )}
                                Design
                            </Button>
                        ) : (
                            <Button variant="ghost" size="sm" disabled className="opacity-40 gap-1.5">
                                <FileText size={14} />
                                Design
                            </Button>
                        )}

                        {hasTasks ? (
                            <Button variant="ghost" size="sm" disabled className="text-success gap-1.5">
                                <CheckCircle2 size={14} />
                                Tasks
                            </Button>
                        ) : canGenerateTasks ? (
                            <Button
                                size="sm"
                                onClick={handleTasks}
                                disabled={isWorkflowProcessing}
                            >
                                {isWorkflowProcessing && tasksMutation.isPending ? (
                                    <Loader2 size={14} className="mr-1.5 animate-spin" />
                                ) : (
                                    <Sparkles size={14} className="mr-1.5" />
                                )}
                                Tasks
                            </Button>
                        ) : (
                            <Button variant="ghost" size="sm" disabled className="opacity-40 gap-1.5">
                                <FileText size={14} />
                                Tasks
                            </Button>
                        )}
                    </div>
                </CardContent>
            </Card>

            {/* Validation Result Banner */}
            {validationResult && !validationResult.valid && (
                <div className="shrink-0 mx-4 mt-2">
                    <Card className="border-warning/50 bg-warning/5">
                        <CardContent className="p-3 text-sm text-warning">
                            <AlertCircle className="w-4 h-4 inline mr-2" />
                            Validation failed:
                            <ul className="mt-1 ml-6 list-disc">
                                {validationResult.errors.map((err, i) => (
                                    <li key={i}>{err}</li>
                                ))}
                            </ul>
                        </CardContent>
                    </Card>
                </div>
            )}

            {/* Content */}
            <div className="flex-1 overflow-auto p-4">
                            {/* Overview Section */}
                            <Section title="Overview" color="blue" defaultOpen={true}>
                                <p className="text-sm text-foreground whitespace-pre-wrap p-2">
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
                                            <span className="text-sm text-foreground">{req.description}</span>
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
                                                <CheckCircle2 size={14} className="text-success" />
                                            </div>
                                            <p className="text-sm text-foreground mt-1">{sc.metric}</p>
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
                                                <AlertCircle size={14} className="mt-1 flex-shrink-0 text-warning" />
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
                                    icon={<Lightbulb size={16} className="text-warning" />}
                                >
                                    <div className="space-y-3 mt-2">
                                        {spec.clarifications.map((cl, i) => (
                                            <div key={i} className={`p-3 rounded-lg border ${cl.resolved ? 'border-success/30 bg-success/5' : 'border-warning/30 bg-warning/5'}`}>
                                                <div className="flex items-center gap-2 mb-2">
                                                    <Badge variant={cl.resolved ? 'success' : 'warning'}>{cl.topic}</Badge>
                                                    {cl.resolved && <CheckCircle2 size={14} className="text-success" />}
                                                </div>
                                                <p className="text-sm text-foreground">{cl.question}</p>
                                                {cl.resolved && (
                                                    <p className="text-sm text-success mt-2">
                                                        <span className="font-medium">Answer:</span> {cl.resolved}
                                                    </p>
                                                )}
                                            </div>
                                        ))}
                                    </div>
                                </Section>
                            )}
            </div>

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
                    <span className="w-2 h-2 rounded-full bg-success"></span>
                    <span>Read-only view</span>
                </div>
            </div>
        </div>
    );
};
