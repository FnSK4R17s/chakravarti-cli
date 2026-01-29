import React, { useState } from 'react';
import { useQuery, useMutation } from '@tanstack/react-query';
import {
    FlaskConical, Play, FileSearch, FileEdit, BarChart3,
    Loader2, CheckCircle2, XCircle, AlertTriangle, Clock,
    ChevronDown, ChevronRight, Beaker, Bot, MessageCircle, Send
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
import { TestFixModal } from './TestFixModal';

// Types
interface TestResult {
    success: boolean;
    total: number;
    passed: number;
    failed: number;
    skipped: number;
    duration_ms: number;
    failures: TestFailure[];
    framework: string;
}

interface TestFailure {
    name: string;
    file: string;
    line?: number;
    message: string;
    stdout?: string;
    stderr?: string;
}

interface TestPlanOutput {
    plan_id: string;
    base_branch: string;
    changed_files: ChangedFileInfo[];
    proposed_tests: ProposedTest[];
}

interface ChangedFileInfo {
    path: string;
    change_type: string;
    lines_added: number;
    lines_removed: number;
    has_tests: boolean;
}

interface ProposedTest {
    target_file: string;
    test_file: string;
    description: string;
    priority: string;
}

interface CoverageResult {
    total: number;
    covered: number;
    uncovered: number;
    coverage_percent: number;
}

interface Agent {
    id: string;
    name: string;
    model: string;
    is_test_writer?: boolean;
}

// API functions
const fetchTestWriterAgent = async (): Promise<{ agent: Agent | null }> => {
    const res = await fetch('/api/test/agent');
    return res.json();
};

const runTests = async (base: string): Promise<{ success: boolean; result?: TestResult; error?: string }> => {
    const res = await fetch('/api/test/run', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ base }),
    });
    return res.json();
};

const planTests = async (base: string): Promise<{ success: boolean; plan?: TestPlanOutput; error?: string }> => {
    const res = await fetch('/api/test/plan', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ base }),
    });
    return res.json();
};

const writeTests = async (base: string, run: boolean, customPrompt?: string): Promise<{ success: boolean; message?: string; error?: string }> => {
    const res = await fetch('/api/test/write', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ base, run, custom_prompt: customPrompt }),
    });
    return res.json();
};

const checkCoverage = async (base: string): Promise<{ success: boolean; coverage?: CoverageResult; error?: string }> => {
    const res = await fetch('/api/test/coverage', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ base }),
    });
    return res.json();
};

