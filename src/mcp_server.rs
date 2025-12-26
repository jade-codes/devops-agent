mod config;
mod git_workflow;
mod scanner;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

const SERVER_NAME: &str = "devops-agent";
const SERVER_VERSION: &str = "0.1.0";

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    eprintln!("🤖 DevOps Agent MCP Server starting...");
    eprintln!("💡 Connect this server to VS Code Copilot via MCP settings");

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(e) => {
                eprintln!("Failed to parse request: {}", e);
                continue;
            }
        };

        let response = handle_request(request).await;
        let response_str = serde_json::to_string(&response)?;

        writeln!(stdout, "{}", response_str)?;
        stdout.flush()?;
    }

    Ok(())
}

async fn handle_request(request: JsonRpcRequest) -> JsonRpcResponse {
    let id = request.id.clone();

    match request.method.as_str() {
        "initialize" => handle_initialize(id),
        "tools/list" => handle_tools_list(id),
        "tools/call" => handle_tool_call(id, request.params).await,
        _ => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: format!("Method not found: {}", request.method),
            }),
        },
    }
}

fn handle_initialize(id: Option<Value>) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(json!({
            "protocolVersion": "2024-11-05",
            "serverInfo": {
                "name": SERVER_NAME,
                "version": SERVER_VERSION
            },
            "capabilities": {
                "tools": {}
            }
        })),
        error: None,
    }
}

fn handle_tools_list(id: Option<Value>) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(json!({
            "tools": [
                {
                    "name": "scan_repository",
                    "description": "Scan a repository and identify files that need to be analyzed against the checklist. Returns list of files and their content.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "repo_path": {
                                "type": "string",
                                "description": "Path to the repository to scan"
                            },
                            "checklist_path": {
                                "type": "string",
                                "description": "Path to checklist.yaml configuration",
                                "default": "checklist.yaml"
                            }
                        },
                        "required": ["repo_path"]
                    }
                },
                {
                    "name": "check_guidelines",
                    "description": "Run 'make run-guidelines' if it exists in the repository and return the results.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "repo_path": {
                                "type": "string",
                                "description": "Path to the repository"
                            }
                        },
                        "required": ["repo_path"]
                    }
                },
                {
                    "name": "create_fix_branch",
                    "description": "Create a new git branch for fixing an issue. Branch name will be 'devops-agent/fix-{issue_id}'",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "repo_path": {
                                "type": "string",
                                "description": "Path to the repository"
                            },
                            "issue_id": {
                                "type": "string",
                                "description": "Issue identifier (e.g., '123' or 'security-check')"
                            }
                        },
                        "required": ["repo_path", "issue_id"]
                    }
                },
                {
                    "name": "commit_and_push",
                    "description": "Commit all changes and push to remote. Assumes you're on the correct branch.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "repo_path": {
                                "type": "string",
                                "description": "Path to the repository"
                            },
                            "message": {
                                "type": "string",
                                "description": "Commit message"
                            },
                            "branch_name": {
                                "type": "string",
                                "description": "Branch name to push"
                            }
                        },
                        "required": ["repo_path", "message", "branch_name"]
                    }
                },
                {
                    "name": "create_pull_request",
                    "description": "Create a GitHub pull request for the current branch. Requires GITHUB_TOKEN environment variable.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "repo_path": {
                                "type": "string",
                                "description": "Path to the repository"
                            },
                            "branch_name": {
                                "type": "string",
                                "description": "Branch name containing the changes"
                            },
                            "title": {
                                "type": "string",
                                "description": "PR title"
                            },
                            "body": {
                                "type": "string",
                                "description": "PR description"
                            }
                        },
                        "required": ["repo_path", "branch_name", "title", "body"]
                    }
                },
                {
                    "name": "complete_workflow",
                    "description": "Execute complete workflow: create branch, commit changes, push, and create PR. Use this after fixes have been applied.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "repo_path": {
                                "type": "string",
                                "description": "Path to the repository"
                            },
                            "issue_id": {
                                "type": "string",
                                "description": "Issue identifier"
                            },
                            "commit_message": {
                                "type": "string",
                                "description": "Commit message"
                            },
                            "pr_title": {
                                "type": "string",
                                "description": "Pull request title"
                            },
                            "pr_body": {
                                "type": "string",
                                "description": "Pull request description"
                            }
                        },
                        "required": ["repo_path", "issue_id", "commit_message", "pr_title", "pr_body"]
                    }
                }
            ]
        })),
        error: None,
    }
}

