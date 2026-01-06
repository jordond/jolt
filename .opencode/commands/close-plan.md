# /close-plan <issue-number>

Create a PR to complete a plan. The PR will auto-close the issue when merged.

## Procedure

1. **Verify completion** - Ensure all tasks in the issue are done
2. **Commit changes** - If uncommitted work exists, create a commit
3. **Push branch** - `git push -u origin <branch-name>`
4. **Create PR** with this format:
   ```bash
   gh pr create --title "<descriptive title>" --body "## Summary

   <1-3 bullet points describing what was done>

   ## Changes

   - `path/to/file.rs` - <what changed>

   Fixes #<issue-number>"
   ```
5. **Post final update** - Comment on the issue with PR link and summary
