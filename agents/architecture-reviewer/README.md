# Architecture Reviewer Agent

Reviews codebase architecture, detects patterns and anti-patterns, identifies architectural issues, and suggests improvements.

## Features

- **Pattern Detection**: Identifies Singleton, Factory, Repository, Service, Controller, Model patterns
- **Circular Dependencies**: Detects module dependency cycles
- **God Objects**: Finds files with too many responsibilities
- **Tight Coupling**: Identifies modules with excessive dependencies
- **Architectural Layers**: Verifies separation of concerns
- **Metrics**: Calculates module count, lines of code, dependency graphs
- **Multi-Language**: Supports Rust, Python, JavaScript, TypeScript

## Usage

```bash
# Analyze current directory
architecture-reviewer

# Analyze specific path
architecture-reviewer --path /path/to/project

# Filter by severity
architecture-reviewer --severity high

# Output as JSON
architecture-reviewer --format json

# Output as Markdown report
architecture-reviewer --format markdown

# Create GitHub issues
architecture-reviewer --create-issues --repo owner/repo

# Complete workflow
architecture-reviewer --path ./src --create-issues --repo myorg/myproject --format markdown
```

## Detected Issues

### Circular Dependencies (High)
Modules that depend on each other creating a cycle. Breaking these improves testability and maintainability.

### God Objects (High)
Files with > 500 lines and > 20 functions. These violate Single Responsibility Principle.

### Tight Coupling (Medium)
Modules depending on > 15 other modules. Indicates poor abstraction and difficult testing.

### Missing Tests (Medium)
No dedicated test directory structure. Tests should be organized separately.

### Unclear Layers (Low)
No separation between models, controllers, services. Clear layering improves maintainability.

## Detected Patterns

- **Singleton**: Static instances, lazy initialization
- **Factory**: `create()` or `new_()` methods
- **Repository**: Data access abstraction
- **Service**: Business logic layer
- **Controller**: Request handlers
- **Model**: Data structures

## Output Examples

### Console
```
📈 Architecture Metrics:
   Modules: 25
   Lines of code: 3420
   Avg lines per module: 136

✨ Design Patterns Detected:
   Singleton in src/config.rs (80% confidence)
   Repository in src/data/user_repo.rs (70% confidence)

⚠️  Architectural Issues:

🔴 Circular dependency detected [architecture]
   Circular dependency between src/api.rs and src/handlers.rs
   💡 Break the cycle by introducing an interface

🟡 Tight coupling detected [coupling]
   src/main.rs depends on 18 modules
   💡 Reduce dependencies using interfaces
```

### Markdown Report
```markdown
# Architecture Review Report

## Overview
- **Modules:** 25
- **Total Lines:** 3420
- **Patterns Detected:** 2
- **Issues Found:** 2

## Design Patterns
- **Singleton** in src/config.rs (confidence: 80%)
- **Repository** in src/data/user_repo.rs (confidence: 70%)

## Issues

### Circular dependency detected [High]
**Description:** Circular dependency between src/api.rs and src/handlers.rs
**Locations:**
- src/api.rs
- src/handlers.rs
**Suggestion:** Break the cycle by introducing an interface
```

## Integration

```bash
# Review architecture
architecture-reviewer --create-issues --repo owner/repo

# Identify refactoring needs
refactor-analyzer --threshold 7

# Implement fixes
todo-resolver --issue 42

# Verify improvements
architecture-reviewer --severity high
```

## Severity Levels

- **High**: Critical architectural flaws (circular dependencies, god objects)
- **Medium**: Design smells (tight coupling, missing tests)
- **Low**: Suggestions (layering improvements, organization)

## Testing

```bash
cargo test
```

Tests cover:
- Severity parsing
- Module name extraction
- Pattern detection
- Dependency extraction
- Path exclusion logic