// Subcomponents
const TestResultCard: React.FC<{ result: TestResult }> = ({ result }) => {
    const [expandedFailures, setExpandedFailures] = useState<Set<number>>(new Set());

    const toggleFailure = (idx: number) => {
        setExpandedFailures(prev => {
            const next = new Set(prev);
            if (next.has(idx)) next.delete(idx);
            else next.add(idx);
            return next;
        });
    };

    return (
        <div className="space-y-4">
            {/* Summary */}
            <div className="grid grid-cols-2 md:grid-cols-5 gap-3">
                <Card className="p-4">
                    <div className="text-sm text-muted-foreground">Total</div>
                    <div className="text-2xl font-bold">{result.total}</div>
                </Card>
                <Card className="p-4 border-accent-green/30">
                    <div className="text-sm text-muted-foreground flex items-center gap-1">
                        <CheckCircle2 size={14} className="text-accent-green" /> Passed
                    </div>
                    <div className="text-2xl font-bold text-accent-green">{result.passed}</div>
                </Card>
                <Card className="p-4 border-destructive/30">
                    <div className="text-sm text-muted-foreground flex items-center gap-1">
                        <XCircle size={14} className="text-destructive" /> Failed
                    </div>
                    <div className="text-2xl font-bold text-destructive">{result.failed}</div>
                </Card>
                <Card className="p-4 border-accent-amber/30">
                    <div className="text-sm text-muted-foreground flex items-center gap-1">
                        <AlertTriangle size={14} className="text-accent-amber" /> Skipped
                    </div>
                    <div className="text-2xl font-bold text-accent-amber">{result.skipped}</div>
                </Card>
                <Card className="p-4">
                    <div className="text-sm text-muted-foreground flex items-center gap-1">
                        <Clock size={14} /> Duration
                    </div>
                    <div className="text-2xl font-bold">{(result.duration_ms / 1000).toFixed(2)}s</div>
                </Card>
            </div>

            {/* Framework */}
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
                <Beaker size={14} />
                <span>Framework: <span className="text-foreground font-medium">{result.framework}</span></span>
            </div>

            {/* Failures */}
            {result.failures.length > 0 && (
                <div className="space-y-2">
                    <h4 className="font-medium text-destructive flex items-center gap-2">
                        <XCircle size={16} /> Failed Tests ({result.failures.length})
                    </h4>
                    {result.failures.map((failure, idx) => (
                        <Card key={idx} className="border-destructive/30">
                            <Collapsible open={expandedFailures.has(idx)} onOpenChange={() => toggleFailure(idx)}>
                                <CollapsibleTrigger asChild>
                                    <button className="w-full flex items-center justify-between px-4 py-3 hover:bg-destructive/5 transition-colors">
                                        <div className="flex items-center gap-3">
                                            <div className="text-muted-foreground">
                                                {expandedFailures.has(idx) ? <ChevronDown size={16} /> : <ChevronRight size={16} />}
                                            </div>
                                            <XCircle size={16} className="text-destructive" />
                                            <span className="font-mono text-sm">{failure.name}</span>
                                        </div>
                                        <span className="text-xs text-muted-foreground font-mono">{failure.file}</span>
                                    </button>
                                </CollapsibleTrigger>
                                <CollapsibleContent>
                                    <div className="border-t border-border bg-muted/30 p-4 space-y-2">
                                        <div className="text-sm text-destructive">{failure.message}</div>
                                        {failure.stdout && (
                                            <pre className="text-xs font-mono bg-background p-2 rounded overflow-x-auto">
                                                {failure.stdout}
                                            </pre>
                                        )}
                                    </div>
                                </CollapsibleContent>
                            </Collapsible>
                        </Card>
                    ))}
                </div>
            )}
        </div>
    );
};

