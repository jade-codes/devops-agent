# Note Scanner Agent

Scans code for important inline notes, observations, and documentation that needs attention.

## Features

- **12 Note Types**: NOTE, IMPORTANT, WARNING, CAUTION, PERF, PERFORMANCE, OPTIMIZE, REFACTOR, DEPRECATED, REVIEW, QUESTION, CONSIDER
- **Severity Levels**: Automatic severity assignment based on keywords and note type
- **Categories**: Groups notes by purpose (documentation, safety, performance, technical-debt, code-quality, clarification, enhancement)
- **Multiple Output Formats**: Console (with emoji), JSON, Markdown
- **GitHub Integration**: Automatically create issues with proper labels
- **Smart Filtering**: Filter by minimum severity level

## Usage

```bash
# Scan current directory
note-scanner

# Scan specific path
note-scanner --path /path/to/code

# Filter by severity
note-scanner --severity medium

# Output as JSON
note-scanner --format json

# Output as Markdown
note-scanner --format markdown

# Create GitHub issues
note-scanner --create-issues --repo owner/repo

# Combined example
note-scanner --path ./src --severity high --create-issues --repo myorg/myproject
```

## Note Types and Severity

### High Severity (Automatic)
- `WARNING` - Safety concerns
- `CAUTION` - Potential issues
- `DEPRECATED` - Code marked for removal
- Any note containing: "critical", "security", "unsafe", "panic", "crash"

### Medium Severity
- `IMPORTANT` - Significant observations
- `REVIEW` - Needs code review
- `REFACTOR` - Technical debt

### Low Severity
- `NOTE` - General observations
- `PERF` / `PERFORMANCE` / `OPTIMIZE` - Performance notes
- `QUESTION` - Clarifications needed
- `CONSIDER` - Enhancement ideas

## Output Examples

### Console Output
```
🔴 WARNING [safety] src/core.rs:45
   Unsafe pointer dereference without null check
🟡 REFACTOR [technical-debt] src/utils.rs:123
   This function is too complex, should be split
🟢 PERF [performance] src/processor.rs:89
   Could use binary search instead of linear
```

### GitHub Issue
**Title:** `WARNING: Unsafe pointer dereference without null check`

**Labels:** `priority: high`, `safety`

**Body:**
```
**File:** src/core.rs:45
**Type:** WARNING
**Category:** safety
**Severity:** High

Unsafe pointer dereference without null check
```

## Integration with Other Bots

```bash
# Scan for notes, create issues, then resolve them
note-scanner --create-issues --repo owner/repo
todo-resolver --issue 42
```

## Supported Languages

- Rust (.rs)
- Python (.py)
- JavaScript/TypeScript (.js, .ts)
- Go (.go)
- Java (.java)
- C/C++ (.c, .cpp, .h, .hpp)
- Ruby (.rb)
- PHP (.php)

## Testing

```bash
cargo test
```

All tests pass with comprehensive coverage of:
- Severity parsing and determination
- Filtering logic
- File type detection
- Note scanning patterns
- String truncation
