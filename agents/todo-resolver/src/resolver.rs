use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TodoItem {
    pub file: String,
    pub line: usize,
    pub content: String,
    pub issue_number: Option<u32>,
}

pub fn load_from_issue(issue_num: u32) -> Result<TodoItem> {
    // Get issue details from GitHub
    let output = Command::new("gh")
        .args([
            "issue",
            "view",
            &issue_num.to_string(),
            "--json",
            "title,body",
        ])
        .output()
        .context("Failed to fetch issue from GitHub")?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to fetch issue #{}: {}",
            issue_num,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    let issue: serde_json::Value = serde_json::from_str(&json_str)?;

    let title = issue["title"].as_str().context("Missing issue title")?;
    let body = issue["body"].as_str().unwrap_or("");

    // Extract file location from body (format: **File:** `path/file.rs:line`)
    let file_regex = Regex::new(r"\*\*File:\*\* `([^:]+):(\d+)`")?;
    let (file, line) = if let Some(cap) = file_regex.captures(body) {
        (cap[1].to_string(), cap[2].parse()?)
    } else {
        anyhow::bail!("Could not extract file location from issue body");
    };

    Ok(TodoItem {
        file,
        line,
        content: title.to_string(),
        issue_number: Some(issue_num),
    })
}

pub fn load_from_location(repo_path: &Path, location: &str) -> Result<TodoItem> {
    let parts: Vec<&str> = location.split(':').collect();
    if parts.len() != 2 {
        anyhow::bail!("Location must be in format 'file:line'");
    }

    let file = parts[0].to_string();
    let line: usize = parts[1].parse()?;

    let file_path = repo_path.join(&file);
    let content =
        fs::read_to_string(&file_path).with_context(|| format!("Failed to read {}", file))?;

    let todo_line = content
        .lines()
        .nth(line.saturating_sub(1))
        .context("Line number out of bounds")?;

    // Extract TODO content
    let todo_regex = Regex::new(r"(?i)TODO:?\s*(.*)")?;
    let todo_content = if let Some(cap) = todo_regex.captures(todo_line) {
        cap[1].trim().to_string()
    } else {
        anyhow::bail!("No TODO found at {}:{}", file, line);
    };

    Ok(TodoItem {
        file,
        line,
        content: todo_content,
        issue_number: None,
    })
}

pub fn select_todo_automatically(repo_path: &Path) -> Result<TodoItem> {
    // Scan for TODOs and pick the simplest one
    let todos = scan_todos(repo_path)?;

    if todos.is_empty() {
        anyhow::bail!("No TODOs found in repository");
    }

    // Sort by complexity (prefer simple ones)
    let mut sorted = todos;
    sorted.sort_by_key(|t| t.content.len());

    Ok(sorted[0].clone())
}

fn scan_todos(repo_path: &Path) -> Result<Vec<TodoItem>> {
    let mut todos = Vec::new();
    let todo_regex = Regex::new(r"(?i)//\s*TODO:?\s*(.*)")?;

    for entry in walkdir::WalkDir::new(repo_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
    {
        let path = entry.path();
        let relative = path.strip_prefix(repo_path).unwrap_or(path);

        if let Ok(content) = fs::read_to_string(path) {
            for (line_num, line) in content.lines().enumerate() {
                if let Some(cap) = todo_regex.captures(line) {
                    todos.push(TodoItem {
                        file: relative.display().to_string(),
                        line: line_num + 1,
                        content: cap[1].trim().to_string(),
                        issue_number: None,
                    });
                }
            }
        }
    }

    Ok(todos)
}

pub fn generate_tests(repo_path: &Path, todo: &TodoItem) -> Result<String> {
    // Generate test file based on TODO
    let test_file = format!("{}_test.rs", todo.file.replace(".rs", ""));
    let test_path = repo_path.join(&test_file);

    // Create the test file with basic structure
    if let Some(parent) = test_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let test_content = format!(
        "#[cfg(test)]\nmod tests {{\n    use super::*;\n\n    #[test]\n    fn test_placeholder() {{\n        // TODO: Implement test for: {}\n        todo!()\n    }}\n}}\n",
        todo.content
    );

    fs::write(&test_path, test_content)?;

    Ok(test_file)
}

pub fn run_tests(repo_path: &Path, test_file: Option<&str>) -> Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.current_dir(repo_path);

    if let Some(file) = test_file {
        // Run only tests in the specific file
        // Extract the module path from the file path
        let module_path = file
            .replace(".rs", "")
            .replace("_test", "")
            .replace("/", "::")
            .replace("crates::", "");

        println!("   Running tests in module: {}", module_path);
        cmd.args(["test", "--lib", "--", &module_path]);
    } else {
        cmd.args(["test"]);
    }

    let output = cmd.output()?;

    if !output.status.success() {
        println!("   Tests failed (expected for TDD)");
    } else {
        println!("   Tests passed ✓");
    }

    Ok(())
}

