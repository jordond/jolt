# /plan <description>

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

## Labels & Mode Keywords

When creating issues, add labels that control agent behavior. The "Workon Prompt" must include the corresponding keyword:

| Label | Keyword in Prompt | Effect |
|-------|-------------------|--------|
| `ultrawork` | Include "ultrawork" | Maximum effort mode - parallel agents, exhaustive verification |
| `analyze` | Include "analyze" | Analysis mode - gather context before implementation |

**Example with ultrawork label:**
```markdown
## Workon Prompt

> **Mode:** ultrawork
>
> **Start here:** <specific first action to take>
>
> **Key files:** `path/to/file1.rs`, `path/to/file2.rs`
>
> **Context:** <any important background needed to begin>
```

## Notes

- The "Workon Prompt" section is essential - it tells `/workon` how to begin
- Use `feature` label for new functionality, `enhancement` for improvements, `bug` for fixes
- Add `ultrawork` label for complex tasks requiring maximum effort
- Add `analyze` label for tasks requiring deep research before implementation
- Keep implementation steps atomic and checkable
