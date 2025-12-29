//! chore-bot - GitHub Copilot agent orchestrator
//!
//! Spawns GitHub Copilot agents for automated workflows:
//! - test: Add tests for open testing issues
//! - feature: Implement features from issues
//! - bug: Fix bugs from issues
//! - chore: Complete chores/tech debt
//! - approve: Rerun pending workflow runs

mod subagent;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

/// Load a prompt template from the agent's directory
fn load_prompt(agent: &str) -> Result<String> {
    let prompt_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("agents")
        .join(agent)
        .join("prompt.md");
    Ok(std::fs::read_to_string(prompt_path)?)
}

/// Render a template by replacing {{key}} placeholders
fn render_template(template: &str, vars: &[(&str, &str)]) -> String {
    let mut result = template.to_string();
    for (key, value) in vars {
        result = result.replace(&format!("{{{{{}}}}}", key), value);
    }
    result
}

#[derive(Parser, Debug)]
#[command(name = "chore-bot")]
#[command(about = "Spawns GitHub Copilot agents for automated workflows")]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Spawn agents to add tests (one per module batch)
    Test {
        /// Repository path
        #[arg(short, long)]
        repo_path: PathBuf,

        /// Max PRs to create (one per module)
        #[arg(short, long, default_value = "5")]
        max_prs: u8,
    },

    /// Spawn agent to implement a feature
    Feature {
        /// Repository path
        #[arg(short, long)]
        repo_path: PathBuf,

        /// GitHub issue number
        #[arg(short, long)]
        issue: u32,
    },

    /// Spawn agents to fix bugs
    Bug {
        /// Repository path
        #[arg(short, long)]
        repo_path: PathBuf,

        /// Max bugs to fix
        #[arg(short, long, default_value = "3")]
        max_bugs: u8,
    },

    /// Spawn agents for chores/tech debt
    Chore {
        /// Repository path
        #[arg(short, long)]
        repo_path: PathBuf,

        /// Max chores to resolve
        #[arg(short, long, default_value = "5")]
        max_chores: u8,
    },

    /// Spawn agent with custom task description
    Custom {
        /// Repository path
        #[arg(short, long)]
        repo_path: PathBuf,

        /// Task description
        #[arg(short, long)]
        task: String,
    },

    /// Approve all pending workflow runs
    Approve {
        /// Repository path
        #[arg(short, long)]
        repo_path: PathBuf,
    },

    /// Analyze coverage and create issues for untested functions (uses agents/coverage)
    Coverage {
        /// Repository path
        #[arg(short, long)]
        repo_path: PathBuf,

        /// Coverage threshold (0-100)
        #[arg(short, long, default_value = "90")]
        threshold: u8,

        /// Create GitHub issues for untested functions
        #[arg(long)]
        create_issues: bool,
    },

    /// Scan for TODO/FIXME comments and create issues (uses agents/todo-scanner)
    Scan {
        /// Repository path
        #[arg(short, long)]
        repo_path: PathBuf,

        /// Create GitHub issues for TODOs without issue references
        #[arg(long)]
        create_issues: bool,

        /// Dry run - show what would be created
        #[arg(long)]
        dry_run: bool,
    },

    /// Batch create GitHub issues from JSON (uses agents/issue-creator)
    CreateIssues {
        /// Repository path
        #[arg(short, long)]
        repo_path: PathBuf,

        /// JSON file with issues to create
        #[arg(short, long)]
        batch: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Test { repo_path, max_prs } => run_test(&repo_path, max_prs)?,
        Commands::Feature { repo_path, issue } => run_feature(&repo_path, issue)?,
        Commands::Bug {
            repo_path,
            max_bugs,
        } => run_bug(&repo_path, max_bugs)?,
        Commands::Chore {
            repo_path,
            max_chores,
        } => run_chore(&repo_path, max_chores)?,
        Commands::Custom { repo_path, task } => run_custom(&repo_path, &task)?,
        Commands::Approve { repo_path } => run_approve(&repo_path)?,
        Commands::Coverage {
            repo_path,
            threshold,
            create_issues,
        } => run_coverage(&repo_path, threshold, create_issues)?,
        Commands::Scan {
            repo_path,
            create_issues,
            dry_run,
        } => run_scan(&repo_path, create_issues, dry_run)?,
        Commands::CreateIssues { repo_path, batch } => run_create_issues(&repo_path, &batch)?,
    }

    Ok(())
}

