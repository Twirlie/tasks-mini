---
title: "MVP PRD: Personal Kanban Task Manager"
status: "ready-for-agent"
created: "2026-05-10T07:20:00Z"
updated: "2026-05-10T07:28:00Z"
labels: ["prd", "mvp", "enhancement", "ready-for-agent"]
---

# MVP PRD: Personal Kanban Task Manager

## Problem Statement

The user needs a lightweight, fast, personal task management application to organize work and life. Existing solutions are either too complex (enterprise features, team collaboration) or too simple (basic todo lists without workflow visualization). The user wants a Kanban-style interface with drag-and-drop functionality that stays out of the way and focuses on simplicity.

## Solution

A cross-platform desktop application built with Rust, Tauri, and Leptos. Single-board Kanban with customizable columns. Tasks support titles and optional descriptions. Full drag-and-drop for moving tasks between columns and reordering within columns. Column management with undo/redo. Local file storage with automatic backups.

## User Stories

### Task Management

1. As a user, I want to create a new task with a title, so that I can capture work I need to do
2. As a user, I want to add an optional description to a task, so that I can include additional context or notes
3. As a user, I want to view all my tasks organized by column, so that I can see my workflow at a glance
4. As a user, I want to edit a task's title and description, so that I can update details as work evolves
5. As a user, I want to delete a task, so that I can remove work that is no longer relevant
6. As a user, I want to see when a task was created and last updated, so that I can track work history
7. As a user, I want to see when a task was completed (moved to Done), so that I can reference completion dates

### Column Management

8. As a user, I want to start with default columns (Backlog, Todo, In Progress, Done), so that I have a standard Kanban workflow ready to use
9. As a user, I want to add a new column, so that I can customize my workflow for my specific needs
10. As a user, I want to rename a column, so that I can use terminology that matches my work style
11. As a user, I want to delete a column, so that I can remove workflow stages I don't need
12. As a user, I want tasks from a deleted column to automatically move to the first column (Backlog), so that I don't lose work
13. As a user, I want to undo a column operation (add, rename, delete), so that I can recover from mistakes
14. As a user, I want to redo an undone column operation, so that I can restore a change I accidentally undid

### Drag-and-Drop Workflow

15. As a user, I want to drag a task from one column to another, so that I can visually track progress through my workflow
16. As a user, I want to drag tasks within a column to reorder them, so that I can prioritize work within each stage
17. As a user, I want drag-and-drop to have smooth animations, so that the interface feels responsive and polished
18. As a user, I want the new order to persist after I drop a task, so that my prioritization is remembered

### Data Persistence & Safety

19. As a user, I want my tasks to be saved automatically to local files, so that I don't lose work when I close the app
20. As a user, I want backups created before every write operation, so that I can recover from data corruption
21. As a user, I want the last 5 backups kept per workflow, so that I have multiple recovery points
22. As a user, I want schema versioning in my data files, so that the app can migrate data as the app evolves
23. As a user, I want to store data in `~/.local/share/tasks-mini/`, so that my tasks are in a standard location

### UI/UX

24. As a user, I want a clean, minimalistic design, so that the interface doesn't distract from my work
25. As a user, I want dark mode support, so that I can use the app comfortably in low-light environments
26. As a user, I want the app to follow my system preference for dark/light mode, so that it matches my environment
27. As a user, I want a responsive layout that adapts to my window size, so that I can use the app on different screen sizes

## Implementation Decisions

### Module Architecture

Following the ports and adapters pattern with modular architecture as specified in the mission statement. Deep modules with simple interfaces encapsulate complex functionality.

**Modules to build:**

1. **`domain`** — Core types and business rules
   - Types: `Task`, `Column`, `Board`
   - Validation: title max 200 chars, description max 2000 chars
   - Invariants: column names unique, task order non-negative integers
   - Pure logic, no I/O dependencies

2. **`storage_port`** — Storage interface trait
   - `Storage` trait defining CRUD operations
   - Return type: `Result<T, StorageError>`
   - Enables swapping implementations (JSON, SQLite, etc.)

3. **`json_storage`** — File-based JSON persistence
   - Implements `Storage` trait
   - Data directory: `~/.local/share/tasks-mini/workflow/`
   - File format: `tasks.json` with schema version field
   - Auto-backup: `tasks.json.backup` before writes
   - Keep last 5 backups with timestamps
   - Schema versioning for future migrations

4. **`column_service`** — Column operations with side effects
   - Handles add, rename, delete column operations
   - On delete: moves tasks to first column (Backlog)
   - Validation: column names unique, max 50 chars
   - Delegates persistence to storage adapter

