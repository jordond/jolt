---
description: Create a PR to complete a plan, auto-closing the issue when merged
argument-hint: <issue-number>
---

<command-instruction>
Create a PR to complete a plan. The PR will auto-close the issue when merged.

## Procedure

1. **Verify completion** - Ensure all tasks in the issue are done
2. **Commit changes** - If uncommitted work exists, create a commit
3. **Push branch** - `git push -u origin <branch-name>`
4. **Create PR** with this format:
   ```bash
   gh pr create --title "<descriptive title>" --body "$(cat <<'EOF'
   ## Summary

   <1-3 bullet points describing what was done>

   ## Changes

   - `path/to/file.rs` - <what changed>

   Fixes #<issue-number>
   EOF
   )"
   ```
5. **Post final update** - Comment on the issue with PR link and summary
</command-instruction>

<current-context>
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
