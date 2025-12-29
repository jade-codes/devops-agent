//! GitHub Copilot Agent spawning utilities
//!
//! This module provides helpers for spawning GitHub Copilot agents via `gh agent-task create`.

use anyhow::Result;
use std::path::Path;
use std::process::Command;

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
pub fn group_by_module(
    repo_path: &Path,
    issues: &[u32],
) -> Result<Vec<(String, Vec<(u32, String)>)>> {
    use std::collections::HashMap;

    let mut batches: HashMap<String, Vec<(u32, String)>> = HashMap::new();

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
fn categorize_by_path(title: &str) -> String {
    let patterns = [
        ("syster_lsp::server::core", "lsp-server-core"),
        ("syster_lsp::server::helpers", "lsp-helpers"),
        ("syster_lsp::ServerState", "lsp-server-state"),
        ("syster::semantic::symbol_table", "symbol-table"),
        ("syster::semantic::graphs::symmetric", "symmetric-graph"),
        ("syster::semantic::graphs::one_to_one", "one-to-one-graph"),
        ("syster::semantic::graphs::one_to_many", "one-to-many-graph"),
        ("syster::semantic::workspace::core", "workspace-core"),
        ("syster::semantic::workspace::file", "workspace-file"),
        (
            "syster::semantic::adapters::kerml::selection",
            "kerml-selection",
        ),
        ("syster::semantic::adapters::kerml::inlay", "kerml-inlay"),
        (
            "syster::semantic::adapters::kerml::folding",
            "kerml-folding",
        ),
        (
            "syster::semantic::adapters::kerml::validator",
            "kerml-validator",
        ),
        ("syster::semantic::adapters::sysml", "sysml-adapters"),
        ("syster::semantic::types::diagnostic", "types-diagnostic"),
        ("syster::semantic::types::error", "types-error"),
        (
            "syster::semantic::types::semantic_role",
            "types-semantic-role",
        ),
        ("syster::semantic::processors", "semantic-processors"),
        ("syster::syntax::sysml::visitor", "sysml-visitor"),
        ("syster::syntax::sysml::parser", "sysml-parser"),
        ("syster::syntax::sysml::ast", "sysml-ast-utils"),
        ("syster::syntax::kerml::parser", "kerml-parser"),
        ("syster::syntax::file", "syntax-file"),
        ("syster::core::parse_result", "core-parse-result"),
        ("syster::core", "core-misc"),
    ];

    for (pattern, module) in patterns {
        if title.contains(pattern) {
            return module.to_string();
        }
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
                &format!("repos/{{owner}}/{{repo}}/actions/runs/{}/rerun", run_id),
                "--method",
                "POST",
            ])
            .current_dir(repo_path)
            .output()?;

        results.push((run_id, rerun_result.status.success()));
    }

    Ok(results)
}
