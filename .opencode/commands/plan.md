---
description: Create a plan for a feature or task, producing a GitHub issue as source of truth
argument-hint: <description>
---

<command-instruction>
Create a plan for a feature or task. Produces a GitHub issue as the source of truth.

## Procedure

1. **Research** - Explore codebase to understand scope and constraints
2. **Draft** - Create `./scratchpad/plan-<slug>.md` with the template below
3. **Review** - Present draft to user for feedback
4. **Finalize** - Once approved, create GitHub issue:
   ```bash
   gh issue create --title "<title>" --body-file ./scratchpad/plan-<slug>.md --label "feature"
   ```
5. **Cleanup** - Delete scratchpad file after issue is created

## Issue Template

The scratchpad file MUST follow this format to work with `/workon`:

```markdown
## Summary

<1-3 sentence problem statement>

## Requirements

- [ ] Requirement 1
- [ ] Requirement 2
- [ ] Requirement 3

## Proposed Approach

<Brief description of the implementation strategy>

## Implementation Steps

- [ ] Step 1 - <description>
- [ ] Step 2 - <description>
- [ ] Step 3 - <description>

## Open Questions

- <Any unresolved decisions or unknowns>

---

## Workon Prompt

> **Start here:** <specific first action to take>
>
> **Key files:** `path/to/file1.rs`, `path/to/file2.rs`
>
> **Context:** <any important background needed to begin>
```

## Notes

- The "Workon Prompt" section is essential - it tells `/workon` how to begin
</command-instruction>

<current-context>
<open-issues>
!`gh issue list --state open --limit 10 --json number,title,labels --jq '.[] | "- #\(.number) \(.title) [\(.labels | map(.name) | join(", "))]"' 2>/dev/null || echo "no issues"`
</open-issues>
<scratchpad-plans>
!`ls -1 ./scratchpad/plan-*.md 2>/dev/null || echo "no drafts"`
</scratchpad-plans>
<project-structure>
!`ls -1 src/`
</project-structure>
</current-context>
