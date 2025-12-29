use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use std::process::Command;

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

    /// Run coverage analysis and create issues
    CoverageWorkflow {
        /// Repository path
        #[arg(short, long)]
        repo_path: PathBuf,

        /// Coverage threshold
        #[arg(short, long, default_value = "90")]
        threshold: u8,
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
        Commands::CoverageWorkflow {
            repo_path,
            threshold,
        } => {
            run_coverage_workflow(&repo_path, threshold).await?;
        }
    }

    Ok(())
}

/// Test Workflow: Find missing tests → Implement them
async fn run_test_workflow(repo_path: &Path, _threshold: u8, max_todos: u8) -> Result<()> {
    println!("🧪 Starting Test Workflow");
    println!("========================\n");

    // Step 1: Get list of testing issues (already created by coverage agent)
    println!("📋 Step 1: Fetching testing issues...");
    let issues = get_coverage_issues(repo_path)?;

    if issues.is_empty() {
        println!("⚠️  No testing issues found. Run coverage analysis first:");
        println!("   coverage --repo-path . --threshold 90 --create-issues");
        return Ok(());
    }

    println!("✅ Found {} testing issues\n", issues.len());

    let issues_to_resolve = issues.into_iter().take(max_todos as usize);

    // Step 2: Spawn agent task for each issue
    for (idx, issue_num) in issues_to_resolve.enumerate() {
        println!(
            "\n🤖 Step 2.{}: Spawning agent for issue #{}...",
            idx + 1,
            issue_num
        );

        // Fetch issue details to create task description
        let issue_details = Command::new("gh")
            .args([
                "issue",
                "view",
                &issue_num.to_string(),
                "--json",
                "title,body",
            ])
            .current_dir(repo_path)
            .output()?;

        if !issue_details.status.success() {
            println!("⚠️  Failed to fetch issue #{}: skipping", issue_num);
            continue;
        }

        let issue_json: serde_json::Value =
            serde_json::from_slice(&issue_details.stdout).unwrap_or_default();
        let title = issue_json["title"].as_str().unwrap_or("");

        // Create agent task to generate tests for this issue
        let task_description = format!(
            "Generate comprehensive tests for the function mentioned in issue #{}.

Issue: {}

Requirements:
- Read the source code to understand the function
- Generate working, compilable tests (no TODO comments or placeholders)
- Create tests in a separate test file (e.g., filename_test.rs)
- Ensure test file is properly included in the module
- Run cargo test to ensure they compile and pass
- Commit changes with message: 'test: Add tests for <function> (closes #{})'
- Create a pull request

If the function cannot be tested without significant setup, skip it and report why.",
            issue_num, title, issue_num
        );

        // Invoke GitHub Copilot agent via gh CLI
        println!("   Spawning agent task...");
        let agent_result = Command::new("gh")
            .args(["agent-task", "create", &task_description])
            .current_dir(repo_path)
            .output()?;

        if agent_result.status.success() {
            println!("✅ Agent task spawned for issue #{}", issue_num);
        } else {
            println!("⚠️  Failed to spawn agent for issue #{}", issue_num);
            println!("{}", String::from_utf8_lossy(&agent_result.stderr));
        }
    }

    println!("\n✅ Test workflow complete!");
    Ok(())
}

/// Feature Workflow: Implement feature using agent task
async fn run_feature_workflow(repo_path: &Path, issue: u32) -> Result<()> {
    println!("🚀 Starting Feature Workflow");
    println!("===========================\n");

    // Fetch issue details
    println!("📋 Fetching issue details...");
    let issue_details = Command::new("gh")
        .args(["issue", "view", &issue.to_string(), "--json", "title,body"])
        .current_dir(repo_path)
        .output()?;

    if !issue_details.status.success() {
        anyhow::bail!("Failed to fetch issue #{}", issue);
    }

    let issue_json: serde_json::Value =
        serde_json::from_slice(&issue_details.stdout).unwrap_or_default();
    let title = issue_json["title"].as_str().unwrap_or("");
    let body = issue_json["body"].as_str().unwrap_or("");

    // Create comprehensive task description for the agent
    let task_description = format!(
        "Implement the feature described in issue #{}.

Issue: {}

Description:
{}

Requirements:
1. Review the architecture and codebase
2. Scan for related TODOs that might be relevant
3. Implement the feature following best practices
4. Add comprehensive tests
5. Run all tests to ensure they pass
6. Commit changes with clear, descriptive commit message
7. Create a pull request referencing issue #{}

Please provide a complete, working implementation.",
        issue, title, body, issue
    );

    // Spawn agent task
    println!("\n🤖 Spawning agent to implement feature...");
    let agent_result = Command::new("gh")
        .args(["agent-task", "create", &task_description])
        .current_dir(repo_path)
        .output()?;

    if agent_result.status.success() {
        println!("\n✅ Agent task spawned for issue #{}", issue);
        println!("{}", String::from_utf8_lossy(&agent_result.stdout));
    } else {
        println!("\n❌ Failed to spawn agent:");
        println!("{}", String::from_utf8_lossy(&agent_result.stderr));
        anyhow::bail!("Agent task creation failed");
    }

    Ok(())
}

