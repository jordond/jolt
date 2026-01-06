---
description: Create a PR to complete a plan, auto-closing the issue when merged
---

<command-instruction>
BEFORE ANYTHING ELSE, ASK THE USER FOR AN ISSUE NUMBER. THEN USE THAT ISSUE NUMBER FOR THE REMAINING WORK. DO NOT PROCEED WITHOUT AN ISSUE NUMBER.

Create a PR to complete a plan. The PR will auto-close the issue when merged.

## Procedure (Step 0 - REQUIRED)

**Verify correct branch**

After getting the issue number, verify the current branch is associated with that issue:

```bash
CURRENT_BRANCH=$(git branch --show-current)
ISSUE_NUM=<issue-number>

# Check if branch name contains issue number (e.g., feat/issue-42-description or fix/issue-42-description)
if [[ ! "$CURRENT_BRANCH" =~ issue-${ISSUE_NUM} ]]; then
  echo "WARNING: Current branch ($CURRENT_BRANCH) does not appear to be for issue #$ISSUE_NUM"
fi

# Also verify we're not on main/master
if [[ "$CURRENT_BRANCH" == "main" || "$CURRENT_BRANCH" == "master" ]]; then
  echo "ERROR: Cannot create PR from main/master branch"
fi
```

- If on `main`/`master`: **STOP** - Cannot create PR from main branch
- If branch doesn't match the issue, **STOP** and ask user:
  - "You're on `<current-branch>` but completing issue #X. Expected a branch like `feat/issue-X-*` or `fix/issue-X-*`. Continue anyway? (YES / NO)"
  - If NO: List branches that might match the issue and offer to switch
  - If YES: Proceed with caution

## Prerequisites

- **Copilot Review Extension** (optional but recommended):
  ```bash
  gh extension install ChrisCarini/gh-copilot-review
  ```

## Procedure

1. **Verify completion** - Ensure all tasks in the issue are done
2. **Commit changes** - If uncommitted work exists, create a commit
3. **Push branch** - `git push -u origin <branch-name>`
4. **Create PR** and capture the URL:
   ```bash
   PR_URL=$(gh pr create --title "<descriptive title>" --body "$(cat <<'EOF'
   ## Summary

   <1-3 bullet points describing what was done>

   ## Changes

   - `path/to/file.rs` - <what changed>

   Fixes #<issue-number>
   EOF
   )")
   echo "Created PR: $PR_URL"
   ```
5. **Request Copilot review** (if extension installed):
   ```bash
   gh copilot-review "$PR_URL"
   ```
6. **Post final update** - Comment on the issue with PR link and summary
</command-instruction>

<current-context>
<in-progress-issues>
!`gh issue list --label "in-progress" --json number,title --jq '.[] | "- #\(.number) \(.title)"' 2>/dev/null || echo "none"`
</in-progress-issues>
<git-status>
!`git status --porcelain`
</git-status>
<current-branch>
!`git branch --show-current`
</current-branch>
<unpushed-commits>
!`git log origin/$(git branch --show-current)..HEAD --oneline 2>/dev/null || echo "no remote tracking"`
</unpushed-commits>
<recent-commits>
!`git log --oneline -5`
</recent-commits>
</current-context>
