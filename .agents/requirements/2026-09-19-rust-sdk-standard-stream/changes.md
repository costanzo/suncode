# Changes

## Source

- Add the core subscription poll primitive.
- Implement standard `Stream` and `FusedStream` behavior in the Rust SDK.
- Keep direct async, blocking, and nonblocking receive compatibility.

## Contracts and generated artifacts

- Document standard stream item and terminal semantics.
- No generated artifacts or native ABI changes.

## Configuration and persistence

- No changes.

## Tests

- Add `StreamExt::next`, close, fused, and one-shot lag coverage.

## Documentation

- Add this delivery package and update durable SDK, contract, feature, specification, and decision records.
