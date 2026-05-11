---
title: "Undo/Redo Implementation Gap - No UI/Hotkeys"
labels: ["needs-triage"]
---

# Undo/Redo Implementation Gap

## Problem
The application has backend code for undo/redo functionality, but there is no way for users to access it through the UI or hotkeys.

## Current State
- Backend undo/redo logic exists (confirmed in codebase)
- No keyboard shortcuts implemented (Ctrl+Z, Ctrl+Y, etc.)
- No UI buttons or menu items for undo/redo
- Users cannot access the undo/redo functionality

## Expected Behavior
- Users should be able to undo actions with keyboard shortcuts (Ctrl+Z/Cmd+Z)
- Users should be able to redo actions with keyboard shortcuts (Ctrl+Y/Cmd+Y or Ctrl+Shift+Z/Cmd+Shift+Z)
- Optional: UI buttons/menus for undo/redo for discoverability

## Impact
- Critical usability issue - users cannot recover from mistakes
- Backend functionality is inaccessible
- Poor user experience

## Areas to Investigate
- Frontend hotkey binding implementation
- UI component integration for undo/redo buttons
- Connection between frontend UI and backend undo/redo logic
- State management for undo/redo stack visibility

## Notes
This is a implementation gap, not a bug in existing functionality. The backend work appears to be complete but the frontend integration is missing.
