# TODO Scanner Agent

Automatically scans code for TODO/FIXME/NOTE comments and creates GitHub issues for items without issue references.

## Installation

```bash
cd todo-scanner
cargo build --release
```

## Usage

### Scan repository (console output)
```bash
todo-scanner --repo-path ../syster
```

### Scan and create GitHub issues
```bash
todo-scanner --repo-path ../syster --create-issues
```

### Dry run (see what would be created)
```bash
todo-scanner --repo-path ../syster --create-issues --dry-run
```

### JSON output
```bash
todo-scanner --repo-path ../syster --output json > todos.json
```

### Markdown report
```bash
todo-scanner --repo-path ../syster --output markdown > TODOS.md
```

## Checklist Rule Enforced

**Code Quality: No TODO/FIXME without issue reference**
- Severity: warning
- Description: TODO and FIXME comments should reference an issue number or have a plan

## Features

- ✅ Scans TODO, FIXME, NOTE, HACK, XXX comments
- ✅ Detects existing issue references: `(#123)`, `(issue #123)`, `(gh-123)`
- ✅ Skips comments that already have issue references
- ✅ Creates GitHub issues with proper labels (bug/enhancement/documentation)
- ✅ Runs standalone - no dependencies on other bots
- ✅ Test coverage for core functionality

## Examples

### Input Code
```rust
// TODO: Add error handling here
fn process() {}

// TODO (#45): Optimize this function
fn optimize() {}

// FIXME: Memory leak in this section
fn leak() {}

// NOTE: This is just informational
fn info() {}
```

### Output
```
📋 Found 4 TODO/FIXME/NOTE comments

test.rs:1 [TODO] Add error handling here
test.rs:4 [TODO] Optimize this function (ref: (#45))
test.rs:7 [FIXME] Memory leak in this section
test.rs:10 [NOTE] This is just informational

🚀 Creating GitHub issues...
  ✓ Created: https://github.com/user/repo/issues/46 (TODO: Add error handling)
  ✓ Created: https://github.com/user/repo/issues/47 (FIXME: Memory leak)
✅ Created 2 issues
```

## Running Tests

```bash
cargo test
```

## Integration with CI/CD

Add to `.github/workflows/todo-scanner.yml`:
```yaml
name: TODO Scanner
on: [push, pull_request]
jobs:
  scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Scan TODOs
        run: |
          cargo run --manifest-path todo-scanner/Cargo.toml -- \
            --repo-path . \
            --create-issues \
            --dry-run
```
