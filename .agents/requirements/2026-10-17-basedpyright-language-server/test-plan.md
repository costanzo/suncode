# Test Plan

## Scope

Design-system catalog, Rust runtime and persistence, SDK bindings, Avalonia Settings behavior, and a live installed-server handshake.

## Unit tests

- Version-2 persistence compatibility and version-4 basedpyright settings.
- Rust SDK settings mapping, environment redaction, and project isolation.
- C# request serialization and desktop item projection.

## Integration and conformance tests

- Rust SDK and C ABI contract tests.
- Avalonia build and test suite.
- Design-system production build and direct route rendering.
- Live basedpyright stdio initialization.

## Regression checks

- Existing jdtls and vtsls routes remain functional.
- Unscoped catalog disables all three entries.

## Manual checks

- Catalog with three server rows.
- basedpyright default, custom configuration path, dirty, reset, save, indexing, ready, failed, and retry states.
- Light and dark themes at desktop and constrained widths.
- Keyboard focus, long paths, and overflow.

## Commands and results

- `npm run build`: passed; Vite retained the existing bundle-size warning.
- Targeted `prettier --write`: completed with no formatting changes.
- Headless Chrome screenshots: desktop catalog, desktop detail, and narrow detail passed visual inspection.
- `git diff --check`: passed.
- `cargo test -p suncode-data -p suncode-lsp --no-fail-fast`: passed, 46 tests.
- `cargo test` in `sdks/rust`: passed, 36 tests.
- `cargo test` in `sdks/c`: passed, 4 tests.
- `dotnet test tests/SunCode.Desktop.Tests.csproj --no-restore`: passed, 322 tests.
- Live `basedpyright-langserver --stdio` initialization: passed with 21 capability fields.

## Residual risks

- Interpreter discovery is focused-tested through deterministic project paths; conda, pyenv, and Windows installations remain environment-dependent.
