# Agent Configuration

## Agent skills

### Issue tracker

GitHub issues using the `gh` CLI. See `docs/agents/issue-tracker.md`.

### Triage labels

Default triage label vocabulary. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context layout with root `CONTEXT.md` and `docs/adr/`. See `docs/agents/domain.md`.

## Development discipline

### TDD (mandatory)

All feature implementation MUST follow Test-Driven Development using the `tdd` skill:

1. **RED first**: Write one failing test for the next behavior
2. **GREEN**: Write minimal code to make it pass
3. **REFACTOR**: Clean up while staying green
4. **Repeat**: One vertical slice at a time — never batch all tests then all implementation

Agents MUST NOT:
- Write implementation before a failing test exists
- Write multiple tests before making each one pass (horizontal slicing)
- Skip the refactor step after a cycle completes
- Treat "Tests" as a final checklist item instead of the driving force

### Tauri Dev Server usage

Assume Tauri Dev Server is already running, DO NOT start your own. If you find it is not running, ask the user to start it. If you need to test changes, use the existing server. Tauri MCP can be used to view and interact with the frontend.

