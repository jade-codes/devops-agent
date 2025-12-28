use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

use crate::config::ChecklistConfig;
use crate::scanner::{FileToAnalyze, ProjectContext};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnalysisResult {
    pub file_path: String,
    pub findings: Vec<Finding>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Finding {
    pub category: String,
    pub rule: String,
    pub severity: String,
    pub passed: bool,
    pub message: String,
    pub line_number: Option<usize>,
}

#[derive(Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: usize,
    messages: Vec<Message>,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ClaudeResponse {
    content: Vec<ContentBlock>,
}

#[derive(Deserialize)]
struct ContentBlock {
    text: String,
}

pub async fn analyze_files(
    files: &[FileToAnalyze],
    config: &ChecklistConfig,
    project_context: &ProjectContext,
) -> Result<Vec<AnalysisResult>> {
    let api_key =
        env::var("ANTHROPIC_API_KEY").context("ANTHROPIC_API_KEY environment variable not set")?;

    let client = Client::new();
    let mut results = Vec::new();

    // Add project-level checks if needed
    if project_context.has_run_guidelines {
        let guidelines_result = check_run_guidelines(project_context, config)?;
        results.push(guidelines_result);
    }

    for file in files {
        println!("  Analyzing: {}", file.relative_path);
        let result = analyze_single_file(&client, &api_key, file, config).await?;
        results.push(result);
    }

    Ok(results)
}

async fn analyze_single_file(
    client: &Client,
    api_key: &str,
    file: &FileToAnalyze,
    config: &ChecklistConfig,
) -> Result<AnalysisResult> {
    let prompt = build_analysis_prompt(file, config);

    let request = ClaudeRequest {
        model: "claude-sonnet-4-20250514".to_string(),
        max_tokens: 4096,
        messages: vec![Message {
            role: "user".to_string(),
            content: prompt,
        }],
    };

    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&request)
        .send()
        .await
        .context("Failed to send request to Claude API")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(anyhow!("Claude API error {status}: {body}"));
    }

    let claude_response: ClaudeResponse = response
        .json()
        .await
        .context("Failed to parse Claude response")?;

    let analysis_text = claude_response
        .content
        .first()
        .map(|c| c.text.as_str())
        .unwrap_or("");

    let findings = parse_claude_response(analysis_text, config)?;

    Ok(AnalysisResult {
        file_path: file.relative_path.clone(),
        findings,
    })
}

fn build_analysis_prompt(file: &FileToAnalyze, config: &ChecklistConfig) -> String {
    let mut prompt = String::new();

    prompt.push_str("You are a code review assistant. Analyze the following code file against a checklist of rules.\n\n");
    prompt.push_str(&format!("File: {}\n\n", file.relative_path));
    prompt.push_str("Code:\n```\n");
    prompt.push_str(&file.content);
    prompt.push_str("\n```\n\n");

    prompt.push_str("Checklist Rules:\n");
    for (i, item) in config.items.iter().enumerate() {
        prompt.push_str(&format!(
            "{}. [{}] {} - {} (severity: {})\n",
            i + 1,
            item.category,
            item.rule,
            item.description,
            item.severity
        ));
    }

    prompt.push_str("\n\nFor each rule, determine if the code passes or fails. ");
    prompt.push_str("\n\nSPECIAL INSTRUCTIONS FOR TEST COVERAGE:");
    prompt.push_str("\n- Identify all public functions/methods that lack corresponding tests");
    prompt.push_str(
        "\n- Check if test files exist (files with .test., _test., or in tests/ directory)",
    );
    prompt.push_str("\n- For functions with complex logic (multiple branches, loops, error handling), verify comprehensive test coverage");
    prompt.push_str("\n- Flag any exported APIs without basic test coverage as errors");
    prompt.push_str("\n- Check test functions for conditional logic (if/else/match/for/while). Tests should be simple and procedural");
    prompt.push_str("\n- Flag tests containing branches or loops as they make tests harder to understand and debug");
    prompt.push_str("\n\nProvide your analysis in the following JSON format:\n\n");
    prompt.push_str("[\n");
    prompt.push_str("  {\n");
    prompt.push_str("    \"rule_number\": 1,\n");
    prompt.push_str("    \"passed\": true,\n");
    prompt.push_str("    \"message\": \"Explanation of why it passed or failed\",\n");
    prompt.push_str("    \"line_number\": 42\n");
    prompt.push_str("  }\n");
    prompt.push_str("]\n\n");
    prompt.push_str("Include line_number only if you can identify a specific line. ");
    prompt.push_str("Only include rules that have findings (failures or notable passes). ");
    prompt.push_str("Respond ONLY with the JSON array, no other text.");

    prompt
}

#[derive(Deserialize)]
struct ClaudeFindings {
    rule_number: usize,
    passed: bool,
    message: String,
    line_number: Option<usize>,
}

fn parse_claude_response(response: &str, config: &ChecklistConfig) -> Result<Vec<Finding>> {
    // Extract JSON from response (Claude might include markdown code blocks)
    let json_str = if let Some(start) = response.find('[') {
        if let Some(end) = response.rfind(']') {
            &response[start..=end]
        } else {
            response
        }
    } else {
        response
    };

    let claude_findings: Vec<ClaudeFindings> =
        serde_json::from_str(json_str).context("Failed to parse Claude's JSON response")?;

    let mut findings = Vec::new();

    for cf in claude_findings {
        if cf.rule_number == 0 || cf.rule_number > config.items.len() {
            continue;
        }

        let item = &config.items[cf.rule_number - 1];
        findings.push(Finding {
            category: item.category.clone(),
            rule: item.rule.clone(),
            severity: item.severity.clone(),
            passed: cf.passed,
            message: cf.message.clone(),
            line_number: cf.line_number,
        });
    }

    Ok(findings)
}

fn check_run_guidelines(
    project_context: &ProjectContext,
    config: &ChecklistConfig,
) -> Result<AnalysisResult> {
    let mut findings = Vec::new();

    // Find the run-guidelines rule in config
    let rule_item = config
        .items
        .iter()
        .find(|item| item.rule.contains("run-guidelines"));

    if let Some(rule) = rule_item {
        if let Some(output) = &project_context.run_guidelines_output {
            let passed = output.contains("Exit code: 0");

            let message = if passed {
                "make run-guidelines executed successfully".to_string()
            } else {
                format!("make run-guidelines failed:\n{output}")
            };

            findings.push(Finding {
                category: rule.category.clone(),
                rule: rule.rule.clone(),
                severity: rule.severity.clone(),
                passed,
                message,
                line_number: None,
            });
        }
    }

    Ok(AnalysisResult {
        file_path: "PROJECT_ROOT".to_string(),
        findings,
    })
}
