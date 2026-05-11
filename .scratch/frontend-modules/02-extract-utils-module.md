---
title: "Extract Utils Module with Drag & Drop"
status: "needs-triage"
created: "2025-06-17T12:00:00Z"
updated: "2025-06-17T12:00:00Z"
labels: ["frontend", "refactoring", "utils", "drag-drop"]
---

## What to build

Extract drag & drop utilities and MoveTaskArgs from `src/app.rs` into a dedicated `src/utils.rs` module, consolidating with the existing `src/drag_drop.rs`. This isolates cross-cutting utility logic and makes it reusable across components.

## Acceptance criteria

- [ ] Consolidate drag & drop functions from `app.rs` and `drag_drop.rs` into `src/utils.rs`
- [ ] Move MoveTaskArgs struct to utils module
- [ ] Update `src/app.rs` to import utilities from the new module
- [ ] Add unit tests for drag & drop utility functions with mock data
- [ ] Verify drag & drop functionality still works after refactoring
- [ ] Remove or deprecate the old `src/drag_drop.rs` file

## Blocked by

- #1 Extract Domain Types Module (needs domain types for MoveTaskArgs)

## User stories covered

2. As a developer, I want drag & drop logic to be isolated as a utility, so that I can reuse it across different components and test it independently.
8. As a developer, I want utility functions to be organized in a dedicated module, so that I can reuse them across components and add new utilities as the application grows.

## TDD Cycles

1. `test drag_start function` → consolidate on_drag_start function → verify data transfer format
2. `test drop handling` → consolidate on_drop function → test with mock drag events
3. `test MoveTaskArgs serialization` → move struct to utils → verify IPC compatibility
4. `test component imports` → update app.rs imports → ensure drag & drop components work
