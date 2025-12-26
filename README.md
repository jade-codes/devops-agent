# 🤖 DevOps Agent

A Rust-based code review agent that scans your repository and runs automated checklists using Claude AI. Perfect for maintaining code quality, security standards, and best practices in your GitHub repositories.

## ✨ Features

- 🔍 **Automated Code Analysis** - Scans code files and checks against customizable rules
- 🤖 **Claude AI Integration** - Uses Claude to perform intelligent code review
- 📋 **Flexible Checklists** - YAML-based configuration for custom rules and patterns
- 🎯 **PR-Focused** - Can analyze only changed files in pull requests
- 📊 **Multiple Output Formats** - Console, Markdown, and JSON reports
- 💬 **GitHub Integration** - Automatically posts analysis results as PR comments
- ⚡ **Fast & Efficient** - Built in Rust for maximum performance

## 🚀 Quick Start

### Prerequisites

- Rust 1.70 or higher
- An Anthropic API key ([get one here](https://console.anthropic.com/))
- GitHub repository (for GitHub Actions integration)

### Local Installation

1. **Clone the repository:**
   ```bash
   git clone https://github.com/your-username/devops-agent.git
   cd devops-agent
   ```

2. **Build the project:**
   ```bash
   cargo build --release
   ```

3. **Set your Anthropic API key:**
   ```bash
   export ANTHROPIC_API_KEY="your-api-key-here"
   ```

4. **Run the agent:**
   ```bash
   ./target/release/devops-agent --repo-path /path/to/your/project
   ```

## 📋 Configuration

### Checklist Configuration

Create or modify `checklist.yaml` to define your code review rules:

```yaml
name: "My Code Quality Checklist"
description: "Custom rules for my project"

file_patterns:
  - "**/*.rs"
  - "**/*.py"
  - "**/*.js"

exclude_patterns:
  - "**/target/**"
  - "**/node_modules/**"

items:
  - category: "Security"
    rule: "No hardcoded secrets"
    description: "Check for API keys, passwords, or tokens in code"
    severity: "error"

  - category: "Testing"
    rule: "Public functions have tests"
    description: "All public functions should have unit tests"
    severity: "warning"

  - category: "Build & CI"
    rule: "Make run-guidelines passes"
    description: "If the project has a Makefile with 'run-guidelines' target, it must pass"
    severity: "error"
```

**Severity Levels:**
- `error` - Critical issues that should block merging
- `warning` - Important issues that should be addressed
- `info` - Suggestions and best practices

### Project-Level Checks

The agent automatically detects and runs project-level checks:

**`make run-guidelines`**: If your target project has a Makefile with a `run-guidelines` target, the agent will:
- Automatically detect it
- Run `make run-guidelines` in the target repository
- Report failures as errors with full output

Example `run-guidelines` target for your project:
```makefile
.PHONY: run-guidelines

run-guidelines:
	@echo "Running project checks..."
	@cargo fmt --check
	@cargo clippy -- -D warnings
	@cargo test
	@./scripts/custom-checks.sh
	@echo "✅ All checks passed!"
```

## 🎯 Usage

### Basic Usage

```bash
# Analyze current directory
devops-agent

# Analyze specific repository
devops-agent --repo-path /path/to/repo

# Use custom checklist
devops-agent --checklist my-rules.yaml

# Output as markdown
devops-agent --output markdown

# Output as JSON
devops-agent --output json
```

### PR Mode (GitHub Actions)

```bash
# Only analyze changed files in PR
devops-agent --pr-only

# Post results as PR comment
devops-agent --pr-only --post-comment
```

## 🔧 GitHub Actions Setup

### Step 1: Add Secrets

In your GitHub repository, add the following secrets:

1. Go to **Settings** → **Secrets and variables** → **Actions**
2. Add `ANTHROPIC_API_KEY` with your Anthropic API key

### Step 2: Copy Workflow File

The workflow file is already included at `.github/workflows/devops-agent.yml`. It will:

- ✅ Trigger on pull requests to main/master/develop
- ✅ Build and run devops-agent
- ✅ Analyze only changed files
- ✅ Post results as PR comments automatically

### Step 3: Customize (Optional)

Edit `.github/workflows/devops-agent.yml` to:

- Change trigger branches
- Modify build options
- Adjust when it runs

## 📊 Output Examples

### Console Output

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  🤖 DevOps Agent ANALYSIS REPORT
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📊 Files Analyzed: 5
✅ Passed Checks:  12
❌ Errors:         2
⚠️  Warnings:       3
ℹ️  Info:           1

📄 src/main.rs
─────────────────────────────────────────────────
❌ [Security] No hardcoded secrets
   Line: 42
   Found what appears to be an API key on line 42
```

### Markdown Report

The agent generates nicely formatted markdown reports perfect for GitHub PR comments with:
- Summary statistics
- Categorized findings
- Line numbers for issues
- Severity indicators

## 🛠️ Development

### Project Structure

```
devops-agent/
├── src/
│   ├── main.rs         # Entry point and CLI
│   ├── config.rs       # Configuration loading
│   ├── scanner.rs      # Repository file scanning
│   ├── analyzer.rs     # Claude AI integration
│   ├── reporter.rs     # Report generation
│   └── github.rs       # GitHub API integration
├── checklist.yaml      # Default checklist
├── Cargo.toml          # Rust dependencies
└── .github/
    └── workflows/
        └── devops-agent.yml  # GitHub Action workflow
```

### Running Tests

```bash
cargo test
```

### Building for Release

```bash
cargo build --release
```

The binary will be at `target/release/devops-agent`.

## 🎨 Customization Ideas

- Add language-specific rules (e.g., Rust-specific patterns)
- Create multiple checklist profiles (security-focused, documentation-focused, etc.)
- Integrate with other CI/CD platforms (GitLab CI, Jenkins)
- Add automatic issue creation for critical findings
- Generate HTML reports
- Track metrics over time

## 🤝 Contributing

Contributions are welcome! Feel free to:

- Add new checklist rules
- Improve error handling
- Add tests
- Enhance reporting formats
- Fix bugs

## 📝 License

[MIT License](LICENSE) - feel free to use this in your projects!

## 🙋 FAQ

**Q: How much does it cost to run?**
A: Depends on your Anthropic API usage. Each file analysis uses Claude API tokens based on file size and checklist complexity.

**Q: Can I use this locally without GitHub Actions?**
A: Yes! Just run the binary directly with your ANTHROPIC_API_KEY set.

**Q: Does it support other languages besides Rust?**
A: Yes! The checklist can be configured for any text-based files (Python, JavaScript, Go, etc.).

**Q: Can I run this on private repositories?**
A: Yes, works with both public and private repos when using GitHub Actions.

**Q: How do I add custom rules?**
A: Edit `checklist.yaml` and add new items under the `items` section.

## 🔗 Resources

- [Anthropic Claude API Documentation](https://docs.anthropic.com/)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Rust Programming Language](https://www.rust-lang.org/)

---

Made with ❤️ using Rust and Claude AI
