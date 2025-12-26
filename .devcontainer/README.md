# Dev Container for DevOps Agent

This dev container includes everything you need to develop and run the DevOps Agent.

## Included

✅ **Rust toolchain** (latest stable)
✅ **Cargo** (Rust package manager)
✅ **Git** (version control)
✅ **GitHub CLI** (for authentication and PR management)
✅ **VS Code extensions**:
- Rust Analyzer (LSP for Rust)
- Better TOML (for Cargo.toml)
- Crates (dependency management)
- GitHub Copilot + Chat

## Getting Started

### 1. Open in Dev Container

In VS Code:
- Open Command Palette (Ctrl+Shift+P / Cmd+Shift+P)
- Select: `Dev Containers: Reopen in Container`
- Wait for container to build (~2-3 minutes first time)

### 2. Set Environment Variables

Create a `.env` file or set in terminal:

```bash
export GITHUB_TOKEN="your-github-token-here"
```

Get token from: https://github.com/settings/tokens
Required permissions: `repo`, `workflow`

### 3. Build and Run

The project is automatically built on container creation. To rebuild:

```bash
# Build CLI version
cargo build --release

# Build MCP server
cargo build --release --bin devops-agent-mcp

# Run tests
cargo test

# Format code
cargo fmt

# Check with clippy
cargo clippy
```

### 4. Configure MCP for Copilot

The MCP server is already built. To use it with VS Code Copilot:

1. Add to your user settings (Ctrl+, → "mcp"):
```json
{
  "mcpServers": {
    "devops-agent": {
      "command": "/workspaces/devops-agent/target/release/devops-agent-mcp",
      "env": {
        "GITHUB_TOKEN": "${env:GITHUB_TOKEN}"
      }
    }
  }
}
```

2. Reload VS Code
3. Start using devops-agent with Copilot!

## Usage Examples

### CLI Mode
```bash
# Analyze current directory
./target/release/devops-agent

# Analyze specific repo
./target/release/devops-agent --repo-path /path/to/repo

# Output as markdown
./target/release/devops-agent --output markdown
```

### MCP Mode (with Copilot)
```
You: "Use devops-agent to scan examples/ and find issues"
Copilot: [scans and reports issues]

You: "Fix all security issues and create PRs"
Copilot: [fixes and creates PRs automatically]
```

## SSH Key Mounting

Your local `~/.ssh` directory is mounted (read-only) so you can:
- Push to GitHub with SSH
- Clone private repositories
- Use existing SSH keys

## Troubleshooting

**Container won't start:**
- Check Docker is running
- Try: `Dev Containers: Rebuild Container`

**GitHub token issues:**
- Verify token in environment: `echo $GITHUB_TOKEN`
- Check token permissions at GitHub settings

**Cargo build fails:**
- Clean and rebuild: `cargo clean && cargo build`
- Update dependencies: `cargo update`

**MCP server not connecting:**
- Check binary exists: `ls -la target/release/devops-agent-mcp`
- Make executable: `chmod +x target/release/devops-agent-mcp`
- Check VS Code output panel for errors

## Development Tips

- **Hot reload**: Use `cargo watch -x run` for auto-rebuild on changes
- **Debug mode**: Build without `--release` for faster compilation + debug symbols
- **Test specific module**: `cargo test scanner`
- **Check coverage**: Install `cargo-tarpaulin` for test coverage reports

## Next Steps

1. Set your `GITHUB_TOKEN` environment variable
2. Try scanning the examples: `./target/release/devops-agent --repo-path examples/`
3. Configure MCP and use with Copilot
4. Start automating your code reviews! 🚀