5. **`undo_redo`** — Command pattern for reversible operations
   - Commands: `AddColumn`, `RenameColumn`, `DeleteColumn`
   - History stack with configurable max depth
   - Execute forward/backward
   - Persists to storage after each operation
   - Does NOT contain task-movement logic (handled by column_service)

6. **`tauri_commands`** — Tauri IPC command handlers
   - Thin translation layer from IPC to domain
   - Error handling: convert domain errors to serializable responses
   - No business logic

7. **`ui`** — Leptos frontend components
   - Kanban board view with columns and tasks
   - Task form (create/edit)
   - Column management UI
   - Undo/redo buttons
   - Drag-and-drop integration
   - Dark mode toggle

### Data Model

```
Board {
  id: String,
  name: String,
  columns: Vec<Column>,
  schema_version: u32,
}

Column {
  id: String,
  name: String,
  order: u32,
}

Task {
  id: String,
  title: String,
  description: Option<String>,
  column_id: String,
  order: u32,
  created_at: DateTime<Utc>,
  updated_at: DateTime<Utc>,
  completed_at: Option<DateTime<Utc>>,
}
```

### API Contracts

**Tauri Commands:**
- `get_board()` -> Result<Board, Error>
- `create_task(title, description, column_id)` -> Result<Task, Error>
- `update_task(id, title, description)` -> Result<Task, Error>
- `delete_task(id)` -> Result<(), Error>
- `move_task(id, column_id, order)` -> Result<Task, Error>
- `add_column(name)` -> Result<Column, Error>
- `rename_column(id, name)` -> Result<Column, Error>
- `delete_column(id)` -> Result<(), Error>
- `undo()` -> Result<(), Error>
- `redo()` -> Result<(), Error>

### Error Handling

- Custom error types using `thiserror`
- Storage errors: IO, serialization, validation
- Domain errors: validation failures, not found
- UI shows user-friendly messages, logs technical details with `tracing`

## Testing Decisions

### Testing Philosophy

- Test external behavior, not implementation details
- Each module has its own tests and is independently testable
- Mock external dependencies where possible
- Use TDD (Test-Driven Development) approach: Red-Green-Refactor cycle

### Module Test Coverage

**`domain`** — Full unit test coverage
- Validation rules (title length, description length)
- Business invariants (unique column names, task ordering)
- Edge cases: empty strings, boundary values

**`storage_port`** — Trait verification via mock implementations
- Mock storage for testing dependent modules
- Verify contract compliance

**`json_storage`** — Integration tests with temp directories
- Read/write roundtrips
- Backup creation and rotation
- Schema versioning
- Error handling: corrupt files, permission errors

**`column_service`** — Unit tests with mocked storage
- Add column validation
- Rename column (including duplicate name handling)
- Delete column (task movement behavior)
- Error propagation

**`undo_redo`** — Unit tests
- Execute command
- Undo single operation
- Redo undone operation
- Stack depth limits
- History persistence

**`tauri_commands`** — Minimal tests (thin layer)
- Verify error conversion
- Mock storage for integration tests

**`ui`** — Component tests where feasible
- Form validation
- State updates

### Test Structure

- Tests co-located with source: `#[cfg(test)]` modules
- Integration tests in `tests/` directory
- Test utilities and mocks in `test_utils/` module

## Out of Scope

The following features are explicitly excluded from MVP and planned for future versions:

- **Multiple workflow boards** — Single board only for MVP
- **Tags** — Task categorization via tags
- **Due dates** — Date tracking on tasks
- **Comments** — Post-task creation notes
- **Search and filter** — Full-text search and filtering by status/tags
- **Keyboard shortcuts** — `n` for new task, `f` for search, arrow navigation, etc.
- **Column reordering** — Drag to reorder columns (add/delete/rename only)
- **Undo/redo for tasks** — Only column operations have undo/redo
- **Export functionality** — ZIP export of workflow
- **Data migration UI** — Automatic migrations only, no manual migration tools

## Further Notes

### Security Considerations

- No sensitive data storage (no auth, no personal info)
- Local file storage only, no network connectivity
- Standard filesystem permissions apply

### Performance Targets

- Task creation: < 100ms from click to visible task
- Drag-and-drop: 60fps animations, < 50ms response to drop
- Search: N/A for MVP (deferred)
- Memory: Minimal footprint, lazy loading not needed for single-board personal use

### Future Considerations

The modular architecture and schema versioning are designed to support:
- Multi-board feature (Board becomes a first-class entity)
- Tags and due dates (extend Task type)
- Comments (new entity type)
- Sync/backup to cloud (new storage adapter)

### Documentation

- Doc comments for all public interfaces
- Complex logic explained inline
- This PRD serves as the high-level specification
