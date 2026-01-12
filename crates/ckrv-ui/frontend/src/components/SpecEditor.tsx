import React, { useState, useEffect } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
    ChevronDown, ChevronRight, Edit3, Check, X, Plus, Trash2,
    Code, LayoutGrid, List, AlertCircle, CheckCircle2, Circle,
    FileText, ArrowLeft, Save, Loader2, RotateCcw
} from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible';
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Input } from '@/components/ui/input';
import { Textarea } from '@/components/ui/textarea';

// Types matching backend
interface UserStoryAcceptance {
    given: string;
    when: string;
    then: string;
}

interface UserStory {
    id: string;
    title: string;
    priority: string;
    description: string;
    acceptance: UserStoryAcceptance[];
}

interface Requirement {
    id: string;
    description: string;
}

interface SuccessCriterion {
    id: string;
    metric: string;
}

interface SpecDetail {
    id: string;
    goal: string;
    constraints: string[];
    acceptance: string[];
    user_stories: UserStory[];
    requirements: Requirement[];
    success_criteria: SuccessCriterion[];
    assumptions: string[];
}

interface SpecListItem {
    name: string;
    path: string;
    has_tasks: boolean;
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

const saveSpec = async (name: string, spec: SpecDetail): Promise<{ success: boolean; message?: string }> => {
    const res = await fetch('/api/specs/save', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name, spec }),
    });
    return res.json();
};

