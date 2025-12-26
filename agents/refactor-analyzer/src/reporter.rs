use anyhow::{Context, Result};
use std::process::Command;

use crate::analyzer::RefactorCandidate;

/// Output candidates as JSON
pub fn output_json(candidates: &[RefactorCandidate]) -> Result<()> {
    let json = serde_json::to_string_pretty(candidates)?;
    println!("{}", json);
    Ok(())
}

/// Output candidates as Markdown
pub fn output_markdown(candidates: &[RefactorCandidate]) -> Result<()> {
    println!("# Refactoring Candidates\n");

    for candidate in candidates {
        println!("## {} ({})", candidate.function, candidate.file);
        println!("**Lines:** {}-{}", candidate.line_start, candidate.line_end);
        println!("**Complexity:** {}/10", candidate.complexity_score);
        println!("**Priority Score:** {:.2}", candidate.priority_score());
        println!("\n**Metrics:**");
        println!("- Lines of code: {}", candidate.lines_of_code);
        println!("- Nesting depth: {}", candidate.nesting_depth);
        println!("- Parameters: {}", candidate.num_parameters);

        if !candidate.issues.is_empty() {
            println!("\n**Issues:**");
            for issue in &candidate.issues {
                println!("- {}", issue);
            }
        }
        println!();
    }

    Ok(())
}

/// Output candidates to console
pub fn output_console(candidates: &[RefactorCandidate]) -> Result<()> {
    for candidate in candidates {
        let priority_icon = if candidate.priority_score() >= 7.0 {
            "🔴"
        } else if candidate.priority_score() >= 4.0 {
            "🟡"
        } else {
            "🟢"
        };

        println!(
            "{} {} ({}:{})",
            priority_icon, candidate.function, candidate.file, candidate.line_start
        );
        println!(
            "   Complexity: {}/10 | Priority: {:.1} | Lines: {} | Nesting: {} | Params: {}",
            candidate.complexity_score,
            candidate.priority_score(),
            candidate.lines_of_code,
            candidate.nesting_depth,
            candidate.num_parameters
        );

        if !candidate.issues.is_empty() {
            for issue in &candidate.issues {
                println!("   ⚠️  {}", issue);
            }
        }
        println!();
    }
    Ok(())
}

/// Create GitHub issues for refactoring candidates
pub fn create_github_issues(candidates: &[RefactorCandidate], repo: &str) -> Result<()> {
    println!("\n🚀 Creating GitHub issues...");

    for candidate in candidates {
        let title = format!(
            "Refactor: {} ({}/10 complexity)",
            candidate.function, candidate.complexity_score
        );

        let body = format!(
            "**File:** {}:{}-{}\n**Complexity:** {}/10\n**Priority Score:** {:.2}\n\n**Metrics:**\n- Lines of code: {}\n- Nesting depth: {}\n- Parameters: {}\n\n**Issues:**\n{}\n\n**Suggested Actions:**\n- Break into smaller functions\n- Reduce nesting depth\n- Simplify conditional logic\n- Extract reusable components",
            candidate.file,
            candidate.line_start,
            candidate.line_end,
            candidate.complexity_score,
            candidate.priority_score(),
            candidate.lines_of_code,
            candidate.nesting_depth,
            candidate.num_parameters,
            candidate
                .issues
                .iter()
                .map(|i| format!("- {}", i))
                .collect::<Vec<_>>()
                .join("\n")
        );

        let label = if candidate.priority_score() >= 7.0 {
            "priority: high"
        } else if candidate.priority_score() >= 4.0 {
            "priority: medium"
        } else {
            "priority: low"
        };

        let output = Command::new("gh")
            .args([
                "issue",
                "create",
                "--repo",
                repo,
                "--title",
                &title,
                "--body",
                &body,
                "--label",
                label,
                "--label",
                "refactoring",
                "--label",
                "technical-debt",
            ])
            .output()
            .context("Failed to create GitHub issue")?;

        if output.status.success() {
            let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!("   ✓ Created: {}", url);
        } else {
            eprintln!("   ✗ Failed: {}", String::from_utf8_lossy(&output.stderr));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_json() {
        let candidates = vec![RefactorCandidate {
            file: "test.rs".to_string(),
            function: "test_func".to_string(),
            line_start: 10,
            line_end: 50,
            complexity_score: 8,
            lines_of_code: 40,
            nesting_depth: 4,
            num_parameters: 3,
            issues: vec!["High complexity".to_string()],
        }];

        assert!(output_json(&candidates).is_ok());
    }

    #[test]
    fn test_output_console() {
        let candidates = vec![RefactorCandidate {
            file: "test.rs".to_string(),
            function: "test_func".to_string(),
            line_start: 10,
            line_end: 50,
            complexity_score: 8,
            lines_of_code: 40,
            nesting_depth: 4,
            num_parameters: 3,
            issues: vec![],
        }];

        assert!(output_console(&candidates).is_ok());
    }
}
