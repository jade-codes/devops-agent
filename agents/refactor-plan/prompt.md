```markdown
Create a refactoring plan for: {{path}}

## MISSION

Analyze the specified code path holistically to understand its architecture, identify bottlenecks, and produce a comprehensive refactoring plan.

## CRITICAL REQUIREMENTS

### 1. HOLISTIC CODE ANALYSIS
- Trace the ENTIRE flow from entry point to exit
- Map all function calls, dependencies, and data transformations
- Identify the boundaries between modules/components
- Document the current state before proposing changes

### 2. ARCHITECTURE UNDERSTANDING
- Identify the design patterns currently in use
- Map the dependency graph (what depends on what)
- Understand the data flow and state management
- Note any abstractions, traits, or interfaces involved
- Identify coupling between components

### 3. BOTTLENECK IDENTIFICATION
Analyze for these specific issues:
- **Performance**: O(n²) loops, redundant computations, unnecessary allocations
- **Complexity**: Deep nesting, long functions, high cyclomatic complexity
- **Coupling**: Tight dependencies, circular references, god objects
- **Duplication**: Repeated logic, copy-paste code, similar patterns
- **Abstraction**: Leaky abstractions, wrong abstraction level, missing abstractions
- **Error Handling**: Inconsistent error patterns, swallowed errors, poor error context
- **Testability**: Hard-to-test code, hidden dependencies, side effects

### 4. IMPACT ASSESSMENT
For each identified issue:
- Severity: Critical / High / Medium / Low
- Effort: Large / Medium / Small
- Risk: Breaking changes, API changes, behavior changes
- Dependencies: What else would need to change

### 5. REFACTORING STRATEGY
Propose a phased approach:
- **Phase 1**: Safe, low-risk improvements (formatting, naming, small extractions)
- **Phase 2**: Structural changes (extract functions/modules, introduce abstractions)
- **Phase 3**: Architectural changes (redesign patterns, restructure dependencies)

### 6. OUTPUT FORMAT

Produce a structured plan with:

```
## Executive Summary
[One paragraph overview of current state and recommended approach]

## Architecture Overview
[Diagram or description of current architecture]

## Flow Analysis
[Step-by-step trace of the code path]

## Identified Issues

### Issue 1: [Name]
- **Location**: [file:line]
- **Type**: [Performance/Complexity/Coupling/etc.]
- **Severity**: [Critical/High/Medium/Low]
- **Description**: [What the problem is]
- **Impact**: [Why it matters]
- **Proposed Solution**: [How to fix it]
- **Effort**: [Large/Medium/Small]
- **Risk**: [What could break]

[Repeat for each issue]

## Refactoring Phases

### Phase 1: Quick Wins (Low Risk)
- [ ] Task 1
- [ ] Task 2

### Phase 2: Structural Improvements (Medium Risk)
- [ ] Task 1
- [ ] Task 2

### Phase 3: Architectural Changes (Higher Risk)
- [ ] Task 1
- [ ] Task 2

## Recommended Approach
[Prioritized list of what to tackle first and why]

## Testing Strategy
[How to ensure refactoring doesn't break functionality]
```

### 7. PRINCIPLES TO FOLLOW
- Prefer incremental changes over big-bang rewrites
- Each refactoring step should leave code working
- Maintain or improve test coverage at each step
- Document architectural decisions
- Consider backwards compatibility

### 8. DO NOT
- Implement any changes (this is a PLANNING agent only)
- Make assumptions without reading the code
- Propose changes without understanding impact
- Skip the holistic analysis in favor of quick fixes

```
