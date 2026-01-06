# /pr-feedback <pr-number>

Address PR review comments one by one with user confirmation.

## Arguments

- `<pr-number>` - The pull request number to process feedback for

## Procedure

1. **Fetch PR review comments**
   ```bash
   gh api repos/:owner/:repo/pulls/<pr-number>/comments --paginate | jq -r '.[] | "---\n## Comment \(.id)\n**File:** \(.path):\(.line)\n**Suggestion:**\n\(.body)\n"'
   ```

2. **Present summary table**
   - Show all comments in a numbered table with:
     - File and line number
     - Brief description of the issue
     - Priority (High/Medium/Low based on severity)
   - Group related comments (e.g., all deprecation warnings together)

3. **Process comments one by one**
   - For each comment, show:
     - The file and line number
     - The full issue description
     - The suggested fix (if provided)
   - Ask user for confirmation:
     - **YES** - Fix this comment
     - **SKIP** - Move to next comment
     - **YES ALL [GROUP]** - Fix all related comments together (e.g., "YES ALL CHRONO" for deprecation fixes)
   - If fixing:
     - Read the relevant file section
     - Apply the fix
     - Do NOT verify yet (defer to end)
     - Mark as fixed and move to next

4. **Skip verification during fixes**
   - Do NOT run `cargo check`, `clippy`, or `lsp_diagnostics` after each fix
   - This speeds up the feedback loop
   - All verification happens at the end

5. **Final verification (after all comments processed)**
   ```bash
   cargo fmt --all --check
   cargo clippy --all-targets --all-features -- -D warnings
   cargo build
   cargo test
   ```
   - If any check fails, report and offer to fix

6. **Commit and push**
   - Show summary of changes:
     - Number of comments fixed
     - Number of comments skipped
     - Files modified
   - Ask user for commit message or suggest:
     ```
     fix: address PR #<number> review feedback
     
     - <brief list of changes>
     ```
   - Commit and push:
     ```bash
     git add -A
     git commit -m "<message>"
     git push
     ```

## Comment Classification

When presenting comments, classify by priority:

| Priority | Criteria |
|----------|----------|
| High | Bugs, security issues, key conflicts, breaking changes |
| Medium | Deprecation warnings, missing error handling, code quality |
| Low | Style suggestions, minor optimizations, optional improvements |

## Grouping Related Comments

Identify and group related comments to offer batch fixes:

| Group | Trigger | Example |
|-------|---------|---------|
| CHRONO | Multiple `and_hms_opt` deprecations | "YES ALL CHRONO" |
| TIMEOUT | Multiple timeout-related issues | "YES ALL TIMEOUT" |
| ERROR | Multiple error handling improvements | "YES ALL ERROR" |

## Example Session

```
## PR #42 Review Comments Summary

| # | File | Issue | Priority |
|---|------|-------|----------|
| 1 | src/input.rs:166 | Key conflict in handler | High |
| 2 | src/main.rs:1197 | Deprecated and_hms_opt | Medium |
| 3 | src/main.rs:1256 | Deprecated and_hms_opt | Medium |

---

## Comment #1 - Key Conflict (HIGH)

**File:** `src/input.rs:166`

**Issue:** Pressing 'h' conflicts with vim navigation...

**Suggested Fix:**
[code block]

---

**Proceed with Comment #1?** (YES / SKIP)

> YES

**Comment #1 FIXED**

---

## Comment #2 - Deprecated Method (MEDIUM)

**File:** `src/main.rs:1197`

**Issue:** and_hms_opt is deprecated...

**Note:** Comments #2, #3 are related deprecation warnings.

**Proceed with Comment #2?** (YES / YES ALL CHRONO / SKIP)

> YES ALL CHRONO

**Comments #2, #3 FIXED**

---

## All Comments Addressed!

| # | Status |
|---|--------|
| 1 | FIXED |
| 2 | FIXED |
| 3 | FIXED |

**Running verification...**

[verification output]

**Ready to commit?** (YES / NO)
```

## Handling False Positives

If a comment appears incorrect or already resolved:

1. Check the actual code to verify
2. If already correct, note: "**SKIP this comment?** (YES to skip / NO to investigate)"
3. Mark as SKIPPED with reason: "Code already correct" or "False positive"

## Notes

- Always fetch the latest PR comments before starting
- Defer ALL verification to the end for speed
- Offer grouped fixes for related issues to reduce confirmation fatigue
- If verification fails, do NOT auto-commit - report and offer fixes first
- The final push updates the PR automatically
