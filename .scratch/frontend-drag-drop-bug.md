---
title: "Frontend Drag and Drop Bug"
type: "bug"
priority: "high"
status: "open"
component: "frontend"
created: "2026-05-11"
---

# Frontend Drag and Drop Bug

## Issue Description
Tasks can be picked up and dragged, but dropping them does nothing. The drop functionality is not working properly.

## Steps to Reproduce
1. Pick up a task (this works)
2. Drag the task to a new location
3. Drop the task
4. Observe that nothing happens - the task is not placed in the new location

## Expected Behavior
When dropping a task, it should be placed in the new location and the UI should update accordingly.

## Actual Behavior
Dropping a task has no effect - the task is not moved and the UI remains unchanged.

## Environment
- Frontend: Leptos application
- Backend: Tauri
- Issue location: Frontend drag and drop implementation

## Priority
High - Core functionality is broken

## Notes
- Task pickup works correctly
- Dragging works correctly  
- Only the drop functionality is affected
- This appears to be a frontend issue, not backend
