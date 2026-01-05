import React, { useState, useEffect } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
    ChevronDown, ChevronRight, Edit3, Check, X, Plus, Trash2,
    Code, LayoutGrid, List, AlertCircle, CheckCircle2, Circle,
    FileText, ArrowLeft, Save, Loader2, RotateCcw
} from 'lucide-react';

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

// Collapsible Section Component
const Section: React.FC<{
    title: string;
    count?: number;
    children: React.ReactNode;
    defaultOpen?: boolean;
    color?: 'slate' | 'blue' | 'green' | 'amber' | 'purple';
}> = ({ title, count, children, defaultOpen = true, color = 'slate' }) => {
    const [isOpen, setIsOpen] = useState(defaultOpen);
    const colorClasses = {
        slate: 'border-gray-600 bg-gray-800/50',
        blue: 'border-blue-600 bg-blue-900/30',
        green: 'border-green-600 bg-green-900/30',
        amber: 'border-amber-600 bg-amber-900/30',
        purple: 'border-purple-600 bg-purple-900/30'
    };

    return (
        <div className={`border rounded-lg mb-3 ${colorClasses[color]}`}>
            <button
                onClick={() => setIsOpen(!isOpen)}
                className="w-full px-4 py-3 flex items-center justify-between text-left font-medium text-gray-200 hover:bg-white/5 transition-colors rounded-t-lg"
            >
                <div className="flex items-center gap-2">
                    {isOpen ? <ChevronDown size={18} /> : <ChevronRight size={18} />}
                    <span>{title}</span>
                    {count !== undefined && (
                        <span className="text-xs bg-gray-700 px-2 py-0.5 rounded-full text-gray-300">{count}</span>
                    )}
                </div>
            </button>
            {isOpen && <div className="px-4 pb-4 bg-gray-900/50 rounded-b-lg border-t border-gray-700">{children}</div>}
        </div>
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
                    <textarea
                        value={editValue}
                        onChange={(e) => setEditValue(e.target.value)}
                        className="flex-1 p-2 border rounded text-sm min-h-[80px] bg-gray-800 border-gray-600 text-gray-200 focus:ring-2 focus:ring-cyan-500 focus:border-cyan-500 outline-none"
                        autoFocus
                    />
                ) : (
                    <input
                        value={editValue}
                        onChange={(e) => setEditValue(e.target.value)}
                        className="flex-1 p-2 border rounded text-sm bg-gray-800 border-gray-600 text-gray-200 focus:ring-2 focus:ring-cyan-500 focus:border-cyan-500 outline-none"
                        autoFocus
                    />
                )}
                <button onClick={save} className="p-1.5 text-green-400 hover:bg-green-900/50 rounded"><Check size={16} /></button>
                <button onClick={cancel} className="p-1.5 text-red-400 hover:bg-red-900/50 rounded"><X size={16} /></button>
            </div>
        );
    }

    return (
        <div
            onClick={() => setIsEditing(true)}
            className={`cursor-pointer hover:bg-gray-800 p-2 rounded border border-transparent hover:border-gray-600 transition-all group ${className}`}
        >
            <span className="text-sm text-gray-300 whitespace-pre-wrap">{value}</span>
            <Edit3 size={14} className="inline ml-2 text-gray-500 opacity-0 group-hover:opacity-100 transition-opacity" />
        </div>
    );
};

// Priority Badge
const PriorityBadge: React.FC<{ priority: string }> = ({ priority }) => {
    const colors: Record<string, string> = {
        P1: 'bg-red-900/50 text-red-300 border-red-700',
        P2: 'bg-amber-900/50 text-amber-300 border-amber-700',
        P3: 'bg-green-900/50 text-green-300 border-green-700'
    };
    return (
        <span className={`text-xs font-medium px-2 py-0.5 rounded border ${colors[priority] || colors.P3}`}>
            {priority}
        </span>
    );
};

