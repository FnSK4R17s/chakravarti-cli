import React, { useState, useMemo } from 'react';
import { useQuery } from '@tanstack/react-query';
import {
    GitCompare, ChevronDown, ChevronRight, Plus, Minus,
    FileCode2, FilePlus2, FileX2, FileEdit, Loader2,
    ArrowRight, Check, RefreshCw
} from 'lucide-react';
import { Card, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible';
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select';

// Types
interface FileDiff {
    path: string;
    status: 'added' | 'modified' | 'deleted' | 'renamed';
    additions: number;
    deletions: number;
    diff: string;
}

interface DiffStats {
    files_changed: number;
    insertions: number;
    deletions: number;
}

interface DiffResponse {
    success: boolean;
    base_branch: string;
    target_branch: string;
    files: FileDiff[];
    stats: DiffStats;
    error?: string;
}

interface BranchesResponse {
    success: boolean;
    current: string;
    branches: string[];
    error?: string;
}

// API functions
const fetchBranches = async (): Promise<BranchesResponse> => {
    const res = await fetch('/api/diff/branches');
    return res.json();
};

const fetchDiff = async (base: string, target: string): Promise<DiffResponse> => {
    const params = new URLSearchParams();
    if (base) params.set('base', base);
    if (target) params.set('target', target);
    const res = await fetch(`/api/diff?${params}`);
    return res.json();
};

// Status icon mapping
const statusIcons: Record<string, React.ElementType> = {
    added: FilePlus2,
    modified: FileEdit,
    deleted: FileX2,
    renamed: FileCode2,
};

const statusColors: Record<string, string> = {
    added: 'text-accent-green',
    modified: 'text-accent-amber',
    deleted: 'text-destructive',
    renamed: 'text-accent-cyan',
};

// Parse diff content into lines with styling
function parseDiffLines(diff: string): { type: 'header' | 'add' | 'remove' | 'context' | 'hunk'; content: string }[] {
    const lines = diff.split('\n');
    return lines.map(line => {
        if (line.startsWith('+++') || line.startsWith('---')) {
            return { type: 'header', content: line };
        } else if (line.startsWith('@@')) {
            return { type: 'hunk', content: line };
        } else if (line.startsWith('+')) {
            return { type: 'add', content: line };
        } else if (line.startsWith('-')) {
            return { type: 'remove', content: line };
        }
        return { type: 'context', content: line };
    });
}

// File diff component using Collapsible
const FileDiffView: React.FC<{ file: FileDiff; isExpanded: boolean; onToggle: () => void }> = ({
    file, isExpanded, onToggle
}) => {
    const StatusIcon = statusIcons[file.status] || FileCode2;
    const statusColor = statusColors[file.status] || 'text-muted-foreground';
    const diffLines = useMemo(() => parseDiffLines(file.diff), [file.diff]);

    return (
        <Card className="overflow-hidden">
            <Collapsible open={isExpanded} onOpenChange={onToggle}>
                <CollapsibleTrigger asChild>
                    <button className="w-full flex items-center justify-between px-4 py-3 hover:bg-accent/50 transition-colors">
                        <div className="flex items-center gap-3">
                            <div className="text-muted-foreground">
                                {isExpanded ? <ChevronDown size={16} /> : <ChevronRight size={16} />}
                            </div>
                            <StatusIcon size={16} className={statusColor} />
                            <span className="font-mono text-sm text-foreground">{file.path}</span>
                        </div>
                        <div className="flex items-center gap-4 text-sm">
                            {file.additions > 0 && (
                                <Badge variant="success" className="flex items-center gap-1">
                                    <Plus size={12} />
                                    {file.additions}
                                </Badge>
                            )}
                            {file.deletions > 0 && (
                                <Badge variant="destructive" className="flex items-center gap-1">
                                    <Minus size={12} />
                                    {file.deletions}
                                </Badge>
                            )}
                        </div>
                    </button>
                </CollapsibleTrigger>
                <CollapsibleContent>
                    <div className="border-t border-border bg-muted/30 overflow-x-auto">
                        <pre className="text-xs font-mono p-0">
                            {diffLines.map((line, i) => {
                                let bgColor = '';
                                let textColor = 'text-muted-foreground';

                                if (line.type === 'add') {
                                    bgColor = 'bg-accent-green/10';
                                    textColor = 'text-accent-green';
                                } else if (line.type === 'remove') {
                                    bgColor = 'bg-destructive/10';
                                    textColor = 'text-destructive';
                                } else if (line.type === 'hunk') {
                                    bgColor = 'bg-accent-cyan/10';
                                    textColor = 'text-accent-cyan';
                                } else if (line.type === 'header') {
                                    textColor = 'text-muted-foreground';
                                }

                                return (
                                    <div
                                        key={i}
                                        className={`px-4 py-0.5 ${bgColor} ${textColor} border-l-2 ${line.type === 'add' ? 'border-accent-green' :
                                            line.type === 'remove' ? 'border-destructive' :
                                                'border-transparent'
                                            }`}
                                    >
                                        {line.content || ' '}
                                    </div>
                                );
                            })}
                        </pre>
                    </div>
                </CollapsibleContent>
            </Collapsible>
        </Card>
    );
};

// Branch selector using shadcn Select
const BranchSelector: React.FC<{
    value: string;
    onChange: (val: string) => void;
    branches: string[];
    label: string;
}> = ({ value, onChange, branches, label }) => {
    return (
        <div className="flex flex-col gap-1">
            <label className="text-xs text-muted-foreground">{label}</label>
            <Select value={value} onValueChange={onChange}>
                <SelectTrigger className="w-[180px]">
                    <SelectValue placeholder="Select branch" />
                </SelectTrigger>
                <SelectContent>
                    {branches.map(branch => (
                        <SelectItem key={branch} value={branch}>{branch}</SelectItem>
                    ))}
                </SelectContent>
            </Select>
        </div>
    );
};

// Main DiffViewer component
export default function DiffViewer() {
    const [baseBranch, setBaseBranch] = useState<string>('');
    const [targetBranch, setTargetBranch] = useState<string>('');
    const [expandedFiles, setExpandedFiles] = useState<Set<string>>(new Set());

    // Fetch branches
    const { data: branchesData, isLoading: loadingBranches } = useQuery({
        queryKey: ['branches'],
        queryFn: fetchBranches,
    });

    // Initialize base/target when branches load
    React.useEffect(() => {
        if (branchesData?.branches && !baseBranch) {
            // Default base to main or master
            const defaultBase = branchesData.branches.find(b => b === 'main' || b === 'master') || branchesData.branches[0];
            setBaseBranch(defaultBase || '');
            setTargetBranch(branchesData.current || 'HEAD');
        }
    }, [branchesData, baseBranch]);

    // Fetch diff when branches are selected
    const { data: diffData, isLoading: loadingDiff, refetch } = useQuery({
        queryKey: ['diff', baseBranch, targetBranch],
        queryFn: () => fetchDiff(baseBranch, targetBranch),
        enabled: !!baseBranch && !!targetBranch,
    });

    const toggleFile = (path: string) => {
        setExpandedFiles(prev => {
            const next = new Set(prev);
            if (next.has(path)) {
                next.delete(path);
            } else {
                next.add(path);
            }
            return next;
        });
    };

    const expandAll = () => {
        if (diffData?.files) {
            setExpandedFiles(new Set(diffData.files.map(f => f.path)));
        }
    };

    const collapseAll = () => {
        setExpandedFiles(new Set());
    };

    const isLoading = loadingBranches || loadingDiff;
    const branches = branchesData?.branches || [];

    return (
        <div className="h-full flex flex-col bg-background text-foreground">
            {/* Header */}
            <Card className="shrink-0 rounded-none border-x-0 border-t-0">
                <CardHeader className="pb-4">
                    <div className="flex items-center justify-between">
                        <div>
                            <CardTitle className="text-xl flex items-center gap-3">
                                <GitCompare className="text-primary" size={24} />
                                Diff Viewer
                            </CardTitle>
                            <p className="text-muted-foreground text-sm mt-1">Compare changes between branches</p>
                        </div>

                        {/* Branch selectors */}
                        <div className="flex items-center gap-4">
                            <BranchSelector
                                label="Base"
                                value={baseBranch}
                                onChange={setBaseBranch}
                                branches={branches}
                            />
                            <div className="pt-5 text-muted-foreground">
                                <ArrowRight size={20} />
                            </div>
                            <BranchSelector
                                label="Compare"
                                value={targetBranch}
                                onChange={setTargetBranch}
                                branches={['HEAD', ...branches]}
                            />
                            <Button
                                variant="outline"
                                size="icon"
                                onClick={() => refetch()}
                                disabled={isLoading}
                                className="mt-5"
                                title="Refresh"
                            >
                                <RefreshCw size={18} className={isLoading ? 'animate-spin' : ''} />
                            </Button>
                        </div>
                    </div>
                </CardHeader>
            </Card>

            {/* Stats bar */}
            {diffData?.stats && (
                <div className="shrink-0 px-6 py-3 border-b border-border flex items-center gap-6 text-sm bg-muted/30">
                    <div className="flex items-center gap-2">
                        <FileCode2 size={16} className="text-muted-foreground" />
                        <span className="text-foreground">{diffData.stats.files_changed} files changed</span>
                    </div>
                    <Badge variant="success" className="flex items-center gap-1">
                        <Plus size={14} />
                        {diffData.stats.insertions} insertions
                    </Badge>
                    <Badge variant="destructive" className="flex items-center gap-1">
                        <Minus size={14} />
                        {diffData.stats.deletions} deletions
                    </Badge>
                    <div className="flex-1" />
                    <Button variant="ghost" size="sm" onClick={expandAll}>
                        Expand all
                    </Button>
                    <Button variant="ghost" size="sm" onClick={collapseAll}>
                        Collapse all
                    </Button>
                </div>
            )}

            {/* Content */}
            <ScrollArea className="flex-1 p-6">
                {isLoading ? (
                    <div className="flex items-center justify-center h-64">
                        <Loader2 className="animate-spin text-muted-foreground" size={32} />
                    </div>
                ) : diffData?.files && diffData.files.length > 0 ? (
                    <div className="space-y-3">
                        {diffData.files.map(file => (
                            <FileDiffView
                                key={file.path}
                                file={file}
                                isExpanded={expandedFiles.has(file.path)}
                                onToggle={() => toggleFile(file.path)}
                            />
                        ))}
                    </div>
                ) : (
                    <div className="flex flex-col items-center justify-center h-64 text-muted-foreground">
                        <Check size={48} className="text-accent-green mb-4" />
                        <p className="text-lg font-medium text-foreground">No differences</p>
                        <p className="text-sm mt-1">
                            {baseBranch} and {targetBranch} are identical
                        </p>
                    </div>
                )}
            </ScrollArea>
        </div>
    );
}
