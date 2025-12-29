# 🤖 Chore Bot - GitHub Copilot Agent Orchestrator

A Rust-based automation tool that spawns GitHub Copilot agents to handle development tasks. Automatically processes GitHub issues and creates pull requests for tests, features, bugs, and chores.

## ✨ Features

- 🧪 **Test Workflow** - Batch issues by module and spawn agents to add comprehensive tests
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

## 🔧 Customizing Prompts

Edit the markdown files in `prompts/` to customize agent instructions. Templates use `{{variable}}` syntax for dynamic values.

## 📋 Commands

| Command | Description |
|---------|-------------|
| `test` | Spawn agents to add tests for issues labeled `testing` |
| `feature` | Spawn agent to implement a specific feature issue |
| `bug` | Spawn agents to fix issues labeled `bug` |
| `chore` | Spawn agents for issues labeled `chore` |
| `approve` | Rerun all workflows with `action_required` status |
| `custom` | Spawn agent with custom task description |
   git clone https://github.com/your-username/chore-bot.git
   cd chore-bot
   ```

2. **Build the project:**
   ```bash
   cargo build --release
   ```

3. **Clone your target repository:**
   ```bash
   cd repos/
   git clone https://github.com/your-org/your-repo.git
   cd ..
   ```

## 🎯 Usage

### Coverage Workflow

Analyze test coverage and create GitHub issues for untested functions:

```bash
./target/release/orchestrator coverage-workflow \
  --repo-path ./repos/your-repo \
  --create-issues
```

This will:
1. Run `cargo llvm-cov --cobertura` on the target repo
2. Parse coverage data to identify functions below 90% threshold
3. Create GitHub issues with 'testing' label for each untested function

### Test Workflow

Process testing issues and create PRs with generated tests:

```bash
./target/release/orchestrator test-workflow \
  --repo-path ./repos/your-repo \
  --max-todos 5
```

This will:
1. Fetch open issues with 'testing' label that don't have PRs
2. For each issue:
   - Generate appropriate tests based on function type
   - Run tests to verify they pass
   - Create a branch and commit the test file
   - Push and open a pull request

### Direct Todo-Resolver Usage

Process a specific issue manually:

```bash
./agents/todo-resolver/target/release/todo-resolver \
  --repo-path ./repos/your-repo \
  --issue 123 \
  --create-pr
```

## 🧪 Test Generation

The todo-resolver intelligently generates tests based on function signatures:

### PartialEq Implementations

For types implementing `PartialEq`, generates tests that:
- Verify identical instances are equal
- Test each field independently to ensure all are checked
- Create instances with actual field values (not placeholders)

Example generated test:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literalnumber_eq_identical() {
        let val1 = LiteralNumber { 
            literal_expression: LiteralExpression::default(), 
            literal: 2.0 
        };
        let val2 = LiteralNumber { 
            literal_expression: LiteralExpression::default(), 
            literal: 2.0 
        };
        assert_eq!(val1, val2, "Identical instances should be equal");
    }

    #[test]
    fn test_literalnumber_ne_diff_literal() {
        let val1 = LiteralNumber { 
            literal_expression: LiteralExpression::default(), 
            literal: 2.0 
        };
        let val2 = LiteralNumber { 
            literal_expression: LiteralExpression::default(), 
            literal: 3.0 
        };
        assert_ne!(val1, val2, "Instances with different literal should not be equal");
    }
}
```

### Constructor Functions

Generates tests for `new()` and similar constructors with various inputs.

### Clone Implementations

Generates tests verifying cloned instances are equal and independent.

### Generic Functions

Generates tests covering happy path, edge cases, and error conditions.

## 🔧 Configuration

### Pre-commit Hook Optimization

The system automatically configures target repositories to run fast pre-commit checks:

```bash
# In target repo: .git/hooks/pre-commit
# Only runs formatting and linting, not full test suite
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
```

This ensures commits are fast while tests run in CI.

### Coverage Threshold

Coverage issues are created for functions with <90% coverage. This can be adjusted in `agents/coverage/src/main.rs`.

## 🛠️ Development

### Project Structure

