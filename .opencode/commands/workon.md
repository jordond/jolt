# /workon <issue-number | search-query>

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
   - If `ultrawork` label present: Include "ultrawork" keyword in prompt context
   - If `analyze` label present: Include "analyze" keyword in prompt context
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
   - Apply mode based on labels:
     - `ultrawork`: Maximum effort - spawn parallel agents, exhaustive verification
     - `analyze`: Analysis first - gather context with explore/librarian agents before implementation
   - Start with codebase exploration based on the issue requirements

## Example Usage

```bash
# By issue number
/workon 42

# By search query
/workon battery forecast

# By partial match
/workon "graph design"
```

## Mode Labels

| Label | Keyword | Behavior |
|-------|---------|----------|
| `ultrawork` | "ultrawork" | Maximum effort mode - parallel agents, exhaustive search, full verification |
| `analyze` | "analyze" | Analysis mode - deep context gathering before any implementation |

When an issue has these labels, the Workon Prompt MUST contain the keyword to trigger the mode.

## Notes

- This command pairs with `/plan` which creates issues with workon-friendly prompts
- Use `/update-plan` during work to track progress
- Use `/close-plan` when complete to create a PR
