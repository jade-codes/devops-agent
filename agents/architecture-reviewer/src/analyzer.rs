use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    Singleton,
    Factory,
    Observer,
    Strategy,
    Repository,
    Service,
    Controller,
    Model,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pattern {
    pub pattern_type: PatternType,
    pub location: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub title: String,
    pub description: String,
    pub severity: Severity,
    pub category: String,
    pub locations: Vec<String>,
    pub suggestion: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArchitectureReport {
    pub module_count: usize,
    pub total_lines: usize,
    pub patterns: Vec<Pattern>,
    pub issues: Vec<Issue>,
    pub dependencies: HashMap<String, Vec<String>>,
}

/// Parse severity string
pub fn parse_severity(s: &str) -> Result<Severity> {
    match s.to_lowercase().as_str() {
        "low" => Ok(Severity::Low),
        "medium" => Ok(Severity::Medium),
        "high" => Ok(Severity::High),
        _ => anyhow::bail!("Invalid severity: {}", s),
    }
}

/// Analyze architecture of codebase
pub fn analyze_architecture(path: &str) -> Result<ArchitectureReport> {
    let mut module_count = 0;
    let mut total_lines = 0;
    let mut patterns = Vec::new();
    let mut issues = Vec::new();
    let mut dependencies: HashMap<String, Vec<String>> = HashMap::new();
    let mut all_modules = HashSet::new();

    // First pass: collect all modules
    for entry in WalkDir::new(path)
        .into_iter()
        .filter_entry(|e| !is_excluded(e.path()))
    {
        let entry = entry?;
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension() {
                if ext == "rs" || ext == "py" || ext == "js" || ext == "ts" {
                    module_count += 1;
                    let module_name = get_module_name(entry.path());
                    all_modules.insert(module_name);

                    let content = fs::read_to_string(entry.path())?;
                    total_lines += content.lines().count();

                    // Detect patterns
                    patterns.extend(detect_patterns(entry.path(), &content));

                    // Analyze dependencies
                    let deps = extract_dependencies(&content);
                    dependencies.insert(entry.path().display().to_string(), deps);
                }
            }
        }
    }

    // Second pass: detect architectural issues
    issues.extend(detect_circular_dependencies(&dependencies));
    issues.extend(detect_god_objects(path)?);
    issues.extend(detect_tight_coupling(&dependencies, &all_modules));
    issues.extend(detect_missing_separation(path)?);

    Ok(ArchitectureReport {
        module_count,
        total_lines,
        patterns,
        issues,
        dependencies,
    })
}

/// Check if path should be excluded
fn is_excluded(path: &Path) -> bool {
    let excluded = ["target", "node_modules", ".git", "dist", "build", "vendor"];
    path.components().any(|c| {
        if let Some(s) = c.as_os_str().to_str() {
            excluded.contains(&s)
        } else {
            false
        }
    })
}

/// Get module name from path
fn get_module_name(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string()
}

/// Detect design patterns in code
pub fn detect_patterns(path: &Path, content: &str) -> Vec<Pattern> {
    let mut patterns = Vec::new();

    // Singleton pattern
    if content.contains("static") && content.contains("lazy_static") {
        patterns.push(Pattern {
            pattern_type: PatternType::Singleton,
            location: path.display().to_string(),
            confidence: 0.8,
        });
    }

    // Factory pattern
    if content.contains("fn create") || content.contains("fn new_") {
        patterns.push(Pattern {
            pattern_type: PatternType::Factory,
            location: path.display().to_string(),
            confidence: 0.6,
        });
    }

    // Repository pattern
    if content.contains("Repository") || content.contains("repo") {
        patterns.push(Pattern {
            pattern_type: PatternType::Repository,
            location: path.display().to_string(),
            confidence: 0.7,
        });
    }

    patterns
}

/// Extract dependencies from code
fn extract_dependencies(content: &str) -> Vec<String> {
    let mut deps = Vec::new();
    let use_pattern = Regex::new(r"use\s+([\w:]+)").unwrap();

    for capture in use_pattern.captures_iter(content) {
        if let Some(dep) = capture.get(1) {
            deps.push(dep.as_str().to_string());
        }
    }

    deps
}

/// Detect circular dependencies
fn detect_circular_dependencies(dependencies: &HashMap<String, Vec<String>>) -> Vec<Issue> {
    let mut issues = Vec::new();

    // Simple cycle detection (A -> B -> A)
    for (module, deps) in dependencies {
        for dep in deps {
            if let Some(transitive_deps) = dependencies.get(dep) {
                if transitive_deps.iter().any(|d| d.contains(module)) {
                    issues.push(Issue {
                        title: "Circular dependency detected".to_string(),
                        description: format!("Circular dependency between {} and {}", module, dep),
                        severity: Severity::High,
                        category: "architecture".to_string(),
                        locations: vec![module.clone(), dep.clone()],
                        suggestion: "Break the cycle by introducing an interface or abstracting shared logic".to_string(),
                    });
                }
            }
        }
    }

    issues
}

