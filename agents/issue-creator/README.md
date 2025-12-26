# Issue Creator Agent

Creates GitHub issues from structured input (CLI args, JSON, or batch files).

## Installation

```bash
cd issue-creator
cargo build --release
```

## Usage

### Create single issue from CLI
```bash
issue-creator \
  --title "Fix memory leak" \
  --body "Memory leak in function X" \
  --labels "bug,priority-high" \
  --assignees "username"
```

### Create from JSON file
```bash
issue-creator --from-json issue.json
```

**issue.json:**
```json
{
  "title": "Add authentication",
  "body": "Implement OAuth2 authentication",
  "labels": ["enhancement", "security"],
  "priority": "high"
}
```

### Batch create from JSON array
```bash
issue-creator --batch issues.json
```

**issues.json:**
```json
[
  {
    "title": "Fix bug #1",
    "body": "Description",
    "labels": ["bug"]
  },
  {
    "title": "Add feature #1",
    "body": "Description",
    "labels": ["enhancement"],
    "priority": "medium"
  }
]
```

### Dry run
```bash
issue-creator --batch issues.json --dry-run
```

### Save issue URLs
```bash
issue-creator --batch issues.json --output created-issues.txt
```

## Features

- ✅ Create single or batch issues
- ✅ Load from JSON files
- ✅ Automatic priority labels (low/medium/high/critical)
- ✅ Input validation (title length, required fields)
- ✅ Dry run mode
- ✅ Save created issue URLs
- ✅ Runs standalone with TDD tests

## JSON Schema

```json
{
  "title": "string (required, max 256 chars)",
  "body": "string (optional)",
  "labels": ["string"],
  "assignees": ["string"],
  "milestone": "string",
  "priority": "low|medium|high|critical"
}
```

## Priority Labels

The agent automatically adds priority labels:
- `priority: "low"` → adds `priority-low` label
- `priority: "medium"` → adds `priority-medium` label
- `priority: "high"` → adds `priority-high` label
- `priority: "critical"` → adds `priority-critical` label

## Integration with Other Bots

Other bots can use this agent by:

1. **Generate JSON:**
```rust
let issue = IssueRequest {
    title: "TODO: Fix this".to_string(),
    body: Some("Found in file.rs:42".to_string()),
    labels: Some(vec!["todo".to_string()]),
    priority: Some("medium".to_string()),
};
serde_json::to_writer(file, &issue)?;
```

2. **Call issue-creator:**
```bash
issue-creator --from-json /tmp/issue.json
```

## Examples

### Create from coverage agent output
```bash
# Coverage agent generates issues.json
coverage --repo-path . --output json > coverage-issues.json

# Create issues
issue-creator --batch coverage-issues.json
```

### Pipeline multiple bots
```bash
# Scan TODOs → generate JSON
todo-scanner --repo-path . --output json > todos.json

# Transform to issue format (using jq)
cat todos.json | jq '[.[] | {title: .title, body: .content, labels: ["todo"]}]' > issues.json

# Create issues
issue-creator --batch issues.json
```

## Running Tests

```bash
cargo test
```

## CLI Help

```bash
issue-creator --help
```
