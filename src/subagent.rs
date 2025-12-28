use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentRequest {
    pub agent: String,
    pub args: Vec<String>,
    pub working_dir: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentResponse {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

/// Run a subagent and return structured results
pub async fn run_subagent(request: AgentRequest) -> Result<AgentResponse> {
    // Use absolute path to the agent binary (binary name = agent name)
    let agent_path = std::env::current_dir()?.join(format!(
        "agents/{}/target/release/{}",
        request.agent, request.agent
    ));

    let mut cmd = Command::new(&agent_path);
    cmd.args(&request.args);

    if let Some(dir) = request.working_dir {
        cmd.current_dir(dir);
    }

    let output = cmd.output()?;

    Ok(AgentResponse {
        success: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        exit_code: output.status.code().unwrap_or(-1),
    })
}

/// Coverage Agent - Find missing tests
pub async fn run_coverage_agent(
    repo_path: &Path,
    threshold: u8,
    create_issues: bool,
) -> Result<AgentResponse> {
    let mut args = vec![
        "--repo-path".to_string(),
        repo_path.display().to_string(),
        "--threshold".to_string(),
        threshold.to_string(),
    ];

    if create_issues {
        args.push("--create-issues".to_string());
    }

    run_subagent(AgentRequest {
        agent: "coverage".to_string(),
        args,
        working_dir: None,
    })
    .await
}

/// TODO Scanner - Find TODO items
pub async fn run_todo_scanner(repo_path: &Path, create_issues: bool) -> Result<AgentResponse> {
    let mut args = vec!["--repo-path".to_string(), repo_path.display().to_string()];

    if create_issues {
        args.push("--create-issues".to_string());
    }

    run_subagent(AgentRequest {
        agent: "todo-scanner".to_string(),
        args,
        working_dir: None,
    })
    .await
}

/// TODO Resolver - Implement fixes
pub async fn run_todo_resolver(
    repo_path: &Path,
    issue_number: u32,
    create_pr: bool,
) -> Result<AgentResponse> {
    let mut args = vec![
        "--repo-path".to_string(),
        ".".to_string(), // Use . since we set working_dir to repo_path
        "--issue".to_string(),
        issue_number.to_string(),
    ];

    if create_pr {
        args.push("--create-pr".to_string());
    }

    run_subagent(AgentRequest {
        agent: "todo-resolver".to_string(),
        args,
        working_dir: Some(repo_path.to_path_buf()),
    })
    .await
}

/// Architecture Reviewer - Analyze design
pub async fn run_architecture_reviewer(repo_path: &Path) -> Result<AgentResponse> {
    run_subagent(AgentRequest {
        agent: "architecture-reviewer".to_string(),
        args: vec!["--repo-path".to_string(), repo_path.display().to_string()],
        working_dir: None,
    })
    .await
}

/// Feature Implementer - Implement features from specs
pub async fn run_feature_implementer(repo_path: &Path, issue_number: u32) -> Result<AgentResponse> {
    run_subagent(AgentRequest {
        agent: "feature-implementer".to_string(),
        args: vec![
            "--repo-path".to_string(),
            repo_path.display().to_string(),
            "--issue".to_string(),
            issue_number.to_string(),
        ],
        working_dir: None,
    })
    .await
}
