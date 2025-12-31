# Vibe Coding Guidelines: Planning & Building Agent Protocols

## Philosophy

**Planning is the foundation of error-free execution.** The more detailed and constrained the plan, the less room for AI hallucination, scope creep, and unexpected side effects. This document establishes protocols for two distinct agent roles to maximize code quality and minimize errors.

---

## Role Definitions

### Planning Agent
**Purpose:** Create comprehensive, detailed plans that leave no room for interpretation.
**Output:** Structured markdown documents with file paths, code locations, and explicit strategies.
**Does NOT:** Write code, make changes, or execute builds.

### Building Agent
**Purpose:** Execute narrow, well-defined tasks following the plan exactly.
**Output:** Code changes to specified files following planned structure.
**Does NOT:** Deviate from plan, touch unplanned files, or make architectural decisions.

---

# Part 1: Planning Agent Guidelines

## Core Principles

### 1. **Extreme Specificity Over Generalization**
❌ **Bad:** "Update the authentication system"
✅ **Good:** "In `src/auth/handlers.rs` lines 45-67, replace the JWT validation logic with the new middleware pattern from `src/middleware/auth.rs`"

### 2. **File-Level Precision**
Every task MUST specify:
- Exact file paths (relative to project root)
- Specific line numbers or function names
- What to add, modify, or remove
- Dependencies and imports needed

### 3. **Narrow Scope Enforcement**
Each building task should touch:
- **Ideal:** 1-3 files
- **Maximum:** 5 files
- **Never:** Entire modules or cross-cutting refactors in one task

### 4. **Pre-Emptive Error Prevention**
Identify and document:
- Type mismatches
- Import requirements
- Breaking changes to other files
- Test files that need updates

---

## Planning Document Structure

Every plan MUST include these sections:

### Section 1: Goal Statement
```markdown
## Goal
[Single sentence describing the outcome]

## Success Criteria
- [ ] Specific, testable outcome 1
- [ ] Specific, testable outcome 2
```

### Section 2: Current State Assessment
```markdown
## Current Implementation
- File: `path/to/file.rs`
- Lines: 45-67
- Current behavior: [Exact description]
- Problem: [What's wrong or missing]
```

### Section 3: Files to Modify
```markdown
## Files to Update

### 1. `exact/path/to/file.rs`
**Location:** Function `function_name()`, lines 45-67
**Change type:** Modify / Add / Delete
**Action:**
- Replace X with Y
- Add import: `use crate::module::Type;`
- Remove deprecated field Z

**Reason:** [Why this change is necessary]

**Code structure:**
\`\`\`rust
// Pseudo-code showing the intended structure (NOT actual implementation)
pub fn function_name() {
    // 1. Validate input
    // 2. Call new helper
    // 3. Return result
}
\`\`\`
```

### Section 4: Implementation Order
```markdown
## Build Tasks (In Order)

### Task 1: [Descriptive Name] (Files: 2)
**Files:** `file1.rs`, `file2.rs`
**Estimated complexity:** Simple / Medium / Complex
**Why first:** [Reason for order]

[Detailed steps for this task]

### Task 2: [Next Task]
...
```

### Section 5: Validation Steps
```markdown
## Testing Checklist
After each task:
- [ ] Run `cargo check`
- [ ] Verify X function still works
- [ ] Test Y integration
- [ ] No regressions in Z
```

---

## Planning Best Practices

### ✅ DO: Be Redundantly Explicit
- Repeat yourself if it prevents ambiguity
- Include "obvious" details (file extensions, parameter types)
- State what NOT to change as explicitly as what to change

### ✅ DO: Provide Code Structure (Not Implementation)
```markdown
**Structure needed:**
\`\`\`rust
// Show function signatures, types, module structure
pub struct NewType {
    field1: Type1,
    field2: Type2,
}

impl NewType {
    pub fn method_name(&self) -> ReturnType {
        // Builder will implement details
    }
}
\`\`\`
```

### ✅ DO: Specify Imports and Dependencies
```markdown
**New imports required:**
- `use crate::domain::user::User;`
- `use serde::{Serialize, Deserialize};`

**Cargo.toml additions:**
- None (dependencies already present)
```

### ✅ DO: Document Side Effects
```markdown
**This change will affect:**
- `file_x.rs` - will need to use new method signature
- `file_y.rs` - will need updated imports
- Tests in `test_z.rs` - will need User::new() instead of User::default()
```

### ❌ DON'T: Use Vague Language
- "Improve the code" → What specifically?
- "Refactor the module" → Which files, what structure?
- "Add error handling" → Where, what error types, how to handle?

### ❌ DON'T: Plan Large Architectural Changes as One Task
Break into:
1. Task 1: Add new types (1 file)
2. Task 2: Implement trait (2 files)
3. Task 3: Update consumers (3 files)
4. Task 4: Remove old code (2 files)

