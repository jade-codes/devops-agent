//! GitHub Copilot Agent spawning utilities
//!
//! This module provides helpers for spawning GitHub Copilot agents via `gh agent-task create`.

use anyhow::Result;
use std::path::Path;
use std::process::Command;

/// Issue with number and title
pub type IssueWithTitle = (u32, String);

/// Batch of issues grouped by module name
pub type IssueBatch = (String, Vec<IssueWithTitle>);

/// Response from spawning a GitHub agent task
#[derive(Debug)]
pub struct AgentTaskResult {
    pub success: bool,
    pub message: String,
}

/// Spawn a GitHub Copilot agent task
pub fn spawn_agent(repo_path: &Path, task_description: &str) -> Result<AgentTaskResult> {
    let output = Command::new("gh")
        .args(["agent-task", "create", task_description])
        .current_dir(repo_path)
        .output()?;

    Ok(AgentTaskResult {
        success: output.status.success(),
        message: if output.status.success() {
            String::from_utf8_lossy(&output.stdout).to_string()
        } else {
            String::from_utf8_lossy(&output.stderr).to_string()
        },
    })
}

/// Fetch issue details from GitHub
pub fn fetch_issue(repo_path: &Path, issue_num: u32) -> Result<Option<(String, String)>> {
    let output = Command::new("gh")
        .args([
            "issue",
            "view",
            &issue_num.to_string(),
            "--json",
            "title,body",
        ])
        .current_dir(repo_path)
        .output()?;

    if !output.status.success() {
        return Ok(None);
    }

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap_or_default();
    let title = json["title"].as_str().unwrap_or("").to_string();
    let body = json["body"].as_str().unwrap_or("").to_string();

    Ok(Some((title, body)))
}

/// Fetch issue title only
pub fn fetch_issue_title(repo_path: &Path, issue_num: u32) -> Result<Option<String>> {
    let output = Command::new("gh")
        .args(["issue", "view", &issue_num.to_string(), "--json", "title"])
        .current_dir(repo_path)
        .output()?;

    if !output.status.success() {
        return Ok(None);
    }

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap_or_default();
    Ok(json["title"].as_str().map(|s| s.to_string()))
}

/// List issues by label
pub fn list_issues_by_label(repo_path: &Path, label: &str) -> Result<Vec<u32>> {
    let output = Command::new("gh")
        .args([
            "issue",
            "list",
            "--label",
            label,
            "--state",
            "open",
            "--limit",
            "150",
            "--json",
            "number",
            "--jq",
            ".[].number",
        ])
        .current_dir(repo_path)
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let issues: Vec<u32> = stdout
        .lines()
        .filter_map(|line| line.trim().parse().ok())
        .collect();

    Ok(issues)
}

/// List open PR numbers
pub fn list_open_prs(repo_path: &Path) -> Result<std::collections::HashSet<u32>> {
    let output = Command::new("gh")
        .args([
            "pr",
            "list",
            "--state",
            "open",
            "--limit",
            "150",
            "--json",
            "number",
            "--jq",
            ".[].number",
        ])
        .current_dir(repo_path)
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let prs: std::collections::HashSet<u32> = stdout
        .lines()
        .filter_map(|line| line.trim().parse().ok())
        .collect();

    Ok(prs)
}

/// Group issues by module based on function path in title
pub fn group_by_module(repo_path: &Path, issues: &[u32]) -> Result<Vec<IssueBatch>> {
    use std::collections::HashMap;

    let mut batches: HashMap<String, Vec<IssueWithTitle>> = HashMap::new();

    for &issue_num in issues {
        if let Some(title) = fetch_issue_title(repo_path, issue_num)? {
            let module = categorize_by_path(&title);
            batches.entry(module).or_default().push((issue_num, title));
        }
    }

    // Sort by batch size descending
    let mut result: Vec<_> = batches.into_iter().collect();
    result.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

    Ok(result)
}

/// Categorize an issue title into a module name based on function path
/// Extracts last 2 segments from Rust paths like `foo::bar::baz` -> `bar-baz`
fn categorize_by_path(title: &str) -> String {
    let re = regex::Regex::new(r"(\w+::)+\w+").unwrap();

    if let Some(m) = re.find(title) {
        let path = m.as_str();
        let segments: Vec<&str> = path.split("::").collect();

        // Use last 2 segments (or 1 if short path)
        let category = if segments.len() >= 2 {
            format!(
                "{}-{}",
                segments[segments.len() - 2],
                segments[segments.len() - 1]
            )
        } else {
            segments.last().unwrap_or(&"misc").to_string()
        };

        return category.to_lowercase().replace('_', "-");
    }

    "misc".to_string()
}

