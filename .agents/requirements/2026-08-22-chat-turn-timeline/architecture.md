# Architecture

## Current state

Rust persists normalized messages, turns, calls, and tool uses. Live events carry turn correlation, but the session snapshot returns only provider-neutral message bodies and loses IDs and turn ownership. Avalonia projects those flat messages directly into chat rows.

## Proposed design

Extend the session snapshot with normalized `conversationTurns`. Each turn contains its state, ordered correlated user/assistant messages, and ordered tool uses. The existing flat `messages` field remains for compatibility.

Avalonia projects each normalized turn into chat items. User and final assistant messages are primary items. Earlier assistant messages and all tool uses are process items owned by the turn. Active turns expose process items; terminal turns collapse them by default. The first retained process item anchors the control so expanded content reads from the control downward before the final response. Expansion is transient presentation state.

Live events update the same turn-owned items by stable message and tool-call identifiers. Terminal `turn.state` events collapse process items when a final assistant response exists.

## Boundaries and dependencies

- Rust remains the sole SQLite owner and constructs snapshot DTOs.
- The hand-written SDK contract is updated in documentation and both language implementations.
- Avalonia owns expansion state and rendering only.

## Data and control flow

1. Desktop loads a session snapshot.
2. Rust queries normalized turn/message/tool records and returns correlated turns.
3. Desktop atomically projects chat items in turn order.
4. Live message and tool events update the current turn timeline.
5. A terminal turn state hides process items by default without removing them.
6. The user toggles transient expansion from the control before the retained process region.

## Security and failure handling

Tool request/result payloads are existing redacted normalized values. No credentials or authorization headers are added. Malformed optional content is represented as unavailable rather than failing the whole session snapshot.

## Compatibility and migration

The existing `messages` snapshot field remains unchanged. `conversationTurns` is additive. No schema migration is required.

## Risks and rollback

Large histories create more snapshot data and UI rows. Process rows are compact and collapsed after completion; rollback can remove the additive DTO and revert Avalonia to flat messages without changing persisted data.

## Open questions

- None.
