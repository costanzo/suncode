# Requirement

## Background

Project Explorer currently exposes file nodes without a content view. Developers need to inspect a project file without leaving the active desktop project window or losing the current session context.

## Goals

- Open a selected project or registered dependency file in the Workspace center area.
- Provide a read-only syntax-highlighted viewing surface backed by AvaloniaEdit and TextMate in the production client.
- Let users return to conversation history by selecting a session from the left navigation.
- Establish the interaction and visual contract in the design system before production implementation.

## Non-goals

- Editing, saving, formatting, searching, replacing, or language-server features.
- Changing dependency authority; dependency files remain read-only inputs.
- Adding a separate editor window or a production web editor.

## Requirements

1. Clicking a file node in Explorer selects it and replaces the central Conversation view with the file viewer.
2. The viewer displays the file name/path, detected language, read-only status, line numbers, selectable text, syntax highlighting, and horizontal scrolling for long lines.
3. Selecting a session from the left session list restores that session's Conversation view.
4. Empty, loading, read failure, and constrained-width states remain explicit and non-editable.
5. AvaloniaEdit and TextMate are production dependencies only after approval; the design system is specification tooling.

## Edge cases

- A file disappears or cannot be read: show a bounded read-failure state without exposing an editable control.
- An empty file: show the file identity and an empty-document state while retaining the read-only affordance.
- Very long lines or deeply nested paths: preserve horizontal inspection rather than resizing the Workspace.
- A session is selected while a file is open: clear the file view and restore the selected session conversation.

## Acceptance criteria

- The Workspace design-system navigation contains an `Editor` child route.
- The route demonstrates ready, empty, loading, read-failure, and constrained-width states in both themes.
- The Workspace overview demonstrates Explorer-to-editor and session-to-conversation transitions.
- Production implementation remains untouched until this draft is confirmed.

## Open questions

- Confirm the exact production package versions and TextMate language registry coverage during implementation.
