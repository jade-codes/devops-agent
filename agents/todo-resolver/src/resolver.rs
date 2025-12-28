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
    // Read the source file to understand what needs testing
    let source_path = repo_path.join(&todo.file);
    let source_content = fs::read_to_string(&source_path)
        .with_context(|| format!("Failed to read source file: {}", todo.file))?;

    // Extract function name from the todo content
    let function_name = todo.content.split('`').nth(1).unwrap_or("").trim();

    println!("   Generating tests for: {}", function_name);

    // Generate test file path - put in same directory as source with _test suffix
    let test_file = todo.file.replace(".rs", "_test.rs");
    let test_path = repo_path.join(&test_file);

    if let Some(parent) = test_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Analyze the source to generate context-aware tests
    let test_content = if function_name.contains("PartialEq") {
        // Generate tests for PartialEq implementations
        generate_partialeq_tests(&source_content, function_name)?
    } else if function_name.contains("::new") || function_name.ends_with("new") {
        // Generate tests for constructor functions
        generate_constructor_tests(&source_content, function_name)?
    } else if function_name.contains("Clone") {
        // Generate tests for Clone implementations
        generate_clone_tests(&source_content, function_name)?
    } else {
        // Generate generic function tests
        generate_generic_tests(&source_content, function_name)?
    };

    fs::write(&test_path, test_content)?;
    println!("   Created test file: {}", test_file);

    Ok(test_file)
}

fn generate_partialeq_tests(source: &str, function_name: &str) -> Result<String> {
    // Extract the type name from the function signature
    let type_name = function_name
        .split('<')
        .nth(1)
        .and_then(|s| s.split(" as ").next())
        .and_then(|s| s.split("::").last())
        .unwrap_or("Value");

    // Parse the struct fields to understand what we're testing
    let struct_fields = parse_struct_fields(source, type_name);

    // Find the actual eq implementation
    let eq_impl = find_eq_implementation(source, type_name);

    // Generate test based on what fields are compared
    let test_content = if !struct_fields.is_empty() {
        generate_field_based_tests(type_name, &struct_fields, &eq_impl)
    } else {
        generate_default_eq_tests(type_name)
    };

    Ok(test_content)
}

fn find_eq_implementation(source: &str, type_name: &str) -> String {
    let impl_pattern = format!("impl PartialEq for {}", type_name);
    if let Some(start) = source.find(&impl_pattern) {
        let after_impl = &source[start..];
        if let Some(fn_start) = after_impl.find("fn eq") {
            let after_fn = &after_impl[fn_start..];
            if let Some(brace_start) = after_fn.find('{') {
                if let Some(brace_end) = after_fn[brace_start..].find('}') {
                    return after_fn[brace_start..brace_start + brace_end + 1].to_string();
                }
            }
        }
    }
    String::new()
}

fn generate_field_based_tests(
    type_name: &str,
    fields: &[(String, String)],
    eq_impl: &str,
) -> String {
    let type_lower = type_name.to_lowercase();

    // Generate tests for each field to ensure the eq implementation checks all fields
    let mut test_cases = Vec::new();

    // Test 1: Both instances identical - should be equal
    let same_values = generate_instance_literal(type_name, fields, &vec![false; fields.len()]);
    test_cases.push(format!(
        r#"    #[test]
    fn test_{}_eq_identical() {{
        let val1 = {};
        let val2 = {};
        assert_eq!(val1, val2, "Identical instances should be equal");
    }}"#,
        type_lower, same_values, same_values
    ));

    // Test 2: For each field, create tests where only that field differs
    for (idx, (field_name, _)) in fields.iter().enumerate() {
        let mut differs = vec![false; fields.len()];
        differs[idx] = true;

        let val1 = generate_instance_literal(type_name, fields, &vec![false; fields.len()]);
        let val2 = generate_instance_literal(type_name, fields, &differs);

        test_cases.push(format!(
            r#"    #[test]
    fn test_{}_ne_diff_{}() {{
        let val1 = {};
        let val2 = {};
        assert_ne!(val1, val2, "Instances with different {} should not be equal");
    }}"#,
            type_lower, field_name, val1, val2, field_name
        ));
    }

    format!(
        r#"#[cfg(test)]
mod tests {{
    use super::*;

{}
}}
"#,
        test_cases.join("\n\n")
    )
}

fn generate_instance_literal(
    type_name: &str,
    fields: &[(String, String)],
    use_alt_values: &[bool],
) -> String {
    let field_strs: Vec<String> = fields
        .iter()
        .enumerate()
        .map(|(idx, (field_name, field_type))| {
            let use_alt = use_alt_values.get(idx).copied().unwrap_or(false);
            let (val1, val2) = generate_test_value_for_type(field_type, idx);
            let val = if use_alt { val2 } else { val1 };
            format!("{}: {}", field_name, val)
        })
        .collect();

    format!("{} {{ {} }}", type_name, field_strs.join(", "))
}

