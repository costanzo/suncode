# Architecture

## Current state

The design-system Settings specimen includes global and project settings, providers, built-in agents, and MCP server management. It has no Language Server Protocol configuration or runtime-status surface.

## Proposed design

Add a first-level `Language servers` Settings destination. The page reuses the established quiet list, native editor window, enable toggle, retry/edit/delete actions, and confirmation dialog while introducing language scope and indexing progress.

## Boundaries and dependencies

- Changes are limited to `design-system/`, `DESIGN.md`, and this delivery record.
- React/Vite remains specification and review tooling only.
- No Avalonia, Rust, SDK, persistence, or protocol implementation is added.
- Existing shared buttons, dropdowns, dialogs, icons, native window frame, tokens, and Settings composition are reused.

## Data and control flow

The specimen keeps illustrative in-memory language-server rows. Add and edit actions open a separate native-window specimen. Enable and retry actions briefly enter an indexing state before resolving. Delete uses the shared confirmation dialog. These interactions demonstrate intended behavior but do not persist or launch a process.

## Security and failure handling

The design-system page does not add a standalone authority or semantic-capability banner. Production process authority, sandboxing, undo, and server-request behavior remain part of the future implementation contract rather than this compact configuration surface.

## Compatibility and migration

The new destination is additive. Existing Settings destinations and interactions remain unchanged.

## Risks and rollback

The principal risk is confusing language servers with MCP servers or implying IDE editing features. Distinct language, scope, and status presentation mitigate this. Rollback removes the page, styles, and design contract without affecting production code.

## Open questions

- Production server discovery, persistence, and policy shape are intentionally deferred.
