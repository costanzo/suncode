# Architecture

## Current state

Avalonia owns Workspace and Settings presentation state in each window and ViewModel. Agent SQLite owns durable agent, project, session, configuration, and operation state. The recent-content switcher retains at most 20 entries only for the project-window lifetime.

## Proposed design

Add a desktop-owned `UiStateStore` backed by `<data-directory>/ui-state.json`. It loads a versioned document once, returns immutable state snapshots, accepts global/project updates, and coalesces writes through one process-wide timer. A synchronous close/exit flush covers changes still inside the debounce interval.

The document has a global Settings section and a dictionary keyed by SDK project ID. Each project record owns Workspace regions, panel dimensions, normal window geometry/final state, current content, last selected session, and recent content.

## Boundaries and dependencies

- Avalonia owns this file because every stored value is presentation/navigation state.
- Rust remains the only SQLite owner and remains authoritative for projects, dependencies, sessions, and file reads.
- The JSON file does not duplicate agent content, configuration, credentials, authority, operations, or audit state.
- Project and session IDs are SDK-issued identities. Project file references remain relative and use opaque dependency IDs.

## Data and control flow

1. Application initialization creates the shared store and reads the JSON document once.
2. Opening a project reads its snapshot before selecting the default session.
3. The ViewModel loads authoritative dependency/session lists, filters stale references, then restores the last selected session and central file/session.
4. The Workspace window restores validated normal bounds and then its maximized/full-screen state.
5. Navigation, content, panel-resize, and window-geometry changes replace the project's in-memory snapshot and schedule a write.
6. Settings restores its global navigation snapshot after controls are initialized; later navigation changes update the same store.
7. Window close and application exit synchronously flush pending state.

## Security and failure handling

The document contains no credentials, prompts, responses, file contents, canonical project roots, or external dependency roots. Relative paths receive structural validation before restoration. Reads reject unsupported versions and malformed documents. Writes create the parent directory, serialize to a same-directory temporary file, flush it, and atomically replace the destination. Read/write failures are diagnostic-only and cannot block application use.

## Compatibility and migration

Version 1 starts from defaults when no file exists. Missing fields use defaults and unknown JSON fields are ignored. A newer schema version is not overwritten during that application run, preventing an older client from destroying newer state.

## Risks and rollback

The main risks are stale references, resize write pressure, concurrent project-window updates, and off-screen geometry. Authoritative SDK reconciliation, debouncing, one shared locked store, and display clamping address them. Rollback can stop consuming and writing `ui-state.json` without changing agent data.

## Open questions

- None.