async fn handle_tool_call(id: Option<Value>, params: Option<Value>) -> JsonRpcResponse {
    let params = match params {
        Some(p) => p,
        None => {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32602,
                    message: "Missing params".to_string(),
                }),
            }
        }
    };

    let tool_name = params["name"].as_str().unwrap_or("");
    let arguments = &params["arguments"];

    let result = match tool_name {
        "scan_repository" => tool_scan_repository(arguments).await,
        "check_guidelines" => tool_check_guidelines(arguments).await,
        "create_fix_branch" => tool_create_fix_branch(arguments).await,
        "commit_and_push" => tool_commit_and_push(arguments).await,
        "create_pull_request" => tool_create_pull_request(arguments).await,
        "complete_workflow" => tool_complete_workflow(arguments).await,
        _ => Err(anyhow::anyhow!("Unknown tool: {}", tool_name)),
    };

    match result {
        Ok(content) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "content": [{
                    "type": "text",
                    "text": content
                }]
            })),
            error: None,
        },
        Err(e) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code: -32000,
                message: format!("Tool execution failed: {}", e),
            }),
        },
    }
}

async fn tool_scan_repository(args: &Value) -> Result<String> {
    let repo_path = args["repo_path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("repo_path required"))?;
    let checklist_path = args["checklist_path"].as_str().unwrap_or("checklist.yaml");

    let repo_path = std::path::Path::new(repo_path);
    let checklist_path = std::path::Path::new(checklist_path);

    let config = config::load_checklist(checklist_path)?;
    let files = scanner::scan_repository(repo_path, &config, false)?;

    let mut output = format!("📊 Scanned Repository\n\n");
    output.push_str(&format!("Found {} files to analyze:\n\n", files.len()));

    for file in &files {
        output.push_str(&format!("### File: {}\n", file.relative_path));
        output.push_str(&format!("```\n{}\n```\n\n", file.content));
    }

    output.push_str(&format!("\n📋 Checklist Rules ({}):\n", config.items.len()));
    for (i, item) in config.items.iter().enumerate() {
        output.push_str(&format!(
            "{}. [{}] {} - {}\n",
            i + 1,
            item.severity,
            item.rule,
            item.description
        ));
    }

    Ok(output)
}

async fn tool_check_guidelines(args: &Value) -> Result<String> {
    let repo_path = args["repo_path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("repo_path required"))?;

    let repo_path = std::path::Path::new(repo_path);
    let context = scanner::get_project_context(repo_path)?;

    if !context.has_run_guidelines {
        return Ok("❌ No 'make run-guidelines' target found in repository".to_string());
    }

    if let Some(output) = &context.run_guidelines_output {
        Ok(format!("🔧 Make run-guidelines Results:\n\n{}", output))
    } else {
        Ok("⚠️ run-guidelines target found but not executed".to_string())
    }
}

async fn tool_create_fix_branch(args: &Value) -> Result<String> {
    let repo_path = args["repo_path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("repo_path required"))?;
    let issue_id = args["issue_id"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("issue_id required"))?;

    let workflow = git_workflow::GitWorkflow::new(repo_path.to_string());
    let branch_name = format!("devops-agent/fix-{}", issue_id);

    workflow.create_branch(&branch_name)?;

    Ok(format!(
        "✅ Created and checked out branch: {}\n\nYou can now make changes to fix the issue.",
        branch_name
    ))
}

async fn tool_commit_and_push(args: &Value) -> Result<String> {
    let repo_path = args["repo_path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("repo_path required"))?;
    let message = args["message"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("message required"))?;
    let branch_name = args["branch_name"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("branch_name required"))?;

    let workflow = git_workflow::GitWorkflow::new(repo_path.to_string());

    let commit_sha = workflow.commit_changes(message)?;
    workflow.push_branch(branch_name)?;

    Ok(format!(
        "✅ Committed and pushed changes\n\nCommit: {}\nBranch: {}",
        commit_sha, branch_name
    ))
}

async fn tool_create_pull_request(args: &Value) -> Result<String> {
    let repo_path = args["repo_path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("repo_path required"))?;
    let branch_name = args["branch_name"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("branch_name required"))?;
    let title = args["title"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("title required"))?;
    let body = args["body"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("body required"))?;

    let workflow = git_workflow::GitWorkflow::new(repo_path.to_string());
    let (pr_number, pr_url) = workflow
        .create_pull_request(branch_name, title, body)
        .await?;

    Ok(format!(
        "✅ Created Pull Request\n\nPR #{}: {}",
        pr_number, pr_url
    ))
}

async fn tool_complete_workflow(args: &Value) -> Result<String> {
    let repo_path = args["repo_path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("repo_path required"))?;
    let issue_id = args["issue_id"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("issue_id required"))?;
    let commit_message = args["commit_message"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("commit_message required"))?;
    let pr_title = args["pr_title"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("pr_title required"))?;
    let pr_body = args["pr_body"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("pr_body required"))?;

    let workflow = git_workflow::GitWorkflow::new(repo_path.to_string());
    let result = workflow
        .complete_workflow(issue_id, commit_message, pr_title, pr_body)
        .await?;

    Ok(format!(
        "✅ Complete Workflow Executed\n\n\
        Branch: {}\n\
        Commit: {}\n\
        PR #{}: {}\n\n\
        🎉 Ready to move to the next issue!",
        result.branch_name,
        result.commit_sha,
        result.pr_number.unwrap_or(0),
        result.pr_url.as_deref().unwrap_or("N/A")
    ))
}
