# Changes

## Source

- Add basedpyright to the design-system language-server catalog.
- Add its server-specific Settings detail specimen and direct route.
- Add the immutable Rust basedpyright catalog adapter, detection, version-4 persistence, settings mapping, and runtime reconciliation.
- Add basedpyright-specific Avalonia Settings fields, normalized dirty tracking, save/reset, detection, failure, and retry behavior.

## Contracts and generated artifacts

- Bump the native SDK ABI from 20 to 21.
- Add typed Rust, C ABI, and C# basedpyright configuration operations and DTOs.

## Configuration and persistence

- Store version-4 basedpyright configuration in the existing language-server JSON column while preserving older records.

## Tests

- Add focused persistence, SDK mapping, environment redaction, project isolation, and managed serialization tests.
- Run Rust LSP/data, Rust SDK, C ABI, Avalonia, design-system, and live protocol checks.

## Documentation

- Update `DESIGN.md`, the SDK contract, product status, stable feature documentation, Settings README, and guide copy.