/// Quality Workflow: Run coverage and create GitHub agent tasks for improvements
async fn run_quality_workflow(repo_path: &Path) -> Result<()> {
    println!("🔍 Starting Quality Workflow");
    println!("==========================\n");

    // Use the coverage agent (still useful for analysis)
    println!("📊 Running coverage analysis...");
    let coverage_result = Command::new("./agents/coverage/target/release/coverage")
        .args([
            "--repo-path",
            repo_path.display().to_string().as_str(),
            "--threshold",
            "80",
        ])
        .output()?;

    println!("{}", String::from_utf8_lossy(&coverage_result.stdout));

    if !coverage_result.status.success() {
        println!("⚠️  Coverage analysis had issues");
        println!("{}", String::from_utf8_lossy(&coverage_result.stderr));
    }

    println!("\n✅ Quality workflow complete!");
    println!("   Run 'test-workflow' to generate tests for low-coverage functions");
    Ok(())
}

/// Custom workflow - spawn an agent task with custom instructions
async fn run_custom_workflow(repo_path: &Path, task_description: &str) -> Result<()> {
    println!("🎯 Starting Custom Workflow");
    println!("==========================\n");

    println!("🤖 Spawning custom agent task...");
    let agent_result = Command::new("gh")
        .args(["agent-task", "create", task_description])
        .current_dir(repo_path)
        .output()?;

    if agent_result.status.success() {
        println!("\n✅ Agent task spawned");
        println!("{}", String::from_utf8_lossy(&agent_result.stdout));
    } else {
        println!("\n❌ Failed to spawn agent:");
        println!("{}", String::from_utf8_lossy(&agent_result.stderr));
        anyhow::bail!("Agent task failed");
    }

    Ok(())
}

/// Bug Workflow: Spawn agent tasks to fix bugs
async fn run_bug_workflow(repo_path: &Path, max_bugs: u8) -> Result<()> {
    println!("🐛 Starting Bug Workflow");
    println!("=====================\n");

    // Get bug issues from GitHub
    println!("📋 Fetching bug issues...");
    let issues = get_bug_issues(repo_path)?;

    if issues.is_empty() {
        println!("⚠️  No bug issues found");
        return Ok(());
    }

    println!("✅ Found {} bug issues\n", issues.len());

    let issues_to_resolve = issues.into_iter().take(max_bugs as usize);

    // Spawn agent task for each bug
    for (idx, issue_num) in issues_to_resolve.enumerate() {
        println!(
            "\n🤖 Step {}: Spawning agent for issue #{}...",
            idx + 1,
            issue_num
        );

        // Fetch issue details
        let issue_details = Command::new("gh")
            .args([
                "issue",
                "view",
                &issue_num.to_string(),
                "--json",
                "title,body",
            ])
            .current_dir(repo_path)
            .output()?;

        if !issue_details.status.success() {
            println!("⚠️  Failed to fetch issue #{}: skipping", issue_num);
            continue;
        }

        let issue_json: serde_json::Value =
            serde_json::from_slice(&issue_details.stdout).unwrap_or_default();
        let title = issue_json["title"].as_str().unwrap_or("");
        let body = issue_json["body"].as_str().unwrap_or("");

        // Create agent task to fix the bug
        let task_description = format!(
            "Fix the bug described in issue #{}.

Issue: {}

Description:
{}

Requirements:
- Analyze the bug and identify root cause
- Implement a fix
- Add tests to verify the fix and prevent regression
- Run all tests to ensure nothing breaks
- Commit with message: 'fix: <description> (closes #{})'
- Create a pull request

Please provide a complete solution.",
            issue_num, title, body, issue_num
        );

        println!("   Spawning agent task...");
        let agent_result = Command::new("gh")
            .args(["agent-task", "create", &task_description])
            .current_dir(repo_path)
            .output()?;

        if agent_result.status.success() {
            println!("✅ Agent task spawned for issue #{}", issue_num);
        } else {
            println!("⚠️  Failed to spawn agent for issue #{}", issue_num);
            println!("{}", String::from_utf8_lossy(&agent_result.stderr));
        }
    }

    println!("\n✅ Bug workflow complete!");
    Ok(())
}

