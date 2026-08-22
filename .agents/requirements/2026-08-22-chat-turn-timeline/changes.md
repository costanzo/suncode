# Changes

## Source

- Added a Rust persistence projection that groups normalized messages and tool uses by conversation turn.
- Added the additive SDK `conversationTurns` snapshot field while preserving the existing flat `messages` projection.
- Added Avalonia turn-owned process items, stable message/tool update behavior, terminal collapse, retained expansion, and final-assistant presentation state.
- Added compact assistant/tool process rows and an expand/collapse control before the retained process region and final response; process rows have no copy action.
- Anchored the control before the retained process region so expanded work appears below the control rather than above it.

## Contracts and generated artifacts

- Updated the hand-written runtime SDK contract documentation and shared snapshot test vector for `conversationTurns`.

## Configuration and persistence

- No schema or configuration change.

## Tests

- Added persistence grouping and SDK serialization coverage.
- Added Avalonia snapshot, live ordering, idempotency, streaming, tool update, terminal collapse, expansion, missing-final, and tool-call-text coverage.

## Documentation

- Updated the Avalonia feature record, runtime specification, migration change log, and this requirement package.

- Added this delivery record.
