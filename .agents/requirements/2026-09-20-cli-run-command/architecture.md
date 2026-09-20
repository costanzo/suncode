# Architecture

## Current state

`apps/cli` embeds `AsyncAgentSdk`, disables Browser and Computer host capabilities, and implements administrative commands with text/JSONL output and explicit shutdown.

## Proposed design

Add a one-shot `run` adapter inside the existing CLI. Re-export the typed `TurnResponse` from `sdks/rust` so the client never imports core directly. Use `futures_util::StreamExt` and Tokio signal selection over the SDK's typed session stream and turn submission future.

## Boundaries and dependencies

The CLI depends only on `sdks/rust` and terminal, argument, serialization, async, and test libraries. Project/session/turn/provider/policy/operation behavior remains SDK/core-owned. Mock-provider integration tests may host a local HTTP endpoint but production CLI code never contacts providers directly.

## Data and control flow

1. Resolve CLI and `SUNCODE_` configuration.
2. Open SDK with restricted host capabilities.
3. Read prompt, open project, and create a primary session.
4. Atomically watch the new session.
5. Submit the turn while consuming typed events.
6. Re-watch on lag and drain ready tail events after submission completes.
7. Convert `TurnResponse` into final text/JSONL outcome or exit 4 suspension.
8. Consume SDK shutdown, then write the success report.

## Security and failure handling

Prompt stdin is never reused for an approval or question. Credentials remain SQLite-owned. Cancellation uses the active SDK turn ID. Browser and Computer remain unavailable. Provider content is emitted only through normalized SDK DTOs/events.

## Compatibility and migration

The command is additive. Existing administrative grammar, schema-versioned JSONL envelopes, exit statuses, SDK defaults, C ABI, desktop client, and persistence are unchanged.

## Risks and rollback

The principal race is submission completion becoming ready in the same scheduler poll as queued events. Explicit tail draining prevents final deltas and lifecycle events from being skipped. Removing the additive command and SDK re-export rolls back this slice without data migration.

## Open questions

- None.
