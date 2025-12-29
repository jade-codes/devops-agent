Complete chore #{{issue}}.

**{{title}}**

{{body}}

## CRITICAL REQUIREMENTS

### 1. UNDERSTAND THE SCOPE
- Read the chore description carefully
- Identify all files/areas that need changes
- Plan the refactoring before starting

### 2. MAINTAIN EXISTING BEHAVIOR
- Chores should NOT change functionality
- If refactoring, ensure behavior is preserved
- Be careful with public API changes

### 3. IMPROVE CODE QUALITY
- Follow existing patterns and conventions
- Remove dead code if applicable
- Add/update documentation as needed
- Improve type safety where possible

### 4. UPDATE TESTS IF NEEDED
- If refactoring changes test structure, update tests
- Ensure all existing tests still pass
- Add tests for previously untested code if applicable

### 5. COMMIT WITH PROPER MESSAGE
Commit message: `chore: {{title}} (closes #{{issue}})`

### 6. VERIFY BEFORE PR
- `make run-guidelines` must pass
- All tests must pass
- Then create PR

Create a single PR completing this chore.
