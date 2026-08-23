# Test Plan

## Scope

Schema, argument validation, network policy, redirects, HTTP response bounds, HTML conversion, artifacts, and desktop approval presentation.

## Unit tests

- Assert the exact seven-tool registry and WebFetch schema.
- Assert WebFetch requires approval and Full Control recognizes it.
- Assert invalid URL, credentials, format, and timeout fail before policy.
- Fetch a local HTML response and verify Markdown conversion and script removal.
- Verify plain-text conversion and declared charset decoding.
- Reject non-text MIME types.
- Retain output beyond 64 KiB in a readable managed artifact.

## Integration and conformance tests

- Run the Rust workspace tests.
- Run the Avalonia desktop tests.

## Regression checks

- Run Rust formatting and production clippy.
- Run `git diff --check` and inspect the final diff.

## Manual checks

- Inspect the approval summary for a WebFetch URL.

## Commands and results

- Focused operations WebFetch tests: 7 passed.
- Focused core argument validation and registry tests: passed.
- Focused Avalonia approval tests: 3 passed.
- `cargo test --manifest-path runtime/Cargo.toml --workspace`: 35 database, 3 LLM, 36 runtime, and 31 operations tests passed.
- `dotnet test apps/desktop-avalonia-tests/SunCode.Desktop.Tests.csproj --no-restore`: 39 passed.
- `cargo fmt --manifest-path runtime/Cargo.toml --all -- --check`: passed.
- `cargo clippy --manifest-path runtime/Cargo.toml --workspace --lib -- -D warnings`: passed.
- `git diff --check`: passed.
- `cargo clippy --manifest-path runtime/Cargo.toml --workspace --all-targets -- -D warnings`: blocked by unrelated existing test warnings in `runtime/crates/operations/src/git.rs:569` (`len() >= 1`) and `runtime/crates/core/src/agent.rs:2126` (discarded enumerate index).

## Residual risks

- Cancellation during a blocked transport read is bounded by the request timeout rather than guaranteed immediate.
- Image and binary content remains unsupported until tool-result attachments exist.
- Two pre-existing test-only clippy warnings remain outside this change; production library targets are clean.
