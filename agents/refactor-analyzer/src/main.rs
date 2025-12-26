use anyhow::Result;
use clap::Parser;

mod analyzer;
mod reporter;

#[derive(Parser, Debug)]
#[command(name = "refactor-analyzer")]
#[command(about = "Identifies code that needs refactoring based on complexity metrics")]
struct Args {
    /// Directory to analyze
    #[arg(short, long, default_value = ".")]
    path: String,

    /// Minimum complexity score to report (1-10)
    #[arg(short, long, default_value_t = 5)]
    threshold: u8,

    /// Output format (console, json, markdown)
    #[arg(short = 'f', long, default_value = "console")]
    format: String,

    /// Create GitHub issues for refactoring candidates
    #[arg(long)]
    create_issues: bool,

    /// GitHub repository (owner/repo)
    #[arg(short = 'r', long)]
    repo: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("🔍 Refactor Analyzer Agent");
    println!("   Analyzing: {}", args.path);
    println!("   Threshold: {}/10", args.threshold);

    // Analyze codebase
    let candidates = analyzer::analyze_directory(&args.path, args.threshold)?;

    println!("\n📊 Found {} refactoring candidates", candidates.len());

    // Sort by priority
    let mut sorted = candidates;
    sorted.sort_by(|a, b| b.priority_score().partial_cmp(&a.priority_score()).unwrap());

    // Output results
    match args.format.as_str() {
        "json" => reporter::output_json(&sorted)?,
        "markdown" => reporter::output_markdown(&sorted)?,
        _ => reporter::output_console(&sorted)?,
    }

    // Create issues if requested
    if args.create_issues {
        if let Some(repo) = args.repo {
            reporter::create_github_issues(&sorted, &repo)?;
        } else {
            eprintln!("⚠️  --repo required when --create-issues is used");
        }
    }

    Ok(())
}
