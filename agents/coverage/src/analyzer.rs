use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CoverageData {
    pub overall_percentage: f32,
    pub files: Vec<FileCoverage>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileCoverage {
    pub path: String,
    pub coverage_percentage: f32,
    pub lines_covered: usize,
    pub lines_total: usize,
    pub uncovered_lines: Vec<usize>,
    pub functions: Vec<FunctionCoverage>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionCoverage {
    pub name: String,
    pub line: usize,
    pub coverage_percentage: f32,
    pub is_covered: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UncoveredItem {
    pub file: String,
    pub function: String,
    pub line: usize,
    pub coverage_percentage: f32,
    pub item_type: UncoveredType,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum UncoveredType {
    Function,
    PublicFunction,
    TestFunction,
}

impl UncoveredItem {
    pub fn title(&self) -> String {
        let type_str = match self.item_type {
            UncoveredType::PublicFunction => "public function",
            UncoveredType::Function => "function",
            UncoveredType::TestFunction => "test function",
        };
        format!("test: Add tests for {} `{}`", type_str, self.function)
    }

    pub fn severity(&self) -> &str {
        match self.item_type {
            UncoveredType::PublicFunction => "error",
            UncoveredType::Function => "warning",
            UncoveredType::TestFunction => "info",
        }
    }
}

pub fn run_coverage(repo_path: &Path) -> Result<CoverageData> {
    // Run cargo tarpaulin
    let output = Command::new("cargo")
        .args([
            "tarpaulin",
            "--out",
            "Xml",
            "--output-dir",
            ".",
            "--skip-clean",
            "--exclude-files",
            "target/*",
        ])
        .current_dir(repo_path)
        .output()
        .context(
            "Failed to run cargo tarpaulin. Is it installed? Run: cargo install cargo-tarpaulin",
        )?;

    if !output.status.success() {
        anyhow::bail!(
            "cargo tarpaulin failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Load the generated cobertura.xml
    let coverage_file = repo_path.join("cobertura.xml");
    load_coverage(&coverage_file)
}

pub fn load_coverage(coverage_file: &Path) -> Result<CoverageData> {
    let content = fs::read_to_string(coverage_file)
        .context("Failed to read coverage file. Run cargo tarpaulin first.")?;

    parse_cobertura(&content)
}

fn parse_cobertura(xml: &str) -> Result<CoverageData> {
    // Parse cobertura XML (simplified parser for demo)
    // In production, use a proper XML parser like quick-xml

    let overall_regex = regex::Regex::new(r#"line-rate="([0-9.]+)""#)?;
    let overall_percentage = if let Some(cap) = overall_regex.captures(xml) {
        cap[1].parse::<f32>()? * 100.0
    } else {
        0.0
    };

    // For now, return simplified data structure
    // In production, parse full XML to extract all files, lines, and functions
    Ok(CoverageData {
        overall_percentage,
        files: vec![],
    })
}

pub fn find_uncovered(coverage: &CoverageData, threshold: f32) -> Vec<UncoveredItem> {
    let mut uncovered = Vec::new();

    for file in &coverage.files {
        if file.coverage_percentage < threshold {
            for func in &file.functions {
                if func.coverage_percentage < threshold {
                    let item_type = if func.name.starts_with("test_") {
                        UncoveredType::TestFunction
                    } else if func.name.starts_with("pub ") {
                        UncoveredType::PublicFunction
                    } else {
                        UncoveredType::Function
                    };

                    uncovered.push(UncoveredItem {
                        file: file.path.clone(),
                        function: func.name.clone(),
                        line: func.line,
                        coverage_percentage: func.coverage_percentage,
                        item_type,
                    });
                }
            }
        }
    }

    uncovered
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cobertura_extracts_overall_percentage() {
        let xml = r#"<?xml version="1.0" ?>
<coverage line-rate="0.85" branch-rate="0.75">
</coverage>"#;

        let coverage = parse_cobertura(xml).unwrap();
        assert_eq!(coverage.overall_percentage, 85.0);
    }

    #[test]
    fn test_find_uncovered_filters_by_threshold() {
        let coverage = CoverageData {
            overall_percentage: 70.0,
            files: vec![FileCoverage {
                path: "src/lib.rs".to_string(),
                coverage_percentage: 60.0,
                lines_covered: 60,
                lines_total: 100,
                uncovered_lines: vec![10, 20, 30],
                functions: vec![
                    FunctionCoverage {
                        name: "pub covered_func".to_string(),
                        line: 1,
                        coverage_percentage: 95.0,
                        is_covered: true,
                    },
                    FunctionCoverage {
                        name: "uncovered_func".to_string(),
                        line: 10,
                        coverage_percentage: 50.0,
                        is_covered: false,
                    },
                ],
            }],
        };

        let uncovered = find_uncovered(&coverage, 80.0);
        assert_eq!(uncovered.len(), 1);
        assert_eq!(uncovered[0].function, "uncovered_func");
    }

    #[test]
    fn test_uncovered_item_title_generation() {
        let item = UncoveredItem {
            file: "src/lib.rs".to_string(),
            function: "process_data".to_string(),
            line: 42,
            coverage_percentage: 30.0,
            item_type: UncoveredType::PublicFunction,
        };

        assert_eq!(
            item.title(),
            "test: Add tests for public function `process_data`"
        );
    }

    #[test]
    fn test_public_function_has_error_severity() {
        let item = UncoveredItem {
            file: "src/lib.rs".to_string(),
            function: "api_func".to_string(),
            line: 1,
            coverage_percentage: 0.0,
            item_type: UncoveredType::PublicFunction,
        };

        assert_eq!(item.severity(), "error");
    }
}
