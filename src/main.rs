mod analyzer;
mod config;
mod github;
mod reporter;
mod scanner;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "devops-agent")]
#[command(about = "A agent that scans repositories and runs checklists using Claude", long_about = None)]
struct Args {
    /// Path to the repository to scan
    #[arg(short, long, default_value = ".")]
    repo_path: PathBuf,

    /// Path to the checklist configuration file
    #[arg(short, long, default_value = "checklist.yaml")]
    checklist: PathBuf,

    /// Only scan files changed in the current PR (requires GitHub context)
    #[arg(long)]
    pr_only: bool,

    /// Output format: console, markdown, or json
    #[arg(short, long, default_value = "console")]
    output: String,

    /// Post results as a GitHub PR comment
    #[arg(long)]
    post_comment: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    println!("🤖 DevOps Agent starting...");
    println!("📂 Repository: {:?}", args.repo_path);
    println!("📋 Checklist: {:?}", args.checklist);

    // Load configuration
    let config = config::load_checklist(&args.checklist)?;
    println!("✅ Loaded {} checklist items", config.items.len());

    // Get project context (check for Makefile, run-guidelines, etc.)
    let project_context = scanner::get_project_context(&args.repo_path)?;
    if project_context.has_run_guidelines {
        println!("🔧 Found 'make run-guidelines' target");
    }

    // Scan repository
    let files = scanner::scan_repository(&args.repo_path, &config, args.pr_only)?;
    println!("📁 Found {} files to analyze", files.len());

    if files.is_empty() {
        println!("No files to analyze. Exiting.");
        return Ok(());
    }

    // Analyze with Claude
    let results = analyzer::analyze_files(&files, &config, &project_context).await?;
    println!("🔍 Analysis complete");

    // Generate report
    let report = reporter::generate_report(&results, &args.output)?;

    // Output report
    match args.output.as_str() {
        "json" => println!("{}", serde_json::to_string_pretty(&report)?),
        _ => println!("\n{}", report),
    }

    // Post to GitHub if requested
    if args.post_comment {
        github::post_pr_comment(&report).await?;
        println!("💬 Posted comment to GitHub PR");
    }

    println!("✅ DevOps Agent complete!");
    Ok(())
}
