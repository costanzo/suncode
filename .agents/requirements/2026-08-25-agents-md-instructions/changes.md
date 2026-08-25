# Changes

## Source

- Added bounded root and nested `AGENTS.md` discovery in Rust core.
- Injected root guidance into provider system context on every model call.
- Attached nested guidance to successful read results with current-turn deduplication.

## Contracts and generated artifacts

- Documented repository instruction context in the agent SDK/provider-input contract.

## Configuration and persistence

- Added no configuration or database state. Nested guidance uses existing tool-result persistence.

## Tests

- Added root message, nested precedence, deduplication, direct-read, and provider/read round-trip coverage.

## Documentation

- Updated current agent feature and Rust core specification facts.
