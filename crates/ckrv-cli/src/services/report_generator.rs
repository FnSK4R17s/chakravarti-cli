//! Report generator - generate Markdown reports for test and QA results.

use serde::Serialize;

use super::test_framework::TestResult;

/// Severity level for QA issues
#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Critical,
    Major,
    Minor,
    Info,
}

impl Severity {
    pub fn emoji(&self) -> &'static str {
        match self {
            Severity::Critical => "🔴",
            Severity::Major => "🟡",
            Severity::Minor => "🟢",
            Severity::Info => "ℹ️",
        }
    }
    
    pub fn label(&self) -> &'static str {
        match self {
            Severity::Critical => "Critical",
            Severity::Major => "Major",
            Severity::Minor => "Minor",
            Severity::Info => "Info",
        }
    }
}

/// Category of QA issue
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueCategory {
    CodeQuality,
    PotentialBug,
    ErrorHandling,
    Security,
    Performance,
    Documentation,
    BestPractice,
}

impl IssueCategory {
    pub fn label(&self) -> &'static str {
        match self {
            IssueCategory::CodeQuality => "Code Quality",
            IssueCategory::PotentialBug => "Potential Bug",
            IssueCategory::ErrorHandling => "Error Handling",
            IssueCategory::Security => "Security",
            IssueCategory::Performance => "Performance",
            IssueCategory::Documentation => "Documentation",
            IssueCategory::BestPractice => "Best Practice",
        }
    }
}

/// A QA issue found by the agent
#[derive(Debug, Clone, Serialize)]
pub struct QAIssue {
    pub id: String,
    pub file: String,
    pub line: Option<u32>,
    pub severity: Severity,
    pub category: IssueCategory,
    pub message: String,
    pub suggestion: Option<String>,
}

/// Summary of QA review
#[derive(Debug, Clone, Serialize)]
pub struct QASummary {
    pub total_issues: u32,
    pub critical: u32,
    pub major: u32,
    pub minor: u32,
    pub info: u32,
    pub files_reviewed: u32,
    pub verdict: String,
}

impl QASummary {
    pub fn from_issues(issues: &[QAIssue], files_reviewed: u32) -> Self {
        let critical = issues.iter().filter(|i| i.severity == Severity::Critical).count() as u32;
        let major = issues.iter().filter(|i| i.severity == Severity::Major).count() as u32;
        let minor = issues.iter().filter(|i| i.severity == Severity::Minor).count() as u32;
        let info = issues.iter().filter(|i| i.severity == Severity::Info).count() as u32;
        
        let verdict = if critical > 0 {
            "fail".to_string()
        } else if major > 0 {
            "review".to_string()
        } else {
            "pass".to_string()
        };
        
        QASummary {
            total_issues: issues.len() as u32,
            critical,
            major,
            minor,
            info,
            files_reviewed,
            verdict,
        }
    }
}

/// Generate Markdown report for test results
pub fn generate_test_report(result: &TestResult, branch: &str, base: &str) -> String {
    let mut report = String::new();
    
    report.push_str("# Test Results\n\n");
    report.push_str(&format!("**Branch**: {}\n", branch));
    report.push_str(&format!("**Base**: {}\n", base));
    report.push_str(&format!("**Framework**: {}\n", result.framework));
    report.push_str(&format!("**Date**: {}\n\n", chrono::Utc::now().format("%Y-%m-%d %H:%M UTC")));
    
    // Summary table
    report.push_str("## Summary\n\n");
    report.push_str("| Metric | Value |\n");
    report.push_str("|--------|-------|\n");
    report.push_str(&format!("| Status | {} |\n", if result.success { "✅ PASS" } else { "❌ FAIL" }));
    report.push_str(&format!("| Total | {} |\n", result.total));
    report.push_str(&format!("| Passed | {} |\n", result.passed));
    report.push_str(&format!("| Failed | {} |\n", result.failed));
    report.push_str(&format!("| Skipped | {} |\n", result.skipped));
    report.push_str(&format!("| Duration | {:.2}s |\n", result.duration_ms as f64 / 1000.0));
    report.push_str("\n");
    
    // Failures section
    if !result.failures.is_empty() {
        report.push_str("## Failures\n\n");
        for failure in &result.failures {
            report.push_str(&format!("### {}\n\n", failure.name));
            if let Some(ref file) = failure.file {
                if let Some(line) = failure.line {
                    report.push_str(&format!("**File**: {}:{}\n\n", file, line));
                } else {
                    report.push_str(&format!("**File**: {}\n\n", file));
                }
            }
            report.push_str(&format!("```\n{}\n```\n\n", failure.message));
        }
    }
    
    // Next steps
    report.push_str("## Next Steps\n\n");
    if result.success {
        report.push_str("- ✅ All tests passed\n");
        report.push_str("- Run `ckrv qa review` for code quality check\n");
        report.push_str("- Run `ckrv promote` to create PR\n");
    } else {
        report.push_str("- ❌ Fix failing tests\n");
        report.push_str("- Run `ckrv test run` to re-verify\n");
    }
    
    report
}

