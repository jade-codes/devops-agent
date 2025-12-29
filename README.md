# 🤖 Chore Bot

A Rust CLI that spawns GitHub Copilot agents to automate development tasks. Processes GitHub issues and creates pull requests for tests, features, bugs, and chores.

## ✨ Features

- 🧪 **Test Workflow** - Batch issues by module and spawn agents to add tests
- 🚀 **Feature Workflow** - Spawn agents to implement features from issues
- 🐛 **Bug Workflow** - Spawn agents to fix bugs with regression tests
- 🧹 **Chore Workflow** - Spawn agents for tech debt and refactoring
- ✅ **Workflow Approval** - Automatically rerun pending CI workflows
- 📝 **Customizable Prompts** - Markdown templates for agent instructions
- 🎯 **Smart Batching** - Groups related issues to minimize merge conflicts

## 🚀 Quick Start

### Prerequisites

- Rust 1.70+
- GitHub CLI (`gh`) authenticated
- GitHub Copilot with agent-task access

### Installation

```bash
cargo build --release
```

### Usage

```bash
# Spawn agents to add tests (batched by module)
./target/release/chore-bot test --repo-path /path/to/repo --max-prs 5

# Spawn agent for a feature
./target/release/chore-bot feature --repo-path /path/to/repo --issue 123

# Spawn agents to fix bugs
./target/release/chore-bot bug --repo-path /path/to/repo --max-bugs 3

# Spawn agents for chores
./target/release/chore-bot chore --repo-path /path/to/repo --max-chores 5

# Approve pending workflow runs
./target/release/chore-bot approve --repo-path /path/to/repo

# Custom task
./target/release/chore-bot custom --repo-path /path/to/repo --task "Your task description"
```

## 📁 Project Structure

```
src/
├── main.rs      # CLI and workflow logic
└── subagent.rs  # GitHub API helpers

prompts/
├── test.md      # Test workflow prompt template
├── feature.md   # Feature workflow prompt template
├── bug.md       # Bug workflow prompt template
└── chore.md     # Chore workflow prompt template
```

## 📋 Commands

| Command | Description |
|---------|-------------|
| `test` | Spawn agents to add tests for issues labeled `testing` |
| `feature` | Spawn agent to implement a specific feature issue |
| `bug` | Spawn agents to fix issues labeled `bug` |
| `chore` | Spawn agents for issues labeled `chore` |
| `approve` | Rerun all workflows with `action_required` status |
| `custom` | Spawn agent with custom task description |

## 🔧 Customizing Prompts

Edit the markdown files in `prompts/` to customize agent instructions. Templates use `{{variable}}` syntax:

- `{{issue_numbers}}` - Comma-separated issue numbers
- `{{issue_titles}}` - Issue titles for context
- `{{task}}` - Custom task description

## 🎯 How It Works

1. **Fetches issues** from GitHub with the appropriate label
2. **Groups by module** based on file paths in issue titles
3. **Spawns Copilot agents** using `gh agent-task create`
4. **Agents create PRs** with the requested changes
5. **Approve command** reruns any pending workflow approvals

## 🛠️ Development

```bash
# Build
cargo build --release

# Run tests
cargo test

# Format code
cargo fmt

# Check lints
cargo clippy
```

## 📝 License

MIT
