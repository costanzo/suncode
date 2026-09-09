# Requirement

## Background

Workspace navigation, recently viewed content, central content selection, window geometry, and Settings navigation currently reset when their windows are recreated. Users should return to the project and Settings locations they last used.

## Goals

- Restore recently viewed files and sessions independently for each project.
- Restore the selected central file or session independently for each project.
- Restore extensible left, right, and bottom Workspace regions independently for each project.
- Restore Workspace window position, size, state, and resizable panel dimensions independently for each project.
- Restore the last global Settings destination, including a selected provider destination.
- Keep frequent UI changes in memory and persist them with bounded, atomic writes.

## Non-goals

- Automatically reopening projects at application startup.
- Persisting file contents, session messages, prompts, credentials, transient loading state, selections inside review tools, or Explorer expansion state.
- Moving desktop presentation state into agent SQLite configuration.
- Changing Workspace or Settings visual styling.

## Requirements

1. Desktop UI state is stored in one versioned JSON file under the SunCode application data directory.
2. The application reads the file once and shares one in-memory state store across desktop windows.
3. Changes are debounced, periodically bounded, and flushed when relevant windows or the application close.
4. Writes use a temporary file and atomic replacement so interruption cannot leave a partially written state file.
5. Left Workspace state is represented by `closed`, `sessions`, or `explorer` rather than independent booleans.
6. Right Workspace state is represented by `closed` or `review`, leaving room for future sidebar kinds.
7. Bottom Workspace state is represented by `closed`, `git`, `providerTrace`, or `toolActivity`.
8. Normal window position and dimensions, the final maximized/full-screen state, navigation width, review width, and bottom-drawer height are project-scoped.
9. Restored window geometry is clamped to a currently available display working area.
10. Recent content remains most-recent-first, stable-identity deduplicated, and limited to 20 entries per project.
11. File identities contain only a registered dependency ID when applicable and a root-relative path. Session identities contain only the SDK-issued session ID.
12. Settings restores the last first-level page, the provider subsection when applicable, and provider navigation expansion.
13. Missing, invalid, stale, corrupt, or future state falls back without preventing application or project startup.

## Edge cases

- A removed display must not leave a restored window off screen.
- Maximized and full-screen windows retain their last normal bounds for a later return to normal state.
- Removed or archived sessions are omitted and the first available session is used as a fallback.
- Removed dependencies invalidate their file entries.
- A missing file may retain its identity long enough to show the existing bounded read error, but a failed restored current file falls back to a valid session.
- Unknown sidebar, drawer, page, or content kinds fall back to defaults.
- A corrupt JSON file is preserved as a timestamped local backup before defaults are used.

## Acceptance criteria

- Reopening a project restores its last valid window and Workspace presentation state.
- The Workspace content switcher is populated after application restart and remains project-specific.
- Reopening Settings returns to the last valid destination, including Appearance and provider pages.
- Rapid resize and navigation changes do not cause one disk write per UI event.
- Focused serialization, restoration, fallback, geometry, and bounded-history tests pass.

## Open questions

- None.
