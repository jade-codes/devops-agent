use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeatureType {
    NewFunction,
    NewModule,
    ApiEndpoint,
    UiComponent,
    DataModel,
    Integration,
    Enhancement,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Complexity {
    Simple,
    Moderate,
    Complex,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FeatureSpec {
    pub description: String,
    pub feature_type: FeatureType,
    pub complexity: Complexity,
    pub affected_files: Vec<String>,
    pub dependencies: Vec<String>,
    pub acceptance_criteria: Vec<String>,
}

/// Analyze feature from description or GitHub issue
pub fn analyze_feature(feature: &str, repo_path: &str) -> Result<FeatureSpec> {
    // Check if it's a GitHub issue number
    if let Some(issue_num) = parse_issue_number(feature) {
        analyze_from_issue(issue_num, repo_path)
    } else {
        analyze_from_description(feature, repo_path)
    }
}

/// Parse issue number from string
pub fn parse_issue_number(s: &str) -> Option<u32> {
    let re = Regex::new(r"^#?(\d+)$").ok()?;
    re.captures(s)?.get(1)?.as_str().parse().ok()
}

/// Analyze feature from GitHub issue
fn analyze_from_issue(issue_num: u32, _repo_path: &str) -> Result<FeatureSpec> {
    // In real implementation, fetch from GitHub API
    Ok(FeatureSpec {
        description: format!("Feature from issue #{}", issue_num),
        feature_type: FeatureType::Enhancement,
        complexity: Complexity::Moderate,
        affected_files: vec![],
        dependencies: vec![],
        acceptance_criteria: vec![],
    })
}

/// Analyze feature from text description
fn analyze_from_description(description: &str, repo_path: &str) -> Result<FeatureSpec> {
    let feature_type = determine_feature_type(description);
    let complexity = determine_complexity(description, repo_path)?;
    let affected_files = identify_affected_files(description, repo_path)?;
    let acceptance_criteria = extract_acceptance_criteria(description);

    Ok(FeatureSpec {
        description: description.to_string(),
        feature_type,
        complexity,
        affected_files,
        dependencies: vec![],
        acceptance_criteria,
    })
}

/// Determine feature type from description
pub fn determine_feature_type(description: &str) -> FeatureType {
    let lower = description.to_lowercase();

    if lower.contains("function") || lower.contains("method") {
        FeatureType::NewFunction
    } else if lower.contains("module") || lower.contains("package") {
        FeatureType::NewModule
    } else if lower.contains("api") || lower.contains("endpoint") || lower.contains("route") {
        FeatureType::ApiEndpoint
    } else if lower.contains("ui") || lower.contains("component") || lower.contains("view") {
        FeatureType::UiComponent
    } else if lower.contains("model") || lower.contains("schema") || lower.contains("database") {
        FeatureType::DataModel
    } else if lower.contains("integration") || lower.contains("connect") {
        FeatureType::Integration
    } else {
        FeatureType::Enhancement
    }
}

/// Determine complexity based on description and codebase context
pub fn determine_complexity(description: &str, _repo_path: &str) -> Result<Complexity> {
    let word_count = description.split_whitespace().count();
    let has_complex_keywords = description.to_lowercase().split_whitespace().any(|w| {
        matches!(
            w,
            "complex" | "multiple" | "integration" | "system" | "architecture"
        )
    });

    if word_count > 50 || has_complex_keywords {
        Ok(Complexity::Complex)
    } else if word_count > 20 {
        Ok(Complexity::Moderate)
    } else {
        Ok(Complexity::Simple)
    }
}

/// Identify files that will be affected
fn identify_affected_files(description: &str, repo_path: &str) -> Result<Vec<String>> {
    let mut files = Vec::new();

    // Extract file mentions from description
    let file_pattern = Regex::new(r"[\w/]+\.\w+")?;
    for capture in file_pattern.captures_iter(description) {
        if let Some(matched) = capture.get(0) {
            let file_path = matched.as_str();
            let full_path = format!("{}/{}", repo_path, file_path);
            if std::path::Path::new(&full_path).exists() {
                files.push(file_path.to_string());
            }
        }
    }

    Ok(files)
}

/// Extract acceptance criteria from description
pub fn extract_acceptance_criteria(description: &str) -> Vec<String> {
    let mut criteria = Vec::new();

    // Look for bullet points or numbered lists
    for line in description.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('-') || trimmed.starts_with('*') || trimmed.starts_with("- ") {
            criteria.push(trimmed.trim_start_matches('-').trim().to_string());
        } else if let Some(stripped) = trimmed.strip_prefix(|c: char| c.is_ascii_digit()) {
            if stripped.starts_with('.') || stripped.starts_with(')') {
                criteria.push(stripped[1..].trim().to_string());
            }
        }
    }

    criteria
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_issue_number() {
        assert_eq!(parse_issue_number("#42"), Some(42));
        assert_eq!(parse_issue_number("42"), Some(42));
        assert_eq!(parse_issue_number("123"), Some(123));
        assert_eq!(parse_issue_number("not-a-number"), None);
    }

    #[test]
    fn test_determine_feature_type() {
        assert_eq!(
            determine_feature_type("Add a new function to calculate sum"),
            FeatureType::NewFunction
        );
        assert_eq!(
            determine_feature_type("Create API endpoint for users"),
            FeatureType::ApiEndpoint
        );
        assert_eq!(
            determine_feature_type("Build UI component for login"),
            FeatureType::UiComponent
        );
        assert_eq!(
            determine_feature_type("Add database model for products"),
            FeatureType::DataModel
        );
    }

    #[test]
    fn test_determine_complexity() {
        let simple = "Add button";
        let moderate = "Add user authentication with email validation checks and password hashing with bcrypt for secure login and account recovery features across the application";
        let complex = "Build complex multi-tenant integration system with real-time sync";

        assert_eq!(
            determine_complexity(simple, ".").unwrap(),
            Complexity::Simple
        );
        assert_eq!(
            determine_complexity(moderate, ".").unwrap(),
            Complexity::Moderate
        );
        assert_eq!(
            determine_complexity(complex, ".").unwrap(),
            Complexity::Complex
        );
    }

    #[test]
    fn test_extract_acceptance_criteria() {
        let description =
            "Feature:\n- Must handle errors\n- Should validate input\n* Return proper status";
        let criteria = extract_acceptance_criteria(description);

        assert_eq!(criteria.len(), 3);
        assert!(criteria.contains(&"Must handle errors".to_string()));
        assert!(criteria.contains(&"Should validate input".to_string()));
    }

    #[test]
    fn test_extract_numbered_criteria() {
        let description = "Requirements:\n1. Fast response\n2. Secure\n3) Scalable";
        let criteria = extract_acceptance_criteria(description);

        assert!(criteria.len() >= 2); // At least the first two
    }
}