/// Chore Workflow: Spawn agent tasks for tech debt and chores
async fn run_chore_workflow(repo_path: &Path, max_chores: u8) -> Result<()> {
    println!("🧹 Starting Chore Workflow");
    println!("========================\n");

    // Get chore issues from GitHub
    println!("📋 Fetching chore issues...");
    let issues = get_chore_issues(repo_path)?;

    if issues.is_empty() {
        println!("⚠️  No chore issues found");
        return Ok(());
    }

    println!("✅ Found {} chore issues\n", issues.len());

    // Spawn agent task for each chore
    for (idx, issue_num) in issues.into_iter().take(max_chores as usize).enumerate() {
        println!(
            "\n🤖 Step {}: Spawning agent for chore #{}...",
            idx + 1,
            issue_num
        );

        // Fetch issue details
        let issue_details = Command::new("gh")
            .args([
                "issue",
                "view",
                &issue_num.to_string(),
                "--json",
                "title,body",
            ])
            .current_dir(repo_path)
            .output()?;

        if !issue_details.status.success() {
            println!("⚠️  Failed to fetch issue #{}: skipping", issue_num);
            continue;
        }

        let issue_json: serde_json::Value =
            serde_json::from_slice(&issue_details.stdout).unwrap_or_default();
        let title = issue_json["title"].as_str().unwrap_or("");
        let body = issue_json["body"].as_str().unwrap_or("");

        // Create agent task for the chore
        let task_description = format!(
            "Complete the chore/tech debt task described in issue #{}.

Issue: {}

Description:
{}

Requirements:
- Implement the improvement or refactoring
- Ensure code quality and maintainability
- Add or update tests as needed
- Run all tests to verify
- Commit with message: 'chore: <description> (closes #{})'
- Create a pull request

Please provide a complete solution.",
            issue_num, title, body, issue_num
        );

        println!("   Launching agent task...");
        let agent_result = Command::new("gh")
            .args(["agent-task", "create", &task_description])
            .current_dir(repo_path)
            .output()?;

        if agent_result.status.success() {
            println!("✅ Agent task spawned for chore #{}", issue_num);
        } else {
            println!("⚠️  Failed to spawn agent for chore #{}", issue_num);
            println!("{}", String::from_utf8_lossy(&agent_result.stderr));
        }
    }

    println!("\n✅ Chore workflow complete!");
    Ok(())
}

/// Coverage Workflow: Analyze coverage and create issues for GitHub agents
async fn run_coverage_workflow(repo_path: &Path, threshold: u8) -> Result<()> {
    println!("📊 Starting Coverage Workflow");
    println!("============================\n");

    // Run coverage analysis and create issues
    println!(
        "🔍 Running coverage analysis with threshold {}%...",
        threshold
    );
    let coverage_result = Command::new("./agents/coverage/target/release/coverage")
        .args([
            "--repo-path",
            repo_path.display().to_string().as_str(),
            "--threshold",
            &threshold.to_string(),
            "--create-issues",
        ])
        .output()?;

    println!("{}", String::from_utf8_lossy(&coverage_result.stdout));

    if !coverage_result.status.success() {
        println!("❌ Coverage analysis failed:");
        println!("{}", String::from_utf8_lossy(&coverage_result.stderr));
        return Ok(());
    }

    println!("\n✅ Coverage analysis complete");
    println!("   Run 'test-workflow' to spawn agents that generate tests");
    Ok(())
}

/// Helper to get coverage issues from GitHub (excluding those with linked PRs)
fn get_coverage_issues(repo_path: &Path) -> Result<Vec<u32>> {
    use std::process::Command;

    // Get all open testing issues
    let output = Command::new("gh")
        .args([
            "issue",
            "list",
            "--label",
            "testing",
            "--state",
            "open",
            "--json",
            "number",
            "--jq",
            ".[].number",
        ])
        .current_dir(repo_path)
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let all_issues: Vec<u32> = stdout
        .lines()
        .filter_map(|line| line.trim().parse().ok())
        .collect();

    // Get all open PRs to check which issues already have PRs
    let pr_output = Command::new("gh")
        .args([
            "pr",
            "list",
            "--state",
            "open",
            "--json",
            "number",
            "--jq",
            ".[].number",
        ])
        .current_dir(repo_path)
        .output()?;

    let pr_stdout = String::from_utf8_lossy(&pr_output.stdout);
    let pr_numbers: std::collections::HashSet<u32> = pr_stdout
        .lines()
        .filter_map(|line| line.trim().parse().ok())
        .collect();

    // Filter out issues that have matching PR numbers (assuming PR number == issue number)
    let issues_without_prs: Vec<u32> = all_issues
        .into_iter()
        .filter(|issue_num| !pr_numbers.contains(issue_num))
        .collect();

    Ok(issues_without_prs)
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
