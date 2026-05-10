# Tasks Mini — Domain Context

## Product

Personal task management desktop app. Kanban-style interface with drag-and-drop. Lightweight, fast, simple. No network, no auth, no sensitive data.

## Core Domain Types

- **Board** — a workflow board containing columns and tasks. Has a `schema_version` for migration. One active board at a time.
- **Column** — a status lane within a board (e.g. Backlog, Todo, In Progress, Done). Has unique name per board, max 50 chars. Ordered by `order` field.
- **Task** — a unit of work. Belongs to one column. Has title (required, max 200 chars), description (optional, max 2000 chars), due date, tags, comments, and timestamps (created, updated, completed). Ordered within its column by `order` field.

## Architecture

### Pattern

Ports and adapters (hexagonal). Business logic in pure Rust modules; infrastructure behind trait boundaries.

### Module layout (backend — `src-tauri/src/`)

Each entity owns its types, validation, errors, and service logic in a self-contained module. Services depend on the `Storage` trait, never on concrete adapters.

- `task/` — Task entity module
  - `types.rs` — `Task` struct, `TaskError`, validation (title required ≤ 200 chars, description ≤ 2000 chars)
  - `service.rs` — `create_task`, `update_task`, `delete_task`, `move_task` (sets `completed_at` when moved to Done)
  - `mod.rs` — re-exports public API
- `column/` — Column entity module
  - `types.rs` — `Column` struct, `ColumnError`, validation (name required ≤ 50 chars, unique per board)
  - `service.rs` — `add_column`, `rename_column`, `delete_column` (relocates tasks to first column)
  - `mod.rs` — re-exports public API
- `board/` — Board entity module
  - `types.rs` — `Board` struct, `BoardError`, validation (unique column names, non-empty name)
  - `service.rs` — `read_board`, default board construction (Backlog, Todo, In Progress, Done)
  - `mod.rs` — re-exports public API
- `storage_port/` — `Storage` trait (port) defining all persistence operations, `StorageError`, `MockStorage` for testing.
- `json_storage/` — JSON file adapter implementing `Storage`. Data at `~/.local/share/tasks-mini/workflow/`. Auto-backup before writes, last 5 kept.
- `undo_redo/` — undo/redo for column changes. Depends on `column::service`.
- `lib.rs` — Tauri IPC command handlers (thin translation layer, no business logic).

### Frontend (`src/`)

Leptos CSR (client-side rendering). Communicates with Tauri backend via `invoke()` IPC.

### Cross-cutting

- Error handling: `thiserror` custom error types, `Result<T, AppError>` everywhere. `tracing` for structured JSON logs.
- IDs: `uuid::Uuid` v4.
- Timestamps: `chrono::DateTime<Utc>`.
- Serde: all domain types derive `Serialize`, `Deserialize`.

## Tech Stack

- **Backend**: Rust, Tauri 2
- **Frontend**: Rust, Leptos 0.8 (CSR), TailwindCSS
- **Build**: Trunk (frontend bundler), Cargo workspace

## Platform

Cross-platform desktop: Windows, Linux, macOS.

## Key Behaviours

- Drag-and-drop reorders tasks within and across columns.
- Deleting a column relocates its tasks to the first column; cannot delete the last column.
- Schema versioning in JSON files with incremental migrations and pre-migration backup.
- Undo/redo for column changes.
- Default board ships with four columns: Backlog, Todo, In Progress, Done.

## Terminology

| Term | Meaning |
|---|---|
| Board | A workflow board — the top-level container for columns and tasks |
| Column | A status lane within a board |
| Task | A unit of work living in a column |
| Storage | The persistence trait (port) — not the concrete implementation |
| JsonStorage | The JSON file adapter that implements Storage |
| MockStorage | In-memory test double that implements Storage |
| `task/` module | Task entity — types, validation, service (CRUD + reordering) |
| `column/` module | Column entity — types, validation, service (CRUD + task relocation) |
| `board/` module | Board entity — types, validation, service (read + default construction) |
| Undo/Redo | Command-pattern history for column mutations |
| Schema version | Integer in JSON files driving incremental migration |
