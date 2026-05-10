---
title: "Task entity module (types only)"
status: "ready-for-agent"
created: "2026-05-10T08:16:00Z"
updated: "2026-05-10T08:16:00Z"
labels: ["mvp", "enhancement", "ready-for-agent", "module:task"]
---

## Parent

MVP PRD: `.scratch/mvp/prd.md`

## What to build

Task entity types and validation. This is the types-only module — the service logic lives in issue 06 (`task/service.rs`). The `Task` struct and `TaskError` enum are defined here so that `column/` and other modules can depend on `crate::task::types`.

This module replaces the `Task`-related code currently in `domain/mod.rs` — see issue 01.

## Acceptance criteria

- [ ] `Task` struct with id, title, description, column_id, order, created_at, updated_at, completed_at
- [ ] `TaskError` enum with Validation, NotFound variants
- [ ] Validation: title required max 200 chars, description optional max 2000 chars
- [ ] `Task::new()` constructor validates inputs and generates UUID + timestamps
- [ ] TDD cycles for types and validation (see Agent Brief)

## Blocked by

None — this is a pure types module with no I/O or storage dependencies.

---

> *This issue was created to split the flat `domain/` module into entity-level modules per the updated architecture in CONTEXT.md.*

## Agent Brief

**Goal:** Create the `task` entity module in `src-tauri/src/task/` with `types.rs` and `mod.rs`. Service logic comes later in issue 06.

**`types.rs` — types and validation:**
- `Task` struct — id (String/UUID), title (String), description (Option<String>), column_id (String), order (u32), created_at (DateTime<Utc>), updated_at (DateTime<Utc>), completed_at (Option<DateTime<Utc>>). Derive `Serialize`, `Deserialize`, `Clone`, `Debug`.
- `TaskError` enum — `Validation(String)`, `NotFound(String)`. Use `thiserror`.
- `Task::new(title, description, column_id, order) -> Result<Task, TaskError>` — validates title non-empty and ≤ 200 chars, description ≤ 2000 chars, generates UUID, sets timestamps
- Constants: `MAX_TITLE_LEN = 200`, `MAX_DESCRIPTION_LEN = 2000`

**`mod.rs`** — re-exports `types::*`

**Implementation notes:**
- Migrate Task-related code from `crate::domain` into this module
- No storage dependency — this is pure types and validation
- `TaskError` here covers type-level validation only; service-level errors (Storage, TaskNotFound, ColumnNotFound, etc.) go in `service.rs` in issue 06

**TDD Cycles** (execute one at a time, RED→GREEN→REFACTOR):
1. `Task::new rejects empty title` → validation logic → no refactor
2. `Task::new rejects title over 200 chars` → length validation → no refactor
3. `Task::new rejects description over 2000 chars` → description validation → no refactor
4. `Task::new with valid inputs succeeds` → constructor + UUID + timestamps → no refactor
5. `Task::new with no description sets None` → optional field → no refactor
6. `Task::new sets order value correctly` → order field → no refactor
