use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;
use walkdir::WalkDir;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Note {
    pub file: String,
    pub line: usize,
    pub note_type: String,
    pub content: String,
    pub severity: Severity,
    pub category: String,
}

/// Parse severity string into enum
pub fn parse_severity(s: &str) -> Result<Severity> {
    match s.to_lowercase().as_str() {
        "low" => Ok(Severity::Low),
        "medium" => Ok(Severity::Medium),
        "high" => Ok(Severity::High),
        _ => anyhow::bail!("Invalid severity: {}", s),
    }
}

/// Scan a directory for important notes
pub fn scan_directory(path: &str) -> Result<Vec<Note>> {
    let mut notes = Vec::new();

    for entry in WalkDir::new(path)
        .into_iter()
        .filter_entry(|e| !is_excluded(e.path()))
    {
        let entry = entry?;
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension() {
                if is_code_file(ext.to_str().unwrap_or("")) {
                    if let Ok(file_notes) = scan_file(entry.path()) {
                        notes.extend(file_notes);
                    }
                }
            }
        }
    }

    Ok(notes)
}

/// Check if path should be excluded
fn is_excluded(path: &Path) -> bool {
    let excluded = ["target", "node_modules", ".git", "dist", "build", "vendor"];
    path.components().any(|c| {
        if let Some(s) = c.as_os_str().to_str() {
            excluded.contains(&s)
        } else {
            false
        }
    })
}

/// Check if file extension indicates a code file
fn is_code_file(ext: &str) -> bool {
    matches!(
        ext,
        "rs" | "py" | "js" | "ts" | "go" | "java" | "cpp" | "c" | "h" | "hpp" | "rb" | "php"
    )
}

/// Scan a single file for notes
pub fn scan_file(path: &Path) -> Result<Vec<Note>> {
    let content = fs::read_to_string(path)?;
    let mut notes = Vec::new();

    // Patterns for different note types
    let patterns = vec![
        (r"//\s*NOTE:\s*(.+)", "NOTE", "documentation"),
        (r"//\s*IMPORTANT:\s*(.+)", "IMPORTANT", "documentation"),
        (r"//\s*WARNING:\s*(.+)", "WARNING", "safety"),
        (r"//\s*CAUTION:\s*(.+)", "CAUTION", "safety"),
        (r"//\s*PERF:\s*(.+)", "PERF", "performance"),
        (r"//\s*PERFORMANCE:\s*(.+)", "PERFORMANCE", "performance"),
        (r"//\s*OPTIMIZE:\s*(.+)", "OPTIMIZE", "performance"),
        (r"//\s*REFACTOR:\s*(.+)", "REFACTOR", "technical-debt"),
        (r"//\s*DEPRECATED:\s*(.+)", "DEPRECATED", "technical-debt"),
        (r"//\s*REVIEW:\s*(.+)", "REVIEW", "code-quality"),
        (r"//\s*QUESTION:\s*(.+)", "QUESTION", "clarification"),
        (r"//\s*CONSIDER:\s*(.+)", "CONSIDER", "enhancement"),
    ];

    for (pattern_str, note_type, category) in patterns {
        let pattern = Regex::new(pattern_str)?;
        for (line_num, line) in content.lines().enumerate() {
            if let Some(captures) = pattern.captures(line) {
                if let Some(content_match) = captures.get(1) {
                    let severity = determine_severity(note_type, content_match.as_str());
                    notes.push(Note {
                        file: path.display().to_string(),
                        line: line_num + 1,
                        note_type: note_type.to_string(),
                        content: content_match.as_str().trim().to_string(),
                        severity,
                        category: category.to_string(),
                    });
                }
            }
        }
    }

    Ok(notes)
}

/// Determine severity based on note type and content
pub fn determine_severity(note_type: &str, content: &str) -> Severity {
    let content_lower = content.to_lowercase();

    // High severity keywords
    if content_lower.contains("critical")
        || content_lower.contains("security")
        || content_lower.contains("unsafe")
        || content_lower.contains("panic")
        || content_lower.contains("crash")
    {
        return Severity::High;
    }

    // Note type based severity
    match note_type {
        "WARNING" | "CAUTION" | "DEPRECATED" => Severity::High,
        "IMPORTANT" | "REVIEW" | "REFACTOR" => Severity::Medium,
        _ => Severity::Low,
    }
}

/// Filter notes by minimum severity
pub fn filter_by_severity(notes: &[Note], min_severity: Severity) -> Vec<Note> {
    notes
        .iter()
        .filter(|n| n.severity >= min_severity)
        .cloned()
        .collect()
}

