# Test Plan

## Scope

Verification covers the catalog and rust-analyzer-specific Settings surface, version-5 persistence, Rust SDK mapping and isolation, C ABI 22, C# DTO serialization, and Avalonia presentation models.

## Unit tests

- Data tests cover backward-compatible deserialization and version-5 persistence acceptance.
- Rust SDK tests cover settings mapping, environment redaction, catalog order, and project isolation.
- C# tests cover request serialization and the desktop item projection.

## Integration and conformance tests

- Build the complete design-system application.
- Confirm the direct rust-analyzer route resolves without runtime errors.
- Build and test the C ABI and Avalonia client against ABI 22.

## Regression checks

- Confirm jdtls, vtsls, and basedpyright detail dispatch remains unchanged.
- Confirm the catalog reports four supported servers.
- Confirm Save and Reset are disabled for an unchanged draft and enabled after editing.

## Manual checks

- Review desktop and constrained-width layouts.
- Review light and dark themes.
- Exercise custom features, custom target, advanced disclosure, detection, enablement, save, reset, and retry states.

## Commands and results

- `npm run build` from `design-system/`: passed (268 modules transformed).
- `git diff --check`: passed.
- Headless browser review: passed for the four-entry catalog, rust-analyzer direct route, light and dark themes, and 760px constrained width.
- Interaction review: Save and Reset start disabled; custom Cargo features and custom target fields appear on selection; Save enables after a draft change.
- `cargo test -p suncode-data`: passed, 26 tests.
- `cargo test --manifest-path sdks/rust/Cargo.toml`: passed, 37 tests.
- `cargo test --manifest-path sdks/c/Cargo.toml`: passed, 4 tests.
- `dotnet test apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj --no-restore`: passed, 324 tests.
- Focused Rust, C#, and design-system formatting checks: passed.
- Initial live `rust-analyzer --version` preflight before follow-up: unavailable because the rustup component was not installed at that time.
- Follow-up live initialization after component installation: passed with the same filtered environment, initialization payload, and workspace settings used by SunCode; the process remained alive after configuration.
- `cargo test -p suncode-lsp`: passed, 22 tests, including bounded/redacted stderr propagation.

## Residual risks

- None specific to the reported startup failure.
