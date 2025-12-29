//! Orchestrator - Spawns GitHub Copilot agents for various workflows
//!
//! Uses `gh agent-task create` to spawn agents that work on issues.

mod subagent;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(name = "orchestrator")]
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
    }

    Ok(())
}

/// Spawn agents to handle testing issues (one per module batch)
fn run_test(repo_path: &Path, max_prs: u8) -> Result<()> {
    println!("🧪 Test Workflow (batched by module)\n");

    // Get testing issues without existing PRs
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

    // Group by module
    let batches = subagent::group_by_module(repo_path, &issues)?;
    println!("Grouped into {} module batches:\n", batches.len());

    for (module, issues) in &batches {
        println!("  {}: {} issues", module, issues.len());
    }
    println!();

    // Spawn one agent per module batch
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

        let task = format!(
            r#"Add comprehensive tests for the **{module}** module.

## Issues to Resolve
{issue_list}

## CRITICAL REQUIREMENTS

### 1. TEST THROUGH PUBLIC API ONLY
- Do NOT make private methods public just for testing
- Test internal logic through public entrypoint functions
- If a function is private, test it via the public function that calls it

### 2. VERIFY LOGIC CORRECTNESS
- READ and UNDERSTAND the implementation before writing tests
- Check if the logic makes sense and is correct
- If you find bugs, note them but still test current behavior

### 3. ONE TEST FILE FOR THIS BATCH
- Create ONE test file: `{module_snake}_test.rs`
- All tests for this batch go in that single file
- Add module declaration to mod.rs: `#[cfg(test)] mod {module_snake}_test;`

### 4. QUALITY TESTS ONLY
- NO TODO comments or placeholder tests
- Test edge cases: empty inputs, error conditions, boundaries
- Descriptive test names explaining what's tested

### 5. CLOSE ALL ISSUES IN ONE COMMIT
Commit message: `test: Add comprehensive tests for {module} ({closes_str})`

### 6. VERIFY BEFORE PR
- `make run-guidelines` must pass
- Then create PR

Create a single PR resolving all {count} issues."#,
            module = module,
            issue_list = issue_list,
            module_snake = module_snake,
            closes_str = closes_str,
            count = issues.len()
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

/// Spawn agent to implement a feature
fn run_feature(repo_path: &Path, issue: u32) -> Result<()> {
    println!("🚀 Feature Workflow\n");

    let (title, body) = match subagent::fetch_issue(repo_path, issue)? {
        Some(details) => details,
        None => {
            println!("Failed to fetch issue #{}", issue);
            return Ok(());
        }
    };

    let task = format!(
        r#"Implement the feature from issue #{issue}.

**{title}**

{body}

## Requirements
1. Review the codebase architecture
2. Implement the feature
3. Add tests
4. Commit: `feat: {title} (closes #{issue})`
5. Create PR"#,
        issue = issue,
        title = title,
        body = body
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

/// Spawn agents to fix bugs
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

        let task = format!(
            r#"Fix bug #{issue}.

**{title}**

{body}

## Requirements
1. Analyze root cause
2. Implement fix
3. Add regression test
4. Commit: `fix: {title} (closes #{issue})`
5. Create PR"#,
            issue = issue,
            title = title,
            body = body
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

/// Spawn agents for chores
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

        let task = format!(
            r#"Complete chore #{issue}.

**{title}**

{body}

## Requirements
1. Implement the improvement
2. Update tests if needed
3. Commit: `chore: {title} (closes #{issue})`
4. Create PR"#,
            issue = issue,
            title = title,
            body = body
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

/// Spawn agent with custom task
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

/// Approve all pending workflow runs
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
