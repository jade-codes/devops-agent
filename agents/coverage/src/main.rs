mod analyzer;
mod reporter;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "coverage")]
#[command(about = "Analyzes code coverage and creates issues for untested code")]
struct Args {
    /// Path to the repository to analyze
    #[arg(short, long, default_value = ".")]
    repo_path: PathBuf,

    /// Coverage threshold percentage (0-100)
    #[arg(short, long, default_value = "80")]
    threshold: f32,

    /// Create GitHub issues for uncovered code
    #[arg(long)]
    create_issues: bool,

    /// Output format: console, json, markdown, or csv
    #[arg(short, long, default_value = "console")]
    output: String,

    /// Dry run - show what would be done without creating issues
    #[arg(long)]
    dry_run: bool,

    /// Use existing coverage data instead of running cargo tarpaulin
    #[arg(long)]
    use_existing: bool,

    /// Path to cobertura.xml file (default: cobertura.xml)
    #[arg(long, default_value = "cobertura.xml")]
    coverage_file: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("📊 Coverage Agent starting...");
    println!("📂 Repository: {:?}", args.repo_path);
    println!("🎯 Coverage threshold: {}%", args.threshold);

    // Run or load coverage
    let coverage_data = if args.use_existing {
        println!(
            "📖 Loading existing coverage data from {:?}",
            args.coverage_file
        );
        analyzer::load_coverage(&args.coverage_file)?
    } else {
        println!("🔬 Running cargo tarpaulin...");
        analyzer::run_coverage(&args.repo_path)?
    };

    println!("✅ Coverage analysis complete");
    println!(
        "📈 Overall coverage: {:.1}%",
        coverage_data.overall_percentage
    );

    // Find uncovered items
    let uncovered = analyzer::find_uncovered(&coverage_data, args.threshold);
    println!(
        "📋 Found {} uncovered items below threshold",
        uncovered.len()
    );

    if uncovered.is_empty() {
        println!("✨ Coverage meets threshold!");
        return Ok(());
    }

    // Output results
    match args.output.as_str() {
        "json" => {
            let report =
                reporter::generate_json_report(&coverage_data, &uncovered, args.threshold)?;
            println!("{report}");
        }
        "markdown" => {
            let report =
                reporter::generate_markdown_report(&coverage_data, &uncovered, args.threshold);
            println!("{report}");
        }
        "csv" => {
            let report = reporter::generate_csv_report(&coverage_data, &uncovered, args.threshold);
            println!("{report}");
        }
        _ => {
            reporter::print_console_report(&coverage_data, &uncovered, args.threshold);
        }
    }

    // Create GitHub issues if requested
    if args.create_issues && !args.dry_run {
        println!("\n🚀 Creating GitHub issues...");
        reporter::create_github_issues(&uncovered)?;
        println!("✅ Created {} issues", uncovered.len());
    } else if args.dry_run {
        println!("\n🔬 Dry run - would create {} issues", uncovered.len());
    }

    Ok(())
}
