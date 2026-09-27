# Changes

## Source

- Rust core context selection, model summary fallback, and overflow retry.
- Rust data checkpoint projection and post-boundary replay.
- Provider error classification and content-free request/response-status logging for the OpenAI-compatible adapter.

## Contracts and generated artifacts

- Updated written persistence and SQLite contracts; no generated artifacts.

## Configuration and persistence

- No new settings or tables. Checkpoint data is stored in existing `session_call` rows.

## Tests

- Core cut-point, budget, summary-fit, and overflow-classification tests.
- Data same-millisecond durable replay test.

## Documentation

- Current agent specification, feature record, persistence, and schema contracts updated.
