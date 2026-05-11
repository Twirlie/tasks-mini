---
title: "Refactor Main App Component"
status: "needs-triage"
created: "2025-06-17T12:00:00Z"
updated: "2025-06-17T12:00:00Z"
labels: ["frontend", "refactoring", "app-component", "final-cleanup"]
---

## What to build

Clean up the main `src/app.rs` file to contain only the App component and layout orchestration, removing all extracted domain types, utilities, and UI components. Ensure clean re-exports work through all mod.rs files and the application maintains full functionality.

## Acceptance criteria

- [ ] Remove all extracted code from `src/app.rs` (domain types, utilities, components)
- [ ] Keep only the main App component with layout orchestration logic
- [ ] Ensure all imports work correctly through the new module structure
- [ ] Add integration tests for complete app functionality and IPC communication
- [ ] Verify the entire application compiles and runs successfully after refactoring
- [ ] Confirm consistent module organization with the backend structure

## Blocked by

- #1 Extract Domain Types Module (domain types must be extracted first)
- #2 Extract Utils Module with Drag & Drop (utilities must be extracted first)
- #3 Extract TaskCard Component (TaskCard must be extracted first)
- #4 Extract ColumnView Component (ColumnView must be extracted first)

## User stories covered

4. As a developer, I want the main app file to focus only on layout orchestration, so that I can quickly understand the overall page structure.
5. As a developer, I want consistent module organization with the backend, so that I can navigate between frontend and backend code more easily.
6. As a developer, I want clear re-exports through mod.rs files, so that I can import modules cleanly without deep path knowledge.

## TDD Cycles

1. `test app compilation` → clean up app.rs imports → verify application builds successfully
2. `test app rendering` → ensure App component works → verify board loads and displays correctly
3. `test module re-exports` → verify all imports work → test clean import paths
4. `test integration` → add end-to-end tests → verify complete app functionality including IPC
