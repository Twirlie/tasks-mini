---
title: "Column entity module (types only)"
status: "done"
created: "2026-05-10T08:16:00Z"
updated: "2026-05-10T08:33:00Z"
labels: ["mvp", "enhancement", "done", "module:column"]
---

## Parent

MVP PRD: `.scratch/mvp/prd.md`

## What to build

Column entity types and validation. This is the types-only module — the service logic lives in issue 04 (`column/service.rs`). The `Column` struct and `ColumnError` enum are defined here so that `board/` and other modules can depend on `crate::column::types`.

This module replaces the `Column`-related code currently in `domain/mod.rs` — see issue 01.

## Acceptance criteria

- [ ] `Column` struct with id, name, order
- [ ] `ColumnError` enum with Validation, NotFound variants
- [ ] Validation: name required, max 50 chars
- [ ] `Column::new()` constructor validates inputs
- [ ] TDD cycles for types and validation (see Agent Brief)

## Blocked by

None — this is a pure types module with no I/O or storage dependencies.

---

> *This issue was created to split the flat `domain/` module into entity-level modules per the updated architecture in CONTEXT.md.*

## Agent Brief

**Goal:** Create the `column` entity module in `src-tauri/src/column/` with `types.rs` and `mod.rs`. Service logic comes later in issue 04.

**`types.rs` — types and validation:**
- `Column` struct — id (String/UUID), name (String), order (u32). Derive `Serialize`, `Deserialize`, `Clone`, `Debug`.
- `ColumnError` enum — `Validation(String)`, `NotFound(String)`. Use `thiserror`.
- `Column::new(name, order) -> Result<Column, ColumnError>` — validates name non-empty and ≤ 50 chars, generates UUID
- Constants: `MAX_COLUMN_NAME_LEN = 50`

**`mod.rs`** — re-exports `types::*`

**Implementation notes:**
- Migrate Column-related code from `crate::domain` into this module
- No storage dependency — this is pure types and validation
- `ColumnError` here covers type-level validation only; service-level errors (Storage, CannotDeleteLastColumn, etc.) go in `service.rs` in issue 04

**TDD Cycles** (execute one at a time, RED→GREEN→REFACTOR):
1. `Column::new rejects empty name` → validation logic → no refactor
2. `Column::new rejects name over 50 chars` → length validation → no refactor
3. `Column::new with valid name succeeds` → constructor + UUID generation → no refactor
4. `Column::new sets correct order value` → order field → no refactor
