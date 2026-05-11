# ADR 0001: Frontend Modularization

## Status

Accepted

## Context

The frontend code is currently condensed into a single 459-line `src/app.rs` file containing multiple concerns:
- Domain types (Board, Column, Task structs)
- Drag & drop utilities and logic
- UI components (ColumnView, TaskCard, DraggableTask, DropZone)
- Main app layout

This monolithic structure makes the code harder to maintain, test, and understand as the application grows.

## Decision

Adopt a modular folder structure that separates concerns while avoiding over-engineering:

```
src/
├── domain/
│   ├── mod.rs          // re-exports all types
│   ├── types.rs        // Board, Column, Task structs all in one file
├── components/
│   ├── mod.rs          // re-exports all components  
│   ├── column_view.rs  // ColumnView component
│   ├── task_card.rs    // TaskCard component
├── drag_drop.rs        // utilities + MoveTaskArgs
└── app.rs             // just the App component layout
```

### Module Responsibilities

- **domain/types.rs**: Contains only data transfer objects (Board, Column, Task structs). These are simple serde-serializable structs used for IPC communication with the Tauri backend.
- **drag_drop.rs**: Contains all drag & drop logic including utilities (`on_drag_start`, `on_drop`, `move_task_with_data`) and the `MoveTaskArgs` struct. Treated as a utility, not a component.
- **components/column_view.rs**: Contains the `ColumnView` component responsible for rendering a single column with its tasks and add task functionality.
- **components/task_card.rs**: Contains the `TaskCard` component responsible for rendering individual tasks with delete functionality.
- **app.rs**: Contains only the main `App` component that orchestrates the overall layout and manages the board state.

### Module Structure Pattern

Using the traditional `mod.rs` pattern with re-exports to match the backend structure:

```rust
// domain/mod.rs
pub mod types;
pub use types::*;

// components/mod.rs  
pub mod column_view;
pub mod task_card;
pub use column_view::*;
pub use task_card::*;
```

## Consequences

### Positive

- **Clear separation of concerns**: Each module has a single responsibility
- **Maintainability**: Easier to locate and modify specific functionality
- **Testability**: Components can be tested in isolation
- **Consistency**: Mirrors backend module structure for familiarity
- **Avoids over-engineering**: Groups related types together rather than excessive file fragmentation

### Negative

- **More files**: Increases file count compared to monolithic approach
- **Import complexity**: Requires managing module imports and re-exports

### Neutral

- **Learning curve**: Developers need to understand the modular structure
- **Build time**: Minimal impact due to small file sizes

## Rationale

This approach balances the benefits of modularization with simplicity:

1. **Domain types grouped**: Since frontend types are simple data transfer objects, grouping them in one file avoids unnecessary fragmentation while still separating them from UI logic.

2. **Components separated**: Each UI component gets its own file for focused development and testing.

3. **Drag & drop as utility**: Drag & drop is treated as cross-cutting utility logic rather than a component, making it reusable and keeping components focused on presentation.

4. **Layout-only app.rs**: The main app component focuses solely on layout orchestration, making it easier to understand the overall page structure.

5. **Consistent with backend**: Uses the same `mod.rs` re-export pattern as the backend for developer familiarity.

## Implementation Steps

1. Create `domain/types.rs` with Board, Column, Task structs
2. Create `domain/mod.rs` to re-export domain types
3. Create `drag_drop.rs` with utilities and MoveTaskArgs
4. Create `components/column_view.rs` with ColumnView component
5. Create `components/task_card.rs` with TaskCard component
6. Create `components/mod.rs` to re-export components
7. Update `app.rs` to only contain App component layout with new imports

## Future Considerations

- As components grow more complex, consider splitting them further (e.g., `components/task/` with multiple files)
- If shared utilities grow beyond drag & drop, consider a `utils/` module
- The structure allows for easy addition of new components and domain types
