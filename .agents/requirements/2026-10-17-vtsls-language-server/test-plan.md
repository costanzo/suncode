# Test Plan

## Scope

Design-system catalog, Rust runtime and persistence, SDK bindings, and Avalonia Settings behavior.

## Unit tests

- LSP workspace configuration request and notification behavior.
- Version-2 persistence compatibility and version-3 vtsls settings.
- Rust SDK mapping, redaction, and project isolation.
- C# request serialization and desktop item projection.

## Integration and conformance tests

- Rust SDK and C ABI contract tests.
- Avalonia build and test suite.
- Design-system production build and route rendering.

## Regression checks

- Existing jdtls catalog and detail routes remain functional.
- Unscoped catalog disables both entries.

## Manual checks

- Catalog with two server rows.
- vtsls default, custom TypeScript SDK, advanced, dirty, reset, save, indexing, ready, failed, and retry states.
- Light and dark themes at desktop and constrained widths.
- Keyboard focus, long paths, and overflow.

## Commands and results

- `npm run build`: passed; Vite retained the existing bundle-size warning.
- Targeted `prettier --check`: passed.
- `git diff --check`: passed.
- `cargo test -p suncode-lsp -p suncode-data --no-fail-fast`: passed.
- `cargo test` in `sdks/rust`: passed.
- `cargo test` in `sdks/c`: passed.
- `dotnet test tests/SunCode.Desktop.Tests.csproj --no-restore`: passed, 320 tests.
- `npm run build`: passed; Vite retained the existing bundle-size warning.
- Rust formatting and `git diff --check`: passed.

## Residual risks

- The local machine does not have vtsls installed globally, so verification uses deterministic fake executables and protocol peers rather than a live installed vtsls process.