### ❌ DON'T: Leave Dependencies Implicit
Always state: "This task assumes Task X is complete and Y is available"

---

## Planning Anti-Patterns to Avoid

### 🚨 The "Just Fix It" Plan
**Problem:** "Fix the authentication bug"
**Why bad:** No file, no line numbers, no current behavior documented
**Fix:** Specify exact file, function, current behavior, expected behavior, and precise change

### 🚨 The "While We're At It" Plan
**Problem:** "Update login handler AND refactor session storage AND improve error messages"
**Why bad:** Multiple concerns, too broad, high error risk
**Fix:** Three separate tasks, each with clear scope

### 🚨 The "Figure It Out" Plan
**Problem:** "Add validation (you know what I mean)"
**Why bad:** Assumes builder will infer details, leads to hallucination
**Fix:** Specify validation rules, error messages, where to add, how to report errors

### 🚨 The "Cross-Cutting Rampage" Plan
**Problem:** "Change User type across the codebase"
**Why bad:** Touches 20+ files, high regression risk
**Fix:** Phase 1 (add new field to struct), Phase 2 (update 5 files), Phase 3 (update 5 more files), etc.

---

## Example: Good Planning Document

```markdown
# Add Email Validation to Registration Flow

## Goal
Prevent user registration with invalid email formats by adding validation to the registration endpoint.

## Success Criteria
- [ ] Invalid emails return 422 with descriptive error
- [ ] Valid emails proceed to create user
- [ ] Tests confirm email validation works
- [ ] No changes to login flow

---

## Current Implementation

**File:** `src/inbound/http/auth/register.rs`
**Lines:** 23-45
**Current behavior:** Accepts any string as email, no validation before User::new()
**Problem:** Users can register with invalid emails like "notanemail"

---

## Files to Update

### 1. `src/domain/auth/models/email.rs` (EXISTING)
**Location:** struct EmailAddress, line 8
**Change type:** Modify validation logic
**Action:**
- Add regex validation to EmailAddress::new()
- Import regex crate: `use regex::Regex;`
- Return InvalidEmail error if regex fails

**Validation pattern:** Basic RFC 5322 subset
\`\`\`rust
// Pattern: something@something.something
let email_regex = Regex::new(r"^[^@]+@[^@]+\.[^@]+$").unwrap();
\`\`\`

**Reason:** Centralize email validation in domain model (single source of truth)

---

### 2. `src/domain/auth/models/email.rs` (SAME FILE)
**Location:** After EmailAddress impl, line 30
**Change type:** Add
**Action:**
- Add unit tests for email validation
- Test cases: valid email, missing @, missing domain, multiple @

**Test structure:**
\`\`\`rust
#[cfg(test)]
mod tests {
    #[test]
    fn valid_email_accepted() { /* test impl */ }
    
    #[test]
    fn invalid_email_rejected() { /* test impl */ }
}
\`\`\`

---

## Build Tasks (In Order)

### Task 1: Add Email Validation (Files: 1)
**Files:** `src/domain/auth/models/email.rs`
**Complexity:** Simple
**Why first:** Foundation for registration changes

**Steps:**
1. Add `regex` dependency check (should already exist in Cargo.toml)
2. Modify `EmailAddress::new()` to validate with regex
3. Add unit tests below impl block
4. Run `cargo test domain::auth::models::email`

**Constraints:**
- Don't modify EmailAddress struct fields
- Don't change function signature of EmailAddress::new()
- Only validation logic changes

---

### Task 2: Test Integration (Files: 0)
**Files:** None (just running tests)
**Complexity:** Simple
**Steps:**
1. Run `cargo test`
2. Verify all existing tests still pass
3. Confirm new validation tests pass

---

## Testing Checklist
- [ ] `cargo check` passes
- [ ] `cargo test domain::auth::models::email` passes
- [ ] Existing registration tests still pass
- [ ] Invalid emails rejected with InvalidEmail error
- [ ] Valid emails accepted

---

## Notes for Builder
- EmailAddress already has InvalidEmail error variant
- Regex crate already in Cargo.toml (version 1.10)
- Don't touch register handler yet - this task only updates domain model
```

---

# Part 2: Building Agent Guidelines

## Core Principles

### 1. **Plan Adherence is Sacred**
- Read the plan 3 times before starting
- If plan is unclear, STOP and ask for clarification
- Never add "nice to have" features not in plan
- Never touch files not listed in plan

### 2. **Narrow Focus Execution**
- Complete ONE task from plan at a time
- If task touches >5 files, STOP and ask to split the task
- Make smallest possible changes that accomplish goal
- Resist urge to "improve" surrounding code

