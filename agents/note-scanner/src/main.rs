use anyhow::Result;
use clap::Parser;

mod scanner;

#[derive(Parser, Debug)]
#[command(name = "note-scanner")]
#[command(about = "Scans code for important notes, observations, and documentation gaps")]
struct Args {
    /// Directory to scan
    #[arg(short, long, default_value = ".")]
    path: String,

    /// Output format (console, json, markdown)
    #[arg(short, long, default_value = "console")]
    format: String,

    /// Create GitHub issues for findings
    #[arg(short, long)]
    create_issues: bool,

    /// GitHub repository (owner/repo)
    #[arg(short = 'r', long)]
    repo: Option<String>,

    /// Minimum severity (low, medium, high)
    #[arg(short, long, default_value = "low")]
    severity: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("🔍 Scanning {} for important notes...", args.path);

    let findings = scanner::scan_directory(&args.path)?;

    // Filter by severity
    let severity_level = scanner::parse_severity(&args.severity)?;
    let filtered = scanner::filter_by_severity(&findings, severity_level);

    println!(
        "\n📊 Found {} items (filtered: {})",
        findings.len(),
        filtered.len()
    );

    // Output results
    match args.format.as_str() {
        "json" => scanner::output_json(&filtered)?,
        "markdown" => scanner::output_markdown(&filtered)?,
        _ => scanner::output_console(&filtered)?,
    }

    // Create GitHub issues if requested
    if args.create_issues {
        if let Some(repo) = args.repo {
            scanner::create_github_issues(&filtered, &repo)?;
        } else {
            eprintln!("⚠️  --repo required when --create-issues is used");
        }
    }

    Ok(())
}
