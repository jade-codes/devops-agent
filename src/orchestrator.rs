mod subagent;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(name = "orchestrator")]
#[command(about = "Orchestrates multiple specialized agents")]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run complete testing workflow
    TestWorkflow {
        /// Repository path
        #[arg(short, long)]
        repo_path: PathBuf,

        /// Coverage threshold
        #[arg(short, long, default_value = "80")]
        threshold: u8,

        /// Max TODOs to resolve
        #[arg(short, long, default_value = "5")]
        max_todos: u8,
    },

    /// Run feature development workflow
    FeatureWorkflow {
        /// Repository path
        #[arg(short, long)]
        repo_path: PathBuf,

        /// GitHub issue number
        #[arg(short, long)]
        issue: u32,
    },

    /// Run quality review workflow
    QualityWorkflow {
        /// Repository path
        #[arg(short, long)]
        repo_path: PathBuf,
    },

    /// Custom workflow
    Custom {
        /// Agents to run in order (comma-separated)
        #[arg(short, long)]
        agents: String,

        /// Repository path
        #[arg(short, long)]
        repo_path: PathBuf,
    },

    /// Run bug fixing workflow
    BugWorkflow {
        /// Repository path
        #[arg(short, long)]
        repo_path: PathBuf,

        /// Max bugs to fix
        #[arg(short, long, default_value = "3")]
        max_bugs: u8,
    },

    /// Run chore/tech debt workflow
    ChoreWorkflow {
        /// Repository path
        #[arg(short, long)]
        repo_path: PathBuf,

        /// Max chores to resolve
        #[arg(short, long, default_value = "5")]
        max_chores: u8,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::TestWorkflow {
            repo_path,
            threshold,
            max_todos,
        } => {
            run_test_workflow(&repo_path, threshold, max_todos).await?;
        }
        Commands::FeatureWorkflow { repo_path, issue } => {
            run_feature_workflow(&repo_path, issue).await?;
        }
        Commands::QualityWorkflow { repo_path } => {
            run_quality_workflow(&repo_path).await?;
        }
        Commands::Custom { agents, repo_path } => {
            run_custom_workflow(&repo_path, &agents).await?;
        }
        Commands::BugWorkflow {
            repo_path,
            max_bugs,
        } => {
            run_bug_workflow(&repo_path, max_bugs).await?;
        }
        Commands::ChoreWorkflow {
            repo_path,
            max_chores,
        } => {
            run_chore_workflow(&repo_path, max_chores).await?;
        }
    }

    Ok(())
}

/// Test Workflow: Find missing tests → Implement them
async fn run_test_workflow(repo_path: &Path, threshold: u8, max_todos: u8) -> Result<()> {
    println!("🧪 Starting Test Workflow");
    println!("========================\n");

    // Step 1: Run coverage analysis
    println!("📊 Step 1: Running coverage analysis...");
    let coverage_result = subagent::run_coverage_agent(repo_path, threshold, true).await?;

    if !coverage_result.success {
        println!("❌ Coverage analysis failed:");
        println!("{}", coverage_result.stderr);
        return Ok(());
    }

    println!("✅ Coverage analysis complete\n");

    // Step 2: Get list of issues
    println!("📋 Step 2: Fetching coverage issues...");
    let issues = get_coverage_issues(repo_path)?;
    let issues_to_resolve = issues.into_iter().take(max_todos as usize);

    // Step 3: Resolve each issue
    for (idx, issue_num) in issues_to_resolve.enumerate() {
        println!("\n🔧 Step 3.{}: Resolving issue #{}...", idx + 1, issue_num);

        let resolver_result = subagent::run_todo_resolver(repo_path, issue_num, true).await?;

        if resolver_result.success {
            println!("✅ Issue #{issue_num} resolved");
        } else {
            println!("⚠️  Failed to resolve issue #{issue_num}");
            println!("{}", resolver_result.stderr);
        }
    }

    println!("\n✅ Test workflow complete!");
    Ok(())
}

/// Feature Workflow: Review architecture → Scan TODOs → Implement feature
async fn run_feature_workflow(repo_path: &Path, issue: u32) -> Result<()> {
    println!("🚀 Starting Feature Workflow");
    println!("===========================\n");

    // Step 1: Architecture review
    println!("🏗️  Step 1: Reviewing architecture...");
    let arch_result = subagent::run_architecture_reviewer(repo_path).await?;
    println!("{}", arch_result.stdout);

    // Step 2: Scan for related TODOs
    println!("\n📝 Step 2: Scanning for TODOs...");
    let todo_result = subagent::run_todo_scanner(repo_path, false).await?;
    println!("{}", todo_result.stdout);

    // Step 3: Implement feature
    println!("\n💻 Step 3: Implementing feature...");
    let feature_result = subagent::run_feature_implementer(repo_path, issue).await?;

    if feature_result.success {
        println!("✅ Feature implementation complete");
        println!("{}", feature_result.stdout);
    } else {
        println!("❌ Feature implementation failed:");
        println!("{}", feature_result.stderr);
    }

    println!("\n✅ Feature workflow complete!");
    Ok(())
}