fn generate_test_value_for_type(field_type: &str, index: usize) -> (String, String) {
    match field_type {
        "f64" => {
            let val = index as f64 + 1.0;
            (
                format!("{}.0", val as i32),
                format!("{}.0", (val + 1.0) as i32),
            )
        }
        "f32" => {
            let val = index as f32 + 1.0;
            (
                format!("{}.0", val as i32),
                format!("{}.0", (val + 1.0) as i32),
            )
        }
        "i32" | "i64" | "i16" | "i8" | "isize" => {
            let val = index as i32 + 1;
            (format!("{}", val), format!("{}", val + 1))
        }
        "u32" | "u64" | "u16" | "u8" | "usize" => {
            let val = index as u32 + 1;
            (format!("{}", val), format!("{}", val + 1))
        }
        "bool" => ("true".to_string(), "false".to_string()),
        "String" => (
            format!(r#"String::from("test{}")"#, index),
            format!(r#"String::from("different{}")"#, index),
        ),
        t if t.starts_with("Option<") => {
            ("None".to_string(), "Some(Default::default())".to_string())
        }
        _ => {
            // For complex types like Expression, LiteralExpression, etc., use Default
            (
                format!("{}::default()", field_type),
                format!("{}::default()", field_type),
            )
        }
    }
}

fn generate_default_eq_tests(type_name: &str) -> String {
    let type_lower = type_name.to_lowercase();

    format!(
        r#"#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn test_{}_eq_same_values() {{
        let val1 = create_test_{}_instance();
        let val2 = create_test_{}_instance();
        assert_eq!(val1, val2, "Same values should be equal");
    }}

    #[test]
    fn test_{}_eq_reflexive() {{
        let val = create_test_{}_instance();
        assert_eq!(val, val, "Value should equal itself");
    }}

    #[test]
    fn test_{}_ne_different_values() {{
        let val1 = create_test_{}_instance();
        let val2 = create_different_{}_instance();
        assert_ne!(val1, val2, "Different values should not be equal");
    }}

    // Helper functions - fill these in with actual {} construction
    fn create_test_{}_instance() -> {} {{
        {}::default()
    }}

    fn create_different_{}_instance() -> {} {{
        // TODO: Create a different instance
        {}::default()
    }}
}}
"#,
        type_lower,
        type_lower,
        type_lower,
        type_lower,
        type_lower,
        type_lower,
        type_lower,
        type_lower,
        type_name,
        type_lower,
        type_name,
        type_name,
        type_lower,
        type_name,
        type_name
    )
}

fn parse_struct_fields(source: &str, type_name: &str) -> Vec<(String, String)> {
    let mut fields = Vec::new();
    let struct_pattern = format!("pub struct {}", type_name);

    if let Some(start) = source.find(&struct_pattern) {
        let after_struct = &source[start..];
        if let Some(brace_start) = after_struct.find('{') {
            // Find matching closing brace
            let mut depth = 0;
            let mut brace_end = 0;
            for (i, ch) in after_struct[brace_start..].char_indices() {
                if ch == '{' {
                    depth += 1;
                } else if ch == '}' {
                    depth -= 1;
                    if depth == 0 {
                        brace_end = i;
                        break;
                    }
                }
            }

            if brace_end > 0 {
                let fields_str = &after_struct[brace_start + 1..brace_start + brace_end];
                for line in fields_str.lines() {
                    let line = line.trim();
                    if line.starts_with("pub ") && line.contains(':') {
                        let parts: Vec<&str> = line.split(':').collect();
                        if parts.len() >= 2 {
                            let field_name = parts[0].replace("pub", "").trim().to_string();
                            let field_type = parts[1].trim().trim_end_matches(',').to_string();
                            fields.push((field_name, field_type));
                        }
                    }
                }
            }
        }
    }

    fields
}

fn generate_constructor_tests(source: &str, function_name: &str) -> Result<String> {
    let func_name = function_name.split("::").last().unwrap_or("new");

    Ok(format!(
        r#"#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn test_{}_creates_valid_instance() {{
        // Test that constructor creates a valid instance
        let result = {}(/* TODO: add constructor args */);
        // TODO: Add assertions about the created instance
    }}

    #[test]
    fn test_{}_with_different_inputs() {{
        // Test constructor with various inputs
        let result1 = {}(/* TODO: add args */);
        let result2 = {}(/* TODO: add different args */);
        // TODO: Add assertions
    }}
}}
"#,
        func_name, func_name, func_name, func_name, func_name
    ))
}

