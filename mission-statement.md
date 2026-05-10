# mission statement

This is a personal task management application that helps me organize my work and life. Lightweight, fast, and focused on simplicity. Kanban-style interface with drag-and-drop functionality.

Divide tasks into columns - users can create custom columns to customize their workflow:
- Default columns: Backlog, Todo, In Progress, Done
- Users can add, rename, reorder, and delete columns

Tasks themselves have the following properties:
- Title
- Description (optional)
- Due date (optional)
- Tags (optional)
- Status (column name, user-defined)
- Comments (simple text notes, added by user post-task creation. used for adding notes/further information.)
- Created date
- Updated date
- Completed date

## Features

- Create, read, update, delete tasks
- Drag-and-drop tasks between columns
- Manage columns: add, rename, reorder, delete
- Create and switch between multiple workflow boards
- Filter tasks by status
- Search tasks by title or description
- Manual task ordering with drag-and-drop
- Post comments on tasks
- Undo/redo column changes

## Project Outline

### Tech Stack

- Rust
- Leptos
- Tauri
- TailwindCSS

### Platform Support

Cross-platform desktop app:
- Windows
- Linux
- macOS

### Module Architecture

This project uses Modular architecture pattern to organize code into independent, reusable modules. Ports and adapters pattern is used to separate business logic from infrastructure concerns.
Used on frontend and backend. Independently testable and maintainable.

### Testing Strategy

Use TDD (Test-Driven Development) approach. Red-Green-Refactor cycle. Each module should have its own tests and be independently testable. Mock external dependencies where possible.

### Comments

use doc comments for public interfaces and complex logic.

### Task Persistence

See Data Storage section for workflow-based file structure.

### UI/UX

- Clean, minimalistic design
- Dark mode support
- Responsive layout
- Smooth animations for drag-and-drop

### Performance

- Optimize for fast task creation and updates
- Efficient drag-and-drop implementation
- Minimal memory footprint
- Fast search and filter operations

### Security

- No sensitive data storage
- Local file storage only
- No network connectivity required
- No user authentication needed

### Error Handling

Use `tracing` for logging to `~/.local/share/tasks-mini/tasks-mini-<timestamp>.log`. Keep it simple:

- Custom error types with `thiserror` for storage, validation, and UI errors
- Use `Result<T, AppError>` everywhere
- Log technical details, show user-friendly messages in the UI
- Handle file I/O errors gracefully with retries
- Keep backups of task data
- Structured JSON logs for easy debugging

### Data Storage

- Workflow directory: `~/.local/share/tasks-mini/workflow/`
- Workflow master file: `~/.local/share/tasks-mini/workflow/workflows.json` (lists available boards, active board)
- Each workflow: `~/.local/share/tasks-mini/workflow/<workflow-name>/tasks.json`
- Include schema version in files for migration
- Auto-backup before writes: `tasks.json.backup`
- Keep last 5 backups per workflow
- Export workflow: zip the workflow folder

### Data Validation

- Title: required, max 200 chars
- Description: optional, max 2000 chars
- Tags: optional, max 50 chars each, max 10 tags
- Column names: required, max 50 chars, unique per workflow
- Due date: optional, must be valid date

### Migration Strategy

- Schema version in each JSON file
- On load: if version < current, run migration functions
- Migrations are incremental (v1→v2, v2→v3, etc.)
- Backup before migration
- Fail gracefully if migration fails, restore backup

### Search & Filter

- Search: title, description, tags, comments
- Filter by: status, tags, due date
- Manual task ordering with drag-and-drop

### Keyboard Shortcuts (nice to have)

- `n` - new task
- `f` - focus search
- `Esc` - cancel/close
- Arrow keys - navigation
- `Ctrl+Z` - undo
- `Ctrl+Y` / `Ctrl+Shift+Z` - redo
