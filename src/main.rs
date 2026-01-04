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
        result = result.replace(&format!("{{{{{key}}}}}"), value);
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
    /// Spawn agents to add tests
    Test {
        /// Repository path
        #[arg(short, long)]
        repo_path: PathBuf,

        /// Max agents to spawn
        #[arg(short, long, default_value = "5")]
        max_prs: u8,

        /// Batch by fixed size instead of module (issues per agent)
        #[arg(long)]
        batch_size: Option<u8>,
    },

    /// Spawn agents to implement features
    Feature {
        /// Repository path
        #[arg(short, long)]
        repo_path: PathBuf,

        /// Max agents to spawn
        #[arg(short, long, default_value = "3")]
        max_prs: u8,
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

    /// Comment on PRs with failing pipelines to request fixes
    Nudge {
        /// Repository path
        #[arg(short, long)]
        repo_path: PathBuf,
    },

    /// Handle PRs with merge conflicts (comment to rebase or close)
    Conflicts {
        /// Repository path
        #[arg(short, long)]
        repo_path: PathBuf,

        /// Close conflicting PRs instead of commenting
        #[arg(long)]
        close: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Test {
            repo_path,
            max_prs,
            batch_size,
        } => run_test(&repo_path, max_prs, batch_size)?,
        Commands::Feature { repo_path, max_prs } => run_feature(&repo_path, max_prs)?,
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
        Commands::Nudge { repo_path } => run_nudge(&repo_path)?,
        Commands::Conflicts { repo_path, close } => run_conflicts(&repo_path, close)?,
    }

    Ok(())
}

