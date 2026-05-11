# Tasks Mini — Domain Context

## Product

Personal task management desktop app with Kanban-style interface and drag-and-drop functionality. Lightweight, fast, simple. No network, no auth, no sensitive data. Local-first design with JSON file persistence.

## Core Domain Types

- **Board** — a workflow board containing columns and tasks. Has a `schema_version` for migration. One active board at a time.
- **Column** — a status lane within a board (e.g. Backlog, Todo, In Progress, Done). Has unique name per board (case-insensitive), max 50 chars. Ordered by `order` field.
- **Task** — a unit of work. Belongs to one column. Has title (required, max 200 chars), description (optional, max 2000 chars), and timestamps (created, updated, completed). Ordered within its column by `order` field. **Note**: No due dates, tags, or comments are currently implemented.

## Architecture

### Pattern

Ports and adapters (hexagonal). Business logic in pure Rust modules; infrastructure behind trait boundaries. Frontend-backend communication via Tauri IPC.

### Module layout (backend — `src-tauri/src/`)

Each entity owns its types, validation, errors, and service logic in a self-contained module. Services depend on the `Storage` trait, never on concrete adapters.

- `task/` — Task entity module
  - `types.rs` — `Task` struct, `TaskError`, validation (title required ≤ 200 chars, description ≤ 2000 chars)
  - `service.rs` — `create_task`, `update_task`, `delete_task`, `move_task` (sets `completed_at` when moved to last column)
  - `mod.rs` — re-exports public API
- `column/` — Column entity module
  - `types.rs` — `Column` struct, `ColumnError`, validation (name required ≤ 50 chars, unique per board)
  - `service.rs` — `add_column`, `rename_column`, `delete_column` (relocates tasks to first column, prevents deletion of last column)
  - `mod.rs` — re-exports public API
- `board/` — Board entity module
  - `types.rs` — `Board` struct, `BoardError`, validation (unique column names, non-empty name)
  - `service.rs` — `default_board` construction (Backlog, Todo, In Progress, Done)
  - `mod.rs` — re-exports public API
- `domain/` — **DUPLICATE**: Contains domain types that mirror backend types for frontend serialization. Should be consolidated.
- `storage_port/` — `Storage` trait (port) defining all persistence operations, `StorageError`, `MockStorage` for testing.
- `json_storage/` — JSON file adapter implementing `Storage`. Data at `~/.local/share/tasks-mini/` (debug mode uses temp dir). Auto-backup before writes, last 5 kept.
- `undo_redo/` — Command-pattern implementation for task and column operations. Includes `History`, `Command` trait, and specific command implementations.
- `lib.rs` — Tauri IPC command handlers (thin translation layer, no business logic). Manages `JsonStorage` and `History` state.

### Frontend (`src/`)

Leptos 0.8 CSR (client-side rendering). Modular structure following ADR 0001. Communicates with Tauri backend via `invoke()` IPC.

- `domain/` — Frontend DTOs (Board, Column, Task) with string timestamps for JSON serialization
- `components/` — UI components
  - `column_view.rs` — Column rendering with task list and add task form
  - `task_card.rs` — Individual task display with delete functionality
  - `theme_button.rs` — Dark/light theme toggle
  - `undo_redo.rs` — Undo/redo controls
- `utils/` — Cross-cutting utilities
  - `keyboard.rs` — Keyboard shortcuts setup
  - `theme.rs` — Theme management
  - **Missing**: Drag & drop utilities (referenced but not found)
- `app.rs` — Main application component with board state management and layout orchestration

### Cross-cutting

- Error handling: `thiserror` custom error types, `Result<T, DomainError>` pattern. `tracing` for structured logging.
- IDs: `uuid::Uuid` v4 for all entities.
- Timestamps: `chrono::DateTime<Utc>` in backend, `String` in frontend for JSON serialization.
- Serde: All domain types derive `Serialize`, `Deserialize`.
- Testing: Comprehensive unit tests for all modules using `tokio::test` and `wasm_bindgen_test` for frontend.

## Tech Stack

- **Backend**: Rust, Tauri 2, async/await with `tokio`
- **Frontend**: Rust, Leptos 0.8 (CSR), TailwindCSS, WebAssembly via `wasm-bindgen`
- **Build**: Trunk (frontend bundler), Cargo workspace
- **Testing**: `wasm-bindgen-test` for frontend, `tokio::test` for backend

## Platform

Cross-platform desktop: Windows, Linux, macOS. Debug mode uses temporary directory to avoid development server conflicts.

## Key Behaviours

- Drag-and-drop reorders tasks within and across columns (partially implemented - missing utilities).
- Deleting a column relocates its tasks to the first column; cannot delete the last column.
- Schema versioning in JSON files (currently v1) with incremental migrations and pre-migration backup.
- **Undo/redo for all operations**: Tasks (create, update, delete, move) and columns (add, rename, delete).
- Default board ships with four columns: Backlog, Todo, In Progress, Done.
- **Auto-completion**: Tasks moved to the last column automatically get `completed_at` timestamp set.
- **Theme switching**: Dark/light mode with system preference detection.

## Data Storage

- **Location**: `~/.local/share/tasks-mini/tasks.json` (production), temp directory in debug mode
- **Format**: JSON with schema versioning
- **Backup**: Automatic backup before each write, keeps last 5 versions with timestamps
- **Recovery**: Default board created if no file exists

## IPC Commands (Tauri)

- `get_board` — Load current board state
- `create_task(title, description?, column_id)` — Create new task
- `update_task(id, title?, description?)` — Update existing task
- `delete_task(id)` — Delete task (via undo/redo system)
- `move_task(id, column_id, order)` — Move/reorder task (via undo/redo system)
- `add_column(name)` — Add new column (via undo/redo system)
- `undo` — Undo last operation
- `redo` — Redo last undone operation
- `greet` — Development/test command

## Architecture Issues & Technical Debt

1. **Domain Type Duplication**: Backend and frontend both define Board/Column/Task types, creating maintenance overhead
2. **Missing Frontend Utilities**: Drag & drop utilities referenced in components but not implemented
3. **Inconsistent Error Handling**: Frontend uses different error patterns than backend
4. **Incomplete Feature Set**: Due dates, tags, and comments documented but not implemented
5. **Board Service Mismatch**: CONTEXT.md documents `read_board` but implementation only has `default_board`

## Development Workflow

- **TDD Required**: All feature development must follow test-driven development per AGENTS.md
- **Tauri Dev Server**: Assume server is running, do not start additional instances
- **Modular Frontend**: Follow ADR 0001 structure for all new components
- **Command Pattern**: All mutations go through undo/redo system for consistency

## Terminology

| Term | Meaning |
|---|---|
| Board | A workflow board — the top-level container for columns and tasks |
| Column | A status lane within a board |
| Task | A unit of work living in a column |
| Storage | The persistence trait (port) — not the concrete implementation |
| JsonStorage | The JSON file adapter that implements Storage |
| MockStorage | In-memory test double that implements Storage |
| History | Command-pattern undo/redo manager with configurable depth (default 50) |
| `task/` module | Task entity — types, validation, service (CRUD + reordering) |
| `column/` module | Column entity — types, validation, service (CRUD + task relocation) |
| `board/` module | Board entity — types, validation, service (default construction only) |
| Undo/Redo | Command-pattern history for all mutations (tasks and columns) |
| Schema version | Integer in JSON files driving incremental migration (currently v1) |
| IPC | Inter-Process Communication between frontend (Leptos) and backend (Tauri) |
