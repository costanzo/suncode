# Test Plan

## Scope

Provider schema bootstrap and compatibility, provider configuration preservation, authoritative ABI conformance, SDK startup, and unchanged desktop behavior. The CLI architecture itself is documentation-only in this delivery.

## Unit tests

- Fresh in-memory store seeds Claude with `adapter_type=anthropic`.
- Previous provider constraint is rebuilt.
- Endpoint, API key, enabled state, ordering, and creation time survive the rebuild; update time advances only for the intentional built-in adapter switch.
- C ABI test compares the exported value with the SDK constant.

## Integration and conformance tests

- Rust SDK opens and exercises facade operations against a fresh database.
- C binding exposes the current ABI and native watch behavior.
- Avalonia remains compatible with the current native SDK.

## Regression checks

- Agent workspace tests.
- Rust SDK tests.
- C binding tests.
- Avalonia tests.
- Rust formatting, applicable Clippy, and `git diff --check`.

## Manual checks

- Review `contracts/cli.md` against the architecture ownership and environment-prefix requirements.

## Commands and results

- `cargo test --manifest-path agent/Cargo.toml -p suncode-data previous_provider_constraint_is_rebuilt_for_anthropic_without_losing_configuration --offline` — passed.
- `cargo test --manifest-path agent/Cargo.toml --workspace --all-targets --offline` — passed: 59 core tests, 14 data tests, and all other workspace crate/example tests.
- `cargo test --manifest-path sdks/rust/Cargo.toml --lib --offline` — passed: 29 tests.
- `cargo test --manifest-path sdks/c/Cargo.toml --lib --offline` — passed: 4 tests.
- `dotnet test apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj` — passed: 107 tests.
- `cargo clippy --manifest-path agent/Cargo.toml -p suncode-data -p suncode-database --all-targets --offline -- -D warnings` — passed.
- `cargo clippy --manifest-path sdks/rust/Cargo.toml --all-targets --offline -- -D warnings` — passed.
- `cargo clippy --manifest-path sdks/c/Cargo.toml --all-targets --offline -- -A clippy::missing-safety-doc -D warnings` — passed; the allowance covers the binding's pre-existing unsafe-export documentation debt.
- Rust formatting checks — passed.
- `git diff --check` — passed.

## Residual risks

- General non-interactive CI execution still requires Rust-owned named policy profiles.
