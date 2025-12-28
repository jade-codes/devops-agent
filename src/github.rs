use anyhow::{Context, Result};
use reqwest::Client;
use serde::Serialize;
use std::env;

#[derive(Serialize)]
struct GitHubComment {
    body: String,
}

pub async fn post_pr_comment(report: &str) -> Result<()> {
    let github_token =
        env::var("GITHUB_TOKEN").context("GITHUB_TOKEN environment variable not set")?;

    let repo = env::var("GITHUB_REPOSITORY")
        .context("GITHUB_REPOSITORY not set (are you running in GitHub Actions?)")?;

    let pr_number = env::var("PR_NUMBER")
        .or_else(|_| extract_pr_from_github_ref())
        .context("Could not determine PR number")?;

    let client = Client::new();
    let url = format!("https://api.github.com/repos/{repo}/issues/{pr_number}/comments");

    let comment = GitHubComment {
        body: report.to_string(),
    };

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {github_token}"))
        .header("User-Agent", "devops-agent")
        .header("Accept", "application/vnd.github.v3+json")
        .json(&comment)
        .send()
        .await
        .context("Failed to post comment to GitHub")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("GitHub API error {status}: {body}");
    }

    Ok(())
}

fn extract_pr_from_github_ref() -> Result<String> {
    let github_ref = env::var("GITHUB_REF").context("GITHUB_REF not set")?;

    // GITHUB_REF format for PRs: refs/pull/:prNumber/merge
    if let Some(captures) = github_ref.strip_prefix("refs/pull/") {
        if let Some(pr_num) = captures.split('/').next() {
            return Ok(pr_num.to_string());
        }
    }

    anyhow::bail!("Could not extract PR number from GITHUB_REF: {github_ref}")
}
