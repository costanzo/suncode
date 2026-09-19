# Test Plan

## Scope

Verify local stdio LSP protocol behavior, persistence compatibility, project-scoped runtime lifecycle, semantic tool normalization, SDK contracts, and Avalonia Settings behavior.

## Unit tests

- Content-Length framing and JSON-RPC correlation.
- Initialization, capability parsing, diagnostics, locations, hover, symbols, cancellation, and shutdown.
- Definition validation, revision conflicts, write-only environment patches, and DTO redaction.
- Path/URI normalization and one-based position conversion.

## Integration and conformance tests

- Fake language server transport over duplex stdio plus structured process-launch failure coverage.
- Exact 17-table additive database upgrade.
- Rust/C/C# named method and serialization contracts.
- Avalonia view-model and interaction tests.

## Regression checks

- Existing MCP, agent tool, policy, SDK, database, and Settings tests.
- Existing project/dependency authority rules.

## Manual checks

- Light/dark Settings page, narrow width, add/edit/delete, status, failure, and disabled states.

## Commands and results

- `cargo fmt --all -- --check && cargo test --workspace` in `agent/`: passed; 116 unit tests plus doc tests, including 4 protocol tests and 3 core LSP tests.
- `cargo fmt --all -- --check && cargo test` in `sdks/rust/`: passed; 22 tests.
- `cargo fmt --all -- --check && cargo test` in `sdks/c/`: passed; 2 tests.
- `dotnet build SunCode.Desktop.csproj --no-restore`: passed with 0 warnings and 0 errors.
- `dotnet test tests/SunCode.Desktop.Tests.csproj --no-restore`: passed; 105 tests.
- `npm run build` in `design-system/`: passed with the existing non-blocking Vite chunk-size warning.
- Impeccable source review and detector: passed with no findings; the final reviewer disposition was `ship`.
- `git diff --check`: passed.

## Residual risks

- Real language servers remain optional external dependencies and are not required by the normal test suite.
- The final UI review used the approved design artifact plus Avalonia source/build evidence; no production-native screenshot was captured in this delivery.
