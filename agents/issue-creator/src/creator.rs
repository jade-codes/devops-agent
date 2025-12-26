use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IssueRequest {
    pub title: String,
    pub body: Option<String>,
    pub labels: Option<Vec<String>>,
    pub assignees: Option<Vec<String>>,
    pub milestone: Option<String>,
    pub priority: Option<String>,
}

impl IssueRequest {
    pub fn validate(&self) -> Result<()> {
        if self.title.is_empty() {
            anyhow::bail!("Issue title cannot be empty");
        }
        if self.title.len() > 256 {
            anyhow::bail!("Issue title too long (max 256 characters)");
        }
        Ok(())
    }

    pub fn enhanced_labels(&self) -> Vec<String> {
        let mut labels = self.labels.clone().unwrap_or_default();

        // Add priority label if specified
        if let Some(priority) = &self.priority {
            match priority.to_lowercase().as_str() {
                "low" => labels.push("priority-low".to_string()),
                "medium" => labels.push("priority-medium".to_string()),
                "high" => labels.push("priority-high".to_string()),
                "critical" => labels.push("priority-critical".to_string()),
                _ => {}
            }
        }

        labels
    }
}

pub fn load_issue_from_json(path: &Path) -> Result<IssueRequest> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read issue file: {}", path.display()))?;

    let issue: IssueRequest =
        serde_json::from_str(&content).context("Failed to parse issue JSON")?;

    issue.validate()?;
    Ok(issue)
}

pub fn load_batch_issues(path: &Path) -> Result<Vec<IssueRequest>> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read batch file: {}", path.display()))?;

    let issues: Vec<IssueRequest> =
        serde_json::from_str(&content).context("Failed to parse batch JSON")?;

    for (i, issue) in issues.iter().enumerate() {
        issue
            .validate()
            .with_context(|| format!("Invalid issue at index {i}"))?;
    }

    Ok(issues)
}

pub fn create_issues(issues: &[IssueRequest]) -> Result<Vec<String>> {
    let mut urls = Vec::new();

    for issue in issues {
        let url = create_single_issue(issue)?;
        urls.push(url);
    }

    Ok(urls)
}

fn create_single_issue(issue: &IssueRequest) -> Result<String> {
    let mut cmd = Command::new("gh");
    cmd.args(["issue", "create", "--title", &issue.title]);

    let body = issue.body.as_deref().unwrap_or("");
    cmd.args(["--body", body]);

    let labels = issue.enhanced_labels();
    let labels_str = labels.join(",");
    if !labels_str.is_empty() {
        cmd.args(["--label", &labels_str]);
    }

    if let Some(assignees) = &issue.assignees {
        let assignees_str = assignees.join(",");
        cmd.args(["--assignee", &assignees_str]);
    }

    if let Some(milestone) = &issue.milestone {
        cmd.args(["--milestone", milestone]);
    }

    let output = cmd
        .output()
        .context("Failed to execute gh command. Is gh CLI installed and authenticated?")?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to create issue: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();

    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_issue_validation_requires_title() {
        let issue = IssueRequest {
            title: String::new(),
            body: None,
            labels: None,
            assignees: None,
            milestone: None,
            priority: None,
        };

        assert!(issue.validate().is_err());
    }

    #[test]
    fn test_issue_validation_title_length() {
        let issue = IssueRequest {
            title: "a".repeat(300),
            body: None,
            labels: None,
            assignees: None,
            milestone: None,
            priority: None,
        };

        assert!(issue.validate().is_err());
    }

    #[test]
    fn test_enhanced_labels_adds_priority() {
        let issue = IssueRequest {
            title: "Test".to_string(),
            body: None,
            labels: Some(vec!["bug".to_string()]),
            assignees: None,
            milestone: None,
            priority: Some("high".to_string()),
        };

        let labels = issue.enhanced_labels();
        assert!(labels.contains(&"bug".to_string()));
        assert!(labels.contains(&"priority-high".to_string()));
    }

    #[test]
    fn test_load_issue_from_json() {
        let temp_dir = TempDir::new().unwrap();
        let json_path = temp_dir.path().join("issue.json");

        let json = r#"{
            "title": "Fix bug",
            "body": "Description here",
            "labels": ["bug", "priority-high"]
        }"#;

        fs::write(&json_path, json).unwrap();

        let issue = load_issue_from_json(&json_path).unwrap();
        assert_eq!(issue.title, "Fix bug");
        assert_eq!(issue.body, Some("Description here".to_string()));
        assert_eq!(issue.labels.unwrap().len(), 2);
    }

    #[test]
    fn test_load_batch_issues() {
        let temp_dir = TempDir::new().unwrap();
        let json_path = temp_dir.path().join("batch.json");

        let json = r#"[
            {
                "title": "Issue 1",
                "labels": ["bug"]
            },
            {
                "title": "Issue 2",
                "labels": ["enhancement"]
            }
        ]"#;

        fs::write(&json_path, json).unwrap();

        let issues = load_batch_issues(&json_path).unwrap();
        assert_eq!(issues.len(), 2);
        assert_eq!(issues[0].title, "Issue 1");
        assert_eq!(issues[1].title, "Issue 2");
    }

    #[test]
    fn test_valid_issue_passes_validation() {
        let issue = IssueRequest {
            title: "Valid title".to_string(),
            body: Some("Valid body".to_string()),
            labels: None,
            assignees: None,
            milestone: None,
            priority: None,
        };

        assert!(issue.validate().is_ok());
    }
}
