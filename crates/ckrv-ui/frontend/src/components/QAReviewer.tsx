/**
 * @module QAReviewer
 * @description
 * Quality Assurance interface for reviewing code quality, finding bugs, and generating
 * reports. Provides three main functions: code review, bug scanning, and full report
 * generation using an AI-powered QA agent.
 *
 * @context
 * Rendered as the main content of the QA page in the dashboard. Users run code reviews
 * against a base branch, scan for potential bugs, and generate comprehensive QA reports.
 * Requires a QA agent to be configured in Agent Manager.
 *
 * @dependencies
 * - useQuery/useMutation: React Query for QA operations
 * - shadcn/ui components: Card, Badge, Tabs, ScrollArea for consistent UI
 *
 * @example
 * // Rendered directly as a page component
 * <QAReviewer />
 *
 * // Supports three tabs: Review, Bugs, Report
 * // Issues are grouped by severity (critical, major, minor, info)
 */

// === IMPORTS ===
import React, { useState } from 'react';
import { useQuery, useMutation } from '@tanstack/react-query';
import {
    ShieldCheck, Search, Bug, FileText, Loader2, AlertTriangle,
    AlertCircle, Info, CheckCircle2, ChevronDown, ChevronRight,
    Download, RefreshCw, Shield, FileWarning
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
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { toast } from 'sonner';

// === TYPES ===

/**
 * A quality assurance issue found during code review.
 */
interface QAIssue {
    /** Unique issue identifier */
    id: string;
    /** File path where the issue was found */
    file: string;
    /** Line number of the issue, if applicable */
    line?: number;
    /** Severity: critical blocks release, info is advisory */
    severity: 'critical' | 'major' | 'minor' | 'info';
    /** Issue category (e.g., code_quality, security, performance) */
    category: string;
    /** Description of the issue */
    message: string;
    /** Suggested fix or improvement */
    suggestion?: string;
}

/**
 * Summary statistics from a QA review.
 */
interface QASummary {
    /** Total number of issues found */
    total_issues: number;
    /** Count of critical severity issues */
    critical: number;
    /** Count of major severity issues */
    major: number;
    /** Count of minor severity issues */
    minor: number;
    /** Count of informational issues */
    info: number;
    /** Number of files reviewed */
    files_reviewed: number;
    /** Overall verdict: pass, fail, or needs review */
    verdict: 'pass' | 'fail' | 'review';
}

/**
 * Complete output from a QA review run.
 * 
 * @example
 * const review: QAReviewOutput = {
 *   report_id: 'qa-123',
 *   base_branch: 'main',
 *   issues: [...],
 *   summary: { total_issues: 5, critical: 0, ... }
 * };
 */
interface QAReviewOutput {
    /** Unique report identifier */
    report_id: string;
    /** Base branch used for diff comparison */
    base_branch: string;
    /** List of issues found */
    issues: QAIssue[];
    /** Summary statistics */
    summary: QASummary;
    /** ID of the agent that performed the review */
    agent_id?: string;
}

/**
 * Agent configuration returned from the API.
 * Represents an AI agent capable of performing QA tasks.
 */
interface Agent {
    /** Unique agent identifier */
    id: string;
    /** Display name of the agent */
    name: string;
    /** Model identifier (e.g., 'claude-3-opus') */
    model: string;
    /** Whether this agent is designated as a QA agent */
    is_qa_agent?: boolean;
}

// === API FUNCTIONS ===

const fetchQAAgent = async (): Promise<{ agent: Agent | null }> => {
    const res = await fetch('/api/qa/agent');
    return res.json();
};

const runReview = async (base: string): Promise<{ success: boolean; review?: QAReviewOutput; error?: string }> => {
    const res = await fetch('/api/qa/review', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ base }),
    });
    return res.json();
};

const runBugs = async (base: string): Promise<{ success: boolean; issues?: QAIssue[]; error?: string }> => {
    const res = await fetch('/api/qa/bugs', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ base }),
    });
    return res.json();
};

const runReport = async (base: string, full: boolean): Promise<{ success: boolean; review?: QAReviewOutput; report?: string; error?: string }> => {
    const res = await fetch('/api/qa/report', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ base, full }),
    });
    return res.json();
};

// === CONSTANTS ===

