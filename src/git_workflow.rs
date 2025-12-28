use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowResult {
    pub branch_name: String,
    pub commit_sha: String,
    pub pr_number: Option<u64>,
    pub pr_url: Option<String>,
}

pub struct GitWorkflow {
    repo_path: String,
    github_token: Option<String>,
}

impl GitWorkflow {
    pub fn new(repo_path: String) -> Self {
        let github_token = env::var("GITHUB_TOKEN").ok();
        Self {
            repo_path,
            github_token,
        }
    }

    /// Creates a new branch from main/master
    pub fn create_branch(&self, branch_name: &str) -> Result<()> {
        // Get current branch to return to if needed
        let _current_branch = self.get_current_branch()?;

        // Fetch latest
        self.run_git(&["fetch", "origin"])?;

        // Determine main branch name
        let main_branch = self.get_main_branch_name()?;

        // Create and checkout new branch
        self.run_git(&[
            "checkout",
            "-b",
            branch_name,
            &format!("origin/{main_branch}"),
        ])?;

        println!("✅ Created and checked out branch: {branch_name}");
        Ok(())
    }

    /// Stages and commits all changes
    pub fn commit_changes(&self, message: &str) -> Result<String> {
        // Stage all changes
        self.run_git(&["add", "-A"])?;

        // Commit
        self.run_git(&["commit", "-m", message])?;

        // Get commit SHA
        let sha = self.get_latest_commit_sha()?;
        println!("✅ Committed changes: {sha}");

        Ok(sha)
    }

    /// Pushes current branch to origin
    pub fn push_branch(&self, branch_name: &str) -> Result<()> {
        self.run_git(&["push", "-u", "origin", branch_name])?;
        println!("✅ Pushed branch to origin: {branch_name}");
        Ok(())
    }

    /// Creates a pull request via GitHub API
    pub async fn create_pull_request(
        &self,
        branch_name: &str,
        title: &str,
        body: &str,
    ) -> Result<(u64, String)> {
        let token = self.github_token.as_ref().context("GITHUB_TOKEN not set")?;

        let repo = self.get_repository_info()?;
        let base_branch = self.get_main_branch_name()?;

        let client = reqwest::Client::new();
        let url = format!(
            "https://api.github.com/repos/{}/{}/pulls",
            repo.owner, repo.name
        );

        #[derive(Serialize)]
        struct CreatePR {
            title: String,
            head: String,
            base: String,
            body: String,
        }

        #[derive(Deserialize)]
        struct PRResponse {
            number: u64,
            html_url: String,
        }

        let pr_request = CreatePR {
            title: title.to_string(),
            head: branch_name.to_string(),
            base: base_branch,
            body: body.to_string(),
        };

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {token}"))
            .header("User-Agent", "devops-agent")
            .header("Accept", "application/vnd.github.v3+json")
            .json(&pr_request)
            .send()
            .await
            .context("Failed to create PR")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("GitHub API error {status}: {body}");
        }

        let pr: PRResponse = response.json().await?;
        println!("✅ Created PR #{}: {}", pr.number, pr.html_url);

        Ok((pr.number, pr.html_url))
    }

    /// Complete workflow: branch -> commit -> push -> PR
    pub async fn complete_workflow(
        &self,
        issue_id: &str,
        commit_message: &str,
        pr_title: &str,
        pr_body: &str,
    ) -> Result<WorkflowResult> {
        let branch_name = format!("devops-agent/fix-{issue_id}");

        // Create branch
        self.create_branch(&branch_name)?;

        // Commit changes
        let commit_sha = self.commit_changes(commit_message)?;

        // Push branch
        self.push_branch(&branch_name)?;

        // Create PR
        let (pr_number, pr_url) = self
            .create_pull_request(&branch_name, pr_title, pr_body)
            .await?;

        Ok(WorkflowResult {
            branch_name,
            commit_sha,
            pr_number: Some(pr_number),
            pr_url: Some(pr_url),
        })
    }

    // Helper methods
    fn run_git(&self, args: &[&str]) -> Result<String> {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.repo_path)
            .output()
            .context(format!("Failed to run git {args:?}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Git command failed: {stderr}");
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    fn get_current_branch(&self) -> Result<String> {
        self.run_git(&["rev-parse", "--abbrev-ref", "HEAD"])
    }

    fn get_main_branch_name(&self) -> Result<String> {
        // Try to detect main branch (main or master)
        let branches = self.run_git(&["branch", "-r"])?;
        if branches.contains("origin/main") {
            Ok("main".to_string())
        } else {
            Ok("master".to_string())
        }
    }

    fn get_latest_commit_sha(&self) -> Result<String> {
        self.run_git(&["rev-parse", "HEAD"])
    }

    fn get_repository_info(&self) -> Result<RepoInfo> {
        let remote_url = self.run_git(&["remote", "get-url", "origin"])?;

        // Parse owner/repo from URL
        // Handles: git@github.com:owner/repo.git or https://github.com/owner/repo.git
        let parts: Vec<&str> = if remote_url.contains("github.com:") {
            remote_url.split("github.com:").collect()
        } else {
            remote_url.split("github.com/").collect()
        };

        if parts.len() < 2 {
            anyhow::bail!("Could not parse repository URL: {remote_url}");
        }

        let repo_part = parts[1].trim_end_matches(".git");
        let mut repo_split = repo_part.split('/');

        let owner = repo_split
            .next()
            .context("Could not extract owner")?
            .to_string();
        let name = repo_split
            .next()
            .context("Could not extract repo name")?
            .to_string();

        Ok(RepoInfo { owner, name })
    }
}

#[derive(Debug)]
struct RepoInfo {
    owner: String,
    name: String,
}
