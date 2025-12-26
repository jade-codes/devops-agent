use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;

use crate::analyzer::FeatureSpec;

#[derive(Debug, Serialize, Deserialize)]
pub struct TestCase {
    pub name: String,
    pub test_file: String,
    pub test_code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Implementation {
    pub files_modified: Vec<String>,
    pub lines_added: usize,
}

/// Generate test cases for feature
pub fn generate_tests(spec: &FeatureSpec) -> Result<Vec<TestCase>> {
    let mut tests = Vec::new();

    // Generate test for each acceptance criterion
    for (i, criterion) in spec.acceptance_criteria.iter().enumerate() {
        tests.push(TestCase {
            name: format!("test_criterion_{}", i + 1),
            test_file: format!("tests/{}_test.rs", sanitize_name(&spec.description)),
            test_code: generate_test_code(criterion),
        });
    }

    // Add basic happy path test if no criteria specified
    if tests.is_empty() {
        tests.push(TestCase {
            name: "test_basic_functionality".to_string(),
            test_file: format!("tests/{}_test.rs", sanitize_name(&spec.description)),
            test_code: generate_basic_test(&spec.description),
        });
    }

    Ok(tests)
}

/// Write test to file
pub fn write_test(test: &TestCase, repo_path: &str) -> Result<()> {
    let test_path = format!("{}/{}", repo_path, test.test_file);

    // Create test directory if it doesn't exist
    if let Some(parent) = std::path::Path::new(&test_path).parent() {
        fs::create_dir_all(parent)?;
    }

    // Append test to file or create new
    let mut content = if std::path::Path::new(&test_path).exists() {
        fs::read_to_string(&test_path)?
    } else {
        "#[cfg(test)]\nmod tests {\n    use super::*;\n\n".to_string()
    };

    content.push_str(&test.test_code);
    content.push_str("\n}\n");

    fs::write(&test_path, content)?;
    Ok(())
}

/// Run tests in repository
pub fn run_tests(repo_path: &str) -> Result<bool> {
    let output = Command::new("cargo")
        .args(["test", "--quiet"])
        .current_dir(repo_path)
        .output()
        .context("Failed to run tests")?;

    Ok(output.status.success())
}

/// Implement the feature
pub fn implement_feature(spec: &FeatureSpec, repo_path: &str) -> Result<Implementation> {
    // This is a placeholder - real implementation would:
    // 1. Analyze codebase structure
    // 2. Generate appropriate code
    // 3. Insert into correct locations
    // 4. Handle imports and dependencies

    let implementation_code = generate_implementation_code(spec);

    // For now, create a stub file
    let impl_file = format!("{}/src/{}.rs", repo_path, sanitize_name(&spec.description));

    fs::write(&impl_file, implementation_code)?;

    Ok(Implementation {
        files_modified: vec![impl_file],
        lines_added: 50, // Placeholder
    })
}

/// Create pull request
pub fn create_pr(spec: &FeatureSpec, target_branch: &str, repo_path: &str) -> Result<String> {
    // Create branch
    let branch_name = format!("feature/{}", sanitize_name(&spec.description));

    Command::new("git")
        .args(["checkout", "-b", &branch_name])
        .current_dir(repo_path)
        .output()
        .context("Failed to create branch")?;

    // Stage and commit changes
    Command::new("git")
        .args(["add", "."])
        .current_dir(repo_path)
        .output()?;

    Command::new("git")
        .args(["commit", "-m", &format!("feat: {}", spec.description)])
        .current_dir(repo_path)
        .output()?;

    // Push branch
    Command::new("git")
        .args(["push", "-u", "origin", &branch_name])
        .current_dir(repo_path)
        .output()?;

    // Create PR
    let pr_body = format!(
        "## Feature Implementation\n\n{}\n\n**Type:** {:?}\n**Complexity:** {:?}\n\n### Acceptance Criteria\n{}",
        spec.description,
        spec.feature_type,
        spec.complexity,
        spec.acceptance_criteria
            .iter()
            .map(|c| format!("- {}", c))
            .collect::<Vec<_>>()
            .join("\n")
    );

    let output = Command::new("gh")
        .args([
            "pr",
            "create",
            "--title",
            &format!("feat: {}", spec.description),
            "--body",
            &pr_body,
            "--base",
            target_branch,
        ])
        .current_dir(repo_path)
        .output()
        .context("Failed to create PR")?;

    let pr_url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(pr_url)
}

// Helper functions

fn sanitize_name(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}

fn generate_test_code(criterion: &str) -> String {
    format!(
        r#"    #[test]
    fn test_{}() {{
        // Test: {}
        todo!("Implement test");
    }}
"#,
        sanitize_name(criterion),
        criterion
    )
}

fn generate_basic_test(description: &str) -> String {
    format!(
        r#"    #[test]
    fn test_basic_functionality() {{
        // Test: {}
        todo!("Implement test");
    }}
"#,
        description
    )
}

fn generate_implementation_code(spec: &FeatureSpec) -> String {
    format!(
        r#"// Feature: {}
// Type: {:?}
// Complexity: {:?}

pub fn placeholder() {{
    todo!("Implement feature")
}}
"#,
        spec.description, spec.feature_type, spec.complexity
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer::{Complexity, FeatureType};

    #[test]
    fn test_sanitize_name() {
        assert_eq!(sanitize_name("Add User Login"), "add_user_login");
        assert_eq!(sanitize_name("API Endpoint"), "api_endpoint");
        assert_eq!(sanitize_name("  spaces  "), "spaces");
    }

    #[test]
    fn test_generate_tests_with_criteria() {
        let spec = FeatureSpec {
            description: "Test feature".to_string(),
            feature_type: FeatureType::NewFunction,
            complexity: Complexity::Simple,
            affected_files: vec![],
            dependencies: vec![],
            acceptance_criteria: vec!["Must work".to_string(), "Must be fast".to_string()],
        };

        let tests = generate_tests(&spec).unwrap();
        assert_eq!(tests.len(), 2);
    }

    #[test]
    fn test_generate_tests_without_criteria() {
        let spec = FeatureSpec {
            description: "Test feature".to_string(),
            feature_type: FeatureType::NewFunction,
            complexity: Complexity::Simple,
            affected_files: vec![],
            dependencies: vec![],
            acceptance_criteria: vec![],
        };

        let tests = generate_tests(&spec).unwrap();
        assert_eq!(tests.len(), 1);
        assert_eq!(tests[0].name, "test_basic_functionality");
    }

    #[test]
    fn test_generate_test_code() {
        let code = generate_test_code("handles errors");
        assert!(code.contains("test_handles_errors"));
        assert!(code.contains("todo!"));
    }
}
