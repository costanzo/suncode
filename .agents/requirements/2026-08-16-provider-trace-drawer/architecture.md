# Architecture

Provider trace records are Rust-owned session diagnostics. They sit beside session content and turn projections, but are exposed only through named SDK methods and ordered events. The Avalonia client keeps transient drawer state and does not read SQLite or provider wire structures.

The UI follows the existing Git drawer topology: bottom dock, resizable height, left request list, right detail pane. The drawer is a debug/audit surface, not a persistent dashboard.

## Current state

## Proposed design

## Boundaries and dependencies

## Data and control flow

## Security and failure handling

## Compatibility and migration

## Risks and rollback

## Open questions
