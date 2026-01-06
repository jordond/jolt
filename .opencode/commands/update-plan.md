---
description: Update an existing plan with progress, always including a continuation prompt
argument-hint: <issue-number>
---

<command-instruction>
Update an existing plan with progress. Always includes a continuation prompt.

## Procedure

1. **Fetch current state** - `gh issue view <issue-number>`
2. **Summarize progress** - What was completed, what changed, any blockers
3. **Post update comment** with this format:

   ```markdown
   ## Progress Update - <date>

   ### Completed
   - [x] Task 1
   - [x] Task 2

   ### In Progress
   - [ ] Task 3 (current state: ...)

   ### Blockers / Changes
   - <any issues or scope changes>

   ### Modified Files
   - `path/to/file.rs` - <what changed>

   ---

   ## Workon Prompt

   > **Start here:** <specific next action to take>
   >
   > **Key files:** `path/to/modified/file1.rs`, `path/to/modified/file2.rs`
   >
   > **Context:** <current state and what's been done>
   ```

4. **Update labels** - Add/remove `in-progress` as appropriate
5. **Update checkboxes** - Edit issue body if tasks completed: `gh issue edit <number> --body-file`
</command-instruction>

<current-context>
<in-progress-issues>
!`gh issue list --label "in-progress" --json number,title --jq '.[] | "- #\(.number) \(.title)"' 2>/dev/null || echo "none"`
</in-progress-issues>
<current-branch>
!`git branch --show-current`
</current-branch>
<recent-commits>
!`git log --oneline -5`
</recent-commits>
<modified-files>
!`git diff --name-only HEAD~5 2>/dev/null | head -10 || echo "no recent changes"`
</modified-files>
</current-context>
