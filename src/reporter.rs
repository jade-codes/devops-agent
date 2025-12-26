use anyhow::Result;
use serde::Serialize;

use crate::analyzer::{AnalysisResult, Finding};

#[derive(Serialize)]
pub struct Report {
    pub summary: Summary,
    pub results: Vec<AnalysisResult>,
}

#[derive(Serialize)]
pub struct Summary {
    pub total_files: usize,
    pub total_findings: usize,
    pub errors: usize,
    pub warnings: usize,
    pub info: usize,
    pub passed_checks: usize,
}

pub fn generate_report(results: &[AnalysisResult], format: &str) -> Result<String> {
    let summary = calculate_summary(results);

    match format {
        "json" => generate_json_report(results, &summary),
        "markdown" => generate_markdown_report(results, &summary),
        _ => generate_console_report(results, &summary),
    }
}

fn calculate_summary(results: &[AnalysisResult]) -> Summary {
    let mut total_findings = 0;
    let mut errors = 0;
    let mut warnings = 0;
    let mut info = 0;
    let mut passed_checks = 0;

    for result in results {
        for finding in &result.findings {
            total_findings += 1;
            if finding.passed {
                passed_checks += 1;
            } else {
                match finding.severity.as_str() {
                    "error" => errors += 1,
                    "warning" => warnings += 1,
                    _ => info += 1,
                }
            }
        }
    }

    Summary {
        total_files: results.len(),
        total_findings,
        errors,
        warnings,
        info,
        passed_checks,
    }
}

fn generate_json_report(results: &[AnalysisResult], summary: &Summary) -> Result<String> {
    let report = Report {
        summary: Summary {
            total_files: summary.total_files,
            total_findings: summary.total_findings,
            errors: summary.errors,
            warnings: summary.warnings,
            info: summary.info,
            passed_checks: summary.passed_checks,
        },
        results: results.to_vec(),
    };

    Ok(serde_json::to_string_pretty(&report)?)
}

fn generate_markdown_report(results: &[AnalysisResult], summary: &Summary) -> Result<String> {
    let mut md = String::new();

    md.push_str("# 🤖 DevOps Agent Analysis Report\n\n");

    // Summary
    md.push_str("## 📊 Summary\n\n");
    md.push_str(&format!("- **Files Analyzed**: {}\n", summary.total_files));
    md.push_str(&format!(
        "- **Total Findings**: {}\n",
        summary.total_findings
    ));
    md.push_str(&format!(
        "- **✅ Passed Checks**: {}\n",
        summary.passed_checks
    ));
    md.push_str(&format!("- **❌ Errors**: {}\n", summary.errors));
    md.push_str(&format!("- **⚠️  Warnings**: {}\n", summary.warnings));
    md.push_str(&format!("- **ℹ️  Info**: {}\n\n", summary.info));

    // Details
    if summary.errors + summary.warnings + summary.info > 0 {
        md.push_str("## 🔍 Findings\n\n");

        for result in results {
            let failures: Vec<&Finding> = result.findings.iter().filter(|f| !f.passed).collect();

            if !failures.is_empty() {
                md.push_str(&format!("### 📄 `{}`\n\n", result.file_path));

                for finding in failures {
                    let icon = match finding.severity.as_str() {
                        "error" => "❌",
                        "warning" => "⚠️",
                        _ => "ℹ️",
                    };

                    md.push_str(&format!(
                        "{} **{}**: {}\n",
                        icon, finding.category, finding.rule
                    ));

                    if let Some(line) = finding.line_number {
                        md.push_str(&format!("   - Line: {}\n", line));
                    }

                    md.push_str(&format!("   - {}\n\n", finding.message));
                }
            }
        }
    } else {
        md.push_str("## ✅ All checks passed!\n\n");
        md.push_str("No issues found in the analyzed files.\n");
    }

    Ok(md)
}

fn generate_console_report(results: &[AnalysisResult], summary: &Summary) -> Result<String> {
    let mut output = String::new();

    output.push_str("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    output.push_str("  🤖 DevOps Agent ANALYSIS REPORT\n");
    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n");

    output.push_str(&format!("📊 Files Analyzed: {}\n", summary.total_files));
    output.push_str(&format!("✅ Passed Checks:  {}\n", summary.passed_checks));
    output.push_str(&format!("❌ Errors:         {}\n", summary.errors));
    output.push_str(&format!("⚠️  Warnings:       {}\n", summary.warnings));
    output.push_str(&format!("ℹ️  Info:           {}\n", summary.info));

    if summary.errors + summary.warnings + summary.info > 0 {
        output.push_str("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
        output.push_str("  🔍 FINDINGS\n");
        output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

        for result in results {
            let failures: Vec<&Finding> = result.findings.iter().filter(|f| !f.passed).collect();

            if !failures.is_empty() {
                output.push_str(&format!("\n📄 {}\n", result.file_path));
                output.push_str("─────────────────────────────────────────────────\n");

                for finding in failures {
                    let icon = match finding.severity.as_str() {
                        "error" => "❌",
                        "warning" => "⚠️",
                        _ => "ℹ️",
                    };

                    output.push_str(&format!(
                        "{} [{}] {}\n",
                        icon, finding.category, finding.rule
                    ));

                    if let Some(line) = finding.line_number {
                        output.push_str(&format!("   Line: {}\n", line));
                    }

                    output.push_str(&format!("   {}\n\n", finding.message));
                }
            }
        }
    } else {
        output.push_str("\n✅ All checks passed! No issues found.\n");
    }

    output.push_str("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    Ok(output)
}
