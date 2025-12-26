use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TodoAnalysis {
    pub todo_type: String,
    pub complexity: String,
    pub approach: String,
    pub estimated_lines: usize,
    pub requires_tests: bool,
}

use std::path::Path;

pub fn analyze_todo(repo_path: &Path, todo: &crate::resolver::TodoItem) -> Result<TodoAnalysis> {
    // Read the file and context around the TODO
    let file_path = repo_path.join(&todo.file);
    let content = std::fs::read_to_string(&file_path)
        .with_context(|| format!("Failed to read {}", todo.file))?;

    let lines: Vec<&str> = content.lines().collect();
    let todo_line = lines
        .get(todo.line.saturating_sub(1))
        .context("TODO line out of bounds")?;

    // Analyze based on context
    let complexity = determine_complexity(todo_line, &todo.content);
    let todo_type = determine_type(&todo.content);
    let approach = suggest_approach(&todo_type, &todo.content);
    let estimated_lines = estimate_implementation_size(&complexity);

    Ok(TodoAnalysis {
        todo_type,
        complexity,
        approach,
        estimated_lines,
        requires_tests: should_have_tests(todo_line),
    })
}

fn determine_complexity(line: &str, content: &str) -> String {
    let indicators = [
        ("refactor", 3),
        ("implement", 3),
        ("add", 2),
        ("fix", 2),
        ("update", 1),
        ("move", 1),
        ("extract", 2),
    ];

    for (keyword, weight) in indicators {
        if content.to_lowercase().contains(keyword) {
            return match weight {
                3 => "high".to_string(),
                2 => "medium".to_string(),
                _ => "low".to_string(),
            };
        }
    }

    if line.contains("pub fn") || line.contains("pub struct") {
        "medium".to_string()
    } else {
        "low".to_string()
    }
}

fn determine_type(content: &str) -> String {
    let lower = content.to_lowercase();

    if lower.contains("test") {
        "testing".to_string()
    } else if lower.contains("implement") {
        "implementation".to_string()
    } else if lower.contains("refactor") {
        "refactoring".to_string()
    } else if lower.contains("fix") || lower.contains("bug") {
        "bugfix".to_string()
    } else if lower.contains("add") || lower.contains("feature") {
        "feature".to_string()
    } else {
        "general".to_string()
    }
}

fn suggest_approach(todo_type: &str, content: &str) -> String {
    match todo_type {
        "testing" => "Write unit tests following AAA pattern (Arrange, Act, Assert)".to_string(),
        "implementation" => "Implement the feature with proper error handling".to_string(),
        "refactoring" => {
            "Extract to separate function/module maintaining existing behavior".to_string()
        }
        "bugfix" => "Write failing test first, then fix the bug".to_string(),
        "feature" => "Design API, write tests, implement incrementally".to_string(),
        _ => format!("Implement: {}", content),
    }
}

fn estimate_implementation_size(complexity: &str) -> usize {
    match complexity {
        "high" => 100,
        "medium" => 30,
        _ => 10,
    }
}

fn should_have_tests(line: &str) -> bool {
    // Public functions should have tests
    line.contains("pub fn") || line.contains("pub struct") || line.contains("pub enum")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_complexity_high() {
        let complexity =
            determine_complexity("fn something()", "TODO: Refactor this entire module");
        assert_eq!(complexity, "high");
    }

    #[test]
    fn test_determine_complexity_medium() {
        let complexity = determine_complexity("pub fn api()", "TODO: Add parameter validation");
        assert_eq!(complexity, "medium");
    }

    #[test]
    fn test_determine_complexity_low() {
        let complexity = determine_complexity("let x = 1;", "TODO: Update variable name");
        assert_eq!(complexity, "low");
    }

    #[test]
    fn test_determine_type_testing() {
        let todo_type = determine_type("Add tests for this function");
        assert_eq!(todo_type, "testing");
    }

    #[test]
    fn test_determine_type_implementation() {
        let todo_type = determine_type("Implement caching layer");
        assert_eq!(todo_type, "implementation");
    }

    #[test]
    fn test_determine_type_bugfix() {
        let todo_type = determine_type("Fix memory leak here");
        assert_eq!(todo_type, "bugfix");
    }

    #[test]
    fn test_should_have_tests_for_public_function() {
        assert!(should_have_tests("pub fn calculate() -> i32 {"));
        assert!(should_have_tests("pub struct Data {"));
        assert!(!should_have_tests("let x = 1;"));
    }

    #[test]
    fn test_estimate_implementation_size() {
        assert_eq!(estimate_implementation_size("high"), 100);
        assert_eq!(estimate_implementation_size("medium"), 30);
        assert_eq!(estimate_implementation_size("low"), 10);
    }
}
