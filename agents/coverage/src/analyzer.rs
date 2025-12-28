use anyhow::{Context, Result};
use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};
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
    // Try cargo-llvm-cov first (much faster), fall back to tarpaulin
    println!("🔬 Attempting fast coverage with cargo-llvm-cov...");
    let llvm_cov_result = Command::new("cargo")
        .args([
            "llvm-cov",
            "--cobertura",
            "--output-path",
            "cobertura.xml",
            "--workspace",
            "--release",         // Use release builds (much faster tests)
            "--ignore-run-fail", // Continue even if some tests fail
        ])
        .current_dir(repo_path)
        .output();

    if llvm_cov_result.is_ok() && llvm_cov_result.as_ref().unwrap().status.success() {
        println!("✅ Used cargo-llvm-cov (fast)");
    } else {
        // Fall back to tarpaulin
        println!("⚠️  cargo-llvm-cov not available, using tarpaulin (slower)...");
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
                "--timeout",
                "300", // 5 minute timeout per test
                "--release", // Use release builds for faster execution
                "--lib", // Only test library code (skip bins)
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
    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);

    let mut overall_percentage = 0.0;
    let mut files = Vec::new();
    let mut current_file: Option<FileCoverage> = None;
    let mut current_method_name = String::new();
    let mut in_method = false;
    let mut method_line = 0;
    let mut method_hits = 0;

    loop {
        match reader.read_event() {
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => {
                match e.name().as_ref() {
                    b"coverage" => {
                        // Extract overall line-rate
                        for attr in e.attributes() {
                            if let Ok(attr) = attr {
                                if attr.key.as_ref() == b"line-rate" {
                                    if let Ok(value) = std::str::from_utf8(&attr.value) {
                                        overall_percentage =
                                            value.parse::<f32>().unwrap_or(0.0) * 100.0;
                                    }
                                }
                            }
                        }
                    }
                    b"class" => {
                        // Start a new file
                        let mut filename = String::new();
                        let mut line_rate = 0.0;

                        for attr in e.attributes() {
                            if let Ok(attr) = attr {
                                match attr.key.as_ref() {
                                    b"filename" => {
                                        if let Ok(value) = std::str::from_utf8(&attr.value) {
                                            filename = value.to_string();
                                        }
                                    }
                                    b"line-rate" => {
                                        if let Ok(value) = std::str::from_utf8(&attr.value) {
                                            line_rate = value.parse::<f32>().unwrap_or(0.0) * 100.0;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }

                        if !filename.is_empty() {
                            current_file = Some(FileCoverage {
                                path: filename,
                                coverage_percentage: line_rate,
                                lines_covered: 0,
                                lines_total: 0,
                                uncovered_lines: vec![],
                                functions: vec![],
                            });
                        }
                    }
                    b"method" => {
                        in_method = true;
                        method_hits = 0;

                        for attr in e.attributes() {
                            if let Ok(attr) = attr {
                                match attr.key.as_ref() {
                                    b"name" => {
                                        if let Ok(value) = std::str::from_utf8(&attr.value) {
                                            // Clean up method name
                                            current_method_name = value
                                                .replace("&lt;", "<")
                                                .replace("&gt;", ">")
                                                .replace("::{closure#0}", "");
                                        }
                                    }
                                    b"line-rate" => {
                                        if let Ok(value) = std::str::from_utf8(&attr.value) {
                                            let rate = value.parse::<f32>().unwrap_or(0.0);
                                            method_hits = if rate > 0.0 { 1 } else { 0 };
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    b"line" => {
                        if in_method {
                            for attr in e.attributes() {
                                if let Ok(attr) = attr {
                                    if attr.key.as_ref() == b"number" {
                                        if let Ok(value) = std::str::from_utf8(&attr.value) {
                                            method_line = value.parse::<usize>().unwrap_or(0);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(e)) => {
                match e.name().as_ref() {
                    b"method" => {
                        if in_method && !current_method_name.is_empty() {
                            if let Some(ref mut file) = current_file {
                                let coverage_pct = if method_hits > 0 { 100.0 } else { 0.0 };
                                file.functions.push(FunctionCoverage {
                                    name: current_method_name.clone(),
                                    line: method_line,
                                    coverage_percentage: coverage_pct,
                                    is_covered: method_hits > 0,
                                });
                            }
                            in_method = false;
                            current_method_name.clear();
                            method_line = 0;
                        }
                    }
                    b"class" => {
                        // Finish current file
                        if let Some(file) = current_file.take() {
                            // Only add files with functions
                            if !file.functions.is_empty() {
                                files.push(file);
                            }
                        }
                    }
                    _ => {}
                }
            }
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "Error parsing XML at position {}: {:?}",
                    reader.buffer_position(),
                    e
                ));
            }
            _ => {}
        }
    }

    Ok(CoverageData {
        overall_percentage,
        files,
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
