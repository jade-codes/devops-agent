mod creator;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "issue-creator")]
#[command(about = "Creates GitHub issues from structured input")]
struct Args {
    /// Title of the issue
    #[arg(short, long)]
    title: Option<String>,

    /// Body/description of the issue
    #[arg(short, long)]
    body: Option<String>,

    /// Labels (comma-separated)
    #[arg(short, long)]
    labels: Option<String>,

    /// Assignees (comma-separated)
    #[arg(short, long)]
    assignees: Option<String>,

    /// Milestone
    #[arg(short, long)]
    milestone: Option<String>,

    /// Priority (low, medium, high, critical)
    #[arg(short, long)]
    priority: Option<String>,

    /// JSON file with issue data
    #[arg(long)]
    from_json: Option<PathBuf>,

    /// Batch create from JSON array file
    #[arg(long)]
    batch: Option<PathBuf>,

    /// Dry run - show what would be created
    #[arg(long)]
    dry_run: bool,

    /// Output issue URL(s) to file
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("📝 Issue Creator Agent starting...");

    let issues = if let Some(batch_file) = args.batch {
        // Batch create from JSON array
        println!("📚 Loading batch issues from {:?}", batch_file);
        creator::load_batch_issues(&batch_file)?
    } else if let Some(json_file) = args.from_json {
        // Single issue from JSON
        println!("📄 Loading issue from {:?}", json_file);
        vec![creator::load_issue_from_json(&json_file)?]
    } else {
        // Create from CLI args
        vec![creator::IssueRequest {
            title: args
                .title
                .ok_or_else(|| anyhow::anyhow!("Title is required"))?,
            body: args.body,
            labels: args
                .labels
                .map(|l| l.split(',').map(|s| s.trim().to_string()).collect()),
            assignees: args
                .assignees
                .map(|a| a.split(',').map(|s| s.trim().to_string()).collect()),
            milestone: args.milestone,
            priority: args.priority,
        }]
    };

    println!("📋 Issues to create: {}", issues.len());

    if args.dry_run {
        println!("\n🔬 Dry run - would create:");
        for (i, issue) in issues.iter().enumerate() {
            println!("\n{}. {}", i + 1, issue.title);
            if let Some(body) = &issue.body {
                println!("   {}", body.lines().next().unwrap_or(""));
            }
            if let Some(labels) = &issue.labels {
                println!("   Labels: {}", labels.join(", "));
            }
        }
        return Ok(());
    }

    // Create issues
    println!("\n🚀 Creating issues...");
    let results = creator::create_issues(&issues)?;

    for (i, url) in results.iter().enumerate() {
        println!("  {}. ✓ Created: {}", i + 1, url);
    }

    println!("\n✅ Created {} issues", results.len());

    // Save to output file if specified
    if let Some(output_file) = args.output {
        std::fs::write(&output_file, results.join("\n"))?;
        println!("📁 Saved issue URLs to {:?}", output_file);
    }

    Ok(())
}
