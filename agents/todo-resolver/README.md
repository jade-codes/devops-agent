# TODO Resolver Agent

Automatically resolves TODO items by implementing fixes following Test-Driven Development (TDD).

## Installation

```bash
cd todo-resolver
cargo build --release
```

## Usage

### Resolve TODO from GitHub issue
```bash
todo-resolver --issue 42 --create-pr
```

### Resolve specific TODO by location
```bash
todo-resolver --todo "src/lib.rs:42" --create-pr
```

### Auto-select and resolve simplest TODO
```bash
todo-resolver --auto --create-pr
```

### Dry run (analyze without implementing)
```bash
todo-resolver --issue 42 --dry-run
```

### Skip test generation (only implement)
```bash
todo-resolver --issue 42 --skip-tests
```

## TDD Workflow

The agent follows strict TDD principles:

1. **Write Tests First** 📝
   - Generates failing tests based on TODO requirements
   - Tests follow AAA pattern (Arrange, Act, Assert)

2. **Verify Tests Fail** 🔴
   - Runs tests to ensure they fail initially
   - Confirms tests are valid

3. **Implement Fix** 🔨
   - Writes minimal code to make tests pass
   - Follows best practices and style guidelines

4. **Verify Tests Pass** ✅
   - Runs tests again
   - Ensures implementation is correct

5. **Commit Changes** 📦
   - Creates feature branch
   - Commits with descriptive message
   - References issue if applicable

6. **Create PR** 🚀
   - Opens pull request
   - Includes implementation details
   - Links to original issue

## Features

- ✅ Loads TODOs from GitHub issues
- ✅ Scans repository for TODOs
- ✅ Analyzes complexity and approach
- ✅ Generates appropriate tests
- ✅ Implements fixes following TDD
- ✅ Creates branches and commits
- ✅ Opens pull requests
- ✅ Runs standalone with tests

## Analysis Output

```bash
todo-resolver --todo "src/lib.rs:42" --dry-run
```

```
📊 Analysis:
   Type: implementation
   Complexity: medium
   Suggested approach: Implement the feature with proper error handling
   Estimated lines: 30
   Requires tests: true
```

## TODO Types Supported

- **Testing** - Add missing tests
- **Implementation** - Implement new features
- **Refactoring** - Refactor existing code
- **Bugfix** - Fix bugs with tests first
- **Feature** - Add new functionality
- **General** - Other improvements

## Complexity Levels

- **Low** (~10 lines) - Simple changes
- **Medium** (~30 lines) - Standard features
- **High** (~100 lines) - Complex refactoring

## Integration with Issue Tracker

Load TODOs directly from GitHub issues created by `todo-scanner`:

```bash
# todo-scanner creates issues
todo-scanner --repo-path . --create-issues

# Get issue number from GitHub
gh issue list

# Resolve the TODO
todo-resolver --issue 42 --create-pr
```

## Branch Naming

Branches are auto-generated:
- Format: `todo-resolver/[sanitized-todo-content]`
- Example: `todo-resolver/fix-memory-leak`

## Commit Messages

```
fix: Add error handling for parse failures (closes #42)
```

## Running Tests

```bash
cargo test
```

## Example Session

```bash
$ todo-resolver --issue 15 --create-pr

🔧 TODO Resolver Agent starting...
📂 Repository: "."
📋 Loading TODO from issue #15

📝 Selected TODO:
   File: src/parser.rs:42
   Content: Add error handling for invalid input

✅ Step 1: Writing tests...
   Created: src/parser_test.rs

🔴 Step 2: Running tests (should fail)...
   Tests failed (expected for TDD)

🔨 Step 3: Implementing fix...
   Modified 1 file(s)

✅ Step 4: Running tests (should pass)...
   Tests passed ✓

📦 Step 5: Committing changes...
   Branch: todo-resolver/add-error-handling-for-invalid-input

🚀 Step 6: Creating pull request...
   PR: https://github.com/user/repo/pull/123

✅ TODO resolved successfully!
```

## Limitations

- Currently supports Rust projects only
- Requires manual review of generated code
- Complex TODOs may need human assistance
- Test generation is template-based

## Safety

- ✅ Dry run mode for analysis
- ✅ Creates new branches (never modifies main)
- ✅ All changes are in PRs for review
- ✅ TDD ensures tests validate changes
