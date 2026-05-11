# Architecture Improvements

This document outlines key architectural concerns and proposed improvements for the Tasks Mini codebase.

## 1. Board Aggregate Module

- **Concern**: Board entity module is shallow, with business logic scattered across individual services
- **Problem**: Board-level invariants (column uniqueness, cross-entity operations) are distributed across task/column services, making it hard to maintain board-wide business rules
- **Solution**: Create a deep Board aggregate module that centralizes all board-level operations and invariants
- **Impact**: High - Improves business logic locality and reduces cross-module coupling
- **Complexity Estimation**: Medium - Requires refactoring existing service calls

## 2. IPC Command Layer

- **Concern**: Tauri command handlers contain duplicated business logic and error handling
- **Problem**: Command handlers leak complexity with inline error translation, undo/redo wrapping, and result post-processing
- **Solution**: Create a dedicated command_layer module with generic handlers and centralized error translation
- **Impact**: High - Eliminates code duplication and standardizes IPC patterns
- **Complexity Estimation**: Low-Medium - Mostly refactoring existing handlers

## 3. Frontend Service Layer

- **Concern**: Frontend components directly call Tauri IPC with duplicated async logic
- **Problem**: Error handling, loading states, and data transformation are scattered across components
- **Solution**: Introduce frontend service layer (board_service, task_service) with centralized business logic
- **Impact**: High - Reduces component complexity and improves testability
- **Complexity Estimation**: Medium - Requires extracting logic from components

## 4. Drag & Drop Utilities

- **Concern**: Drag & drop logic is embedded in components, making it hard to test and reuse
- **Problem**: UI concerns are mixed with drag logic, limiting reusability across components
- **Solution**: Create dedicated drag_drop utility module with pure functions and state management
- **Impact**: Medium - Enhances reusability and enables isolated testing
- **Complexity Estimation**: Low - Mostly extracting existing logic

## 5. Storage Port Interface

- **Concern**: Storage trait mixes high-level board operations with low-level entity CRUD
- **Problem**: Interface doesn't provide meaningful abstraction, creating a shallow port
- **Solution**: Split Storage into focused interfaces (BoardStorage, EntityStorage, MigrationStorage)
- **Impact**: Medium - Improves adapter focus and enables better test doubles
- **Complexity Estimation**: Medium - Requires updating all storage implementations