/** Maps severity levels to their visual styling (icon, colors, borders) */
const severityConfig: Record<string, { icon: React.ElementType; color: string; bgColor: string; borderColor: string }> = {
    critical: { icon: AlertCircle, color: 'text-destructive', bgColor: 'bg-destructive/10', borderColor: 'border-destructive/50' },
    major: { icon: AlertTriangle, color: 'text-warning', bgColor: 'bg-warning/10', borderColor: 'border-warning/50' },
    minor: { icon: Info, color: 'text-info', bgColor: 'bg-info/10', borderColor: 'border-info/50' },
    info: { icon: Info, color: 'text-muted-foreground', bgColor: 'bg-muted', borderColor: 'border-border' },
};

/** Maps issue categories to their representative icons */
const categoryIcons: Record<string, React.ElementType> = {
    code_quality: FileText,
    potential_bug: Bug,
    error_handling: AlertTriangle,
    security: Shield,
    performance: RefreshCw,
    documentation: FileText,
    best_practice: CheckCircle2,
};

// === SUB-COMPONENTS ===

// Issue Card Component
/**
 * Props for IssueCard component.
 * Expandable card displaying a QA issue with severity, category, and suggested fix.
 */
interface IssueCardProps {
    /** QA issue to display */
    issue: QAIssue;
    /** Whether the issue details are expanded */
    isExpanded: boolean;
    /** Callback to toggle the expanded state */
    onToggle: () => void;
}

const IssueCard: React.FC<IssueCardProps> = ({
    issue, isExpanded, onToggle
}) => {
    const config = severityConfig[issue.severity] || severityConfig.info;
    const SeverityIcon = config.icon;
    const CategoryIcon = categoryIcons[issue.category] || FileWarning;

    return (
        <Card className={`${config.borderColor} border-l-4`}>
            <Collapsible open={isExpanded} onOpenChange={onToggle}>
                <CollapsibleTrigger asChild>
                    <button className="w-full flex items-center justify-between px-4 py-3 hover:bg-accent/50 transition-colors text-left">
                        <div className="flex items-center gap-3">
                            <div className="text-muted-foreground">
                                {isExpanded ? <ChevronDown size={16} /> : <ChevronRight size={16} />}
                            </div>
                            <SeverityIcon size={16} className={config.color} />
                            <div>
                                <span className="font-medium">{issue.message}</span>
                                <div className="text-xs text-muted-foreground font-mono mt-0.5">
                                    {issue.file}{issue.line && `:${issue.line}`}
                                </div>
                            </div>
                        </div>
                        <div className="flex items-center gap-2">
                            <Badge variant="outline" className="flex items-center gap-1">
                                <CategoryIcon size={12} />
                                {issue.category.replace('_', ' ')}
                            </Badge>
                            <Badge className={`${config.bgColor} ${config.color}`}>
                                {issue.severity}
                            </Badge>
                        </div>
                    </button>
                </CollapsibleTrigger>
                <CollapsibleContent>
                    {issue.suggestion && (
                        <div className="border-t border-border bg-muted/30 p-4">
                            <div className="flex items-start gap-2">
                                <CheckCircle2 size={16} className="text-success shrink-0 mt-0.5" />
                                <div>
                                    <div className="text-sm font-medium text-success">Suggested Fix</div>
                                    <div className="text-sm text-muted-foreground mt-1">{issue.suggestion}</div>
                                </div>
                            </div>
                        </div>
                    )}
                </CollapsibleContent>
            </Collapsible>
        </Card>
    );
};

// Summary Card Component
/**
 * Props for SummaryCard component.
 * Displays review verdict and issue counts by severity.
 */
interface SummaryCardProps {
    /** Summary statistics from a QA review */
    summary: QASummary;
}