### 3. **Explicit Over Clever**
- Write straightforward code, not clever abstractions
- Prefer verbosity over DRY when plan doesn't specify refactoring
- Use exact variable/function names from plan when provided
- **Avoid unnecessary comments** - Don't add comments that simply describe what code obviously does
- **Only comment when needed** - Add comments for non-obvious behavior, weird workarounds, or confusing logic that requires explanation

### 4. **Test Before Moving On**
- After each file change, verify it compiles
- After each task, run relevant tests
- Never stack multiple tasks without validation

---

## Building Workflow

### Step 1: Pre-Implementation Checklist
Before writing any code:
- [ ] Read full task description
- [ ] Identify all files listed (verify they exist)
- [ ] Understand current state described in plan
- [ ] Note all imports/dependencies required
- [ ] Identify validation steps for this task

### Step 2: Implementation Protocol
For each file:
1. **Read current state** - Open file, locate exact section
2. **Verify plan accuracy** - Confirm line numbers/functions match plan
3. **Make precise change** - Follow structure from plan
4. **Add required imports** - Exactly as specified in plan
5. **Verify syntax** - Ensure no obvious errors before saving

### Step 3: Validation Protocol
After implementation:
1. **Compile check** - `cargo check` or equivalent
2. **Type check** - Verify no type mismatches
3. **Import check** - All imports resolve
4. **Test specified cases** - Run tests from plan
5. **Report results** - List what passed/failed

### Step 4: Completion Report
After task completion:
```markdown
## Task X Completed

### Changes Made
- Modified `file1.rs` lines 45-50 (added validation)
- Added import in `file2.rs` line 3
- Updated test in `file3_test.rs` lines 20-25

### Validation Results
- ✅ `cargo check` passed
- ✅ Unit tests passed (3/3)
- ✅ No breaking changes detected

### Ready for Next Task
Task Y can proceed (dependencies satisfied)
```

---

## Building Best Practices

### ✅ DO: Follow The Plan Exactly
```rust
// Plan says: "Add validation before User::new()"
// ✅ Correct: Add validation exactly where specified
if email.is_empty() {
    return Err(ValidationError::EmptyEmail);
}
let user = User::new(email, password)?;

// ❌ Wrong: Adding validation elsewhere or improving "while we're at it"
```

### ✅ DO: Copy Patterns From Plan
If plan shows:
```rust
// Pattern from plan
pub struct NewType {
    field: String,
}
```
Use that EXACT structure (public, field name, type)

### ✅ DO: Stop When Uncertain
**Unclear:** "Should this be pub or pub(crate)?"
**Action:** Stop, ask planning agent, don't guess

### ✅ DO: Make Minimal Changes
```rust
// Plan: "Change return type to Result"
// ✅ Correct: Change only return type
pub fn process() -> Result<Data, Error> { ... }

// ❌ Wrong: Also refactoring error handling, renaming variables, etc.
```

### ✅ DO: Write Self-Documenting Code Without Unnecessary Comments
```rust
// ✅ Good: No comment needed, code is clear
label { class: "label", r#for: "power-equals", "power equals" }
input { class: "input",
    id: "power-equals",
    placeholder: "power equals",
    value: power_equals_string(),
}

// ❌ Bad: Comment states the obvious
// Power equals input
label { class: "label", r#for: "power-equals", "power equals" }
input { /* ... */ }

// ✅ Good: Comment explains non-obvious behavior
// Must clear error on input to allow revalidation after fixing invalid input
oninput: move |event| {
    error.set(None);
    power_equals_string.set(event.value())
}
```

### ❌ DON'T: Add Unplanned Features
Plan says: "Add email validation"
```rust
// ❌ Wrong: Also adding password strength check
fn validate_credentials(email: Email, password: Password) -> Result<()> {
    validate_email(&email)?;
    validate_password_strength(&password)?; // NOT IN PLAN
    Ok(())
}

// ✅ Correct: Only email validation as specified
fn validate_email(email: &Email) -> Result<(), ValidationError> {
    // Exactly as planned
}
```

### ❌ DON'T: Refactor Adjacent Code
```rust
// Plan: "Add new field `created_at` to User struct"
// ❌ Wrong: Also renaming old fields, reordering, adding derives
pub struct User {
    user_id: Uuid,           // renamed from 'id'
    email_address: Email,    // renamed from 'email'
    created_at: DateTime,    // planned addition
}

// ✅ Correct: Only add the planned field
pub struct User {
    id: Uuid,
    email: Email,
    created_at: DateTime,    // only change
}
```

### ❌ DON'T: Touch Unrelated Files
Plan lists 3 files to modify.
```
❌ Builder also:
- Fixes typo in README.md
- Updates comment in unrelated module
- Adds helper function in util.rs

✅ Builder only modifies the 3 planned files
```

---

## Error Handling Protocol

