use anyhow::Result;
use clap::Parser;

mod analyzer;
mod implementer;

#[derive(Parser, Debug)]
#[command(name = "feature-implementer")]
#[command(about = "Implements new features following strict TDD methodology")]
struct Args {
    /// Feature description or GitHub issue number
    #[arg(short, long)]
    feature: String,

    /// Repository path
    #[arg(short, long, default_value = ".")]
    repo: String,

    /// Generate tests first
    #[arg(long, default_value_t = true)]
    test_first: bool,

    /// Create pull request
    #[arg(long)]
    create_pr: bool,

    /// Target branch for PR
    #[arg(long, default_value = "main")]
    target_branch: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("🚀 Feature Implementer Agent");
    println!("   Feature: {}", args.feature);

    // Step 1: Analyze feature requirements
    println!("\n📋 Step 1: Analyzing feature requirements...");
    let spec = analyzer::analyze_feature(&args.feature, &args.repo)?;
    println!("   Type: {:?}", spec.feature_type);
    println!("   Complexity: {:?}", spec.complexity);
    println!("   Files affected: {}", spec.affected_files.len());

    // Step 2: Generate test cases
    if args.test_first {
        println!("\n✅ Step 2: Generating test cases...");
        let tests = implementer::generate_tests(&spec)?;
        println!("   Generated {} test cases", tests.len());

        for test in &tests {
            implementer::write_test(&test, &args.repo)?;
        }
    }

    // Step 3: Verify tests fail (red phase)
    if args.test_first {
        println!("\n🔴 Step 3: Verifying tests fail (RED phase)...");
        if implementer::run_tests(&args.repo)? {
            println!("   ⚠️  Tests passed before implementation - may need review");
        } else {
            println!("   ✓ Tests fail as expected");
        }
    }

    // Step 4: Implement feature
    println!("\n💻 Step 4: Implementing feature...");
    let implementation = implementer::implement_feature(&spec, &args.repo)?;
    println!(
        "   Implemented in {} files",
        implementation.files_modified.len()
    );

    // Step 5: Verify tests pass (green phase)
    println!("\n🟢 Step 5: Verifying tests pass (GREEN phase)...");
    if implementer::run_tests(&args.repo)? {
        println!("   ✓ All tests pass");
    } else {
        println!("   ✗ Tests failed - implementation needs work");
        return Ok(());
    }

    // Step 6: Create PR if requested
    if args.create_pr {
        println!("\n🚀 Step 6: Creating pull request...");
        let pr_url = implementer::create_pr(&spec, &args.target_branch, &args.repo)?;
        println!("   PR: {}", pr_url);
    }

    println!("\n✨ Feature implementation complete!");
    Ok(())
}
