# Changes

## Source

Provider credential form and styles; Settings in-memory provider state; saved mode summaries; review route and navigation; platform/failure fixtures.

## Contracts and generated artifacts

Production contracts now include provider-scoped `keychain`/`plaintext` storage, redacted credential statuses, and ABI 24 methods for selecting and changing storage mode. Vite output is generated and ignored.

## Configuration and persistence

SQLite stores mode/reference/mask for keychain entries or the complete value for explicit plaintext mode. The Rust `suncode-secrets` crate owns OS keychain access and memory secret handling.

## Tests

Rust, CLI, C#/Avalonia, design-system builds, focused storage tests, and diff checks.

## Documentation

Proposed interaction rules in DESIGN.md, Settings review README, and this active delivery package.