const SummaryCard: React.FC<SummaryCardProps> = ({ summary }) => {
    const verdictConfig = {
        pass: { color: 'text-success', bg: 'bg-success/10', icon: CheckCircle2, label: 'Passed' },
        fail: { color: 'text-destructive', bg: 'bg-destructive/10', icon: AlertCircle, label: 'Failed' },
        review: { color: 'text-warning', bg: 'bg-warning/10', icon: AlertTriangle, label: 'Needs Review' },
    };

    const v = verdictConfig[summary.verdict];
    const VerdictIcon = v.icon;

    return (
        <div className="space-y-4">
            {/* Verdict */}
            <Card className={`p-6 ${v.bg} border-none`}>
                <div className="flex items-center justify-center gap-3">
                    <VerdictIcon size={32} className={v.color} />
                    <span className={`text-2xl font-bold ${v.color}`}>{v.label}</span>
                </div>
            </Card>

            {/* Stats */}
            <div className="grid grid-cols-2 md:grid-cols-5 gap-3">
                <Card className="p-4 text-center">
                    <div className="text-sm text-muted-foreground">Total Issues</div>
                    <div className="text-2xl font-bold">{summary.total_issues}</div>
                </Card>
                <Card className="p-4 text-center border-destructive/30">
                    <div className="text-sm text-muted-foreground flex items-center justify-center gap-1">
                        <AlertCircle size={14} className="text-destructive" /> Critical
                    </div>
                    <div className="text-2xl font-bold text-destructive">{summary.critical}</div>
                </Card>
                <Card className="p-4 text-center border-warning/30">
                    <div className="text-sm text-muted-foreground flex items-center justify-center gap-1">
                        <AlertTriangle size={14} className="text-warning" /> Major
                    </div>
                    <div className="text-2xl font-bold text-warning">{summary.major}</div>
                </Card>
                <Card className="p-4 text-center border-info/30">
                    <div className="text-sm text-muted-foreground flex items-center justify-center gap-1">
                        <Info size={14} className="text-info" /> Minor
                    </div>
                    <div className="text-2xl font-bold text-info">{summary.minor}</div>
                </Card>
                <Card className="p-4 text-center">
                    <div className="text-sm text-muted-foreground">Files Reviewed</div>
                    <div className="text-2xl font-bold">{summary.files_reviewed}</div>
                </Card>
            </div>
        </div>
    );
};

// Issues List Component
/**
 * Props for IssuesList component.
 * Groups and displays issues by severity level.
 */
interface IssuesListProps {
    /** List of QA issues to display */
    issues: QAIssue[];
}

const IssuesList: React.FC<IssuesListProps> = ({ issues }) => {
    /** Set of issue IDs that are currently expanded to show details */
    const [expandedIssues, setExpandedIssues] = useState<Set<string>>(new Set());

    /** Toggles the expanded state of an issue card */
    const toggleIssue = (id: string) => {
        setExpandedIssues(prev => {
            const next = new Set(prev);
            if (next.has(id)) next.delete(id);
            else next.add(id);
            return next;
        });
    };

    if (issues.length === 0) {
        return (
            <div className="text-center py-12">
                <CheckCircle2 size={48} className="inline text-success mb-3" />
                <p className="text-lg font-medium text-foreground">No issues found!</p>
                <p className="text-sm text-muted-foreground mt-1">Your code looks great.</p>
            </div>
        );
    }

    // Group by severity
    const grouped = {
        critical: issues.filter(i => i.severity === 'critical'),
        major: issues.filter(i => i.severity === 'major'),
        minor: issues.filter(i => i.severity === 'minor'),
        info: issues.filter(i => i.severity === 'info'),
    };

    return (
        <div className="space-y-4">
            {grouped.critical.length > 0 && (
                <div>
                    <h4 className="font-medium text-destructive flex items-center gap-2 mb-2">
                        <AlertCircle size={16} /> Critical ({grouped.critical.length})
                    </h4>
                    <div className="space-y-2">
                        {grouped.critical.map(issue => (
                            <IssueCard
                                key={issue.id}
                                issue={issue}
                                isExpanded={expandedIssues.has(issue.id)}
                                onToggle={() => toggleIssue(issue.id)}
                            />
                        ))}
                    </div>
                </div>
            )}

            {grouped.major.length > 0 && (
                <div>
                    <h4 className="font-medium text-warning flex items-center gap-2 mb-2">
                        <AlertTriangle size={16} /> Major ({grouped.major.length})
                    </h4>
                    <div className="space-y-2">
                        {grouped.major.map(issue => (
                            <IssueCard
                                key={issue.id}
                                issue={issue}
                                isExpanded={expandedIssues.has(issue.id)}
                                onToggle={() => toggleIssue(issue.id)}
                            />
                        ))}
                    </div>
                </div>
            )}

            {grouped.minor.length > 0 && (
                <div>
                    <h4 className="font-medium text-info flex items-center gap-2 mb-2">
                        <Info size={16} /> Minor ({grouped.minor.length})
                    </h4>
                    <div className="space-y-2">
                        {grouped.minor.map(issue => (
                            <IssueCard
                                key={issue.id}
                                issue={issue}
                                isExpanded={expandedIssues.has(issue.id)}
                                onToggle={() => toggleIssue(issue.id)}
                            />
                        ))}
                    </div>
                </div>
            )}

            {grouped.info.length > 0 && (
                <div>
                    <h4 className="font-medium text-muted-foreground flex items-center gap-2 mb-2">
                        <Info size={16} /> Info ({grouped.info.length})
                    </h4>
                    <div className="space-y-2">
                        {grouped.info.map(issue => (
                            <IssueCard
                                key={issue.id}
                                issue={issue}
                                isExpanded={expandedIssues.has(issue.id)}
                                onToggle={() => toggleIssue(issue.id)}
                            />
                        ))}
                    </div>
                </div>
            )}
        </div>
    );
};

