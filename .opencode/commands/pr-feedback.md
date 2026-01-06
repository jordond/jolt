---
description: Address PR review comments one by one with user confirmation
---

<command-instruction>
BEFORE ANYTHING ELSE, ASK THE USE FOR A PR NUMBER. THEN USE THAT PR NUMBER FOR THE REAMINING WORK.

DO NOT PROCEED UNLESS YOU HAVE THE PR NUMBER, DO NOT LIST ALL PR's, ASK FOR A PR NUMBER.

Address PR review comments one by one with user confirmation.

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

7. **Resolve comments and post summary**

   After pushing, resolve all addressed review comments and post a summary:

   - **Resolve review threads** (for each fixed comment):

     ```bash
     # Get the GraphQL node_id for the review thread
     gh api graphql -f query='
       query($owner: String!, $repo: String!, $pr: Int!) {
         repository(owner: $owner, name: $repo) {
           pullRequest(number: $pr) {
             reviewThreads(first: 100) {
               nodes {
                 id
                 isResolved
                 comments(first: 1) {
                   nodes {
                     databaseId
                     path
                     body
                   }
                 }
               }
             }
           }
         }
       }
     ' -f owner=':owner' -f repo=':repo' -F pr=<pr-number>
     ```

   - **Resolve each thread** (for fixed comments):

     ```bash
     gh api graphql -f query='
       mutation($threadId: ID!) {
         resolveReviewThread(input: {threadId: $threadId}) {
           thread { isResolved }
         }
       }
     ' -f threadId='<thread-node-id>'
     ```

   - **If resolving fails** (e.g., not a review thread), delete the comment:

     ```bash
     gh api -X DELETE repos/:owner/:repo/pulls/comments/<comment-id>
     ```

   - **Post summary comment** on the PR:

     ```bash
     gh pr comment <pr-number> --body "$(cat <<'EOF'
     ## PR Feedback Addressed

     The following review comments have been addressed in the latest push:

     | File | Issue | Status |
     |------|-------|--------|
     | `<path>:<line>` | <brief description> | Fixed |
     | `<path>:<line>` | <brief description> | Skipped |

     **Summary:**
     - **Fixed:** X comments
     - **Skipped:** Y comments

     All addressed comments have been resolved.
     EOF
     )"
     ```

## Comment Classification

When presenting comments, classify by priority:

| Priority | Criteria                                                      |
| -------- | ------------------------------------------------------------- |
| High     | Bugs, security issues, key conflicts, breaking changes        |
| Medium   | Deprecation warnings, missing error handling, code quality    |
| Low      | Style suggestions, minor optimizations, optional improvements |

## Grouping Related Comments

Identify and group related comments to offer batch fixes:

| Group   | Trigger                              | Example           |
| ------- | ------------------------------------ | ----------------- |
| CHRONO  | Multiple `and_hms_opt` deprecations  | "YES ALL CHRONO"  |
| TIMEOUT | Multiple timeout-related issues      | "YES ALL TIMEOUT" |
| ERROR   | Multiple error handling improvements | "YES ALL ERROR"   |

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

> YES

**Committed and pushed!**

**Resolving review comments...**

- Comment #1 (src/input.rs:166) - Resolved
- Comment #2 (src/main.rs:1197) - Resolved
- Comment #3 (src/main.rs:1256) - Resolved

**Posted summary comment to PR #42**

Done! All feedback addressed and comments resolved.
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
- After pushing, always resolve addressed comments and post a summary
- If a comment cannot be resolved (not a review thread), delete it instead
- The summary comment provides a clear audit trail of what was addressed
</command-instruction>

<current-context>
<open-prs>
!`gh pr list --state open --json number,title,reviewDecision --jq '.[] | "- #\(.number) \(.title) [\(.reviewDecision // "pending")]"' 2>/dev/null || echo "no open PRs"`
</open-prs>
<current-branch>
!`git branch --show-current`
</current-branch>
<git-status>
!`git status --porcelain`
</git-status>
</current-context>