/// Output notes as JSON
pub fn output_json(notes: &[Note]) -> Result<()> {
    let json = serde_json::to_string_pretty(notes)?;
    println!("{}", json);
    Ok(())
}

/// Output notes as Markdown
pub fn output_markdown(notes: &[Note]) -> Result<()> {
    println!("# Code Notes Report\n");

    // Group by category
    let mut by_category: std::collections::HashMap<String, Vec<&Note>> =
        std::collections::HashMap::new();
    for note in notes {
        by_category
            .entry(note.category.clone())
            .or_default()
            .push(note);
    }

    for (category, category_notes) in by_category {
        println!("## {}\n", category.replace('-', " ").to_uppercase());
        for note in category_notes {
            println!(
                "- **{}** ({}:{}): {}",
                note.note_type, note.file, note.line, note.content
            );
        }
        println!();
    }

    Ok(())
}

/// Output notes to console
pub fn output_console(notes: &[Note]) -> Result<()> {
    for note in notes {
        let severity_icon = match note.severity {
            Severity::High => "🔴",
            Severity::Medium => "🟡",
            Severity::Low => "🟢",
        };
        println!(
            "{} {} [{}] {}:{}\n   {}",
            severity_icon, note.note_type, note.category, note.file, note.line, note.content
        );
    }
    Ok(())
}

/// Create GitHub issues for notes
pub fn create_github_issues(notes: &[Note], repo: &str) -> Result<()> {
    println!("\n🚀 Creating GitHub issues...");

    for note in notes {
        let title = format!("{}: {}", note.note_type, truncate(&note.content, 60));
        let body = format!(
            "**File:** {}:{}\n**Type:** {}\n**Category:** {}\n**Severity:** {:?}\n\n{}",
            note.file, note.line, note.note_type, note.category, note.severity, note.content
        );

        let label = match note.severity {
            Severity::High => "priority: high",
            Severity::Medium => "priority: medium",
            Severity::Low => "priority: low",
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
                &note.category,
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

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_severity() {
        assert_eq!(parse_severity("low").unwrap(), Severity::Low);
        assert_eq!(parse_severity("medium").unwrap(), Severity::Medium);
        assert_eq!(parse_severity("high").unwrap(), Severity::High);
        assert!(parse_severity("invalid").is_err());
    }

    #[test]
    fn test_determine_severity_keywords() {
        assert_eq!(
            determine_severity("NOTE", "This is a critical security issue"),
            Severity::High
        );
        assert_eq!(
            determine_severity("NOTE", "This might crash the app"),
            Severity::High
        );
        assert_eq!(
            determine_severity("NOTE", "Regular observation"),
            Severity::Low
        );
    }

    #[test]
    fn test_determine_severity_by_type() {
        assert_eq!(determine_severity("WARNING", "Be careful"), Severity::High);
        assert_eq!(
            determine_severity("IMPORTANT", "Need to address"),
            Severity::Medium
        );
        assert_eq!(determine_severity("NOTE", "Just a note"), Severity::Low);
    }

    #[test]
    fn test_filter_by_severity() {
        let notes = vec![
            Note {
                file: "test.rs".to_string(),
                line: 1,
                note_type: "NOTE".to_string(),
                content: "Low".to_string(),
                severity: Severity::Low,
                category: "doc".to_string(),
            },
            Note {
                file: "test.rs".to_string(),
                line: 2,
                note_type: "WARNING".to_string(),
                content: "High".to_string(),
                severity: Severity::High,
                category: "safety".to_string(),
            },
        ];

        let filtered = filter_by_severity(&notes, Severity::Medium);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].severity, Severity::High);
    }

    #[test]
    fn test_is_code_file() {
        assert!(is_code_file("rs"));
        assert!(is_code_file("py"));
        assert!(is_code_file("js"));
        assert!(!is_code_file("txt"));
        assert!(!is_code_file("md"));
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("this is a very long string", 10), "this is...");
    }

    #[test]
    fn test_scan_file_with_notes() {
        let test_file = std::env::temp_dir().join("test_notes.rs");
        fs::write(
            &test_file,
            "// NOTE: This is a test note\n// WARNING: Be careful here\n// PERF: Could be optimized\n",
        )
        .unwrap();

        let notes = scan_file(&test_file).unwrap();
        assert_eq!(notes.len(), 3);
        assert!(notes.iter().any(|n| n.note_type == "NOTE"));
        assert!(notes.iter().any(|n| n.note_type == "WARNING"));
        assert!(notes.iter().any(|n| n.note_type == "PERF"));

        fs::remove_file(test_file).ok();
    }
}
