# Requirement

## Background

The Avalonia application and its tests are both client-specific, but the test project was stored as a sibling under `apps/`. That layout makes the test project look like an independent application and requires an unnecessary relative path in its project reference.

## Goals

- Colocate the Avalonia test project with the Avalonia application.
- Make the test command discoverable from the application README.
- Keep the test project referencing the application through a short, stable relative path.

## Non-goals

- Rename test namespaces or test assemblies.
- Change test behavior or test dependencies.
- Move Rust tests or create a general repository-wide test root.

## Requirements

- Move the test project to `apps/desktop-avalonia/tests/`.
- Update its project reference to `../SunCode.Desktop.csproj`.
- Update current documentation and verification commands.

## Acceptance criteria

- No source or project file remains under `apps/desktop-avalonia-tests/`.
- The colocated test project restores and builds using the Avalonia project reference.
- The test command is documented at `apps/desktop-avalonia/README.md`.
