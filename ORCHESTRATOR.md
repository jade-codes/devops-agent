# Orchestrator Agent

Coordinates multiple specialized agents to execute complex workflows.

## Architecture

```
┌─────────────────┐
│  Orchestrator   │  ← Main coordinator
└────────┬────────┘
         │
         ├─→ Coverage Agent       (Find missing tests)
         ├─→ TODO Scanner         (Find TODO items)
         ├─→ TODO Resolver        (Implement fixes)
         ├─→ Feature Implementer  (Build features)
         └─→ Architecture Reviewer (Analyze design)
```

## Installation

```bash
cd /workspaces/chore-bot
cargo build --release
```

## Usage

### Test Workflow

Finds missing tests and implements them automatically:

```bash
orchestrator test-workflow \
  --repo-path /path/to/repo \
  --threshold 80 \
  --max-todos 10
```

**Steps:**
1. Run coverage analysis
2. Create GitHub issues for untested code
3. Resolve up to N issues using TODO resolver
4. Create PRs for each fix

### Feature Workflow

Reviews architecture, then implements a feature:

```bash
orchestrator feature-workflow \
  --repo-path /path/to/repo \
  --issue 42
```

**Steps:**
1. Architecture review (identify patterns, smells)
2. Scan for related TODOs
3. Implement feature from issue
4. Create PR

### Quality Workflow

Comprehensive quality analysis:

```bash
orchestrator quality-workflow \
  --repo-path /path/to/repo
```

**Steps:**
1. Architecture review
2. Coverage analysis
3. TODO scan
4. Generate combined report

### Custom Workflow

Run specific agents in sequence:

```bash
orchestrator custom \
  --agents "todo-scanner,coverage,architecture-reviewer" \
  --repo-path /path/to/repo
```

## Workflows

### 🧪 Test Workflow
**Goal:** Improve test coverage  
**Agents:** coverage → todo-resolver  
**Output:** PRs with new tests

### 🚀 Feature Workflow
**Goal:** Implement new features  
**Agents:** architecture-reviewer → todo-scanner → feature-implementer  
**Output:** Feature implementation PR

### 🔍 Quality Workflow
**Goal:** Quality assessment  
**Agents:** architecture-reviewer → coverage → todo-scanner  
**Output:** Quality report

## Subagent API

Each agent exposes a standard CLI interface:

```bash
# Coverage Agent
agents/coverage/target/release/coverage \
  --repo-path /path \
  --threshold 80 \
  --create-issues

# TODO Resolver
agents/todo-resolver/target/release/todo-resolver \
  --repo-path /path \
  --issue 42 \
  --create-pr

# Architecture Reviewer
agents/architecture-reviewer/target/release/architecture-reviewer \
  --repo-path /path
```

## Adding New Workflows

1. Create workflow function in `orchestrator.rs`:

```rust
async fn run_my_workflow(repo_path: &PathBuf) -> Result<()> {
    // Step 1: Call first agent
    let result1 = subagent::run_coverage_agent(repo_path, 80, true).await?;
    
    // Step 2: Call second agent
    let result2 = subagent::run_todo_scanner(repo_path, true).await?;
    
    // Step 3: Process results
    println!("Workflow complete!");
    Ok(())
}
```

2. Add to CLI commands:

```rust
#[derive(Subcommand)]
enum Commands {
    MyWorkflow {
        #[arg(short, long)]
        repo_path: PathBuf,
    },
}
```

## Adding New Agents

1. Create agent in `agents/my-agent/`
2. Add wrapper function in `src/subagent.rs`:

```rust
pub async fn run_my_agent(
    repo_path: &PathBuf,
    custom_arg: String,
) -> Result<AgentResponse> {
    run_subagent(AgentRequest {
        agent: "my-agent".to_string(),
        args: vec![
            "--repo-path".to_string(),
            repo_path.display().to_string(),
            "--custom".to_string(),
            custom_arg,
        ],
        working_dir: None,
    })
    .await
}
```

3. Use in workflows:

```rust
let result = subagent::run_my_agent(&repo_path, "value".to_string()).await?;
```

## Benefits

✅ **Composable** - Mix and match agents for different workflows  
✅ **Maintainable** - Each agent is independent and testable  
✅ **Scalable** - Easy to add new agents and workflows  
✅ **Reusable** - Agents can be used standalone or orchestrated  
✅ **Debuggable** - Clear separation of concerns

## Examples

### Nightly Test Coverage Improvement

```bash
# Run every night via cron
orchestrator test-workflow \
  --repo-path /workspaces/syster \
  --threshold 85 \
  --max-todos 20
```

### Pre-Release Quality Gate

```bash
# Before release, run quality checks
orchestrator quality-workflow \
  --repo-path /workspaces/syster
```

### Feature Sprint

```bash
# Implement feature from backlog
for issue in 42 43 44 45; do
  orchestrator feature-workflow \
    --repo-path /workspaces/syster \
    --issue $issue
done
```

## CI/CD Integration

```yaml
# .github/workflows/nightly-improvements.yml
name: Nightly Improvements

on:
  schedule:
    - cron: '0 2 * * *'

jobs:
  improve:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run test workflow
        run: |
          orchestrator test-workflow \
            --repo-path . \
            --max-todos 5
```

## Monitoring

Each workflow produces structured output:

```json
{
  "workflow": "test-workflow",
  "started_at": "2025-12-28T02:00:00Z",
  "steps": [
    {
      "agent": "coverage",
      "success": true,
      "duration_ms": 45000,
      "issues_created": 12
    },
    {
      "agent": "todo-resolver",
      "success": true,
      "issues_resolved": 5,
      "prs_created": 5
    }
  ],
  "completed_at": "2025-12-28T02:15:00Z"
}
```
