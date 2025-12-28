use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChecklistConfig {
    pub name: String,
    pub description: String,
    pub file_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub items: Vec<ChecklistItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChecklistItem {
    pub category: String,
    pub rule: String,
    pub description: String,
    pub severity: String, // "error", "warning", "info"
}

pub fn load_checklist(path: &Path) -> Result<ChecklistConfig> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read checklist file: {path:?}"))?;
    
    let config: ChecklistConfig = serde_yaml::from_str(&content)
        .with_context(|| "Failed to parse checklist YAML")?;
    
    Ok(config)
}
