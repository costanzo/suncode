# Architecture

## Current state

The desktop client keeps window views, shared UI models, and styles in large top-level files.

## Proposed design

Use window-oriented view folders, feature-oriented UI model folders, partial view-model files, and application style/resource files with stable names.

## Boundaries and dependencies

Keep the refactor presentation-only. Do not change Rust SDK ownership or persistence behavior.

## Data and control flow

No new runtime flow; only file organization changes.

## Security and failure handling

No new security surface.

## Compatibility and migration

Keep XAML class names, bindings, and public view-model APIs aligned during the move.

## Risks and rollback

Main risk is namespace drift; verify with build and focused tests.

## Open questions

- None.
