---
description: Begin working on a plan issue by number or search query
argument-hint: <issue-number | search-query>
---

<command-instruction>
Begin working on a plan issue. Accepts an issue number or a search query to find the issue.

## Arguments

- `<issue-number>` - Direct issue number (e.g., `42`)
- `<search-query>` - Text to search for in issue titles (e.g., `"battery forecast"`)

## Procedure

1. **Resolve the issue**
   - If numeric: `gh issue view <number>`
   - If search query: `gh search issues "<query>" --repo :owner/:repo --state open --limit 5`
     - If multiple matches, present options and ask user to confirm
     - If single match, proceed with that issue
     - If no matches, report and stop

2. **Display issue context**
   - Show issue number, title, labels, and body
   - Highlight the "Workon Prompt" section if present (contains key context)
   - Show any existing progress update comments

3. **Check for mode labels** - Inspect issue labels for special modes:
   - Include the labels in the prompt
   - These keywords trigger specific agent behaviors (see label table below)

4. **Set up working branch**
   - Check current branch: `git branch --show-current`
   - If on `main`/`master`, create and checkout feature branch:
     ```bash
     git checkout -b feat/issue-<number>-<slug>  # for features
     git checkout -b fix/issue-<number>-<slug>   # for bugs
     ```
   - If already on a feature branch, confirm it's the right one or offer to switch

5. **Mark as in-progress**
   - Add `in-progress` label: `gh issue edit <number> --add-label "in-progress"`

6. **Create todo list from issue**
   - Parse checkbox items from issue body
   - Create local todo list tracking the implementation tasks

7. **Begin work**
   - If issue has a "Workon Prompt" section, use it to understand:
     - Current state (for continued work)
     - Next step to take
     - Key files to examine
   - Start with codebase exploration based on the issue requirements

## Notes

- This command pairs with `/plan` which creates issues with workon-friendly prompts
- Use `/update-plan` during work to track progress
- Use `/close-plan` when complete to create a PR
</command-instruction>

<current-context>
<open-issues>
!`gh issue list --state open --json number,title,labels --jq '.[] | "- #\(.number) \(.title) [\(.labels | map(.name) | join(", "))]"' 2>/dev/null || echo "no issues"`
</open-issues>
<in-progress-issues>
!`gh issue list --label "in-progress" --json number,title --jq '.[] | "- #\(.number) \(.title)"' 2>/dev/null || echo "none"`
</in-progress-issues>
<current-branch>
!`git branch --show-current`
</current-branch>
<git-status>
!`git status --porcelain`
</git-status>
</current-context>
