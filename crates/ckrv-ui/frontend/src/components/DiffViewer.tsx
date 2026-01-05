import React, { useState, useMemo } from 'react';
import { useQuery } from '@tanstack/react-query';
import {
    GitCompare, ChevronDown, ChevronRight, Plus, Minus,
    FileCode2, FilePlus2, FileX2, FileEdit, Loader2,
    ArrowRight, Check, RefreshCw
} from 'lucide-react';

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
    added: 'text-emerald-400',
    modified: 'text-amber-400',
    deleted: 'text-red-400',
    renamed: 'text-blue-400',
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

// File diff component
const FileDiffView: React.FC<{ file: FileDiff; isExpanded: boolean; onToggle: () => void }> = ({
    file, isExpanded, onToggle
}) => {
    const StatusIcon = statusIcons[file.status] || FileCode2;
    const statusColor = statusColors[file.status] || 'text-gray-400';
    const diffLines = useMemo(() => parseDiffLines(file.diff), [file.diff]);

    return (
        <div className="border border-gray-700 rounded-lg overflow-hidden bg-gray-800/50">
            {/* File header */}
            <button
                onClick={onToggle}
                className="w-full flex items-center justify-between px-4 py-3 hover:bg-gray-700/50 transition-colors"
            >
                <div className="flex items-center gap-3">
                    <div className="text-gray-500">
                        {isExpanded ? <ChevronDown size={16} /> : <ChevronRight size={16} />}
                    </div>
                    <StatusIcon size={16} className={statusColor} />
                    <span className="font-mono text-sm text-gray-200">{file.path}</span>
                </div>
                <div className="flex items-center gap-4 text-sm">
                    {file.additions > 0 && (
                        <span className="text-emerald-400 flex items-center gap-1">
                            <Plus size={12} />
                            {file.additions}
                        </span>
                    )}
                    {file.deletions > 0 && (
                        <span className="text-red-400 flex items-center gap-1">
                            <Minus size={12} />
                            {file.deletions}
                        </span>
                    )}
                </div>
            </button>

            {/* Diff content */}
            {isExpanded && (
                <div className="border-t border-gray-700 bg-gray-900 overflow-x-auto">
                    <pre className="text-xs font-mono p-0">
                        {diffLines.map((line, i) => {
                            let bgColor = '';
                            let textColor = 'text-gray-400';

                            if (line.type === 'add') {
                                bgColor = 'bg-emerald-900/30';
                                textColor = 'text-emerald-300';
                            } else if (line.type === 'remove') {
                                bgColor = 'bg-red-900/30';
                                textColor = 'text-red-300';
                            } else if (line.type === 'hunk') {
                                bgColor = 'bg-blue-900/20';
                                textColor = 'text-blue-400';
                            } else if (line.type === 'header') {
                                textColor = 'text-gray-500';
                            }

                            return (
                                <div
                                    key={i}
                                    className={`px-4 py-0.5 ${bgColor} ${textColor} border-l-2 ${line.type === 'add' ? 'border-emerald-500' :
                                            line.type === 'remove' ? 'border-red-500' :
                                                'border-transparent'
                                        }`}
                                >
                                    {line.content || ' '}
                                </div>
                            );
                        })}
                    </pre>
                </div>
            )}
        </div>
    );
};

// Branch selector dropdown
const BranchSelector: React.FC<{
    value: string;
    onChange: (val: string) => void;
    branches: string[];
    label: string;
}> = ({ value, onChange, branches, label }) => {
    return (
        <div className="flex flex-col gap-1">
            <label className="text-xs text-gray-500">{label}</label>
            <select
                value={value}
                onChange={(e) => onChange(e.target.value)}
                className="bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-sm text-gray-200 focus:outline-none focus:ring-2 focus:ring-cyan-500/50"
            >
                {branches.map(branch => (
                    <option key={branch} value={branch}>{branch}</option>
                ))}
            </select>
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
        <div className="h-full flex flex-col bg-gray-950 text-gray-100">
            {/* Header */}
            <div className="shrink-0 px-6 py-4 border-b border-gray-800 bg-gray-900/50">
                <div className="flex items-center justify-between">
                    <div>
                        <h1 className="text-xl font-semibold flex items-center gap-3">
                            <GitCompare className="text-cyan-400" size={24} />
                            Diff Viewer
                        </h1>
                        <p className="text-gray-500 text-sm mt-1">Compare changes between branches</p>
                    </div>

                    {/* Branch selectors */}
                    <div className="flex items-center gap-4">
                        <BranchSelector
                            label="Base"
                            value={baseBranch}
                            onChange={setBaseBranch}
                            branches={branches}
                        />
                        <div className="pt-5 text-gray-500">
                            <ArrowRight size={20} />
                        </div>
                        <BranchSelector
                            label="Compare"
                            value={targetBranch}
                            onChange={setTargetBranch}
                            branches={['HEAD', ...branches]}
                        />
                        <button
                            onClick={() => refetch()}
                            disabled={isLoading}
                            className="mt-5 p-2 bg-gray-800 hover:bg-gray-700 rounded-lg transition-colors disabled:opacity-50"
                            title="Refresh"
                        >
                            <RefreshCw size={18} className={isLoading ? 'animate-spin' : ''} />
                        </button>
                    </div>
                </div>
            </div>

            {/* Stats bar */}
            {diffData?.stats && (
                <div className="shrink-0 px-6 py-3 border-b border-gray-800 flex items-center gap-6 text-sm">
                    <div className="flex items-center gap-2">
                        <FileCode2 size={16} className="text-gray-500" />
                        <span className="text-gray-300">{diffData.stats.files_changed} files changed</span>
                    </div>
                    <div className="flex items-center gap-2 text-emerald-400">
                        <Plus size={14} />
                        <span>{diffData.stats.insertions} insertions</span>
                    </div>
                    <div className="flex items-center gap-2 text-red-400">
                        <Minus size={14} />
                        <span>{diffData.stats.deletions} deletions</span>
                    </div>
                    <div className="flex-1" />
                    <button
                        onClick={expandAll}
                        className="text-xs text-gray-500 hover:text-gray-300 transition-colors"
                    >
                        Expand all
                    </button>
                    <button
                        onClick={collapseAll}
                        className="text-xs text-gray-500 hover:text-gray-300 transition-colors"
                    >
                        Collapse all
                    </button>
                </div>
            )}

            {/* Content */}
            <div className="flex-1 overflow-auto p-6">
                {isLoading ? (
                    <div className="flex items-center justify-center h-64">
                        <Loader2 className="animate-spin text-gray-500" size={32} />
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
                    <div className="flex flex-col items-center justify-center h-64 text-gray-500">
                        <Check size={48} className="text-emerald-500 mb-4" />
                        <p className="text-lg font-medium text-gray-300">No differences</p>
                        <p className="text-sm mt-1">
                            {baseBranch} and {targetBranch} are identical
                        </p>
                    </div>
                )}
            </div>
        </div>
    );
}
