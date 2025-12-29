# Dev Container for Chore Bot

This dev container includes everything you need to develop and run Chore Bot.

## Included

✅ **Rust toolchain** (latest stable)  
✅ **Cargo** (Rust package manager)  
✅ **Git** (version control)  
✅ **GitHub CLI** (for authentication and agent spawning)  
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

### 2. Authenticate GitHub CLI

```bash
gh auth login
```

Follow the prompts to authenticate. Required for spawning Copilot agents.

### 3. Build and Run

The project is automatically built on container creation. To rebuild:

```bash
cargo build --release
```

## Usage

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
./target/release/chore-bot custom --repo-path /path/to/repo --task "Your task"
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

**GitHub CLI not authenticated:**
- Run: `gh auth login`
- Check status: `gh auth status`

**Cargo build fails:**
- Clean and rebuild: `cargo clean && cargo build`
- Update dependencies: `cargo update`

## Development Tips

- **Hot reload**: Use `cargo watch -x run` for auto-rebuild on changes
- **Debug mode**: Build without `--release` for faster compilation + debug symbols
- **Check lints**: `cargo clippy`
- **Format code**: `cargo fmt`