/// Rerun all workflow runs waiting for approval (action_required)
/// Uses API rerun since `gh run approve` only works for fork PRs
pub fn approve_pending_workflows(repo_path: &Path) -> Result<Vec<(u64, bool)>> {
    // Get workflow runs with action_required conclusion (waiting for approval)
    let output = Command::new("gh")
        .args([
            "run",
            "list",
            "--json",
            "databaseId,conclusion",
            "--jq",
            ".[] | select(.conclusion == \"action_required\") | .databaseId",
        ])
        .current_dir(repo_path)
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let run_ids: Vec<u64> = stdout
        .lines()
        .filter_map(|line| line.trim().parse().ok())
        .collect();

    let mut results = Vec::new();

    for run_id in run_ids {
        // Use API rerun endpoint - works for Copilot actor runs
        let rerun_result = Command::new("gh")
            .args([
                "api",
                &format!("repos/{{owner}}/{{repo}}/actions/runs/{run_id}/rerun"),
                "--method",
                "POST",
            ])
            .current_dir(repo_path)
            .output()?;

        results.push((run_id, rerun_result.status.success()));
    }

    Ok(results)
}

/// PR with failing checks
#[derive(Debug)]
pub struct FailingPr {
    pub number: u32,
    pub title: String,
    pub author: String,
}

/// List PRs with failing CI checks
pub fn list_failing_prs(repo_path: &Path) -> Result<Vec<FailingPr>> {
    let output = Command::new("gh")
        .args([
            "pr",
            "list",
            "--state",
            "open",
            "--limit",
            "100",
            "--json",
            "number,title,author,headRefName,statusCheckRollup",
        ])
        .current_dir(repo_path)
        .output()?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap_or_default();
    let mut failing = Vec::new();

    if let Some(prs) = json.as_array() {
        for pr in prs {
            let checks = pr.get("statusCheckRollup").and_then(|v| v.as_array());

            // Check if any check has failed
            let has_failure = checks.is_some_and(|checks| {
                checks.iter().any(|check| {
                    check.get("conclusion").and_then(|c| c.as_str()) == Some("FAILURE")
                })
            });

            if has_failure {
                failing.push(FailingPr {
                    number: pr.get("number").and_then(|n| n.as_u64()).unwrap_or(0) as u32,
                    title: pr
                        .get("title")
                        .and_then(|t| t.as_str())
                        .unwrap_or("")
                        .to_string(),
                    author: pr
                        .get("author")
                        .and_then(|a| a.get("login"))
                        .and_then(|l| l.as_str())
                        .unwrap_or("")
                        .to_string(),
                });
            }
        }
    }

    Ok(failing)
}

/// Comment on a PR
pub fn comment_on_pr(repo_path: &Path, pr_number: u32, comment: &str) -> Result<bool> {
    let output = Command::new("gh")
        .args(["pr", "comment", &pr_number.to_string(), "--body", comment])
        .current_dir(repo_path)
        .output()?;

    Ok(output.status.success())
}

/// PR with merge conflict info
#[derive(Debug)]
pub struct ConflictingPr {
    pub number: u32,
    pub title: String,
    pub author: String,
}

/// List PRs with merge conflicts (mergeable state is CONFLICTING)
pub fn list_conflicting_prs(repo_path: &Path) -> Result<Vec<ConflictingPr>> {
    let output = Command::new("gh")
        .args([
            "pr",
            "list",
            "--state",
            "open",
            "--limit",
            "100",
            "--json",
            "number,title,author,mergeable",
        ])
        .current_dir(repo_path)
        .output()?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap_or_default();
    let mut conflicting = Vec::new();

    if let Some(prs) = json.as_array() {
        for pr in prs {
            let mergeable = pr.get("mergeable").and_then(|m| m.as_str());

            if mergeable == Some("CONFLICTING") {
                conflicting.push(ConflictingPr {
                    number: pr.get("number").and_then(|n| n.as_u64()).unwrap_or(0) as u32,
                    title: pr
                        .get("title")
                        .and_then(|t| t.as_str())
                        .unwrap_or("")
                        .to_string(),
                    author: pr
                        .get("author")
                        .and_then(|a| a.get("login"))
                        .and_then(|l| l.as_str())
                        .unwrap_or("")
                        .to_string(),
                });
            }
        }
    }

    Ok(conflicting)
}

/// Close a PR
pub fn close_pr(repo_path: &Path, pr_number: u32) -> Result<bool> {
    let output = Command::new("gh")
        .args(["pr", "close", &pr_number.to_string()])
        .current_dir(repo_path)
        .output()?;

    Ok(output.status.success())
}