// === MAIN COMPONENT ===

export default function QAReviewer() {
    // === STATE ===

    // --- Configuration ---
    /** Base branch for diff comparison (defaults to main/master) */
    const [baseBranch, setBaseBranch] = useState('main');

    // --- UI State ---
    /** Active tab in the QA interface: review, bugs, or report */
    const [activeTab, setActiveTab] = useState('review');

    // --- QA Data ---
    /** Results from the code review analysis */
    const [reviewResult, setReviewResult] = useState<QAReviewOutput | null>(null);
    /** Issues found by the bug analysis */
    const [bugIssues, setBugIssues] = useState<QAIssue[] | null>(null);
    /** Full report markdown output */
    const [fullReport, setFullReport] = useState<string | null>(null);

    // === QUERIES ===

    // Fetch QA agent
    const { data: agentData, isLoading: loadingAgent } = useQuery({
        queryKey: ['qa-agent'],
        queryFn: fetchQAAgent,
    });

    // === MUTATIONS ===
    const reviewMutation = useMutation({
        mutationFn: () => runReview(baseBranch),
        onSuccess: (data) => {
            if (data.success && data.review) {
                setReviewResult(data.review);
                toast.success('Code review completed');
            } else {
                toast.error(data.error || 'Failed to run review');
            }
        },
        onError: () => toast.error('Failed to run review'),
    });

    const bugsMutation = useMutation({
        mutationFn: () => runBugs(baseBranch),
        onSuccess: (data) => {
            if (data.success && data.issues) {
                setBugIssues(data.issues);
                toast.success('Bug analysis completed');
            } else {
                toast.error(data.error || 'Failed to run bug analysis');
            }
        },
        onError: () => toast.error('Failed to run bug analysis'),
    });

    const reportMutation = useMutation({
        mutationFn: () => runReport(baseBranch, true),
        onSuccess: (data) => {
            if (data.success) {
                if (data.review) setReviewResult(data.review);
                if (data.report) setFullReport(data.report);
                toast.success('Full report generated');
            } else {
                toast.error(data.error || 'Failed to generate report');
            }
        },
        onError: () => toast.error('Failed to generate report'),
    });

    const isLoading = reviewMutation.isPending || bugsMutation.isPending || reportMutation.isPending;

    // === HANDLERS ===

    /** Downloads the full report as a markdown file */
    const downloadReport = () => {
        if (!fullReport) return;
        const blob = new Blob([fullReport], { type: 'text/markdown' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `qa-report-${new Date().toISOString().split('T')[0]}.md`;
        a.click();
        URL.revokeObjectURL(url);
    };

    // === MAIN RENDER ===

    return (
        <div className="h-full flex flex-col bg-background text-foreground">
            {/* Header */}
            <Card className="shrink-0 rounded-none border-x-0 border-t-0">
                <CardHeader className="pb-4">
                    <div className="flex items-center justify-between">
                        <div>
                            <CardTitle className="text-xl flex items-center gap-3">
                                <ShieldCheck className="text-primary" size={24} />
                                QA Reviewer
                            </CardTitle>
                            <p className="text-muted-foreground text-sm mt-1">Review code quality, find bugs, and generate reports</p>
                        </div>

                        <div className="flex items-center gap-4">
                            {/* Base branch selector */}
                            <div className="flex flex-col gap-1">
                                <label className="text-xs text-muted-foreground">Base Branch</label>
                                <Select value={baseBranch} onValueChange={setBaseBranch}>
                                    <SelectTrigger className="w-[140px]">
                                        <SelectValue placeholder="Select branch" />
                                    </SelectTrigger>
                                    <SelectContent>
                                        <SelectItem value="main">main</SelectItem>
                                        <SelectItem value="master">master</SelectItem>
                                        <SelectItem value="develop">develop</SelectItem>
                                    </SelectContent>
                                </Select>
                            </div>

                            {/* Agent status */}
                            {loadingAgent ? (
                                <Badge variant="outline"><Loader2 size={12} className="animate-spin mr-1" /> Loading...</Badge>
                            ) : agentData?.agent ? (
                                <Badge variant="success">🤖 {agentData.agent.name}</Badge>
                            ) : (
                                <Badge variant="warning">⚠️ No QA agent</Badge>
                            )}
                        </div>
                    </div>
                </CardHeader>
            </Card>

            {/* Tabs */}
            <div className="flex-1 overflow-hidden">
                <Tabs value={activeTab} onValueChange={setActiveTab} className="h-full flex flex-col">
                    <div className="shrink-0 px-6 pt-4 border-b border-border bg-muted/30">
                        <TabsList className="bg-transparent gap-2">
                            <TabsTrigger value="review" className="data-[state=active]:bg-background">
                                <Search size={14} className="mr-1" /> Review
                            </TabsTrigger>
                            <TabsTrigger value="bugs" className="data-[state=active]:bg-background">
                                <Bug size={14} className="mr-1" /> Bugs
                            </TabsTrigger>
                            <TabsTrigger value="report" className="data-[state=active]:bg-background">
                                <FileText size={14} className="mr-1" /> Report
                            </TabsTrigger>
                        </TabsList>
                    </div>

                    <ScrollArea className="flex-1">
                        {/* Review Tab */}
                        <TabsContent value="review" className="p-6 mt-0">
                            <div className="space-y-4">
                                <div className="flex items-center gap-4">
                                    <Button
                                        onClick={() => reviewMutation.mutate()}
                                        disabled={isLoading || !agentData?.agent}
                                    >
                                        {reviewMutation.isPending ? (
                                            <><Loader2 size={16} className="mr-2 animate-spin" /> Reviewing...</>
                                        ) : (
                                            <><Search size={16} className="mr-2" /> Run Code Review</>
                                        )}
                                    </Button>
                                    <span className="text-sm text-muted-foreground">
                                        Analyze code quality of changes vs <code className="bg-muted px-1 rounded">{baseBranch}</code>
                                    </span>
                                </div>

                                {!agentData?.agent && (
                                    <Card className="p-6 border-warning/50 bg-warning/5">
                                        <div className="flex items-start gap-3">
                                            <AlertTriangle className="text-warning shrink-0" size={20} />
                                            <div>
                                                <h4 className="font-medium">No QA Agent Configured</h4>
                                                <p className="text-sm text-muted-foreground mt-1">
                                                    To use QA review, configure an agent with the "QA Agent" role in the Agent Manager.
                                                </p>
                                            </div>
                                        </div>
                                    </Card>
                                )}

                                {reviewResult && (
                                    <>
                                        <SummaryCard summary={reviewResult.summary} />
                                        <IssuesList issues={reviewResult.issues} />
                                    </>
                                )}
                            </div>
                        </TabsContent>

                        {/* Bugs Tab */}
                        <TabsContent value="bugs" className="p-6 mt-0">
                            <div className="space-y-4">
                                <div className="flex items-center gap-4">
                                    <Button
                                        onClick={() => bugsMutation.mutate()}
                                        disabled={isLoading || !agentData?.agent}
                                    >
                                        {bugsMutation.isPending ? (
                                            <><Loader2 size={16} className="mr-2 animate-spin" /> Scanning...</>
                                        ) : (
                                            <><Bug size={16} className="mr-2" /> Find Bugs</>
                                        )}
                                    </Button>
                                    <span className="text-sm text-muted-foreground">
                                        Scan for potential bugs and error handling issues
                                    </span>
                                </div>

                                {bugIssues && <IssuesList issues={bugIssues} />}
                            </div>
                        </TabsContent>

                        {/* Report Tab */}
                        <TabsContent value="report" className="p-6 mt-0">
                            <div className="space-y-4">
                                <div className="flex items-center gap-4">
                                    <Button
                                        onClick={() => reportMutation.mutate()}
                                        disabled={isLoading || !agentData?.agent}
                                    >
                                        {reportMutation.isPending ? (
                                            <><Loader2 size={16} className="mr-2 animate-spin" /> Generating...</>
                                        ) : (
                                            <><FileText size={16} className="mr-2" /> Generate Full Report</>
                                        )}
                                    </Button>
                                    {fullReport && (
                                        <Button variant="outline" onClick={downloadReport}>
                                            <Download size={16} className="mr-2" /> Download
                                        </Button>
                                    )}
                                    <span className="text-sm text-muted-foreground">
                                        Generate comprehensive QA report with all analysis sections
                                    </span>
                                </div>

                                {reviewResult && <SummaryCard summary={reviewResult.summary} />}

                                {fullReport && (
                                    <Card className="p-4">
                                        <pre className="text-sm font-mono whitespace-pre-wrap overflow-x-auto">
                                            {fullReport}
                                        </pre>
                                    </Card>
                                )}
                            </div>
                        </TabsContent>
                    </ScrollArea>
                </Tabs>
            </div>
        </div>
    );
}