/// Quality Workflow: Architecture review → Coverage analysis → TODO scan
async fn run_quality_workflow(repo_path: &Path) -> Result<()> {
    println!("🔍 Starting Quality Workflow");
    println!("==========================\n");

    // Step 1: Architecture review
    println!("🏗️  Step 1: Architecture review...");
    let arch_result = subagent::run_architecture_reviewer(repo_path).await?;
    println!("{}", arch_result.stdout);

    // Step 2: Coverage analysis
    println!("\n📊 Step 2: Coverage analysis...");
    let coverage_result = subagent::run_coverage_agent(repo_path, 80, false).await?;
    println!("{}", coverage_result.stdout);

    // Step 3: TODO scan
    println!("\n📝 Step 3: TODO scan...");
    let todo_result = subagent::run_todo_scanner(repo_path, false).await?;
    println!("{}", todo_result.stdout);

    println!("\n✅ Quality workflow complete!");
    Ok(())
}

/// Custom workflow
async fn run_custom_workflow(repo_path: &Path, agents: &str) -> Result<()> {
    println!("🎯 Starting Custom Workflow");
    println!("==========================\n");

    for agent in agents.split(',') {
        let agent = agent.trim();
        println!("🤖 Running {agent}...");

        let result = subagent::run_subagent(subagent::AgentRequest {
            agent: agent.to_string(),
            args: vec!["--repo-path".to_string(), repo_path.display().to_string()],
            working_dir: None,
        })
        .await?;

        println!("{}", result.stdout);

        if !result.success {
            println!("⚠️  {agent} failed");
        }
    }

    println!("\n✅ Custom workflow complete!");
    Ok(())
}

/// Bug Workflow: Find bugs → Fix them
async fn run_bug_workflow(repo_path: &Path, max_bugs: u8) -> Result<()> {
    println!("🐛 Starting Bug Workflow");
    println!("=====================\n");

    // Step 1: Scan for TODOs marked as bugs
    println!("🔍 Step 1: Scanning for bugs...");
    let todo_result = subagent::run_todo_scanner(repo_path, true).await?;
    println!("{}", todo_result.stdout);

    // Step 2: Get bug issues
    println!("\n📋 Step 2: Fetching bug issues...");
    let issues = get_bug_issues(repo_path)?;
    let issues_to_resolve = issues.into_iter().take(max_bugs as usize);

    // Step 3: Fix each bug
    for (idx, issue_num) in issues_to_resolve.enumerate() {
        println!("\n🔧 Step 3.{}: Fixing bug #{}...", idx + 1, issue_num);

        let resolver_result = subagent::run_todo_resolver(repo_path, issue_num, true).await?;

        if resolver_result.success {
            println!("✅ Bug #{issue_num} fixed");
        } else {
            println!("⚠️  Failed to fix bug #{issue_num}");
            println!("{}", resolver_result.stderr);
        }
    }

    println!("\n✅ Bug workflow complete!");
    Ok(())
}

/// Chore Workflow: Find tech debt → Resolve it
async fn run_chore_workflow(repo_path: &Path, max_chores: u8) -> Result<()> {
    println!("🧹 Starting Chore Workflow");
    println!("========================\n");

    // Step 1: Architecture review for tech debt
    println!("🏗️  Step 1: Reviewing architecture for tech debt...");
    let arch_result = subagent::run_architecture_reviewer(repo_path).await?;
    println!("{}", arch_result.stdout);

    // Step 2: Scan for chore TODOs
    println!("\n📝 Step 2: Scanning for chores...");
    let todo_result = subagent::run_todo_scanner(repo_path, true).await?;
    println!("{}", todo_result.stdout);

    // Step 3: Get chore issues
    println!("\n📋 Step 3: Fetching chore issues...");
    let issues = get_chore_issues(repo_path)?;
    let issues_to_resolve = issues.into_iter().take(max_chores as usize);

    // Step 4: Resolve each chore
    for (idx, issue_num) in issues_to_resolve.enumerate() {
        println!("\n🔧 Step 4.{}: Resolving chore #{}...", idx + 1, issue_num);

        let resolver_result = subagent::run_todo_resolver(repo_path, issue_num, true).await?;

        if resolver_result.success {
            println!("✅ Chore #{issue_num} resolved");
        } else {
            println!("⚠️  Failed to resolve chore #{issue_num}");
            println!("{}", resolver_result.stderr);
        }
    }

    println!("\n✅ Chore workflow complete!");
    Ok(())
}

/// Helper to get coverage issues from GitHub
fn get_coverage_issues(repo_path: &Path) -> Result<Vec<u32>> {
    use std::process::Command;

    let output = Command::new("gh")
        .args([
            "issue",
            "list",
            "--label",
            "coverage",
            "--json",
            "number",
            "--jq",
            ".[].number",
        ])
        .current_dir(repo_path)
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let issues: Vec<u32> = stdout
        .lines()
        .filter_map(|line| line.trim().parse().ok())
        .collect();

    Ok(issues)
}

/// Helper to get bug issues from GitHub
fn get_bug_issues(repo_path: &Path) -> Result<Vec<u32>> {
    use std::process::Command;

    let output = Command::new("gh")
        .args([
            "issue",
            "list",
            "--label",
            "bug",
            "--json",
            "number",
            "--jq",
            ".[].number",
        ])
        .current_dir(repo_path)
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let issues: Vec<u32> = stdout
        .lines()
        .filter_map(|line| line.trim().parse().ok())
        .collect();

    Ok(issues)
}

/// Helper to get chore issues from GitHub
fn get_chore_issues(repo_path: &Path) -> Result<Vec<u32>> {
    use std::process::Command;

    let output = Command::new("gh")
        .args([
            "issue",
            "list",
            "--label",
            "chore",
            "--json",
            "number",
            "--jq",
            ".[].number",
        ])
        .current_dir(repo_path)
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let issues: Vec<u32> = stdout
        .lines()
        .filter_map(|line| line.trim().parse().ok())
        .collect();

    Ok(issues)
}