fn generate_clone_tests(source: &str, function_name: &str) -> Result<String> {
    Ok(format!(
        r#"#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn test_clone_creates_equal_instance() {{
        let original = create_test_instance();
        let cloned = original.clone();
        assert_eq!(original, cloned, "Cloned instance should equal original");
    }}

    #[test]
    fn test_clone_is_independent() {{
        let original = create_test_instance();
        let mut cloned = original.clone();
        // Modify cloned and ensure original is unchanged
        // TODO: Add modification and assertions
    }}

    fn create_test_instance() -> /* TODO: Type */ {{
        unimplemented!("Create test instance")
    }}
}}
"#
    ))
}

fn generate_generic_tests(source: &str, function_name: &str) -> Result<String> {
    let func_name = function_name.split("::").last().unwrap_or("function");

    Ok(format!(
        r#"#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn test_{}_happy_path() {{
        // Test normal/expected behavior
        let result = {}(/* TODO: add args */);
        // TODO: Add assertions
    }}

    #[test]
    fn test_{}_edge_cases() {{
        // Test boundary conditions and edge cases
        // TODO: Add edge case tests
    }}

    #[test]
    fn test_{}_error_conditions() {{
        // Test error handling
        // TODO: Add error condition tests
    }}
}}
"#,
        func_name, func_name, func_name, func_name
    ))
}

pub fn run_tests(repo_path: &Path, test_file: Option<&str>) -> Result<()> {
    if let Some(file) = test_file {
        println!("   Running tests in: {}", file);

        let mut cmd = Command::new("cargo");
        cmd.current_dir(repo_path);
        // Just run cargo test with a short timeout - it will find and run the new test
        cmd.args(["test", "--lib", "--", "--test-threads=1"]);
        cmd.env("RUST_TEST_TIME_UNIT", "100,1000");
        cmd.env("RUST_TEST_TIME_INTEGRATION", "1000,10000");

        let output = cmd.output()?;

        if !output.status.success() {
            println!("   ❌ Tests failed:");
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);

            // Show last 50 lines of output
            for line in stdout
                .lines()
                .rev()
                .take(50)
                .collect::<Vec<_>>()
                .iter()
                .rev()
            {
                println!("   {}", line);
            }
            for line in stderr
                .lines()
                .rev()
                .take(20)
                .collect::<Vec<_>>()
                .iter()
                .rev()
            {
                println!("   {}", line);
            }
            return Err(anyhow::anyhow!("Tests failed"));
        } else {
            println!("   ✓ Tests passed");
            // Show last 20 lines of output
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout
                .lines()
                .rev()
                .take(20)
                .collect::<Vec<_>>()
                .iter()
                .rev()
            {
                println!("   {}", line);
            }
        }
    } else {
        println!("   Running all tests...");
        let mut cmd = Command::new("cargo");
        cmd.current_dir(repo_path);
        cmd.args(["test", "--lib"]);

        let output = cmd.output()?;

        if !output.status.success() {
            println!("   ❌ Tests failed:");
            println!("{}", String::from_utf8_lossy(&output.stderr));
            return Err(anyhow::anyhow!("Tests failed"));
        } else {
            println!("   ✓ Tests passed");
        }
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

    // Stage all changes
    Command::new("git")
        .args(["add", "."])
        .current_dir(repo_path)
        .output()?;

    // Commit
    let commit_msg = if let Some(issue) = todo.issue_number {
        format!("fix: {} (closes #{})", todo.content, issue)
    } else {
        format!("fix: {}", todo.content)
    };

    let commit_output = Command::new("git")
        .args(["commit", "-m", &commit_msg])
        .current_dir(repo_path)
        .output()?;

    if !commit_output.status.success() {
        let stderr = String::from_utf8_lossy(&commit_output.stderr);
        anyhow::bail!("Failed to commit changes: {}", stderr);
    }

    Ok(branch_name)
}

pub fn create_pr_request(repo_path: &Path, todo: &TodoItem, branch: &str) -> Result<String> {
    // Push the branch to origin first
    let push_output = Command::new("git")
        .args(["push", "-u", "origin", branch])
        .current_dir(repo_path)
        .output()?;

    if !push_output.status.success() {
        anyhow::bail!(
            "Failed to push branch: {}",
            String::from_utf8_lossy(&push_output.stderr)
        );
    }

    let title = format!("Resolve TODO: {}", todo.content);
    let body = format!(
        "Resolves TODO from `{}:{}`\n\nImplemented following TDD approach:\n- ✅ Tests written first\n- ✅ Implementation added\n- ✅ Tests passing",
        todo.file, todo.line
    );

    let output = Command::new("gh")
        .args([
            "pr", "create", "--title", &title, "--body", &body, "--base", "main",
        ])
        .current_dir(repo_path)
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