```
chore-bot/
├── src/
│   ├── main.rs           # Entry point
│   ├── orchestrator.rs   # Workflow coordination
│   └── subagent.rs       # Agent invocation helpers
├── agents/
│   ├── coverage/
│   │   ├── src/
│   │   │   ├── main.rs     # Coverage analysis CLI
│   │   │   ├── analyzer.rs # llvm-cov runner & XML parser
│   │   │   └── config.rs   # Configuration
│   │   └── Cargo.toml
│   └── todo-resolver/
│       ├── src/
│       │   ├── main.rs     # Test generation CLI
│       │   └── resolver.rs # Test generation & PR creation
│       └── Cargo.toml
├── repos/                  # Target repositories
│   └── your-repo/
└── Cargo.toml
```

### Building Agents
### Building Agents

Each agent can be built independently:

```bash
# Build coverage agent
cd agents/coverage
cargo build --release

# Build todo-resolver agent
cd agents/todo-resolver
cargo build --release

# Build orchestrator
cd ../..
cargo build --release --bin orchestrator
```

### Running Tests

```bash
cargo test
```

## 📊 Coverage Statistics

Example output from syster LSP project:
- **Total Lines**: 15,474
- **Covered Lines**: 13,750 (88.9%)
- **Uncovered**: 1,724 lines
- **Issues Created**: 377 functions needing tests
- **PRs Generated**: Automated with real, passing tests

## 🔄 Workflow Examples

### Complete Automation

Run both workflows in sequence to go from 0% to high coverage:

```bash
# Step 1: Identify gaps
./target/release/orchestrator coverage-workflow \
  --repo-path ./repos/syster \
  --create-issues

# Step 2: Generate tests (processes all untested functions)
./target/release/orchestrator test-workflow \
  --repo-path ./repos/syster \
  --max-todos 100
```

### Continuous Integration

Process a few issues at a time to spread work across multiple CI runs:

```bash
# Process 5 issues per run
./target/release/orchestrator test-workflow \
  --repo-path ./repos/syster \
  --max-todos 5
```

The orchestrator automatically:
- Skips issues that already have PRs
- Only processes open issues with 'testing' label
- Creates separate branches for each issue
- Links PRs back to issues

## 🎨 Customization

### Adding New Test Generators

To support new function types, edit `agents/todo-resolver/src/resolver.rs`:

```rust
pub fn generate_tests(repo_path: &Path, todo: &TodoItem) -> Result<String> {
    let test_content = if function_name.contains("PartialEq") {
        generate_partialeq_tests(&source_content, function_name)?
    } else if function_name.contains("::new") {
        generate_constructor_tests(&source_content, function_name)?
    } else if function_name.contains("YourTrait") {
        generate_your_trait_tests(&source_content, function_name)?
    } else {
        generate_generic_tests(&source_content, function_name)?
    };
    
    Ok(test_content)
}
```

### Adjusting Coverage Threshold

Edit `agents/coverage/src/main.rs` to change the threshold:

```rust
// Current: functions with <90% coverage
if coverage_rate < 0.90 {
    // Create issue
}
```

## 🤝 Contributing

Contributions welcome! Areas for improvement:

- Support for more test patterns (async functions, error handling, etc.)
- Integration with other coverage tools
- Support for non-Rust languages
- Enhanced test assertions and edge case generation
- Batch PR creation optimizations

## 📝 License

[MIT License](LICENSE)

## 🙋 FAQ

**Q: How accurate are the generated tests?**
A: The tests are syntactically correct and test the actual function implementation. They focus on verifying behavior through assertions, not just compilation.

**Q: Can I review tests before they're merged?**
A: Yes! All tests are submitted as PRs that go through your normal review process.

**Q: What happens if generated tests fail?**
A: The todo-resolver runs tests and won't create a PR if they fail. This ensures only working tests are submitted.

**Q: Does this work with private repositories?**
A: Yes, requires GitHub CLI (`gh`) to be authenticated with appropriate permissions.

**Q: How long does it take to process issues?**
A: Depends on project size. Most issues complete in 2-3 minutes including test generation, execution, and PR creation.

## 🔗 Resources

- [cargo-llvm-cov Documentation](https://github.com/taiki-e/cargo-llvm-cov)
- [GitHub CLI Documentation](https://cli.github.com/)
- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)

---

Made with ❤️ using Rust - Automated testing for the win! 🚀
