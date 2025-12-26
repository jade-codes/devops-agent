use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RefactorCandidate {
    pub file: String,
    pub function: String,
    pub line_start: usize,
    pub line_end: usize,
    pub complexity_score: u8,
    pub lines_of_code: usize,
    pub nesting_depth: u8,
    pub num_parameters: usize,
    pub issues: Vec<String>,
}

impl RefactorCandidate {
    /// Calculate priority score for refactoring
    pub fn priority_score(&self) -> f32 {
        let complexity_weight = self.complexity_score as f32 * 0.4;
        let size_weight = (self.lines_of_code as f32 / 10.0).min(10.0) * 0.3;
        let nesting_weight = self.nesting_depth as f32 * 0.2;
        let params_weight = (self.num_parameters as f32 / 2.0).min(10.0) * 0.1;

        complexity_weight + size_weight + nesting_weight + params_weight
    }
}

/// Analyze directory for refactoring candidates
pub fn analyze_directory(path: &str, threshold: u8) -> Result<Vec<RefactorCandidate>> {
    let mut candidates = Vec::new();

    for entry in WalkDir::new(path)
        .into_iter()
        .filter_entry(|e| !is_excluded(e.path()))
    {
        let entry = entry?;
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension() {
                if ext == "rs" || ext == "py" || ext == "js" || ext == "ts" {
                    if let Ok(file_candidates) = analyze_file(entry.path(), threshold) {
                        candidates.extend(file_candidates);
                    }
                }
            }
        }
    }

    Ok(candidates)
}

/// Check if path should be excluded
fn is_excluded(path: &Path) -> bool {
    let excluded = ["target", "node_modules", ".git", "dist", "build"];
    path.components().any(|c| {
        if let Some(s) = c.as_os_str().to_str() {
            excluded.contains(&s)
        } else {
            false
        }
    })
}

/// Analyze a single file
pub fn analyze_file(path: &Path, threshold: u8) -> Result<Vec<RefactorCandidate>> {
    let content = fs::read_to_string(path)?;
    let mut candidates = Vec::new();

    // Find all functions
    let func_pattern = Regex::new(r"(?m)^\s*(?:pub\s+)?(?:async\s+)?fn\s+(\w+)")?;

    for capture in func_pattern.captures_iter(&content) {
        if let Some(func_name) = capture.get(1) {
            if let Some(candidate) =
                analyze_function(path, &content, func_name.as_str(), threshold)?
            {
                candidates.push(candidate);
            }
        }
    }

    Ok(candidates)
}

/// Analyze a specific function
fn analyze_function(
    file: &Path,
    content: &str,
    func_name: &str,
    threshold: u8,
) -> Result<Option<RefactorCandidate>> {
    // Find function boundaries
    let func_pattern = Regex::new(&format!(r"fn\s+{}\s*\(", regex::escape(func_name)))?;
    let Some(func_match) = func_pattern.find(content) else {
        return Ok(None);
    };

    let start_pos = func_match.start();
    let line_start = content[..start_pos].lines().count() + 1;

    // Find function end (simple brace matching)
    let func_body = &content[start_pos..];
    let end_pos = find_function_end(func_body);
    let line_end = content[..start_pos + end_pos].lines().count() + 1;

    let func_code = &content[start_pos..start_pos + end_pos];
    let lines_of_code = func_code.lines().count();

    // Calculate metrics
    let complexity = calculate_complexity(func_code);
    let nesting = calculate_nesting_depth(func_code);
    let params = count_parameters(func_code);

    // Identify issues
    let mut issues = Vec::new();
    if complexity >= 8 {
        issues.push(format!("High cyclomatic complexity: {}", complexity));
    }
    if lines_of_code > 50 {
        issues.push(format!("Function too long: {} lines", lines_of_code));
    }
    if nesting > 4 {
        issues.push(format!("Deep nesting: {} levels", nesting));
    }
    if params > 5 {
        issues.push(format!("Too many parameters: {}", params));
    }

    if complexity >= threshold {
        Ok(Some(RefactorCandidate {
            file: file.display().to_string(),
            function: func_name.to_string(),
            line_start,
            line_end,
            complexity_score: complexity,
            lines_of_code,
            nesting_depth: nesting,
            num_parameters: params,
            issues,
        }))
    } else {
        Ok(None)
    }
}

/// Find the end of a function body
fn find_function_end(code: &str) -> usize {
    let mut brace_count = 0;
    let mut in_function = false;

    for (i, ch) in code.char_indices() {
        if ch == '{' {
            brace_count += 1;
            in_function = true;
        } else if ch == '}' {
            brace_count -= 1;
            if in_function && brace_count == 0 {
                return i + 1;
            }
        }
    }

    code.len()
}

/// Calculate cyclomatic complexity
pub fn calculate_complexity(code: &str) -> u8 {
    let mut complexity = 1;

    // Count decision points
    for word in code.split_whitespace() {
        if matches!(
            word,
            "if" | "else" | "for" | "while" | "match" | "&&" | "||" | "?" | "case"
        ) {
            complexity += 1;
        }
    }

    complexity.min(10)
}

/// Calculate maximum nesting depth
pub fn calculate_nesting_depth(code: &str) -> u8 {
    let mut max_depth: i32 = 0;
    let mut current_depth: i32 = 0;

    for ch in code.chars() {
        if ch == '{' {
            current_depth += 1;
            max_depth = max_depth.max(current_depth);
        } else if ch == '}' {
            current_depth = current_depth.saturating_sub(1);
        }
    }

    max_depth.min(255) as u8
}

/// Count function parameters
pub fn count_parameters(code: &str) -> usize {
    let param_pattern = match Regex::new(r"fn\s+\w+\s*\((.*?)\)") {
        Ok(p) => p,
        Err(_) => return 0,
    };

    let captures = match param_pattern.captures(code) {
        Some(c) => c,
        None => return 0,
    };

    let params = match captures.get(1) {
        Some(p) => p.as_str(),
        None => return 0,
    };

    if params.trim().is_empty() {
        0
    } else {
        params.split(',').count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_complexity() {
        let simple = "fn test() { return 42; }";
        let complex = "fn test() { if x { for y in z { if a || b { while c { } } } } }";

        assert_eq!(calculate_complexity(simple), 1);
        assert!(calculate_complexity(complex) >= 5);
    }

    #[test]
    fn test_calculate_nesting_depth() {
        let flat = "fn test() { let x = 1; }";
        let nested = "fn test() { { { { } } } }";

        assert_eq!(calculate_nesting_depth(flat), 1);
        assert_eq!(calculate_nesting_depth(nested), 4);
    }

    #[test]
    fn test_count_parameters() {
        let no_params = "fn test() { }";
        let three_params = "fn test(a: i32, b: String, c: bool) { }";

        assert_eq!(count_parameters(no_params), 0);
        assert_eq!(count_parameters(three_params), 3);
    }

    #[test]
    fn test_priority_score() {
        let candidate = RefactorCandidate {
            file: "test.rs".to_string(),
            function: "complex_func".to_string(),
            line_start: 1,
            line_end: 100,
            complexity_score: 10,
            lines_of_code: 80,
            nesting_depth: 5,
            num_parameters: 8,
            issues: vec![],
        };

        let score = candidate.priority_score();
        assert!(score > 5.0); // High priority
    }

    #[test]
    fn test_find_function_end() {
        let code = "fn test() { let x = 1; { nested(); } }";
        let end = find_function_end(code);
        assert_eq!(&code[..end], "fn test() { let x = 1; { nested(); } }");
    }
}
