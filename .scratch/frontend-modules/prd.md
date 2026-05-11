# Frontend Modularization PRD

## Problem Statement

The frontend code is currently condensed into a single 459-line `src/app.rs` file containing multiple concerns: domain types, drag & drop utilities, UI components, and main app layout. This monolithic structure makes the code harder to maintain, test, and understand as the application grows.

## Solution

Refactor the frontend code into a modular folder structure that separates concerns while avoiding over-engineering. The structure will group related functionality together while maintaining clean interfaces between modules.

## User Stories

1. As a developer, I want domain types to be separated from UI logic, so that I can easily understand data structures without wading through component code.

2. As a developer, I want drag & drop logic to be isolated as a utility, so that I can reuse it across different components and test it independently.

3. As a developer, I want UI components to be in separate files, so that I can focus on individual component logic without distraction.

4. As a developer, I want the main app file to focus only on layout orchestration, so that I can quickly understand the overall page structure.

5. As a developer, I want consistent module organization with the backend, so that I can navigate between frontend and backend code more easily.

6. As a developer, I want clear re-exports through mod.rs files, so that I can import modules cleanly without deep path knowledge.

7. As a developer, I want components to be testable in isolation, so that I can write focused unit tests for specific UI behavior.

8. As a developer, I want utility functions to be organized in a dedicated module, so that I can reuse them across components and add new utilities as the application grows.

9. As a developer, I want domain types grouped together, so that I can see all data structures in one place without file fragmentation.

10. As a developer, I want the modular structure to avoid over-engineering, so that the file organization remains manageable for a small application.

## Implementation Decisions

### Module Structure
- Create a `domain/` module containing data transfer objects (Board, Column, Task structs)
- Create a `components/` module containing UI components (ColumnView, TaskCard)
- Create a `utils.rs` utility module containing drag & drop logic and MoveTaskArgs, with room for future utility functions
- Keep `app.rs` focused only on the main App component layout

### Module Organization
- Use traditional `mod.rs` pattern with re-exports to match backend structure
- Group domain types in a single `types.rs` file to avoid over-fragmentation
- Treat utilities as cross-cutting logic, starting with drag & drop functionality
- Each UI component gets its own file for focused development

### Interface Design
- Domain types remain as serde-serializable structs for IPC communication
- Utility functions provide clear public interfaces, starting with drag & drop functions (`on_drag_start`, `on_drop`)
- Components accept props and emit events following Leptos patterns
- Main app component orchestrates layout and manages board state

### Import Strategy
- Use re-exports in mod.rs files for clean imports
- Components import from domain and utils modules as needed
- App component imports from components module for layout assembly

## Testing Decisions

### Testing Philosophy
- Test external behavior rather than implementation details
- Focus on component rendering and user interactions
- Test utility functions independently of UI components, starting with drag & drop functionality
- Verify domain type serialization/deserialization

### Module Testing
- Test utility functions with mock data and events, starting with drag & drop functionality
- Test ColumnView component rendering and task addition
- Test TaskCard component rendering and deletion
- Test domain type serialization for IPC compatibility

### Test Organization
- Unit tests for utility functions, starting with drag & drop functionality
- Component tests using Leptos testing utilities
- Integration tests for IPC communication with backend types

## Out of Scope

- Backend code changes (this is purely a frontend refactoring)
- New features or functionality beyond the modularization
- Performance optimizations (though modular structure may enable future optimizations)
- Build system changes or dependency management
- CSS/styling refactoring (focus is on code organization)

## Further Notes

The modular structure provides a foundation for future growth while maintaining simplicity for the current application size. The approach balances maintainability benefits with appropriate complexity for a small task management application.

The utils module is designed to contain reusable cross-cutting concerns, starting with drag & drop functionality but with room to grow as new utility functions are needed for future features.

The domain types are intentionally simple data transfer objects, focusing on IPC communication rather than business logic validation which remains in the backend.
