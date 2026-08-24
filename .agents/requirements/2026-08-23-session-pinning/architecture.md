# Architecture

## Current state

Session rows are owned by `suncode-db`, exposed by the Rust SDK, and rendered by `DesktopViewModel` and `ProjectSidebar`. The schema now stores pin state directly on `session`.

## Proposed design

Store nullable `pin_at` directly on `session`. The database query derives pinned state from `pin_at IS NOT NULL`, returns the timestamp in `SessionRecord`, and sorts by pinned state, pin time, and activity time. A dedicated Rust SDK method owns validation and mutation; the Avalonia client never opens SQLite.

## Boundaries and dependencies

- `suncode-db`: session column, projection, ordering, and persistence tests.
- `suncode-agent`: typed method and C ABI export.
- Runtime contract: method table and shared response vector.
- Avalonia: P/Invoke declaration, JSON projection, menu handlers, and pin icon.

## Data and control flow

`MenuFlyout -> ProjectSidebar handler -> DesktopViewModel -> AgentSdk -> C ABI -> AgentSdk -> Store.session`; the next `list_sessions` query returns ordered DTOs and the sidebar reloads without changing the selected session content.

## Security and failure handling

The operation is local metadata only and does not grant machine authority. Missing or archived sessions fail closed. Errors use the existing SDK envelope and `RunAsync` status handling.

## Compatibility and migration

The schema gains nullable `session.pin_at`; existing databases are not converted. The new C ABI function is additive under ABI version 1.

## Risks and rollback

Rollback is deleting the dedicated method/UI and clearing `session.pin_at`; existing sessions remain valid.

## Open questions

- None.
