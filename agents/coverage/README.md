# Coverage Agent

Analyzes code coverage and creates GitHub issues for untested functions and methods.

## Installation

```bash
# Install cargo-tarpaulin
cargo install cargo-tarpaulin

# Build coverage
cd coverage
cargo build --release
```

## Usage

### Run coverage analysis
```bash
coverage --repo-path ../syster
```

### Set custom threshold
```bash
coverage --repo-path ../syster --threshold 90
```

### Create GitHub issues for uncovered code
```bash
coverage --repo-path ../syster --create-issues
```

### Use existing coverage data
```bash
# First generate coverage with cargo tarpaulin
cd ../syster
cargo tarpaulin --out Xml

# Then analyze
cd ../coverage
coverage --repo-path ../syster --use-existing
```

### Generate reports

**Markdown:**
```bash
coverage --repo-path ../syster --output markdown > COVERAGE.md
```

**JSON:**
```bash
coverage --repo-path ../syster --output json > coverage.json
```

## Checklist Rules Enforced

**Testing: Public functions have tests**
- Severity: warning
- Description: All public functions and methods should have corresponding unit tests

**Testing: No untested exported APIs**
- Severity: error
- Description: All exported/public APIs must have at least basic test coverage

**Testing: Test coverage for complex logic**
- Severity: error
- Description: Complex functions with multiple branches should have comprehensive test coverage

## Features

- ✅ Runs cargo tarpaulin automatically
- ✅ Parses cobertura.xml coverage reports
- ✅ Identifies uncovered functions by type (public, private, test)
- ✅ Creates prioritized GitHub issues (error/warning/info)
- ✅ Customizable coverage thresholds
- ✅ Multiple output formats (console, markdown, JSON)
- ✅ Runs standalone with TDD tests

## Output Example

```
📊 Coverage Agent starting...
📂 Repository: "../syster"
🎯 Coverage threshold: 80%
🔬 Running cargo tarpaulin...
✅ Coverage analysis complete
📈 Overall coverage: 73.5%
📋 Found 12 uncovered items below threshold

📊 Coverage Report
==================
Overall: 73.5% (threshold: 80.0%)
❌ Below threshold

📋 Uncovered Items (12):
  🔴 src/analyzer.rs:42 - analyze_files (0.0% coverage)
  🔴 src/scanner.rs:18 - scan_repository (15.2% coverage)
  🟡 src/reporter.rs:64 - generate_json_report (45.0% coverage)
  ...
```

## Priority Levels

- 🔴 **Error** (Public functions) - No test coverage for exported APIs
- 🟡 **Warning** (Private functions) - Low coverage for internal functions  
- 🔵 **Info** (Test functions) - Tests themselves need coverage

## Running Tests

```bash
cargo test
```

## CI/CD Integration

Add to `.github/workflows/coverage.yml`:
```yaml
name: Coverage Check
on: [push, pull_request]
jobs:
  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install tarpaulin
        run: cargo install cargo-tarpaulin
      - name: Check coverage
        run: |
          cargo run --manifest-path coverage/Cargo.toml -- \
            --repo-path . \
            --threshold 80 \
            --create-issues
```