const TestPlanCard: React.FC<{ plan: TestPlanOutput }> = ({ plan }) => {
    return (
        <div className="space-y-4">
            {/* Changed Files */}
            <div>
                <h4 className="font-medium mb-2 flex items-center gap-2">
                    <FileSearch size={16} /> Changed Files ({plan.changed_files.length})
                </h4>
                <Card className="overflow-hidden">
                    <table className="w-full text-sm">
                        <thead className="bg-muted/50">
                            <tr>
                                <th className="text-left px-4 py-2 font-medium">File</th>
                                <th className="text-left px-4 py-2 font-medium">Status</th>
                                <th className="text-center px-4 py-2 font-medium">Changes</th>
                                <th className="text-center px-4 py-2 font-medium">Has Tests</th>
                            </tr>
                        </thead>
                        <tbody>
                            {plan.changed_files.map((file, idx) => (
                                <tr key={idx} className="border-t border-border hover:bg-muted/30">
                                    <td className="px-4 py-2 font-mono text-xs">{file.path}</td>
                                    <td className="px-4 py-2">
                                        <Badge variant="outline">{file.change_type}</Badge>
                                    </td>
                                    <td className="px-4 py-2 text-center">
                                        <span className="text-accent-green">+{file.lines_added}</span>
                                        {' / '}
                                        <span className="text-destructive">-{file.lines_removed}</span>
                                    </td>
                                    <td className="px-4 py-2 text-center">
                                        {file.has_tests ? (
                                            <CheckCircle2 className="inline text-accent-green" size={16} />
                                        ) : (
                                            <XCircle className="inline text-destructive" size={16} />
                                        )}
                                    </td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                </Card>
            </div>

            {/* Proposed Tests */}
            {plan.proposed_tests.length > 0 ? (
                <div>
                    <h4 className="font-medium mb-2 flex items-center gap-2">
                        <FileEdit size={16} /> Proposed Tests ({plan.proposed_tests.length})
                    </h4>
                    <div className="space-y-2">
                        {plan.proposed_tests.map((test, idx) => (
                            <Card key={idx} className="p-4">
                                <div className="flex items-start justify-between">
                                    <div>
                                        <div className="font-medium">{test.description}</div>
                                        <div className="text-xs text-muted-foreground font-mono mt-1">
                                            {test.target_file} → {test.test_file}
                                        </div>
                                    </div>
                                    <Badge
                                        variant={test.priority === 'high' ? 'destructive' : test.priority === 'medium' ? 'warning' : 'outline'}
                                    >
                                        {test.priority}
                                    </Badge>
                                </div>
                            </Card>
                        ))}
                    </div>
                </div>
            ) : (
                <div className="text-center py-8 text-muted-foreground">
                    <CheckCircle2 size={48} className="inline text-accent-green mb-2" />
                    <p className="font-medium text-foreground">All changed files have tests!</p>
                </div>
            )}
        </div>
    );
};

const CoverageCard: React.FC<{ coverage: CoverageResult }> = ({ coverage }) => {
    const coverageColor = coverage.coverage_percent >= 80 ? 'text-accent-green' :
        coverage.coverage_percent >= 50 ? 'text-accent-amber' : 'text-destructive';

    return (
        <div className="space-y-4">
            {/* Coverage Ring */}
            <div className="flex items-center justify-center py-8">
                <div className="relative w-40 h-40">
                    <svg className="w-full h-full transform -rotate-90">
                        <circle
                            cx="80" cy="80" r="70"
                            stroke="currentColor"
                            strokeWidth="12"
                            fill="none"
                            className="text-muted"
                        />
                        <circle
                            cx="80" cy="80" r="70"
                            stroke="currentColor"
                            strokeWidth="12"
                            fill="none"
                            strokeDasharray={`${coverage.coverage_percent * 4.4} 440`}
                            strokeLinecap="round"
                            className={coverageColor}
                        />
                    </svg>
                    <div className="absolute inset-0 flex flex-col items-center justify-center">
                        <span className={`text-3xl font-bold ${coverageColor}`}>
                            {coverage.coverage_percent.toFixed(0)}%
                        </span>
                        <span className="text-sm text-muted-foreground">Coverage</span>
                    </div>
                </div>
            </div>

            {/* Stats */}
            <div className="grid grid-cols-3 gap-4">
                <Card className="p-4 text-center">
                    <div className="text-sm text-muted-foreground">Testable Files</div>
                    <div className="text-2xl font-bold">{coverage.total}</div>
                </Card>
                <Card className="p-4 text-center border-accent-green/30">
                    <div className="text-sm text-muted-foreground">With Tests</div>
                    <div className="text-2xl font-bold text-accent-green">{coverage.covered}</div>
                </Card>
                <Card className="p-4 text-center border-destructive/30">
                    <div className="text-sm text-muted-foreground">Without Tests</div>
                    <div className="text-2xl font-bold text-destructive">{coverage.uncovered}</div>
                </Card>
            </div>

            {/* Status Message */}
            <div className={`text-center py-4 rounded-lg ${coverage.coverage_percent >= 80 ? 'bg-accent-green/10' : 'bg-accent-amber/10'}`}>
                {coverage.coverage_percent >= 80 ? (
                    <>
                        <CheckCircle2 size={24} className="inline text-accent-green mb-1" />
                        <p className="font-medium text-accent-green">Good coverage! All changed files have tests.</p>
                    </>
                ) : (
                    <>
                        <AlertTriangle size={24} className="inline text-accent-amber mb-1" />
                        <p className="font-medium text-accent-amber">Coverage below 80%. Run test plan to see what needs tests.</p>
                    </>
                )}
            </div>
        </div>
    );
};

// Main TestRunner component
export default function TestRunner() {
    const [baseBranch, setBaseBranch] = useState('main');
    const [activeTab, setActiveTab] = useState('run');
    const [testResult, setTestResult] = useState<TestResult | null>(null);
    const [testPlan, setTestPlan] = useState<TestPlanOutput | null>(null);
    const [coverage, setCoverage] = useState<CoverageResult | null>(null);
    const [lastError, setLastError] = useState<string | null>(null);
    const [showFixModal, setShowFixModal] = useState(false);
    const [showAgentPrompt, setShowAgentPrompt] = useState(false);
    const [agentPrompt, setAgentPrompt] = useState('');

    // Auto-detect default branch on mount
    React.useEffect(() => {
        fetch('/api/git/default-branch')
            .then(res => res.json())
            .then(data => {
                if (data.branch) {
                    setBaseBranch(data.branch);
                }
            })
            .catch(() => {
                // Keep default 'main'
            });
    }, []);

    // Check if test plan exists and load it
    const [planExists, setPlanExists] = React.useState(false);

    const checkPlanStatus = React.useCallback(() => {
        fetch('/api/test/plan-status')
            .then(res => res.json())
            .then(data => {
                setPlanExists(data.exists);
                // Load the saved plan if it exists
                if (data.exists && data.plan) {
                    setTestPlan(data.plan);
                }
            })
            .catch(() => {
                setPlanExists(false);
            });
    }, []);

    React.useEffect(() => {
        checkPlanStatus();
    }, [checkPlanStatus]);

    // Check if tests have been written
    const [writeExists, setWriteExists] = React.useState(false);
    const [writeStatus, setWriteStatus] = React.useState<{
        completed_at?: string;
        status?: string;
        agent_name?: string;
        worktree_branch?: string;
        base_branch?: string;
    } | null>(null);

    const checkWriteStatus = React.useCallback(() => {
        fetch('/api/test/write-status')
            .then(res => res.json())
            .then(data => {
                setWriteExists(data.exists);
                if (data.exists) {
                    setWriteStatus({
                        completed_at: data.completed_at,
                        status: data.status,
                        agent_name: data.agent_name,
                        worktree_branch: data.worktree_branch,
                        base_branch: data.base_branch,
                    });
                } else {
                    setWriteStatus(null);
                }
            })
            .catch(() => {
                setWriteExists(false);
                setWriteStatus(null);
            });
    }, []);

    React.useEffect(() => {
        checkWriteStatus();
    }, [checkWriteStatus]);

    // Fetch test writer agent
    const { data: agentData, isLoading: loadingAgent } = useQuery({
        queryKey: ['test-agent'],
        queryFn: fetchTestWriterAgent,
    });

    // Mutations
    const runMutation = useMutation({
        mutationFn: () => runTests(baseBranch),
        onSuccess: (data) => {
            if (data.success && data.result) {
                setTestResult(data.result);
                setLastError(null);
                toast.success('Tests completed');
            } else {
                const errorMsg = data.error || 'Failed to run tests';
                setLastError(errorMsg);
                if (data.result) setTestResult(data.result);
                toast.error(errorMsg);
            }
        },
        onError: () => {
            const errorMsg = 'Failed to run tests';
            setLastError(errorMsg);
            toast.error(errorMsg);
        },
    });

    const planMutation = useMutation({
        mutationFn: () => planTests(baseBranch),
        onSuccess: (data) => {
            if (data.success && data.plan) {
                setTestPlan(data.plan);
                toast.success('Test plan generated');
                checkPlanStatus(); // Refresh plan status to show checkmark
            } else {
                toast.error(data.error || 'Failed to generate plan');
            }
        },
        onError: () => toast.error('Failed to generate plan'),
    });

    const writeMutation = useMutation({
        mutationFn: (customPrompt?: string) => writeTests(baseBranch, false, customPrompt),
        onSuccess: (data) => {
            if (data.success) {
                toast.success(data.message || 'Tests written successfully');
                checkWriteStatus(); // Refresh write status to show checkmark
            } else {
                toast.error(data.error || 'Failed to write tests');
            }
        },
        onError: () => toast.error('Failed to write tests'),
    });

    const coverageMutation = useMutation({
        mutationFn: () => checkCoverage(baseBranch),
        onSuccess: (data) => {
            if (data.success && data.coverage) {
                setCoverage(data.coverage);
                toast.success('Coverage calculated');
            } else {
                toast.error(data.error || 'Failed to check coverage');
            }
        },
        onError: () => toast.error('Failed to check coverage'),
    });

    const isLoading = runMutation.isPending || planMutation.isPending || writeMutation.isPending || coverageMutation.isPending;

    return (
        <div className="h-full flex flex-col bg-background text-foreground">
            {/* Header */}
            <Card className="shrink-0 rounded-none border-x-0 border-t-0">
                <CardHeader className="pb-4">
                    <div className="flex items-center justify-between">
                        <div>
                            <CardTitle className="text-xl flex items-center gap-3">
                                <FlaskConical className="text-primary" size={24} />
                                Test Runner
                            </CardTitle>
                            <p className="text-muted-foreground text-sm mt-1">Run, plan, and write tests for your codebase</p>
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
                                <Badge variant="warning">⚠️ No test writer agent</Badge>
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
                            <TabsTrigger value="run" className="data-[state=active]:bg-background">
                                <Play size={14} className="mr-1" /> Run Tests
                            </TabsTrigger>
                            <TabsTrigger value="plan" className="data-[state=active]:bg-background">
                                <FileSearch size={14} className="mr-1" /> Plan
                                {planExists && <CheckCircle2 size={12} className="ml-1 text-green-500" />}
                            </TabsTrigger>
                            <TabsTrigger value="write" className="data-[state=active]:bg-background">
                                <FileEdit size={14} className="mr-1" /> Write
                                {writeExists && <CheckCircle2 size={12} className="ml-1 text-green-500" />}
                            </TabsTrigger>
                            <TabsTrigger value="coverage" className="data-[state=active]:bg-background">
                                <BarChart3 size={14} className="mr-1" /> Coverage
                            </TabsTrigger>
                        </TabsList>
                    </div>

                    <ScrollArea className="flex-1">
                        {/* Run Tests Tab */}
                        <TabsContent value="run" className="p-6 mt-0">
                            <div className="space-y-4">
                                <div className="flex items-center gap-4">
                                    <Button onClick={() => runMutation.mutate()} disabled={isLoading}>
                                        {runMutation.isPending ? (
                                            <><Loader2 size={16} className="mr-2 animate-spin" /> Running...</>
                                        ) : (
                                            <><Play size={16} className="mr-2" /> Run Tests</>
                                        )}
                                    </Button>
                                    <span className="text-sm text-muted-foreground">
                                        Execute the project test suite in a Docker sandbox
                                    </span>
                                </div>

                                {testResult && <TestResultCard result={testResult} />}

                                {/* Error display with AI fix option */}
                                {lastError && (
                                    <Card className="p-4 border-destructive/50 bg-destructive/5">
                                        <div className="flex items-start gap-3">
                                            <XCircle className="text-destructive shrink-0 mt-0.5" size={18} />
                                            <div className="flex-1 min-w-0">
                                                <h4 className="font-medium text-destructive">Test Error</h4>
                                                <pre className="text-xs text-muted-foreground mt-2 whitespace-pre-wrap font-mono bg-background p-2 rounded overflow-x-auto">
                                                    {lastError}
                                                </pre>
                                            </div>
                                        </div>
                                    </Card>
                                )}
                            </div>
                        </TabsContent>

                        {/* Plan Tab */}
                        <TabsContent value="plan" className="p-6 mt-0">
                            <div className="space-y-4">
                                <div className="flex items-center gap-4">
                                    <Button onClick={() => planMutation.mutate()} disabled={isLoading}>
                                        {planMutation.isPending ? (
                                            <><Loader2 size={16} className="mr-2 animate-spin" /> Analyzing...</>
                                        ) : (
                                            <><FileSearch size={16} className="mr-2" /> Generate Plan</>
                                        )}
                                    </Button>
                                    <span className="text-sm text-muted-foreground">
                                        Analyze changes vs <code className="bg-muted px-1 rounded">{baseBranch}</code> and identify test gaps
                                    </span>
                                </div>

                                {testPlan && <TestPlanCard plan={testPlan} />}
                            </div>
                        </TabsContent>

                        {/* Write Tab */}
                        <TabsContent value="write" className="p-6 mt-0">
                            <div className="space-y-4">
                                <div className="flex items-center gap-4">
                                    <Button
                                        onClick={() => writeMutation.mutate(undefined)}
                                        disabled={isLoading || !agentData?.agent}
                                    >
                                        {writeMutation.isPending ? (
                                            <><Loader2 size={16} className="mr-2 animate-spin" /> Writing...</>
                                        ) : (
                                            <><FileEdit size={16} className="mr-2" /> Write Tests</>
                                        )}
                                    </Button>
                                    <span className="text-sm text-muted-foreground">
                                        Use the test writer agent to generate tests for uncovered code
                                    </span>
                                </div>

                                {/* Show completed test results */}
                                {writeExists && writeStatus && (
                                    <Card className="p-4 border-green-500/50 bg-green-500/5">
                                        <div className="flex items-start gap-3">
                                            <CheckCircle2 className="text-green-500 shrink-0 mt-0.5" size={20} />
                                            <div className="flex-1">
                                                <h4 className="font-medium text-green-700 dark:text-green-400">Tests Written Successfully</h4>
                                                <div className="text-sm text-muted-foreground mt-2 space-y-1">
                                                    {writeStatus.completed_at && (
                                                        <div>
                                                            <span className="font-medium">Completed:</span>{' '}
                                                            {new Date(writeStatus.completed_at).toLocaleString()}
                                                        </div>
                                                    )}
                                                    {writeStatus.agent_name && (
                                                        <div>
                                                            <span className="font-medium">Agent:</span> {writeStatus.agent_name}
                                                        </div>
                                                    )}
                                                    {writeStatus.status && (
                                                        <div>
                                                            <span className="font-medium">Status:</span>{' '}
                                                            <Badge variant={writeStatus.status === 'merged' ? 'success' : 'warning'}>
                                                                {writeStatus.status}
                                                            </Badge>
                                                        </div>
                                                    )}
                                                    {writeStatus.worktree_branch && (
                                                        <div>
                                                            <span className="font-medium">Branch:</span>{' '}
                                                            <code className="text-xs bg-muted px-1 py-0.5 rounded">{writeStatus.worktree_branch}</code>
                                                        </div>
                                                    )}
                                                </div>
                                            </div>
                                        </div>
                                    </Card>
                                )}

                                {!agentData?.agent && (
                                    <Card className="p-6 border-accent-amber/50 bg-accent-amber/5">
                                        <div className="flex items-start gap-3">
                                            <AlertTriangle className="text-accent-amber shrink-0" size={20} />
                                            <div>
                                                <h4 className="font-medium">No Test Writer Agent Configured</h4>
                                                <p className="text-sm text-muted-foreground mt-1">
                                                    To use automated test writing, configure an agent with the "Test Writer" role in the Agent Manager.
                                                </p>
                                            </div>
                                        </div>
                                    </Card>
                                )}

                                {agentData?.agent && (
                                    <Card className="p-4">
                                        <div className="flex items-center gap-3">
                                            <div className="w-10 h-10 rounded-full bg-primary/10 flex items-center justify-center">
                                                🤖
                                            </div>
                                            <div>
                                                <div className="font-medium">{agentData.agent.name}</div>
                                                <div className="text-sm text-muted-foreground">{agentData.agent.model}</div>
                                            </div>
                                            <Badge variant="info" className="ml-auto">Test Writer</Badge>
                                        </div>
                                    </Card>
                                )}
                            </div>
                        </TabsContent>

                        {/* Coverage Tab */}
                        <TabsContent value="coverage" className="p-6 mt-0">
                            <div className="space-y-4">
                                <div className="flex items-center gap-4">
                                    <Button onClick={() => coverageMutation.mutate()} disabled={isLoading}>
                                        {coverageMutation.isPending ? (
                                            <><Loader2 size={16} className="mr-2 animate-spin" /> Calculating...</>
                                        ) : (
                                            <><BarChart3 size={16} className="mr-2" /> Check Coverage</>
                                        )}
                                    </Button>
                                    <span className="text-sm text-muted-foreground">
                                        Analyze test coverage of changed files vs <code className="bg-muted px-1 rounded">{baseBranch}</code>
                                    </span>
                                </div>

                                {coverage && <CoverageCard coverage={coverage} />}
                            </div>
                        </TabsContent>
                    </ScrollArea>
                </Tabs>
            </div>

            {/* Floating AI Agent Button - always visible */}
            <div className="fixed bottom-6 right-6 z-50 flex flex-col items-end gap-2">
                {/* Expandable prompt panel */}
                {showAgentPrompt && (
                    <Card className="w-80 p-4 shadow-xl border-primary/20 animate-in slide-in-from-bottom-2">
                        <div className="flex items-center gap-2 mb-3">
                            <Bot className="text-primary" size={18} />
                            <span className="font-medium text-sm">Ask Test Agent</span>
                        </div>
                        <div className="flex gap-2">
                            <input
                                type="text"
                                value={agentPrompt}
                                onChange={(e) => setAgentPrompt(e.target.value)}
                                onKeyDown={(e) => {
                                    if (e.key === 'Enter' && agentPrompt.trim()) {
                                        // TODO: Send prompt to agent
                                        toast.info('Sending to agent...');
                                        writeMutation.mutate(agentPrompt.trim());
                                        setAgentPrompt('');
                                        setShowAgentPrompt(false);
                                    }
                                }}
                                placeholder="e.g., Set up Jest for this project"
                                className="flex-1 px-3 py-2 text-sm border rounded-md bg-background focus:outline-none focus:ring-2 focus:ring-primary/50"
                            />
                            <Button
                                size="sm"
                                onClick={() => {
                                    if (agentPrompt.trim()) {
                                        toast.info('Sending to agent...');
                                        writeMutation.mutate(agentPrompt.trim());
                                        setAgentPrompt('');
                                        setShowAgentPrompt(false);
                                    }
                                }}
                                disabled={!agentPrompt.trim() || writeMutation.isPending}
                            >
                                <Send size={14} />
                            </Button>
                        </div>
                        <div className="text-xs text-muted-foreground mt-2">
                            Press Enter to send • Agent: {agentData?.agent?.name || 'Not configured'}
                        </div>
                    </Card>
                )}

                {/* Main FAB */}
                <button
                    onClick={() => lastError ? setShowFixModal(true) : setShowAgentPrompt(!showAgentPrompt)}
                    className={`w-14 h-14 rounded-full shadow-lg flex items-center justify-center transition-all hover:scale-110 ${showAgentPrompt ? 'rotate-45' : ''
                        }`}
                    style={{
                        background: lastError
                            ? 'linear-gradient(135deg, #ef4444, #dc2626)'
                            : 'linear-gradient(135deg, #8b5cf6, #ec4899)',
                    }}
                    title={lastError ? "Fix error with AI" : "Ask test agent"}
                >
                    {lastError ? (
                        <AlertTriangle className="w-6 h-6 text-white" />
                    ) : (
                        <MessageCircle className="w-6 h-6 text-white" />
                    )}
                </button>
            </div>

            {/* Test Fix Modal */}
            {showFixModal && lastError && (
                <TestFixModal
                    error={lastError}
                    baseBranch={baseBranch}
                    onClose={() => {
                        setShowFixModal(false);
                        // Optionally clear error or re-run tests
                    }}
                />
            )}
        </div>
    );
}
