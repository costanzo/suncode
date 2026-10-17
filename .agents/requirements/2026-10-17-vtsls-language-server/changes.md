# Changes

## Source

- Add vtsls to the design-system language-server catalog.
- Add a server-specific vtsls Settings detail specimen and direct review route.
- Add the immutable Rust vtsls catalog adapter, validation, launch construction, project persistence, and runtime reconciliation.
- Add vtsls-specific Avalonia Settings fields, normalized dirty tracking, save/reset, detection, failure, and retry behavior.

## Contracts and generated artifacts

- Bump the native SDK ABI from 19 to 20.
- Add typed Rust, C ABI, and C# vtsls configuration operations and DTOs.
- Extend the shared LSP client with `workspace/configuration` and `workspace/didChangeConfiguration` support.

## Configuration and persistence

- Store version-3 vtsls configuration in the existing language-server JSON column; preserve version-2 jdtls compatibility.

## Tests

- Add focused LSP configuration negotiation, persistence compatibility, SDK mapping, environment redaction, project isolation, and managed serialization tests.
- Run Rust LSP/data, Rust SDK, C ABI, Avalonia, and design-system checks.

## Documentation

- Update `DESIGN.md`, the SDK contract, product status, feature documentation, and Settings design-system status.