// Collapsible Section Component using shadcn Collapsible
const Section: React.FC<{
    title: string;
    count?: number;
    children: React.ReactNode;
    defaultOpen?: boolean;
    color?: 'slate' | 'blue' | 'green' | 'amber' | 'purple';
}> = ({ title, count, children, defaultOpen = true, color = 'slate' }) => {
    const [isOpen, setIsOpen] = useState(defaultOpen);
    const colorClasses = {
        slate: 'border-border bg-muted/50',
        blue: 'border-[var(--accent-cyan)] bg-[var(--accent-cyan-dim)]',
        green: 'border-[var(--accent-green)] bg-[var(--accent-green-dim)]',
        amber: 'border-[var(--accent-amber)] bg-[var(--accent-amber-dim)]',
        purple: 'border-[var(--accent-purple)] bg-[var(--accent-purple-dim)]'
    };

    return (
        <Collapsible open={isOpen} onOpenChange={setIsOpen}>
            <Card className={`mb-3 ${colorClasses[color]}`}>
                <CollapsibleTrigger asChild>
                    <button className="w-full px-4 py-3 flex items-center justify-between text-left font-medium text-foreground hover:bg-accent/50 transition-colors rounded-t-lg">
                        <div className="flex items-center gap-2">
                            {isOpen ? <ChevronDown size={18} /> : <ChevronRight size={18} />}
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

// Editable Text Component
const EditableText: React.FC<{
    value: string;
    onChange: (value: string) => void;
    multiline?: boolean;
    className?: string;
}> = ({ value, onChange, multiline = false, className = '' }) => {
    const [isEditing, setIsEditing] = useState(false);
    const [editValue, setEditValue] = useState(value);

    useEffect(() => setEditValue(value), [value]);

    const save = () => {
        onChange(editValue);
        setIsEditing(false);
    };

    const cancel = () => {
        setEditValue(value);
        setIsEditing(false);
    };

    if (isEditing) {
        return (
            <div className="flex gap-2 items-start">
                {multiline ? (
                    <Textarea
                        value={editValue}
                        onChange={(e: React.ChangeEvent<HTMLTextAreaElement>) => setEditValue(e.target.value)}
                        className="flex-1 min-h-[80px]"
                        autoFocus
                    />
                ) : (
                    <Input
                        value={editValue}
                        onChange={(e) => setEditValue(e.target.value)}
                        className="flex-1"
                        autoFocus
                    />
                )}
                <Button variant="ghost" size="icon" onClick={save} className="h-8 w-8 text-[var(--accent-green)]">
                    <Check size={16} />
                </Button>
                <Button variant="ghost" size="icon" onClick={cancel} className="h-8 w-8 text-destructive">
                    <X size={16} />
                </Button>
            </div>
        );
    }

    return (
        <div
            onClick={() => setIsEditing(true)}
            className={`cursor-pointer hover:bg-accent p-2 rounded border border-transparent hover:border-border transition-all group ${className}`}
        >
            <span className="text-sm text-secondary-foreground whitespace-pre-wrap">{value}</span>
            <Edit3 size={14} className="inline ml-2 text-muted-foreground opacity-0 group-hover:opacity-100 transition-opacity" />
        </div>
    );
};

// Priority Badge using shadcn Badge
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

// User Story Card using shadcn Card
const UserStoryCard: React.FC<{
    story: UserStory;
    onUpdate: (story: UserStory) => void;
}> = ({ story, onUpdate }) => {
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
                            <EditableText
                                value={story.description}
                                onChange={(v) => onUpdate({ ...story, description: v })}
                                multiline
                            />
                        </div>
                        {story.acceptance && story.acceptance.length > 0 && (
                            <div>
                                <label className="text-xs text-muted-foreground uppercase tracking-wide">Acceptance Criteria</label>
                                {story.acceptance.map((ac, i) => (
                                    <div key={i} className="ml-2 mt-2 text-sm text-muted-foreground space-y-1">
                                        <div><span className="text-[var(--accent-purple)]">Given:</span> {ac.given}</div>
                                        <div><span className="text-[var(--accent-cyan)]">When:</span> {ac.when}</div>
                                        <div><span className="text-[var(--accent-green)]">Then:</span> {ac.then}</div>
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

// List Item with inline edit
const ListItem: React.FC<{
    value: string;
    onUpdate: (value: string) => void;
    onDelete: () => void;
    icon?: React.ElementType;
}> = ({ value, onUpdate, onDelete, icon: Icon = Circle }) => (
    <div className="flex items-start gap-2 group py-1">
        <Icon size={16} className="text-muted-foreground mt-1.5 flex-shrink-0" />
        <EditableText value={value} onChange={onUpdate} className="flex-1" />
        <Button
            variant="ghost"
            size="icon"
            className="h-6 w-6 opacity-0 group-hover:opacity-100 transition-opacity text-muted-foreground hover:text-destructive"
            onClick={onDelete}
        >
            <Trash2 size={14} />
        </Button>
    </div>
);

// Requirement Row
const RequirementRow: React.FC<{
    req: Requirement;
    onUpdate: (req: Requirement) => void;
}> = ({ req, onUpdate }) => (
    <div className="flex items-start gap-3 py-2 border-b border-border last:border-0">
        <Badge variant="info" className="font-mono text-xs flex-shrink-0">{req.id}</Badge>
        <EditableText
            value={req.description}
            onChange={(v) => onUpdate({ ...req, description: v })}
            className="flex-1"
        />
    </div>
);

// View Mode Toggle using shadcn Tabs
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
            <div><span className="text-[var(--accent-purple)]">id:</span> <span className="text-foreground">{spec.id}</span></div>
            <div><span className="text-[var(--accent-purple)]">goal:</span> <span className="text-muted-foreground truncate inline-block max-w-md">{spec.goal.slice(0, 80)}...</span></div>
            <div className="mt-2">
                <span className="text-[var(--accent-purple)]">constraints:</span> <span className="text-muted-foreground">({spec.constraints.length})</span>
            </div>
            <div className="mt-2">
                <span className="text-[var(--accent-purple)]">acceptance:</span> <span className="text-muted-foreground">({spec.acceptance.length})</span>
            </div>
            <div className="mt-2">
                <span className="text-[var(--accent-purple)]">user_stories:</span> <span className="text-muted-foreground">({spec.user_stories.length})</span>
                {spec.user_stories.map(s => (
                    <div key={s.id} className="pl-4 text-muted-foreground">- {s.id}: {s.title}</div>
                ))}
            </div>
            <div className="mt-2">
                <span className="text-[var(--accent-purple)]">requirements:</span> <span className="text-muted-foreground">({spec.requirements.length})</span>
            </div>
        </div>
    </div>
);

// YAML View
const YamlView: React.FC<{ rawYaml?: string }> = ({ rawYaml }) => (
    <pre className="font-mono text-sm bg-muted text-foreground p-4 rounded-lg overflow-auto max-h-[60vh]">
        <code>{rawYaml || '# No YAML content'}</code>
    </pre>
);

// Spec List View using shadcn Card
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
                                        {spec.has_implementation && (
                                            <Badge variant="info">
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
    const [hasChanges, setHasChanges] = useState(false);

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
            setHasChanges(false);
        }
    }, [specDetailData]);

    // Save mutation
    const saveMutation = useMutation({
        mutationFn: () => saveSpec(selectedSpecName!, spec!),
        onSuccess: (data: { success: boolean; message?: string }) => {
            if (data.success) {
                setHasChanges(false);
                queryClient.invalidateQueries({ queryKey: ['specs'] });
            }
        },
    });

    const updateSpec = (updates: Partial<SpecDetail>) => {
        if (spec) {
            setSpec({ ...spec, ...updates });
            setHasChanges(true);
        }
    };

    const updateConstraint = (index: number, value: string) => {
        if (spec) {
            const newConstraints = [...spec.constraints];
            newConstraints[index] = value;
            updateSpec({ constraints: newConstraints });
        }
    };

    const updateAcceptance = (index: number, value: string) => {
        if (spec) {
            const newAcceptance = [...spec.acceptance];
            newAcceptance[index] = value;
            updateSpec({ acceptance: newAcceptance });
        }
    };

    const updateStory = (index: number, story: UserStory) => {
        if (spec) {
            const newStories = [...spec.user_stories];
            newStories[index] = story;
            updateSpec({ user_stories: newStories });
        }
    };

    const updateRequirement = (index: number, req: Requirement) => {
        if (spec) {
            const newReqs = [...spec.requirements];
            newReqs[index] = req;
            updateSpec({ requirements: newReqs });
        }
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
                    </div>
                    <div className="flex items-center gap-3">
                        <ViewToggle view={view} setView={(v) => setView(v as typeof view)} />
                        {hasChanges && (
                            <Button
                                variant="outline"
                                onClick={() => {
                                    if (specDetailData?.spec) {
                                        setSpec(specDetailData.spec);
                                        setRawYaml(specDetailData.raw_yaml);
                                        setHasChanges(false);
                                    }
                                }}
                            >
                                <RotateCcw size={16} className="mr-2" />
                                Discard
                            </Button>
                        )}
                        <Button
                            onClick={() => saveMutation.mutate()}
                            disabled={!hasChanges || saveMutation.isPending}
                        >
                            {saveMutation.isPending ? (
                                <Loader2 size={16} className="mr-2 animate-spin" />
                            ) : (
                                <Save size={16} className="mr-2" />
                            )}
                            Save
                        </Button>
                    </div>
                </CardContent>
            </Card>

            {/* Content */}
            <div className="flex-1 overflow-auto p-4">
                {view === 'visual' && (
                    <>
                        {/* Goal Section */}
                        <Section title="Goal" color="blue" defaultOpen={true}>
                            <EditableText
                                value={spec.goal}
                                onChange={(v) => updateSpec({ goal: v })}
                                multiline
                            />
                        </Section>

                        {/* Constraints */}
                        <Section title="Constraints" count={spec.constraints.length} color="amber">
                            <div className="space-y-1 mt-2">
                                {spec.constraints.map((c, i) => (
                                    <ListItem
                                        key={i}
                                        value={c}
                                        onUpdate={(v) => updateConstraint(i, v)}
                                        onDelete={() => updateSpec({ constraints: spec.constraints.filter((_, j) => j !== i) })}
                                        icon={AlertCircle}
                                    />
                                ))}
                                <Button
                                    variant="ghost"
                                    size="sm"
                                    onClick={() => updateSpec({ constraints: [...spec.constraints, 'New constraint'] })}
                                    className="mt-2"
                                >
                                    <Plus size={16} className="mr-2" /> Add constraint
                                </Button>
                            </div>
                        </Section>

                        {/* Acceptance Criteria */}
                        <Section title="Acceptance Criteria" count={spec.acceptance.length} color="green">
                            <div className="space-y-1 mt-2">
                                {spec.acceptance.map((a, i) => (
                                    <ListItem
                                        key={i}
                                        value={a}
                                        onUpdate={(v) => updateAcceptance(i, v)}
                                        onDelete={() => updateSpec({ acceptance: spec.acceptance.filter((_, j) => j !== i) })}
                                        icon={CheckCircle2}
                                    />
                                ))}
                                <Button
                                    variant="ghost"
                                    size="sm"
                                    onClick={() => updateSpec({ acceptance: [...spec.acceptance, 'New acceptance criterion'] })}
                                    className="mt-2"
                                >
                                    <Plus size={16} className="mr-2" /> Add criterion
                                </Button>
                            </div>
                        </Section>

                        {/* User Stories */}
                        <Section title="User Stories" count={spec.user_stories.length} color="purple">
                            <div className="grid gap-3 mt-2">
                                {spec.user_stories.map((story, i) => (
                                    <UserStoryCard
                                        key={story.id}
                                        story={story}
                                        onUpdate={(s) => updateStory(i, s)}
                                    />
                                ))}
                            </div>
                        </Section>

                        {/* Requirements */}
                        <Section title="Requirements" count={spec.requirements.length} color="slate">
                            <div className="mt-2">
                                {spec.requirements.map((req, i) => (
                                    <RequirementRow
                                        key={req.id}
                                        req={req}
                                        onUpdate={(r) => updateRequirement(i, r)}
                                    />
                                ))}
                            </div>
                        </Section>

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
                            <Button variant="ghost" size="sm">Copy</Button>
                        </CardHeader>
                        <CardContent className="p-0">
                            <YamlView rawYaml={rawYaml} />
                        </CardContent>
                    </Card>
                )}
            </div>

            {/* Status Bar */}
            <div className="shrink-0 px-4 py-2 border-t border-border flex items-center justify-between text-sm text-muted-foreground bg-muted/50">
                <div className="flex items-center gap-4">
                    <span>{spec.user_stories.length} stories</span>
                    <span>{spec.requirements.length} requirements</span>
                    <span>{spec.constraints.length} constraints</span>
                </div>
                <div className="flex items-center gap-2">
                    <span className={`w-2 h-2 rounded-full ${hasChanges ? 'bg-[var(--accent-amber)]' : 'bg-[var(--accent-green)]'}`}></span>
                    <span>{hasChanges ? 'Unsaved changes' : 'All changes saved'}</span>
                </div>
            </div>
        </div>
    );
};
