---
title: "Extract Domain Types Module"
status: "needs-triage"
created: "2025-06-17T12:00:00Z"
updated: "2025-06-17T12:00:00Z"
labels: ["frontend", "refactoring", "domain-types"]
---

## What to build

Extract the domain types (Board, Column, Task structs) from the monolithic `src/app.rs` file into a dedicated `src/domain/` module. This creates a clean separation between data structures and UI logic, making the codebase more maintainable and testable.

## Acceptance criteria

- [ ] Create `src/domain/mod.rs` with re-exports
- [ ] Create `src/domain/types.rs` containing Board, Column, Task structs with serde serialization
- [ ] Update `src/app.rs` to import domain types from the new module
- [ ] Add unit tests for domain type serialization/deserialization
- [ ] Verify the application still compiles and runs after refactoring

## Blocked by

None - can start immediately

## User stories covered

1. As a developer, I want domain types to be separated from UI logic, so that I can easily understand data structures without wading through component code.
9. As a developer, I want domain types grouped together, so that I can see all data structures in one place without file fragmentation.
10. As a developer, I want the modular structure to avoid over-engineering, so that the file organization remains manageable for a small application.

## TDD Cycles

1. `test domain type serialization` → extract Board struct with serde derives → verify JSON round-trip works
2. `test domain type deserialization` → extract Column and Task structs → ensure all types serialize correctly  
3. `test module imports` → create mod.rs with re-exports → verify app.rs can import and use types
4. `test compilation` → update imports in app.rs → ensure application builds and runs successfully
