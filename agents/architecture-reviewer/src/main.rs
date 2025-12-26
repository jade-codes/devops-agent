use anyhow::Result;
use clap::Parser;

mod analyzer;
mod reporter;

#[derive(Parser, Debug)]
#[command(name = "architecture-reviewer")]
#[command(
    about = "Reviews codebase architecture and identifies patterns, anti-patterns, and improvements"
)]
struct Args {
    /// Directory to analyze
    #[arg(short, long, default_value = ".")]
    path: String,

    /// Output format (console, json, markdown)
    #[arg(short = 'f', long, default_value = "console")]
    format: String,

    /// Create GitHub issues for findings
    #[arg(long)]
    create_issues: bool,

    /// GitHub repository (owner/repo)
    #[arg(short = 'r', long)]
    repo: Option<String>,

    /// Show only specific severity (high, medium, low)
    #[arg(short, long)]
    severity: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("🏗️  Architecture Reviewer Agent");
    println!("   Analyzing: {}", args.path);

    // Analyze architecture
    let report = analyzer::analyze_architecture(&args.path)?;

    println!("\n📊 Architecture Analysis Complete");
    println!("   Modules: {}", report.module_count);
    println!("   Patterns detected: {}", report.patterns.len());
    println!("   Issues found: {}", report.issues.len());

    // Filter by severity if specified
    let issues = if let Some(sev) = &args.severity {
        let severity = analyzer::parse_severity(sev)?;
        report
            .issues
            .clone()
            .into_iter()
            .filter(|i| i.severity == severity)
            .collect()
    } else {
        report.issues.clone()
    };

    // Output results
    match args.format.as_str() {
        "json" => reporter::output_json(&report, &issues)?,
        "markdown" => reporter::output_markdown(&report, &issues)?,
        _ => reporter::output_console(&report, &issues)?,
    }

    // Create issues if requested
    if args.create_issues {
        if let Some(repo) = args.repo {
            reporter::create_github_issues(&issues, &repo)?;
        } else {
            eprintln!("⚠️  --repo required when --create-issues is used");
        }
    }

    Ok(())
}