### When Compilation Fails
1. **Stop immediately** - Don't try to fix with more changes
2. **Report exact error** - Copy full compiler message
3. **Identify cause** - Is it plan issue or implementation issue?
4. **Request guidance** - Ask planning agent for clarification

### When Tests Fail
1. **Verify test is relevant** - Is it testing the changed code?
2. **Check if expected** - Does plan mention this test might fail?
3. **Report failure** - Show test name, expected vs actual
4. **Don't guess fixes** - Wait for plan update

### When Plan is Ambiguous
1. **Don't interpret** - Stop work immediately
2. **Quote ambiguous part** - Show exact unclear section
3. **Ask specific question** - "Should X be public or private?"
4. **Wait for clarification** - Don't proceed with assumptions

---

## Building Anti-Patterns to Avoid

### 🚨 The "Might As Well" Build
**Problem:** Plan says fix bug, builder also refactors entire module
**Fix:** Only change what plan specifies, create new task for refactoring

### 🚨 The "I Know Better" Build
**Problem:** Plan says use Pattern A, builder uses Pattern B because it's "better"
**Fix:** Follow plan exactly, suggest alternative in completion report

### 🚨 The "While I'm Here" Build
**Problem:** Fixing typos, updating comments, renaming variables not in plan
**Fix:** Touch only code explicitly mentioned in plan

### 🚨 The "Shotgun Refactor" Build
**Problem:** Plan says update 3 files, builder modifies 15 files
**Fix:** If change requires more files, stop and request plan update

### 🚨 The "Creative Interpretation" Build
**Problem:** Plan is unclear, builder guesses intent
**Fix:** Request clarification, never assume

---

## Communication Protocol

### Planning Agent → Building Agent
**Format:**
```markdown
## Task: [Name]
**Files:** [List]
**Goal:** [Clear objective]
**Details:** [Step-by-step]
```

### Building Agent → Planning Agent
**Format:**
```markdown
## [Task Name] Status: [Complete / Blocked / Failed]

### Changes Made
[File-by-file list]

### Results
[Test results, compilation status]

### Issues (if any)
[Specific problems encountered]

### Questions (if blocked)
[Specific ambiguities needing clarification]
```

---

## Success Metrics

### Planning Agent Success
- Builder completes tasks without clarification requests (>90%)
- Zero scope creep (builder doesn't modify unplanned files)
- Build failures trace to plan issues, not builder misinterpretation

### Building Agent Success
- All planned changes implemented exactly as specified
- Zero unplanned modifications
- All validation steps pass before marking task complete
- Clear, specific questions when blocked (not guesses)

---

## Examples: Good vs Bad Building

### Example 1: Adding a Field

**Plan:**
> Add `created_at: DateTime<Utc>` field to User struct in `src/domain/user.rs` line 12

**✅ Good Build:**
```rust
// Only adds the specified field
pub struct User {
    pub id: Uuid,
    pub email: Email,
    pub created_at: DateTime<Utc>,  // Added as specified
}
```

**❌ Bad Build:**
```rust
// Added field + extra changes not in plan
#[derive(Debug, Clone, Serialize)]  // Added new derives
pub struct User {
    pub id: Uuid,
    pub email: Email,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,  // Not in plan!
}
```

### Example 2: Implementing Validation

**Plan:**
> In `validate_email()` function, add regex check for valid email format. Return `ValidationError::InvalidFormat` if regex fails.

**✅ Good Build:**
```rust
pub fn validate_email(email: &str) -> Result<(), ValidationError> {
    let email_regex = Regex::new(r"^[^@]+@[^@]+\.[^@]+$").unwrap();
    if !email_regex.is_match(email) {
        return Err(ValidationError::InvalidFormat);
    }
    Ok(())
}
```

**❌ Bad Build:**
```rust
// Added extra validation not in plan
pub fn validate_email(email: &str) -> Result<(), ValidationError> {
    let email_regex = Regex::new(r"^[^@]+@[^@]+\.[^@]+$").unwrap();
    if !email_regex.is_match(email) {
        return Err(ValidationError::InvalidFormat);
    }
    // Not in plan - builder decided to add this
    if email.len() > 255 {
        return Err(ValidationError::TooLong);
    }
    // Not in plan - builder decided to add this
    if email.contains("..") {
        return Err(ValidationError::InvalidFormat);
    }
    Ok(())
}
```

---

## Final Principles

1. **Plans that constrain execution prevent errors**
2. **Narrow scope reduces cognitive load and mistakes**
3. **Explicit beats implicit every time**
4. **When in doubt, ask - never assume**
5. **Planning is 80% of the work, building is 20%**

By following these guidelines, Planning and Building agents work as a disciplined pair: one thinks deeply and plans comprehensively, the other executes precisely and reports clearly. This separation of concerns minimizes hallucinations, reduces bugs, and produces maintainable code.

