# Feature Implementer Agent

Implements new features following strict Test-Driven Development (TDD) methodology.

## Features

- **Requirement Analysis**: Parses feature descriptions or GitHub issues
- **Feature Type Detection**: Identifies if it's a function, module, API, UI, data model, or integration
- **Complexity Assessment**: Determines if feature is simple, moderate, or complex
- **Test Generation**: Creates test cases from acceptance criteria
- **TDD Workflow**: RED → GREEN → REFACTOR
- **PR Creation**: Automatically creates pull request with comprehensive description

## Usage

```bash
# Implement from description
feature-implementer --feature "Add user authentication with email validation"

# Implement from GitHub issue
feature-implementer --feature "#42"

# Generate tests only
feature-implementer --feature "Add search function" --test-first

# Full workflow with PR
feature-implementer --feature "New API endpoint" --create-pr --target-branch develop

# Specify repository
feature-implementer --feature "Add logging" --repo /path/to/project
```

## TDD Workflow

### Step 1: Analyze Requirements
```
📋 Analyzing feature requirements...
   Type: ApiEndpoint
   Complexity: Moderate
   Files affected: 3
```

### Step 2: Generate Tests
```
✅ Generating test cases...
   Generated 3 test cases
```

### Step 3: RED Phase
```
🔴 Verifying tests fail...
   ✓ Tests fail as expected
```

### Step 4: Implement Feature
```
💻 Implementing feature...
   Implemented in 3 files
```

### Step 5: GREEN Phase
```
🟢 Verifying tests pass...
   ✓ All tests pass
```

### Step 6: Create PR (Optional)
```
🚀 Creating pull request...
   PR: https://github.com/owner/repo/pull/123
```

## Feature Types

- **NewFunction**: Adding individual functions or methods
- **NewModule**: Creating new modules or packages
- **ApiEndpoint**: REST/GraphQL endpoints
- **UiComponent**: Frontend components
- **DataModel**: Database schemas or data structures
- **Integration**: Third-party integrations
- **Enhancement**: General improvements

## Complexity Levels

- **Simple**: < 20 words, single file, straightforward
- **Moderate**: 20-50 words, multiple files, some dependencies
- **Complex**: > 50 words, architecture changes, system-wide impact

## Acceptance Criteria

The agent automatically extracts acceptance criteria from:
- Bullet points (- or *)
- Numbered lists (1., 2., 3.)
- Feature requirements sections

Example:
```
Feature: Add user search
- Must search by name and email
- Should return results in < 100ms
- Must handle pagination
```

## Integration

```bash
# Create issue for feature request
issue-creator create --title "Add search" --body "Requirements..."

# Implement the feature
feature-implementer --feature "#123" --create-pr

# Run coverage analysis
coverage --repo . --threshold 80
```

## Testing

```bash
cargo test
```

Comprehensive test coverage for:
- Feature type detection
- Complexity assessment
- Acceptance criteria extraction
- Test code generation
- Name sanitization