/// Spawn agents to handle testing issues
fn run_test(repo_path: &Path, max_prs: u8, batch_size: Option<u8>) -> Result<()> {
    println!("🧪 Test Workflow\n");

    let all_issues = subagent::list_issues_by_label(repo_path, "testing")?;
    let issues_with_prs = subagent::list_issues_with_open_prs(repo_path)?;
    let issues: Vec<_> = all_issues
        .into_iter()
        .filter(|n| !issues_with_prs.contains(n))
        .collect();

    if issues.is_empty() {
        println!("No testing issues without open PRs found.");
        return Ok(());
    }

    println!("Found {} issues without PRs", issues.len());

    // Build batches based on mode
    let batches: Vec<(String, Vec<(u32, String)>)> = if let Some(size) = batch_size {
        // Batch by fixed size
        let mut issues_with_titles: Vec<(u32, String)> = Vec::new();
        for &issue_num in &issues {
            if let Some(title) = subagent::fetch_issue_title(repo_path, issue_num)? {
                issues_with_titles.push((issue_num, title));
            }
        }
        issues_with_titles
            .chunks(size as usize)
            .enumerate()
            .map(|(i, chunk)| (format!("batch-{}", i + 1), chunk.to_vec()))
            .collect()
    } else {
        // Batch by module (default)
        subagent::group_by_module(repo_path, &issues)?
    };

    println!("Grouped into {} batches\n", batches.len());
    for (name, issues) in &batches {
        println!("  {}: {} issues", name, issues.len());
    }
    println!();

    let mut spawned = 0;
    for (batch_name, batch) in batches.into_iter().take(max_prs as usize) {
        println!(
            "🤖 Spawning agent for {} ({} issues)...",
            batch_name,
            batch.len()
        );

        let issue_list: String = batch
            .iter()
            .map(|(num, title)| format!("- #{num}: {title}\n"))
            .collect();

        let closes: Vec<_> = batch.iter().map(|(n, _)| format!("closes #{n}")).collect();
        let closes_str = closes.join(", ");
        let module_snake = batch_name.replace('-', "_");
        let count = batch.len().to_string();

        let template = load_prompt("test")?;
        let task = render_template(
            &template,
            &[
                ("module", &batch_name),
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

    println!("\n✅ Spawned {spawned} agents");
    println!("Monitor: gh agent-task list");

    Ok(())
}

fn run_feature(repo_path: &Path, max_prs: u8) -> Result<()> {
    println!("🚀 Feature Workflow\n");

    let all_issues = subagent::list_issues_by_label(repo_path, "enhancement")?;
    let issues_with_prs = subagent::list_issues_with_open_prs(repo_path)?;
    let issues: Vec<_> = all_issues
        .into_iter()
        .filter(|n| !issues_with_prs.contains(n))
        .collect();

    if issues.is_empty() {
        println!("No enhancement issues without open PRs found.");
        return Ok(());
    }

    println!("Found {} issues without PRs\n", issues.len());

    let mut spawned = 0;
    for issue in issues.into_iter().take(max_prs as usize) {
        let (title, body) = match subagent::fetch_issue(repo_path, issue)? {
            Some(details) => details,
            None => continue,
        };

        let issue_str = issue.to_string();
        let template = load_prompt("feature")?;
        let task = render_template(
            &template,
            &[("issue", &issue_str), ("title", &title), ("body", &body)],
        );

        println!("🤖 Spawning agent for #{issue}: {title}...");
        let result = subagent::spawn_agent(repo_path, &task)?;

        if result.success {
            println!("   ✅ Spawned");
            spawned += 1;
        } else {
            println!("   ❌ Failed: {}", result.message);
        }
    }

    println!("\n✅ Spawned {spawned} agents");
    println!("Monitor: gh agent-task list");

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

        println!("Spawning agent for bug #{issue}...");
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

        println!("Spawning agent for chore #{issue}...");
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
            println!("✅ Approved run {run_id}");
        } else {
            println!("❌ Failed to approve run {run_id}");
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

fn run_nudge(repo_path: &Path) -> Result<()> {
    println!("💬 Nudge PRs with Failing Pipelines\n");

    let failing_prs = subagent::list_failing_prs(repo_path)?;

    if failing_prs.is_empty() {
        println!("✅ No PRs with failing pipelines found!");
        return Ok(());
    }

    println!("Found {} PRs with failing checks:\n", failing_prs.len());

    let comment = r#"@copilot This PR has failing CI checks.

Please take a look at the build failures and push a fix. Common issues:
- Compilation errors
- Test failures  
- Linting/formatting issues

Run `make run-guidelines` locally to verify before pushing."#;

    let mut commented = 0;
    for pr in &failing_prs {
        println!("  #{}: {} (by @{})", pr.number, pr.title, pr.author);

        if subagent::comment_on_pr(repo_path, pr.number, comment)? {
            println!("     ✅ Commented");
            commented += 1;
        } else {
            println!("     ❌ Failed to comment");
        }
    }

    println!("\n✅ Commented on {}/{} PRs", commented, failing_prs.len());

    Ok(())
}

fn run_conflicts(repo_path: &Path, close: bool) -> Result<()> {
    println!("🔀 Handle PRs with Merge Conflicts\n");

    let conflicting_prs = subagent::list_conflicting_prs(repo_path)?;

    if conflicting_prs.is_empty() {
        println!("✅ No PRs with merge conflicts found!");
        return Ok(());
    }

    println!(
        "Found {} PRs with merge conflicts:\n",
        conflicting_prs.len()
    );

    let mut handled = 0;

    if close {
        // Close conflicting PRs
        for pr in &conflicting_prs {
            println!("  #{}: {} (by @{})", pr.number, pr.title, pr.author);

            if subagent::close_pr(repo_path, pr.number)? {
                println!("     ✅ Closed");
                handled += 1;
            } else {
                println!("     ❌ Failed to close");
            }
        }
        println!("\n✅ Closed {}/{} PRs", handled, conflicting_prs.len());
    } else {
        // Comment asking to rebase
        let comment = r#"@copilot This PR has merge conflicts.

Please rebase on main and resolve the conflicts, then push again."#;

        for pr in &conflicting_prs {
            println!("  #{}: {} (by @{})", pr.number, pr.title, pr.author);

            if subagent::comment_on_pr(repo_path, pr.number, comment)? {
                println!("     ✅ Commented");
                handled += 1;
            } else {
                println!("     ❌ Failed to comment");
            }
        }
        println!(
            "\n✅ Commented on {}/{} PRs",
            handled,
            conflicting_prs.len()
        );
    }

    Ok(())
}
