use anyhow::{Context, Result};
use glob::Pattern;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

use crate::config::ChecklistConfig;

#[derive(Debug, Clone)]
pub struct FileToAnalyze {
    pub content: String,
    pub relative_path: String,
}

#[derive(Debug, Clone)]
pub struct ProjectContext {
    pub has_run_guidelines: bool,
    pub run_guidelines_output: Option<String>,
}

pub fn scan_repository(
    repo_path: &Path,
    config: &ChecklistConfig,
    pr_only: bool,
) -> Result<Vec<FileToAnalyze>> {
    let files = if pr_only {
        // Get changed files from git/GitHub context
        get_changed_files(repo_path)?
    } else {
        // Scan all files in repository
        scan_all_files(repo_path)?
    };

    // Filter by patterns
    let files = filter_by_patterns(files, config)?;

    Ok(files)
}

pub fn get_project_context(repo_path: &Path) -> Result<ProjectContext> {
    let makefile_path = repo_path.join("Makefile");

    let mut has_run_guidelines = false;
    let mut run_guidelines_output = None;

    if makefile_path.exists() {
        // Check if Makefile contains run-guidelines target
        if let Ok(makefile_content) = fs::read_to_string(&makefile_path) {
            has_run_guidelines = makefile_content.contains("run-guidelines:");

            if has_run_guidelines {
                // Try to run make run-guidelines
                run_guidelines_output = Some(run_make_guidelines(repo_path)?);
            }
        }
    }

    Ok(ProjectContext {
        has_run_guidelines,
        run_guidelines_output,
    })
}

fn run_make_guidelines(repo_path: &Path) -> Result<String> {
    use std::process::Command;

    let output = Command::new("make")
        .arg("run-guidelines")
        .current_dir(repo_path)
        .output()
        .context("Failed to run 'make run-guidelines'")?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    let combined = format!(
        "Exit code: {}\nStdout:\n{}\nStderr:\n{}",
        output.status.code().unwrap_or(-1),
        stdout,
        stderr
    );

    Ok(combined)
}

fn scan_all_files(repo_path: &Path) -> Result<Vec<FileToAnalyze>> {
    let mut files = Vec::new();

    for entry in WalkDir::new(repo_path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            if let Ok(content) = fs::read_to_string(entry.path()) {
                let relative = entry
                    .path()
                    .strip_prefix(repo_path)
                    .unwrap_or(entry.path())
                    .to_string_lossy()
                    .to_string();

                files.push(FileToAnalyze {
                    content,
                    relative_path: relative,
                });
            }
        }
    }

    Ok(files)
}

fn get_changed_files(repo_path: &Path) -> Result<Vec<FileToAnalyze>> {
    // This would integrate with GitHub Actions context
    // For now, we'll use git to get changed files
    use std::process::Command;

    let output = Command::new("git")
        .arg("diff")
        .arg("--name-only")
        .arg("HEAD")
        .current_dir(repo_path)
        .output()
        .context("Failed to run git diff")?;

    if !output.status.success() {
        // If not in a git repo or no changes, scan all files
        return scan_all_files(repo_path);
    }

    let changed_files = String::from_utf8_lossy(&output.stdout);
    let mut files = Vec::new();

    for file_path in changed_files.lines() {
        let full_path = repo_path.join(file_path);
        if full_path.exists() && full_path.is_file() {
            if let Ok(content) = fs::read_to_string(&full_path) {
                files.push(FileToAnalyze {
                    content,
                    relative_path: file_path.to_string(),
                });
            }
        }
    }

    Ok(files)
}

fn filter_by_patterns(
    files: Vec<FileToAnalyze>,
    config: &ChecklistConfig,
) -> Result<Vec<FileToAnalyze>> {
    let include_patterns: Vec<Pattern> = config
        .file_patterns
        .iter()
        .filter_map(|p| Pattern::new(p).ok())
        .collect();

    let exclude_patterns: Vec<Pattern> = config
        .exclude_patterns
        .iter()
        .filter_map(|p| Pattern::new(p).ok())
        .collect();

    let filtered: Vec<FileToAnalyze> = files
        .into_iter()
        .filter(|file| {
            let rel_path = file.relative_path.as_str();

            // Check if matches any include pattern
            let included =
                include_patterns.is_empty() || include_patterns.iter().any(|p| p.matches(rel_path));

            // Check if matches any exclude pattern
            let excluded = exclude_patterns.iter().any(|p| p.matches(rel_path));

            included && !excluded
        })
        .collect();

    Ok(filtered)
}
