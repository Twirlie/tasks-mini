---
title: "Extract ColumnView Component"
status: "needs-triage"
created: "2025-06-17T12:00:00Z"
updated: "2025-06-17T12:00:00Z"
labels: ["frontend", "refactoring", "components", "columnview"]
---

## What to build

Extract the ColumnView component from `src/app.rs` into a dedicated `src/components/column_view.rs` file. This isolates the column UI logic including task addition, rendering, and drag & drop integration.

## Acceptance criteria

- [ ] Create `src/components/column_view.rs` with the ColumnView component
- [ ] Update `src/components/mod.rs` to re-export ColumnView
- [ ] Update `src/app.rs` to import ColumnView from the components module
- [ ] Add component tests for ColumnView rendering and task addition functionality
- [ ] Verify the ColumnView component still renders and functions correctly after refactoring

## Blocked by

- #1 Extract Domain Types Module (needs domain types for Column and Task structs)
- #2 Extract Utils Module with Drag & Drop (needs DropZone component)
- #3 Extract TaskCard Component (needs TaskCard component)

## User stories covered

3. As a developer, I want UI components to be in separate files, so that I can focus on individual component logic without distraction.
7. As a developer, I want components to be testable in isolation, so that I can write focused unit tests for specific UI behavior.

## TDD Cycles

1. `test ColumnView rendering` → extract ColumnView component → verify component renders with column and tasks
2. `test task addition` → extract add task functionality → ensure new task form works correctly
3. `test component imports` → update components mod.rs → verify app.rs can import ColumnView
4. `test drag integration` → ensure DropZone and TaskCard integration works → verify drag & drop still functions
