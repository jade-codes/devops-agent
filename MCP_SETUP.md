# 🤖 DevOps Agent - MCP Mode Setup

## Quick Start with VS Code Copilot

### 1. Build the MCP Server

```bash
cd /path/to/devops-agent
cargo build --release
```

This creates two binaries:
- `target/release/devops-agent` - CLI mode (GitHub Actions)
- `target/release/devops-agent-mcp` - MCP server (VS Code integration)

### 2. Configure VS Code

Add to your VS Code settings (`.vscode/mcp-settings.json` or global settings):

```json
{
  "mcpServers": {
    "devops-agent": {
      "command": "/path/to/devops-agent/target/release/devops-agent-mcp",
      "env": {
        "GITHUB_TOKEN": "ghp_your_token_here"
      }
    }
  }
}
```

### 3. Get GitHub Token

Create a token at: https://github.com/settings/tokens

Required permissions:
- `repo` (full control)
- `workflow` (if you want to trigger actions)

### 4. Restart VS Code

The MCP server will auto-connect to GitHub Copilot.

## Usage with Copilot

### Workflow 1: Scan and Review Code

```
You: "Use devops-agent to scan the examples/ directory and review against our checklist"

Copilot: [calls scan_repository tool]
         [analyzes code with Claude]
         [provides detailed report]
```

### Workflow 2: Fix Issue with PR

```
You: "Fix the hardcoded API key issue in examples/bad_code.rs, 
      create a branch, commit, and open a PR"

Copilot: [scans code]
         [identifies issue]
         [applies fix to file]
         [calls complete_workflow tool]
         
Result: ✅ PR created at github.com/your-repo/pull/123
```

### Workflow 3: Batch Fix Multiple Issues

```
You: "Scan the repo, find all issues, and for each one:
      1. Create a fix branch
      2. Apply the fix
      3. Create a PR
      4. Move to next issue"

Copilot: [iterates through all findings]
         [creates separate PRs for each]
```

## Available MCP Tools

### `scan_repository`
Scans a repository and returns files to analyze.

**Parameters:**
- `repo_path` (required): Path to repository
- `checklist_path` (optional): Path to checklist.yaml

**Example:**
```
"Scan /path/to/your-project with devops-agent"
```

### `check_guidelines`
Runs `make run-guidelines` if it exists.

**Parameters:**
- `repo_path` (required): Path to repository

**Example:**
```
"Check if the project passes run-guidelines"
```

### `create_fix_branch`
Creates a new branch for fixing an issue.

**Parameters:**
- `repo_path` (required): Path to repository
- `issue_id` (required): Issue identifier

**Example:**
```
"Create a branch to fix issue security-001"
```

### `commit_and_push`
Commits all changes and pushes to remote.

**Parameters:**
- `repo_path` (required): Path to repository
- `message` (required): Commit message
- `branch_name` (required): Branch to push

**Example:**
```
"Commit the changes with message 'Fix hardcoded secrets' and push"
```

### `create_pull_request`
Creates a GitHub PR.

**Parameters:**
- `repo_path` (required): Path to repository
- `branch_name` (required): Branch with changes
- `title` (required): PR title
- `body` (required): PR description

**Example:**
```
"Create a PR for the security fixes"
```

### `complete_workflow`
Full automation: branch → commit → push → PR.

**Parameters:**
- `repo_path` (required): Path to repository
- `issue_id` (required): Issue identifier
- `commit_message` (required): Commit message
- `pr_title` (required): PR title
- `pr_body` (required): PR description

**Example:**
```
"Complete the workflow for fixing test coverage issues"
```

## Example Conversations

### Example 1: Quick Review

```
You: "Review the code in /path/to/your-project using devops-agent"

Copilot: Let me scan that repository...
         [Uses scan_repository tool]
         
         Found 5 files with issues:
         
         ❌ src/auth.rs
            - Hardcoded API key on line 42
            - Missing error handling on line 67
         
         ❌ src/db.rs
            - SQL injection vulnerability on line 23
            - No tests for public functions
         
         Would you like me to create PRs to fix these?
```

### Example 2: Auto-Fix with PR

```
You: "Fix all the security issues in my project and create PRs"

Copilot: I'll fix each issue in a separate PR...
         
         [For each security issue:]
         1. Creating branch devops-agent/fix-security-001
         2. Applying fix to remove hardcoded secrets
         3. Committing changes
         4. Creating PR
         
         ✅ Created PR #45: Fix hardcoded API keys
         ✅ Created PR #46: Fix SQL injection in db.rs
         ✅ Created PR #47: Add input validation
         
         All security issues fixed! 3 PRs ready for review.
```

### Example 3: Check Guidelines

```
You: "Does my project pass the run-guidelines check?"

Copilot: [Uses check_guidelines tool]
         
         🔧 Running make run-guidelines...
         
         ❌ Failed with exit code 1
         
         Issues found:
         - cargo fmt --check failed (3 files need formatting)
         - cargo clippy found 2 warnings
         - 1 test failed
         
         Would you like me to fix these and create a PR?
```

## Tips

1. **Be specific**: "Fix the security issue in line 42 of auth.rs"
2. **Batch operations**: "Fix all documentation issues and create one PR"
3. **Iterative**: "Scan, fix first issue, then move to next"
4. **Check first**: "Check guidelines before making changes"

## Troubleshooting

**Server not connecting:**
- Check binary path in mcp-settings.json
- Ensure it's executable: `chmod +x target/release/devops-agent-mcp`
- Check VS Code output panel for errors

**GitHub token issues:**
- Verify token has `repo` permissions
- Check it's not expired
- Ensure it's properly set in environment

**Tool execution fails:**
- Check `repo_path` is absolute
- Ensure you're in a git repository
- Verify GITHUB_TOKEN is set

## Advanced: Custom Checklist

Create project-specific rules in `checklist.yaml`:

```yaml
name: "My Project Rules"
file_patterns:
  - "**/*.rs"
  
items:
  - category: "Architecture"
    rule: "No direct database calls in handlers"
    severity: "error"
    
  - category: "Security"
    rule: "All auth endpoints use rate limiting"
    severity: "error"
```

Then use it:
```
"Scan my project with the custom checklist and fix all violations"
```

---

Ready to automate your code reviews! 🚀
