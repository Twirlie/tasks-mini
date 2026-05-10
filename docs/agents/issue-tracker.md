# Issue Tracker Configuration

## Type: Local Markdown

Issues are tracked as markdown files in this repository under `.scratch/<feature>/`.

## Workflow

- **Creating issues**: Skills create markdown files under `.scratch/<feature-name>/`
- **File format**: Each issue is a separate markdown file with frontmatter
- **Directory structure**: `.scratch/` contains subdirectories for each feature/epic
- **Status tracking**: Issue status is managed through frontmatter fields

## Frontmatter Schema

```yaml
---
title: "Issue Title"
status: "needs-triage" | "needs-info" | "ready-for-agent" | "ready-for-human" | "wontfix"
created: "YYYY-MM-DDTHH:MM:SSZ"
updated: "YYYY-MM-DDTHH:MM:SSZ"
labels: ["label1", "label2"]
assignee: "username" (optional)
---
```

## Skills Integration

- `to-issues`: Creates new issue files under `.scratch/`
- `triage`: Updates issue status and labels in frontmatter
- `to-prd`: Converts conversation context to issue files
- `qa`: Reads and updates existing issues

## Agent Brief Format

Agent briefs MUST include a **TDD Cycles** section instead of a flat "Tests" list. Each cycle is a vertical RED→GREEN→REFACTOR slice:

```markdown
**TDD Cycles** (execute one at a time, RED→GREEN→REFACTOR):
1. `test description` → what to implement → refactor opportunity
2. `test description` → what to implement → refactor opportunity
...
```

This replaces flat `**Tests:** ...` sections. Agents execute one cycle at a time, never batch all tests then all implementation (horizontal slicing).

## Benefits

- Issues are version-controlled with the codebase
- No external dependencies or services required
- Perfect for solo projects or private repositories
- Issues can be edited directly in the IDE
