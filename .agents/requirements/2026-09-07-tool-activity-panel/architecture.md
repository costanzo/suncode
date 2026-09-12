# Architecture

## Current state

Avalonia ChatArea currently renders tool rows inline and emits `ToolDetailRequested`. Git and Provider trace already share a mutually exclusive bottom-drawer slot. Rust emits live-only `tool.output` events and persists terminal tool results in `session_tool_use`.

## Proposed design

Add a ToolActivityViewer bound to a turn-grouped projection of existing tool activity. The conversation projection keeps user and assistant messages, turn markers, and one active-tool summary; the drawer owns detailed tool inspection.

## Boundaries and dependencies

Design-system React is specification only. Production implementation will keep Rust authoritative, route events through the SDK, and keep Avalonia responsible for transient selection, expansion, scroll-follow, and drawer visibility.

## Data and control flow

Persisted session snapshot supplies turns and terminal tool rows. Live `tool.output` events append to the selected running tool's ephemeral output buffer. Clicking the inline active-tool row raises a selection request; the workspace opens Tool activity, expands the turn, and selects the tool.

## Security and failure handling

Request/result payloads remain read-only and redacted according to existing SDK DTOs. Unknown completion and approval states remain explicit; live output is best-effort and terminal result is authoritative.

## Compatibility and migration

No schema or protocol migration is proposed by the design package.

## Risks and rollback

The main risk is overloading the compact drawer. Rollback is limited to hiding the new drawer and restoring existing inline tool rows; no persisted data changes are required.

## Open questions

None for design review.
