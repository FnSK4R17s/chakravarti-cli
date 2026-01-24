import React, { useState } from 'react';
import { useQuery, useMutation } from '@tanstack/react-query';
import {
    FlaskConical, Play, FileSearch, FileEdit, BarChart3,
    Loader2, CheckCircle2, XCircle, AlertTriangle, Clock,
    ChevronDown, ChevronRight, Beaker
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

const writeTests = async (base: string, run: boolean): Promise<{ success: boolean; message?: string; error?: string }> => {
    const res = await fetch('/api/test/write', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ base, run }),
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
                toast.success('Tests completed');
            } else {
                toast.error(data.error || 'Failed to run tests');
            }
        },
        onError: () => toast.error('Failed to run tests'),
    });

    const planMutation = useMutation({
        mutationFn: () => planTests(baseBranch),
        onSuccess: (data) => {
            if (data.success && data.plan) {
                setTestPlan(data.plan);
                toast.success('Test plan generated');
            } else {
                toast.error(data.error || 'Failed to generate plan');
            }
        },
        onError: () => toast.error('Failed to generate plan'),
    });

    const writeMutation = useMutation({
        mutationFn: () => writeTests(baseBranch, false),
        onSuccess: (data) => {
            if (data.success) {
                toast.success(data.message || 'Tests written successfully');
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
                            </TabsTrigger>
                            <TabsTrigger value="write" className="data-[state=active]:bg-background">
                                <FileEdit size={14} className="mr-1" /> Write
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
                                        onClick={() => writeMutation.mutate()}
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
        </div>
    );
}
