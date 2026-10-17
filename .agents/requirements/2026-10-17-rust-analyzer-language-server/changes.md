# Changes

## Source

- Add rust-analyzer to the design-system language-server catalog.
- Add a server-specific rust-analyzer Settings specimen and direct review route.
- Add the immutable Rust catalog adapter, detection, version-5 persistence, settings mapping, and runtime reconciliation.
- Add typed Rust, C ABI, C#, and Avalonia configuration surfaces.

## Contracts and generated artifacts

- Bump the native SDK ABI from 21 to 22.
- Add `configure_rust_analyzer_language_server`; no generated contracts are introduced.

## Configuration and persistence

- Store version-5 rust-analyzer configuration in the existing language-server JSON column.
- Preserve version 1-4 records through defaulted optional fields.

## Tests

- Add focused data, Rust SDK, C# DTO, and desktop model tests.
- Run Rust, native binding, Avalonia, design-system, formatting, and diff checks.

## Documentation

- Update `DESIGN.md`, the Settings review README, and the review guide.
