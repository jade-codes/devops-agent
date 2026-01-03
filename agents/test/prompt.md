Add comprehensive tests for the **{{module}}** module.

## Issues to Resolve
{{issue_list}}

## CRITICAL REQUIREMENTS

### 1. TEST THROUGH PUBLIC API ONLY
- Do NOT make private methods public just for testing
- Test internal logic through public entrypoint functions
- If a function is private, test it via the public function that calls it

### 2. VERIFY LOGIC CORRECTNESS
- READ and UNDERSTAND the implementation before writing tests
- Check if the logic makes sense and is correct
- If you find bugs, note them but still test current behavior

### 3. TEST FILE LOCATION AND NAMING
- **Unit tests**: `{{module_snake}}/tests/tests_*.rs` (tests folder inside module)
- **Integration tests**: `tests/{{module_snake}}/tests_*.rs` (module folder inside root tests/)
- Use prefix `tests_` for all test files (e.g., `tests_selection_range.rs`)
- All tests for this batch go in the appropriate folder
- Example unit test: `server/selection_range.rs` → `server/tests/tests_selection_range.rs`
- Example integration test: `server/selection_range.rs` → `tests/server/tests_selection_range.rs`

### 4. QUALITY TESTS ONLY
- NO TODO comments or placeholder tests
- Test edge cases: empty inputs, error conditions, boundaries
- Descriptive test names explaining what's tested

### 5. CLOSE ALL ISSUES IN ONE COMMIT
Commit message: `test: Add comprehensive tests for {{module}} ({{closes_str}})`

### 6. VERIFY BEFORE PR
- `make run-guidelines` must pass
- Then create PR

Create a single PR resolving all {{count}} issues.
