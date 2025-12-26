mod scanner;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "todo-scanner")]
#[command(about = "Scans code for TODO/FIXME comments and creates GitHub issues")]
struct Args {
    /// Path to the repository to scan
    #[arg(short, long, default_value = ".")]
    repo_path: PathBuf,

    /// File patterns to include (comma-separated)
    #[arg(short, long, default_value = "**/*.rs,**/*.py,**/*.js,**/*.ts")]
    include: String,

    /// File patterns to exclude (comma-separated)
    #[arg(
        short,
        long,
        default_value = "**/target/**,**/node_modules/**,**/dist/**"
    )]
    exclude: String,

    /// Create GitHub issues (requires gh CLI authentication)
    #[arg(long)]
    create_issues: bool,

    /// Output format: console, json, or markdown
    #[arg(short, long, default_value = "console")]
    output: String,

    /// Dry run - show what would be done without creating issues
    #[arg(long)]
    dry_run: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("🔍 TODO Scanner Agent starting...");
    println!("📂 Repository: {:?}", args.repo_path);

    // Scan for TODOs
    let todos = scanner::scan_todos(
        &args.repo_path,
        &parse_patterns(&args.include),
        &parse_patterns(&args.exclude),
    )?;

    println!("📋 Found {} TODO/FIXME/NOTE comments", todos.len());

    if todos.is_empty() {
        println!("✨ No TODOs found!");
        return Ok(());
    }

    // Output results
    match args.output.as_str() {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&todos)?);
        }
        "markdown" => {
            for todo in &todos {
                println!("## {}", todo.title());
                println!("**File:** `{}:{}`", todo.file, todo.line);
                println!("**Type:** {}", todo.todo_type);
                println!("**Content:** {}", todo.content);
                println!();
            }
        }
        _ => {
            for todo in &todos {
                println!("\n{}", todo.display());
            }
        }
    }

    // Create GitHub issues if requested
    if args.create_issues && !args.dry_run {
        println!("\n🚀 Creating GitHub issues...");
        scanner::create_github_issues(&todos)?;
        println!("✅ Created {} issues", todos.len());
    } else if args.dry_run {
        println!("\n🔬 Dry run - would create {} issues:", todos.len());
        for todo in &todos {
            println!("  - {}", todo.title());
        }
    }

    Ok(())
}

fn parse_patterns(patterns: &str) -> Vec<String> {
    patterns
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}
