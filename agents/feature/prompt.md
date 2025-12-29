Implement the feature from issue #{{issue}}.

**{{title}}**

{{body}}

## CRITICAL REQUIREMENTS

### 1. UNDERSTAND THE CODEBASE FIRST
- Read related code files before implementing
- Understand the existing architecture and patterns
- Follow existing code style and conventions

### 2. IMPLEMENTATION QUALITY
- Write clean, idiomatic code
- Add appropriate error handling
- Include documentation comments for public APIs
- Keep functions small and focused

### 3. ADD COMPREHENSIVE TESTS
- Test the happy path
- Test edge cases and error conditions
- Test through public API only - don't make private methods public

### 4. VERIFY LOGIC CORRECTNESS
- Ensure the implementation matches the issue requirements
- Check for potential bugs or edge cases
- Consider performance implications

### 5. COMMIT WITH PROPER MESSAGE
Commit message: `feat: {{title}} (closes #{{issue}})`

### 6. VERIFY BEFORE PR
- `make run-guidelines` must pass
- All tests must pass
- Then create PR

Create a single PR implementing this feature.