/// Spawn agents to handle testing issues (one per module batch)
fn run_test(repo_path: &Path, max_prs: u8) -> Result<()> {
    println!("🧪 Test Workflow (batched by module)\n");

    let all_issues = subagent::list_issues_by_label(repo_path, "testing")?;
    let open_prs = subagent::list_open_prs(repo_path)?;
    let issues: Vec<_> = all_issues
        .into_iter()
        .filter(|n| !open_prs.contains(n))
        .collect();

    if issues.is_empty() {
        println!("No testing issues found.");
        return Ok(());
    }

    println!("Found {} issues total", issues.len());

    let batches = subagent::group_by_module(repo_path, &issues)?;
    println!("Grouped into {} module batches:\n", batches.len());

    for (module, issues) in &batches {
        println!("  {}: {} issues", module, issues.len());
    }
    println!();

    let mut spawned = 0;
    for (module, issues) in batches.into_iter().take(max_prs as usize) {
        if issues.len() < 2 {
            println!("⏭️  Skipping {} (only {} issue)", module, issues.len());
            continue;
        }

        println!(
            "🤖 Spawning agent for {} ({} issues)...",
            module,
            issues.len()
        );

        let issue_list: String = issues
            .iter()
            .map(|(num, title)| format!("- #{}: {}\n", num, title))
            .collect();

        let closes: Vec<_> = issues
            .iter()
            .map(|(n, _)| format!("closes #{}", n))
            .collect();
        let closes_str = closes.join(", ");
        let module_snake = module.replace('-', "_");
        let count = issues.len().to_string();

        let template = load_prompt("test")?;
        let task = render_template(
            &template,
            &[
                ("module", &module),
                ("issue_list", &issue_list),
                ("module_snake", &module_snake),
                ("closes_str", &closes_str),
                ("count", &count),
            ],
        );

        let result = subagent::spawn_agent(repo_path, &task)?;
        if result.success {
            println!("   ✅ Spawned");
            spawned += 1;
        } else {
            println!("   ❌ Failed: {}", result.message);
        }
    }

    println!("\n✅ Spawned {} agents", spawned);
    println!("Monitor: gh agent-task list");

    Ok(())
}

fn run_feature(repo_path: &Path, issue: u32) -> Result<()> {
    println!("🚀 Feature Workflow\n");

    let (title, body) = match subagent::fetch_issue(repo_path, issue)? {
        Some(details) => details,
        None => {
            println!("Failed to fetch issue #{}", issue);
            return Ok(());
        }
    };

    let issue_str = issue.to_string();
    let template = load_prompt("feature")?;
    let task = render_template(
        &template,
        &[("issue", &issue_str), ("title", &title), ("body", &body)],
    );

    println!("Spawning agent for issue #{}...", issue);
    let result = subagent::spawn_agent(repo_path, &task)?;

    if result.success {
        println!("✅ Agent spawned");
    } else {
        println!("❌ Failed: {}", result.message);
    }

    Ok(())
}

fn run_bug(repo_path: &Path, max_bugs: u8) -> Result<()> {
    println!("🐛 Bug Workflow\n");

    let issues = subagent::list_issues_by_label(repo_path, "bug")?;

    if issues.is_empty() {
        println!("No bug issues found.");
        return Ok(());
    }

    println!("Found {} bugs\n", issues.len());

    for issue in issues.into_iter().take(max_bugs as usize) {
        let (title, body) = match subagent::fetch_issue(repo_path, issue)? {
            Some(details) => details,
            None => continue,
        };

        let issue_str = issue.to_string();
        let template = load_prompt("bug")?;
        let task = render_template(
            &template,
            &[("issue", &issue_str), ("title", &title), ("body", &body)],
        );

        println!("Spawning agent for bug #{}...", issue);
        let result = subagent::spawn_agent(repo_path, &task)?;

        if result.success {
            println!("✅ Spawned");
        } else {
            println!("❌ Failed: {}", result.message);
        }
    }

    Ok(())
}

