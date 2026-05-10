---
title: "Board entity module (types + service)"
status: "ready-for-agent"
created: "2026-05-10T08:16:00Z"
updated: "2026-05-10T08:16:00Z"
labels: ["mvp", "enhancement", "ready-for-agent", "module:board"]
---

## Parent

MVP PRD: `.scratch/mvp/prd.md`

## What to build

Board entity module with types, validation, and service logic. The `Board` struct is the top-level container for columns and tasks. Validation enforces unique column names and non-empty board name. Service provides board reading and default board construction (Backlog, Todo, In Progress, Done).

This module replaces the `Board`-related code currently in `domain/mod.rs` — see issue 01.

## Acceptance criteria

- [ ] `Board` struct with id, name, columns, schema_version
- [ ] `BoardError` enum with Validation, DuplicateColumnName, NotFound variants
- [ ] Validation: board name non-empty, column names unique (case-insensitive)
- [ ] `Board::new()` constructor validates inputs
- [ ] `Board::validate()` method checks all invariants
- [ ] Default board construction with 4 columns (Backlog, Todo, In Progress, Done)
- [ ] TDD cycles for types and validation (see Agent Brief)

## Blocked by

- `.scratch/mvp/01c-column-types.md` (board depends on `crate::column::types`)
- `.scratch/mvp/02-storage-port.md` (for service.rs which needs Storage trait)

---

> *This issue was created to split the flat `domain/` module into entity-level modules per the updated architecture in CONTEXT.md.*

## Agent Brief

**Goal:** Create the `board` entity module in `src-tauri/src/board/` with `types.rs`, `service.rs`, and `mod.rs`.

**`types.rs` — types and validation:**
- `Board` struct — id (String/UUID), name (String), columns (Vec<Column>), schema_version (u32). Derive `Serialize`, `Deserialize`, `Clone`, `Debug`.
- `BoardError` enum — `Validation(String)`, `DuplicateColumnName(String)`, `NotFound(String)`. Use `thiserror`.
- `Board::new(name, columns) -> Result<Board, BoardError>` — validates name non-empty, validates column name uniqueness
- `Board::validate(&self) -> Result<(), BoardError>` — checks all invariants (unique column names case-insensitive)
- Import `Column` from `crate::column::types`

**`service.rs` — business logic:**
- `read_board(storage: &dyn Storage) -> Result<Board, BoardError>` — loads board from storage
- `default_board() -> Board` — constructs default board with 4 columns (Backlog, Todo, In Progress, Done) using `Column::new()` and `Board::new()`
- Depends on `crate::storage_port::Storage` and `crate::column::types`

**`mod.rs`** — re-exports `types::*` and `service::*`

**Implementation notes:**
- Migrate Board-related code from `crate::domain` into this module
- `Column` type lives in `crate::column::types` — board depends on column, not the other way around
- Default board columns: Backlog (order 0), Todo (order 1), In Progress (order 2), Done (order 3)
- Schema version starts at 1

**TDD Cycles** (execute one at a time, RED→GREEN→REFACTOR):
1. `Board::new rejects empty name` → validation logic → no refactor
2. `Board::new with valid name and columns succeeds` → constructor → no refactor
3. `Board::new rejects duplicate column names` → uniqueness check → no refactor
4. `Board::new rejects case-insensitive duplicate column names` → case-insensitive check → extract normalize helper
5. `Board::validate returns error for duplicate names` → validate method → no refactor
6. `default_board creates board with 4 columns` → default_board() → no refactor
7. `read_board loads board from storage` → service function → no refactor