// User Story Card
const UserStoryCard: React.FC<{
    story: UserStory;
    onUpdate: (story: UserStory) => void;
}> = ({ story, onUpdate }) => {
    const [expanded, setExpanded] = useState(false);

    return (
        <div className="border border-gray-700 rounded-lg bg-gray-800/50 hover:bg-gray-800 transition-colors">
            <div className="p-4">
                <div className="flex items-start justify-between gap-3">
                    <div className="flex items-center gap-2">
                        <span className="text-xs font-mono text-gray-500">{story.id}</span>
                        <PriorityBadge priority={story.priority} />
                    </div>
                    <button
                        onClick={() => setExpanded(!expanded)}
                        className="text-gray-500 hover:text-gray-300"
                    >
                        {expanded ? <ChevronDown size={18} /> : <ChevronRight size={18} />}
                    </button>
                </div>
                <h4 className="font-medium text-gray-200 mt-2">{story.title}</h4>
                {expanded && (
                    <div className="mt-3 pt-3 border-t border-gray-700 space-y-3">
                        <div>
                            <label className="text-xs text-gray-500 uppercase tracking-wide">Description</label>
                            <EditableText
                                value={story.description}
                                onChange={(v) => onUpdate({ ...story, description: v })}
                                multiline
                            />
                        </div>
                        {story.acceptance && story.acceptance.length > 0 && (
                            <div>
                                <label className="text-xs text-gray-500 uppercase tracking-wide">Acceptance Criteria</label>
                                {story.acceptance.map((ac, i) => (
                                    <div key={i} className="ml-2 mt-2 text-sm text-gray-400 space-y-1">
                                        <div><span className="text-purple-400">Given:</span> {ac.given}</div>
                                        <div><span className="text-cyan-400">When:</span> {ac.when}</div>
                                        <div><span className="text-green-400">Then:</span> {ac.then}</div>
                                    </div>
                                ))}
                            </div>
                        )}
                    </div>
                )}
            </div>
        </div>
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
        <Icon size={16} className="text-gray-500 mt-1.5 flex-shrink-0" />
        <EditableText value={value} onChange={onUpdate} className="flex-1" />
        <button
            onClick={onDelete}
            className="p-1 text-gray-500 hover:text-red-400 opacity-0 group-hover:opacity-100 transition-opacity"
        >
            <Trash2 size={14} />
        </button>
    </div>
);

// Requirement Row
const RequirementRow: React.FC<{
    req: Requirement;
    onUpdate: (req: Requirement) => void;
}> = ({ req, onUpdate }) => (
    <div className="flex items-start gap-3 py-2 border-b border-gray-700 last:border-0">
        <span className="font-mono text-xs text-cyan-400 bg-cyan-900/30 px-2 py-1 rounded flex-shrink-0">{req.id}</span>
        <EditableText
            value={req.description}
            onChange={(v) => onUpdate({ ...req, description: v })}
            className="flex-1"
        />
    </div>
);

// View Mode Toggle
const ViewToggle: React.FC<{
    view: string;
    setView: (v: string) => void;
}> = ({ view, setView }) => (
    <div className="flex bg-gray-800 rounded-lg p-1 gap-1">
        {[
            { id: 'visual', icon: LayoutGrid, label: 'Visual' },
            { id: 'outline', icon: List, label: 'Outline' },
            { id: 'code', icon: Code, label: 'YAML' }
        ].map(({ id, icon: Icon, label }) => (
            <button
                key={id}
                onClick={() => setView(id)}
                className={`flex items-center gap-1.5 px-3 py-1.5 rounded text-sm font-medium transition-all ${view === id
                    ? 'bg-gray-700 text-white shadow-sm'
                    : 'text-gray-400 hover:text-gray-200'
                    }`}
            >
                <Icon size={16} />
                {label}
            </button>
        ))}
    </div>
);

// Outline View
const OutlineView: React.FC<{ spec: SpecDetail }> = ({ spec }) => (
    <div className="font-mono text-sm space-y-1">
        <div className="text-gray-500">spec:</div>
        <div className="pl-4">
            <div><span className="text-purple-400">id:</span> <span className="text-gray-300">{spec.id}</span></div>
            <div><span className="text-purple-400">goal:</span> <span className="text-gray-400 truncate inline-block max-w-md">{spec.goal.slice(0, 80)}...</span></div>
            <div className="mt-2">
                <span className="text-purple-400">constraints:</span> <span className="text-gray-500">({spec.constraints.length})</span>
            </div>
            <div className="mt-2">
                <span className="text-purple-400">acceptance:</span> <span className="text-gray-500">({spec.acceptance.length})</span>
            </div>
            <div className="mt-2">
                <span className="text-purple-400">user_stories:</span> <span className="text-gray-500">({spec.user_stories.length})</span>
                {spec.user_stories.map(s => (
                    <div key={s.id} className="pl-4 text-gray-400">- {s.id}: {s.title}</div>
                ))}
            </div>
            <div className="mt-2">
                <span className="text-purple-400">requirements:</span> <span className="text-gray-500">({spec.requirements.length})</span>
            </div>
        </div>
    </div>
);

// YAML View
const YamlView: React.FC<{ rawYaml?: string }> = ({ rawYaml }) => (
    <pre className="font-mono text-sm bg-gray-900 text-gray-100 p-4 rounded-lg overflow-auto max-h-[60vh]">
        <code>{rawYaml || '# No YAML content'}</code>
    </pre>
);

// Spec List View (when no spec is selected)
const SpecListView: React.FC<{
    specs: SpecListItem[];
    onSelect: (name: string) => void;
    isLoading: boolean;
}> = ({ specs, onSelect, isLoading }) => {
    if (isLoading) {
        return (
            <div className="flex items-center justify-center h-64">
                <Loader2 className="animate-spin text-gray-500" size={24} />
            </div>
        );
    }

    if (specs.length === 0) {
        return (
            <div className="text-center py-12 text-gray-500">
                <FileText size={48} className="mx-auto mb-4 opacity-50" />
                <p>No specifications found</p>
                <p className="text-sm mt-2">Run <code className="bg-gray-800 px-2 py-0.5 rounded">ckrv spec new</code> to create one</p>
            </div>
        );
    }

    return (
        <div className="space-y-2">
            {specs.map((spec) => (
                <button
                    key={spec.name}
                    onClick={() => onSelect(spec.name)}
                    className="w-full text-left p-4 bg-gray-800/50 hover:bg-gray-800 rounded-lg border border-gray-700 transition-colors"
                >
                    <div className="flex items-center justify-between">
                        <div className="flex items-center gap-3">
                            <FileText size={20} className="text-cyan-400" />
                            <div>
                                <h3 className="font-medium text-gray-200">{spec.name}</h3>
                                <div className="flex items-center gap-2 mt-1">
                                    {spec.has_tasks && (
                                        <span className="text-xs bg-green-900/50 text-green-300 px-2 py-0.5 rounded">has tasks</span>
                                    )}
                                    {spec.has_implementation && (
                                        <span className="text-xs bg-purple-900/50 text-purple-300 px-2 py-0.5 rounded">
                                            implemented: {spec.implementation_branch}
                                        </span>
                                    )}
                                </div>
                            </div>
                        </div>
                        <ChevronRight size={20} className="text-gray-500" />
                    </div>
                </button>
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
                    <h1 className="text-2xl font-bold text-gray-100">Specifications</h1>
                    <p className="text-gray-500 mt-1">Select a spec to view and edit</p>
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
                <Loader2 className="animate-spin text-gray-500" size={32} />
            </div>
        );
    }

    return (
        <div className="h-full flex flex-col overflow-hidden">
            {/* Header */}
            <div className="shrink-0 px-4 py-3 border-b border-gray-700 flex items-center justify-between bg-gray-900/50">
                <div className="flex items-center gap-4">
                    <button
                        onClick={() => setSelectedSpecName(null)}
                        className="p-2 hover:bg-gray-800 rounded-lg transition-colors"
                    >
                        <ArrowLeft size={20} className="text-gray-400" />
                    </button>
                    <div>
                        <div className="flex items-center gap-2">
                            <span className="font-mono text-sm bg-gray-800 px-2 py-0.5 rounded text-gray-400">{spec.id}</span>
                        </div>
                    </div>
                </div>
                <div className="flex items-center gap-3">
                    <ViewToggle view={view} setView={(v) => setView(v as typeof view)} />
                    {hasChanges && (
                        <button
                            onClick={() => {
                                if (specDetailData?.spec) {
                                    setSpec(specDetailData.spec);
                                    setRawYaml(specDetailData.raw_yaml);
                                    setHasChanges(false);
                                }
                            }}
                            className="flex items-center gap-2 px-4 py-2 rounded-lg font-medium transition-colors bg-gray-700 hover:bg-gray-600 text-gray-200"
                            title="Discard all changes"
                        >
                            <RotateCcw size={16} />
                            Discard
                        </button>
                    )}
                    <button
                        onClick={() => saveMutation.mutate()}
                        disabled={!hasChanges || saveMutation.isPending}
                        className={`flex items-center gap-2 px-4 py-2 rounded-lg font-medium transition-colors ${hasChanges
                            ? 'bg-cyan-600 hover:bg-cyan-500 text-white'
                            : 'bg-gray-800 text-gray-500 cursor-not-allowed'
                            }`}
                    >
                        {saveMutation.isPending ? (
                            <Loader2 size={16} className="animate-spin" />
                        ) : (
                            <Save size={16} />
                        )}
                        Save
                    </button>
                </div>
            </div>

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
                                <button
                                    onClick={() => updateSpec({ constraints: [...spec.constraints, 'New constraint'] })}
                                    className="flex items-center gap-2 text-sm text-gray-500 hover:text-gray-300 mt-2 py-1"
                                >
                                    <Plus size={16} /> Add constraint
                                </button>
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
                                <button
                                    onClick={() => updateSpec({ acceptance: [...spec.acceptance, 'New acceptance criterion'] })}
                                    className="flex items-center gap-2 text-sm text-gray-500 hover:text-gray-300 mt-2 py-1"
                                >
                                    <Plus size={16} /> Add criterion
                                </button>
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
                                        <div key={i} className="flex items-start gap-2 py-1 text-sm text-gray-400">
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
                    <div className="bg-gray-800/50 rounded-lg border border-gray-700 p-6">
                        <OutlineView spec={spec} />
                    </div>
                )}

                {view === 'code' && (
                    <div className="rounded-lg overflow-hidden border border-gray-700">
                        <div className="bg-gray-800 px-4 py-2 flex items-center justify-between border-b border-gray-700">
                            <span className="text-gray-400 text-sm">spec.yaml</span>
                            <button className="text-xs text-gray-400 hover:text-white">Copy</button>
                        </div>
                        <YamlView rawYaml={rawYaml} />
                    </div>
                )}
            </div>

            {/* Status Bar */}
            <div className="shrink-0 px-4 py-2 border-t border-gray-700 flex items-center justify-between text-sm text-gray-500 bg-gray-900/50">
                <div className="flex items-center gap-4">
                    <span>{spec.user_stories.length} stories</span>
                    <span>{spec.requirements.length} requirements</span>
                    <span>{spec.constraints.length} constraints</span>
                </div>
                <div className="flex items-center gap-2">
                    <span className={`w-2 h-2 rounded-full ${hasChanges ? 'bg-amber-500' : 'bg-green-500'}`}></span>
                    <span>{hasChanges ? 'Unsaved changes' : 'All changes saved'}</span>
                </div>
            </div>
        </div>
    );
};
