// Contract definitions for Test and QA commands
// Path: specs/014-test-qa-commands/contracts/commands.ts

// ============================================================================
// ckrv test run
// ============================================================================

export interface TestRunArgs {
    /** Branch to compare against (default: "main") */
    base?: string;
    /** Run in Docker sandbox */
    sandbox?: boolean;
    /** Output format */
    json?: boolean;
}

export interface TestRunOutput {
    success: boolean;
    total: number;
    passed: number;
    failed: number;
    skipped: number;
    duration_ms: number;
    failures: TestFailure[];
    framework: string;
}

export interface TestFailure {
    name: string;
    file: string;
    line?: number;
    message: string;
    stdout?: string;
    stderr?: string;
}

// ============================================================================
// ckrv test plan
// ============================================================================

export interface TestPlanArgs {
    /** Branch to compare against (default: "main") */
    base?: string;
    /** Output format */
    json?: boolean;
}

export interface TestPlanOutput {
    plan_id: string;
    created_at: string;
    base_branch: string;
    changed_files: ChangedFile[];
    proposed_tests: ProposedTest[];
    coverage_gaps: string[];
}

export interface ChangedFile {
    path: string;
    change_type: "added" | "modified" | "deleted" | "renamed";
    lines_added: number;
    lines_removed: number;
    has_tests: boolean;
}

export interface ProposedTest {
    target_file: string;
    test_file: string;
    test_type: "unit" | "integration" | "e2e" | "contract";
    description: string;
    priority: "high" | "medium" | "low";
}

// ============================================================================
// ckrv test write
// ============================================================================

export interface TestWriteArgs {
    /** Run tests after writing */
    run?: boolean;
    /** Branch to compare against */
    base?: string;
    /** Output format */
    json?: boolean;
}

export interface TestWriteOutput {
    success: boolean;
    tests_written: number;
    files_created: string[];
    files_modified: string[];
    test_results?: TestRunOutput;
    agent_id: string;
}

// ============================================================================
// ckrv qa review
// ============================================================================

export interface QAReviewArgs {
    /** Branch to compare against (default: "main") */
    base?: string;
    /** Output file path */
    output?: string;
    /** Output format */
    json?: boolean;
}

export interface QAReviewOutput {
    report_id: string;
    created_at: string;
    base_branch: string;
    issues: QAIssue[];
    summary: QASummary;
    agent_id: string;
}

export interface QAIssue {
    id: string;
    file: string;
    line?: number;
    severity: "critical" | "major" | "minor" | "info";
    category:
    | "code_quality"
    | "potential_bug"
    | "error_handling"
    | "security"
    | "performance"
    | "documentation"
    | "best_practice";
    message: string;
    suggestion?: string;
}

export interface QASummary {
    total_issues: number;
    critical: number;
    major: number;
    minor: number;
    info: number;
    files_reviewed: number;
    verdict: "pass" | "fail" | "review";
}

// ============================================================================
// ckrv qa report
// ============================================================================

export interface QAReportArgs {
    /** Include all analysis types */
    full?: boolean;
    /** Branch to compare against */
    base?: string;
    /** Output file path */
    output?: string;
    /** Output format */
    json?: boolean;
}

export interface QAReportOutput extends QAReviewOutput {
    /** Additional sections for full report */
    bug_analysis?: BugAnalysis;
    security_scan?: SecurityScan;
}

export interface BugAnalysis {
    potential_bugs: QAIssue[];
    edge_cases: string[];
    missing_validations: string[];
}

export interface SecurityScan {
    vulnerabilities: QAIssue[];
    secrets_detected: boolean;
    permission_issues: string[];
}

// ============================================================================
// Exit Codes
// ============================================================================

export const ExitCodes = {
    SUCCESS: 0,
    TEST_FAILURE: 1,
    QA_CRITICAL: 1,
    USER_CANCELLED: 2,
    CONFIG_ERROR: 3,
    NO_AGENT: 4,
} as const;
