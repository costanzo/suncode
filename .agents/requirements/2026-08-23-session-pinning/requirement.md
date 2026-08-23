# Requirement

## Background

The Avalonia project sidebar lists sessions in recent-activity order. Developers need a durable way to keep important active sessions at the top of the current project's list.

## Goals

- Add Pin and Unpin actions to the existing session overflow menu.
- Persist pin state through the Rust-owned runtime and SQLite.
- Order pinned sessions before unpinned sessions in `list_sessions`.
- Keep pinning project-local, unlimited, and immediately reflected in the sidebar.

## Non-goals

- Cross-project pin state.
- A maximum pin count or manual drag ordering.
- Pinning archived sessions.

## Requirements

- `SessionRecord` and `list_sessions` expose nullable `pinAt`; clients treat non-null as pinned.
- The SDK exposes `set_session_pinned(session_id, pinned)` through the C ABI.
- Pin state is stored directly in `session.pin_at` and is mutated only by the dedicated SDK method.
- Archiving an active session removes its pin state.
- The sidebar keeps the selected session selected after pinning or unpinning.
- A pinned session displays a compact pin icon and the overflow menu toggles between Pin and Unpin.

## Edge cases

- Pinning a missing session returns the normal session-not-found persistence error.
- Pinning an archived session fails closed.
- Unpinning is idempotent and may clear stale metadata.
- A lagged or refreshed client receives pin state from the normalized session list rather than trusting its previous cache.

## Acceptance criteria

- Pinned active sessions sort before unpinned sessions for the same project.
- Archiving a pinned session clears `pinned` and prevents it from being pinned while archived.
- Rust database and runtime tests pass.
- Avalonia tests/build pass with the new compiled bindings and menu handlers.
- Shared runtime vectors document the new method and response field.

## Open questions

- None for the Phase 1 desktop scope.
