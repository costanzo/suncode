# Changes

## Source
- Runtime turn submissions can be queued into an active turn instead of blocking behind the session lock.
- Agent tool batches are preflighted before execution; all-read-only batches may execute concurrently.
- Context compaction now uses estimated token counts and active model limits, with fixed reserve and recent-tail token settings.
- Qt status handling distinguishes queued turn responses from approval responses.

## Contracts and generated artifacts
- The turn response union gains `status: queued` for accepted running-turn submissions.

## Configuration and persistence
- No SQLite schema or durable setting changes.

## Tests
- Added focused Rust tests for queued submit, read-only batch preflight, and token-window compaction.

## Documentation
- Added this dated requirement package.
