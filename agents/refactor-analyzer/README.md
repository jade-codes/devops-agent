# Refactor Analyzer Agent

Identifies code that needs refactoring based on complexity metrics and best practices.

## Features

- **Cyclomatic Complexity**: Measures decision points in code
- **Nesting Depth**: Detects deeply nested structures
- **Function Length**: Identifies overly long functions
- **Parameter Count**: Flags functions with too many parameters
- **Priority Scoring**: Ranks candidates by refactoring urgency
- **Multi-Language**: Supports Rust, Python, JavaScript, TypeScript
- **GitHub Integration**: Creates issues with detailed metrics

## Metrics

### Cyclomatic Complexity (1-10)
Counts decision points: `if`, `else`, `for`, `while`, `match`, `&&`, `||`
- **1-3**: Simple, easy to test
- **4-6**: Moderate complexity
- **7-10**: High complexity, needs refactoring

### Nesting Depth
Maximum number of nested blocks
- **1-2**: Flat, readable
- **3-4**: Moderate nesting
- **5+**: Too deep, hard to understand

### Lines of Code
- **< 20**: Appropriate size
- **20-50**: Getting large
- **> 50**: Too long, should be split

### Parameter Count
- **0-3**: Good
- **4-5**: Acceptable
- **> 5**: Too many, consider parameter object

## Usage

```bash
# Analyze current directory
refactor-analyzer

# Set complexity threshold
refactor-analyzer --threshold 7

# Analyze specific path
refactor-analyzer --path src/

# Output as JSON
refactor-analyzer --format json

# Output as Markdown
refactor-analyzer --format markdown

# Create GitHub issues
refactor-analyzer --create-issues --repo owner/repo

# Complete workflow
refactor-analyzer --path ./src --threshold 6 --create-issues --repo myorg/myproject
```

## Priority Scoring

Priority = (Complexity × 0.4) + (Size × 0.3) + (Nesting × 0.2) + (Params × 0.1)

- **7.0+**: 🔴 High priority - Refactor immediately
- **4.0-6.9**: 🟡 Medium priority - Plan refactoring
- **< 4.0**: 🟢 Low priority - Monitor

## Output Examples

### Console
```
🔴 process_user_data (src/api.rs:145)
   Complexity: 9/10 | Priority: 7.8 | Lines: 85 | Nesting: 6 | Params: 7
   ⚠️  High cyclomatic complexity: 9
   ⚠️  Function too long: 85 lines
   ⚠️  Deep nesting: 6 levels
   ⚠️  Too many parameters: 7
```

### Markdown Report
```markdown
## process_user_data (src/api.rs)
**Lines:** 145-230
**Complexity:** 9/10
**Priority Score:** 7.80

**Metrics:**
- Lines of code: 85
- Nesting depth: 6
- Parameters: 7

**Issues:**
- High cyclomatic complexity: 9
- Function too long: 85 lines
- Deep nesting: 6 levels
```

## Integration

```bash
# Analyze codebase
refactor-analyzer --create-issues --repo owner/repo --format json > refactorings.json

# Pick high-priority item
todo-resolver --issue 42

# Verify improvement
refactor-analyzer --path src/fixed_module.rs
```

## Testing

```bash
cargo test
```

Comprehensive tests for:
- Complexity calculation
- Nesting depth detection
- Parameter counting
- Priority scoring
- Function boundary detection
