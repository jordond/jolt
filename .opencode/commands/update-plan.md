# /update-plan <issue-number>

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

   > **Mode:** <ultrawork|analyze|omit if neither label>
   >
   > **Start here:** <specific next action to take>
   >
   > **Key files:** `path/to/modified/file1.rs`, `path/to/modified/file2.rs`
   >
   > **Context:** <current state and what's been done>
   ```

   **Important:** If the issue has `ultrawork` or `analyze` labels, the Mode line MUST include that keyword to preserve the behavior mode for `/workon`.

4. **Update labels** - Add/remove `in-progress` as appropriate
5. **Update checkboxes** - Edit issue body if tasks completed: `gh issue edit <number> --body-file`
