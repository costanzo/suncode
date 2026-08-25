# Architecture

## Current state

The production Avalonia client is under `apps/desktop-avalonia/`, while its .NET test project is a sibling directory under `apps/desktop-avalonia-tests/`.

## Proposed design

The client owns its tests in `apps/desktop-avalonia/tests/`. The test project remains a separate .NET project and assembly, but its source and project configuration are colocated with the client it validates.

## Boundaries and dependencies

The test project may reference the Avalonia client project. It does not change the Rust SDK ownership boundary or production application packaging.

## Compatibility and migration

This is a source-tree layout change only. Test namespaces, assembly identity, package versions, and runtime behavior remain unchanged.
