# Architecture

## Current state

The Rust store already persists archived status but the desktop loads archived and active sessions into one collection. Restore exists in the SDK; permanent deletion and archive-specific desktop presentation do not.

## Proposed design

Rust remains authoritative. Store deletion uses one explicit SQLite transaction, removes child sessions before the parent, and returns image storage paths for best-effort managed-file cleanup. The desktop keeps separate active and archived projections and uses the existing confirmation window pattern.

## Boundaries and dependencies

The Avalonia client uses only typed C# SDK methods. SQLite access and file cleanup remain Rust-owned. Source image paths are never deleted.

## Data and control flow

The SDK lists all primary sessions; C# partitions by `status`. Archive and restore update status. Permanent delete removes all session-scoped rows, child rows, invocations, configuration, and managed image files.

## Security and failure handling

Archive, rename, restore, delete, and submit enforce session state in Rust. Delete rejects active sessions and rolls back database work on error. Managed image paths are deleted only when they are inside the configured SunCode image directory.

## Compatibility and migration

No schema migration is required because the existing schema has the needed columns and tables.

## Risks and rollback

The explicit delete list must stay synchronized with session-owned tables. The operation is isolated behind one store method and focused tests.