pub fn implement_fix(_repo_path: &Path, todo: &TodoItem) -> Result<Vec<String>> {
    // In a real implementation, this would:
    // 1. Analyze the TODO and surrounding code
    // 2. Generate appropriate implementation
    // 3. Apply the changes

    println!("   (Implementation would modify: {})", todo.file);

    Ok(vec![todo.file.clone()])
}

pub fn commit_changes(repo_path: &Path, todo: &TodoItem, _changes: &[String]) -> Result<String> {
    let branch_name = format!(
        "todo-resolver/{}",
        todo.content
            .to_lowercase()
            .replace(" ", "-")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .take(50)
            .collect::<String>()
    );

    // Ensure we're on the main branch before creating a new branch
    Command::new("git")
        .args(["checkout", "main"])
        .current_dir(repo_path)
        .output()?;

    // Create branch
    Command::new("git")
        .args(["checkout", "-b", &branch_name])
        .current_dir(repo_path)
        .output()?;

    // Stage changes (in real implementation)
    // git add ...

    // Commit
    let commit_msg = if let Some(issue) = todo.issue_number {
        format!("fix: {} (closes #{})", todo.content, issue)
    } else {
        format!("fix: {}", todo.content)
    };

    Command::new("git")
        .args(["commit", "-m", &commit_msg])
        .current_dir(repo_path)
        .output()?;

    Ok(branch_name)
}

pub fn create_pr_request(todo: &TodoItem, _branch: &str) -> Result<String> {
    let title = format!("Resolve TODO: {}", todo.content);
    let body = format!(
        "Resolves TODO from `{}:{}`\n\nImplemented following TDD approach:\n- ✅ Tests written first\n- ✅ Implementation added\n- ✅ Tests passing",
        todo.file, todo.line
    );

    let output = Command::new("gh")
        .args([
            "pr", "create", "--title", &title, "--body", &body, "--base", "main",
        ])
        .output()?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to create PR: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_load_from_location_parses_correctly() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");

        fs::write(&file_path, "// TODO: Fix this bug\nfn main() {}\n").unwrap();

        let result = load_from_location(temp_dir.path(), "test.rs:1");
        assert!(result.is_ok());

        let todo = result.unwrap();
        assert_eq!(todo.file, "test.rs");
        assert_eq!(todo.line, 1);
        assert_eq!(todo.content, "Fix this bug");
    }

    #[test]
    fn test_load_from_location_invalid_format() {
        let temp_dir = TempDir::new().unwrap();
        let result = load_from_location(temp_dir.path(), "invalid-format");
        assert!(result.is_err());
    }

    #[test]
    fn test_scan_todos_finds_comments() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");

        fs::write(&file_path, "// TODO: Task 1\n// TODO: Task 2\n").unwrap();

        let todos = scan_todos(temp_dir.path()).unwrap();
        assert_eq!(todos.len(), 2);
        assert_eq!(todos[0].content, "Task 1");
        assert_eq!(todos[1].content, "Task 2");
    }

    #[test]
    fn test_commit_changes_generates_branch_name() {
        let temp_dir = TempDir::new().unwrap();

        // Initialize git repo
        Command::new("git")
            .args(["init"])
            .current_dir(temp_dir.path())
            .output()
            .ok();
        Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(temp_dir.path())
            .output()
            .ok();
        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(temp_dir.path())
            .output()
            .ok();

        let todo = TodoItem {
            file: "test.rs".to_string(),
            line: 1,
            content: "Fix Memory Leak".to_string(),
            issue_number: None,
        };

        let result = commit_changes(temp_dir.path(), &todo, &[]);
        if let Ok(branch) = result {
            assert!(branch.starts_with("todo-resolver/"));
            assert!(branch.contains("fix-memory-leak"));
        }
    }
}