fn run_chore(repo_path: &Path, max_chores: u8) -> Result<()> {
    println!("🧹 Chore Workflow\n");

    let issues = subagent::list_issues_by_label(repo_path, "chore")?;

    if issues.is_empty() {
        println!("No chore issues found.");
        return Ok(());
    }

    println!("Found {} chores\n", issues.len());

    for issue in issues.into_iter().take(max_chores as usize) {
        let (title, body) = match subagent::fetch_issue(repo_path, issue)? {
            Some(details) => details,
            None => continue,
        };

        let issue_str = issue.to_string();
        let template = load_prompt("chore")?;
        let task = render_template(
            &template,
            &[("issue", &issue_str), ("title", &title), ("body", &body)],
        );

        println!("Spawning agent for chore #{}...", issue);
        let result = subagent::spawn_agent(repo_path, &task)?;

        if result.success {
            println!("✅ Spawned");
        } else {
            println!("❌ Failed: {}", result.message);
        }
    }

    Ok(())
}

fn run_custom(repo_path: &Path, task: &str) -> Result<()> {
    println!("🎯 Custom Workflow\n");

    let result = subagent::spawn_agent(repo_path, task)?;

    if result.success {
        println!("✅ Agent spawned");
    } else {
        println!("❌ Failed: {}", result.message);
    }

    Ok(())
}

fn run_approve(repo_path: &Path) -> Result<()> {
    println!("✅ Approving Pending Workflows\n");

    let results = subagent::approve_pending_workflows(repo_path)?;

    if results.is_empty() {
        println!("No pending workflows to approve.");
        return Ok(());
    }

    for (run_id, success) in &results {
        if *success {
            println!("✅ Approved run {}", run_id);
        } else {
            println!("❌ Failed to approve run {}", run_id);
        }
    }

    let approved = results.iter().filter(|(_, s)| *s).count();
    println!("\n✅ Approved {}/{} workflows", approved, results.len());

    Ok(())
}

fn run_coverage(repo_path: &Path, threshold: u8, create_issues: bool) -> Result<()> {
    println!("📊 Coverage Workflow\n");

    let agent_bin =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("agents/coverage/target/release/coverage");

    if !agent_bin.exists() {
        println!("❌ Coverage agent not built. Run:");
        println!("   cd agents/coverage && cargo build --release");
        return Ok(());
    }

    let mut cmd = std::process::Command::new(&agent_bin);
    cmd.arg("--repo-path").arg(repo_path);
    cmd.arg("--threshold").arg(threshold.to_string());

    if create_issues {
        cmd.arg("--create-issues");
    }

    let status = cmd.status()?;

    if status.success() {
        println!("\n✅ Coverage analysis complete");
    } else {
        println!("\n❌ Coverage analysis failed");
    }

    Ok(())
}

fn run_scan(repo_path: &Path, create_issues: bool, dry_run: bool) -> Result<()> {
    println!("🔍 TODO Scanner\n");

    let agent_bin = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("agents/todo-scanner/target/release/todo-scanner");

    if !agent_bin.exists() {
        println!("❌ TODO scanner not built. Run:");
        println!("   cd agents/todo-scanner && cargo build --release");
        return Ok(());
    }

    let mut cmd = std::process::Command::new(&agent_bin);
    cmd.arg("--repo-path").arg(repo_path);

    if create_issues {
        cmd.arg("--create-issues");
    }

    if dry_run {
        cmd.arg("--dry-run");
    }

    let status = cmd.status()?;

    if status.success() {
        println!("\n✅ Scan complete");
    } else {
        println!("\n❌ Scan failed");
    }

    Ok(())
}

fn run_create_issues(repo_path: &Path, batch: &Path) -> Result<()> {
    println!("📝 Batch Issue Creator\n");

    let agent_bin = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("agents/issue-creator/target/release/issue-creator");

    if !agent_bin.exists() {
        println!("❌ Issue creator not built. Run:");
        println!("   cd agents/issue-creator && cargo build --release");
        return Ok(());
    }

    let mut cmd = std::process::Command::new(&agent_bin);
    cmd.current_dir(repo_path);
    cmd.arg("--batch").arg(batch);

    let status = cmd.status()?;

    if status.success() {
        println!("\n✅ Issues created");
    } else {
        println!("\n❌ Issue creation failed");
    }

    Ok(())
}