/// Detect god objects (large files with many responsibilities)
fn detect_god_objects(path: &str) -> Result<Vec<Issue>> {
    let mut issues = Vec::new();

    for entry in WalkDir::new(path)
        .into_iter()
        .filter_entry(|e| !is_excluded(e.path()))
    {
        let entry = entry?;
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension() {
                if ext == "rs" || ext == "py" || ext == "js" || ext == "ts" {
                    let content = fs::read_to_string(entry.path())?;
                    let lines = content.lines().count();
                    let functions = content.matches("fn ").count();

                    if lines > 500 && functions > 20 {
                        issues.push(Issue {
                            title: "God Object detected".to_string(),
                            description: format!(
                                "{} has {} lines and {} functions - too many responsibilities",
                                entry.path().display(),
                                lines,
                                functions
                            ),
                            severity: Severity::High,
                            category: "architecture".to_string(),
                            locations: vec![entry.path().display().to_string()],
                            suggestion: "Split into smaller, focused modules following Single Responsibility Principle".to_string(),
                        });
                    }
                }
            }
        }
    }

    Ok(issues)
}

/// Detect tight coupling between modules
fn detect_tight_coupling(
    dependencies: &HashMap<String, Vec<String>>,
    _all_modules: &HashSet<String>,
) -> Vec<Issue> {
    let mut issues = Vec::new();

    for (module, deps) in dependencies {
        if deps.len() > 15 {
            issues.push(Issue {
                title: "Tight coupling detected".to_string(),
                description: format!("{} depends on {} modules", module, deps.len()),
                severity: Severity::Medium,
                category: "coupling".to_string(),
                locations: vec![module.clone()],
                suggestion: "Reduce dependencies by using interfaces and dependency injection"
                    .to_string(),
            });
        }
    }

    issues
}

/// Detect missing architectural separation
fn detect_missing_separation(path: &str) -> Result<Vec<Issue>> {
    let mut issues = Vec::new();
    let mut has_tests = false;
    let mut has_models = false;
    let mut has_controllers = false;

    for entry in WalkDir::new(path).max_depth(2) {
        let entry = entry?;
        if let Some(name) = entry.file_name().to_str() {
            if name == "tests" || name == "test" {
                has_tests = true;
            }
            if name.contains("model") {
                has_models = true;
            }
            if name.contains("controller") || name.contains("handler") {
                has_controllers = true;
            }
        }
    }

    if !has_tests {
        issues.push(Issue {
            title: "Missing test organization".to_string(),
            description: "No dedicated test directory found".to_string(),
            severity: Severity::Medium,
            category: "structure".to_string(),
            locations: vec![path.to_string()],
            suggestion: "Create tests/ directory to organize unit and integration tests"
                .to_string(),
        });
    }

    if !has_models && !has_controllers {
        issues.push(Issue {
            title: "Unclear architectural layers".to_string(),
            description: "No clear separation between models, controllers, or handlers".to_string(),
            severity: Severity::Low,
            category: "structure".to_string(),
            locations: vec![path.to_string()],
            suggestion:
                "Consider organizing code into clear layers (models, controllers, services)"
                    .to_string(),
        });
    }

    Ok(issues)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_severity() {
        assert_eq!(parse_severity("low").unwrap(), Severity::Low);
        assert_eq!(parse_severity("medium").unwrap(), Severity::Medium);
        assert_eq!(parse_severity("high").unwrap(), Severity::High);
    }

    #[test]
    fn test_get_module_name() {
        let path = Path::new("src/analyzer.rs");
        assert_eq!(get_module_name(path), "analyzer");
    }

    #[test]
    fn test_detect_patterns() {
        let code_with_singleton =
            "lazy_static! { static ref INSTANCE: MyStruct = MyStruct::new(); }";
        let patterns = detect_patterns(Path::new("test.rs"), code_with_singleton);
        assert!(!patterns.is_empty());
    }

    #[test]
    fn test_extract_dependencies() {
        let code = "use std::fs;\nuse anyhow::Result;\nuse crate::module;";
        let deps = extract_dependencies(code);
        assert_eq!(deps.len(), 3);
        assert!(deps.contains(&"std::fs".to_string()));
    }

    #[test]
    fn test_is_excluded() {
        assert!(is_excluded(Path::new("target/debug/app")));
        assert!(is_excluded(Path::new("node_modules/package")));
        assert!(!is_excluded(Path::new("src/main.rs")));
    }
}
