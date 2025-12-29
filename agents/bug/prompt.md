Fix bug #{{issue}}.

**{{title}}**

{{body}}

## CRITICAL REQUIREMENTS

### 1. UNDERSTAND THE BUG FIRST
- Read the bug report carefully
- Reproduce the issue if possible
- Identify the root cause before fixing

### 2. ROOT CAUSE ANALYSIS
- Don't just patch symptoms - fix the actual cause
- Trace the code path that leads to the bug
- Consider if similar issues exist elsewhere

### 3. IMPLEMENT A CLEAN FIX
- Make the minimal change needed to fix the issue
- Don't introduce new bugs or break existing functionality
- Follow existing code patterns and style

### 4. ADD REGRESSION TEST
- Write a test that would have caught this bug
- Test the specific scenario from the bug report
- Test related edge cases

### 5. COMMIT WITH PROPER MESSAGE
Commit message: `fix: {{title}} (closes #{{issue}})`

### 6. VERIFY BEFORE PR
- `make run-guidelines` must pass
- All tests must pass (especially new regression test)
- Then create PR

Create a single PR fixing this bug.
