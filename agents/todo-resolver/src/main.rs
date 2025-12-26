mod analyzer;
mod resolver;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "todo-resolver")]
#[command(about = "Resolves TODO items by implementing fixes following TDD")]
struct Args {
    /// Path to the repository
    #[arg(short, long, default_value = ".")]
    repo_path: PathBuf,

    /// GitHub issue number to resolve
    #[arg(short, long)]
    issue: Option<u32>,

    /// Scan for TODOs and pick one automatically
    #[arg(long)]
    auto: bool,

    /// Specific TODO to resolve (file:line format)
    #[arg(long)]
    todo: Option<String>,

    /// Create PR after implementing fix
    #[arg(long)]
    create_pr: bool,

    /// Dry run - analyze but don't make changes
    #[arg(long)]
    dry_run: bool,

    /// Skip test generation (only implement the fix)
    #[arg(long)]
    skip_tests: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("🔧 TODO Resolver Agent starting...");
    println!("📂 Repository: {:?}", args.repo_path);

    // Determine which TODO to resolve
    let todo_item = if let Some(issue_num) = args.issue {
        println!("📋 Loading TODO from issue #{}", issue_num);
        resolver::load_from_issue(issue_num)?
    } else if let Some(ref location) = args.todo {
        println!("📍 Loading TODO from: {}", location);
        resolver::load_from_location(&args.repo_path, location)?
    } else if args.auto {
        println!("🎯 Auto-selecting TODO...");
        resolver::select_todo_automatically(&args.repo_path)?
    } else {
        anyhow::bail!("Must specify --issue, --todo, or --auto");
    };

    println!("\n📝 Selected TODO:");
    println!("   File: {}:{}", todo_item.file, todo_item.line);
    println!("   Content: {}", todo_item.content);

    if args.dry_run {
        println!("\n🔬 Dry run - analyzing...");
        let analysis = analyzer::analyze_todo(&args.repo_path, &todo_item)?;
        println!("\n📊 Analysis:");
        println!("   Type: {}", analysis.todo_type);
        println!("   Complexity: {}", analysis.complexity);
        println!("   Suggested approach: {}", analysis.approach);
        return Ok(());
    }

    // TDD Flow: Write tests first
    if !args.skip_tests {
        println!("\n✅ Step 1: Writing tests...");
        let test_file = resolver::generate_tests(&args.repo_path, &todo_item)?;
        println!("   Created: {}", test_file);

        // Verify tests fail initially
        println!("\n🔴 Step 2: Running tests (should fail)...");
        resolver::run_tests(&args.repo_path)?;
    }

    // Implement the fix
    println!("\n🔨 Step 3: Implementing fix...");
    let changes = resolver::implement_fix(&args.repo_path, &todo_item)?;
    println!("   Modified {} file(s)", changes.len());

    // Verify tests pass
    if !args.skip_tests {
        println!("\n✅ Step 4: Running tests (should pass)...");
        resolver::run_tests(&args.repo_path)?;
    }

    // Create branch and commit
    println!("\n📦 Step 5: Committing changes...");
    let branch = resolver::commit_changes(&args.repo_path, &todo_item, &changes)?;
    println!("   Branch: {}", branch);

    // Create PR if requested
    if args.create_pr {
        println!("\n🚀 Step 6: Creating pull request...");
        let pr_url = resolver::create_pr_request(&todo_item, &branch)?;
        println!("   PR: {}", pr_url);
    }

    println!("\n✅ TODO resolved successfully!");

    Ok(())
}