/// Generate Markdown report for QA review
pub fn generate_qa_report(issues: &[QAIssue], branch: &str, base: &str) -> String {
    let mut report = String::new();
    let summary = QASummary::from_issues(issues, 0);
    
    report.push_str("# QA Review\n\n");
    report.push_str(&format!("**Branch**: {}\n", branch));
    report.push_str(&format!("**Compared to**: {}\n", base));
    report.push_str(&format!("**Date**: {}\n\n", chrono::Utc::now().format("%Y-%m-%d %H:%M UTC")));
    
    // Summary
    report.push_str("## Summary\n\n");
    report.push_str(&format!("**Verdict**: {}\n\n", match summary.verdict.as_str() {
        "pass" => "✅ PASS - No critical issues",
        "review" => "🟡 REVIEW - Major issues found",
        "fail" => "🔴 FAIL - Critical issues found",
        _ => "Unknown"
    }));
    
    report.push_str("| Severity | Count |\n");
    report.push_str("|----------|-------|\n");
    report.push_str(&format!("| 🔴 Critical | {} |\n", summary.critical));
    report.push_str(&format!("| 🟡 Major | {} |\n", summary.major));
    report.push_str(&format!("| 🟢 Minor | {} |\n", summary.minor));
    report.push_str(&format!("| ℹ️ Info | {} |\n", summary.info));
    report.push_str(&format!("| **Total** | {} |\n", summary.total_issues));
    report.push_str("\n");
    
    if issues.is_empty() {
        report.push_str("## Issues Found\n\n");
        report.push_str("No issues found. Great job! 🎉\n\n");
        return report;
    }
    
    // Group issues by severity
    let mut critical: Vec<&QAIssue> = issues.iter().filter(|i| i.severity == Severity::Critical).collect();
    let mut major: Vec<&QAIssue> = issues.iter().filter(|i| i.severity == Severity::Major).collect();
    let mut minor: Vec<&QAIssue> = issues.iter().filter(|i| i.severity == Severity::Minor).collect();
    let mut info: Vec<&QAIssue> = issues.iter().filter(|i| i.severity == Severity::Info).collect();
    
    // Sort each group by file
    critical.sort_by(|a, b| a.file.cmp(&b.file));
    major.sort_by(|a, b| a.file.cmp(&b.file));
    minor.sort_by(|a, b| a.file.cmp(&b.file));
    info.sort_by(|a, b| a.file.cmp(&b.file));
    
    report.push_str("## Issues Found\n\n");
    
    if !critical.is_empty() {
        report.push_str(&format!("### 🔴 Critical ({})\n\n", critical.len()));
        for issue in critical {
            report.push_str(&format_issue(issue));
        }
    }
    
    if !major.is_empty() {
        report.push_str(&format!("### 🟡 Major ({})\n\n", major.len()));
        for issue in major {
            report.push_str(&format_issue(issue));
        }
    }
    
    if !minor.is_empty() {
        report.push_str(&format!("### 🟢 Minor ({})\n\n", minor.len()));
        for issue in minor {
            report.push_str(&format_issue(issue));
        }
    }
    
    if !info.is_empty() {
        report.push_str(&format!("### ℹ️ Info ({})\n\n", info.len()));
        for issue in info {
            report.push_str(&format_issue(issue));
        }
    }
    
    report
}

fn format_issue(issue: &QAIssue) -> String {
    let mut s = String::new();
    
    let location = if let Some(line) = issue.line {
        format!("{}:{}", issue.file, line)
    } else {
        issue.file.clone()
    };
    
    s.push_str(&format!("**[{}]** {} - `{}`\n", issue.category.label(), issue.message, location));
    
    if let Some(ref suggestion) = issue.suggestion {
        s.push_str(&format!("  - **Fix**: {}\n", suggestion));
    }
    
    s.push_str("\n");
    s
}
