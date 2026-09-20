# Changes

## Source

- Add `run` grammar and execution to `apps/cli`.
- Render typed SDK events and terminal turn responses.
- Add first/second interrupt behavior and lag re-watch.
- Re-export `TurnResponse` from the Rust SDK facade.

## Contracts and generated artifacts

- Update `contracts/cli.md` and durable feature/specification records.
- No generated contracts or bindings.

## Configuration and persistence

- Add CLI use of `SUNCODE_MODEL` and `SUNCODE_REASONING_EFFORT`.
- No schema or persisted configuration changes.

## Tests

- Add argument tests and mock-provider process integration coverage for JSONL events, final response, stdin, text stream separation, and environment selection.

## Documentation

- Update CLI README, product/architecture status, contributor guidance, decision index, and feature records.
