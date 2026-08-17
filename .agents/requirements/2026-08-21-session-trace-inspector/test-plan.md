# Test Plan

## Scope

Session trace summary/detail DTOs, provider cache usage parsing, Avalonia trace tree projection, and the ProjectWorkspace trace drawer.

## Unit tests

- Verify turn summaries include turns with and without calls.
- Verify call detail includes correlated messages and tool uses.
- Verify OpenAI-compatible cache token fields are normalized.
- Verify cache hit rate and unavailable usage presentation.

## Integration and conformance tests

- Update shared runtime SDK vectors for additive trace fields.
- Verify existing C ABI provider trace methods still return valid envelopes.

## Regression checks

- Run Rust workspace tests and strict focused Clippy.
- Build the Avalonia desktop application.
- Run formatting and diff checks.

## Manual checks

- Inspect expanded/collapsed turns, call selection, filtering, loading, empty, failed, and cache-missing states.
- Verify the drawer remains usable at minimum and typical project-window sizes.

## Commands and results

- `cargo test --workspace`: passed (59 unit tests and all doc tests).
- `cargo fmt --all -- --check`: passed.
- `cargo clippy -p suncode-db --all-targets -- -D warnings`: passed.
- `cargo clippy -p suncode-llm --all-targets -- -D warnings`: passed.
- `cargo clippy -p suncode-runtime --lib -- -D warnings`: passed.
- `dotnet build apps/desktop-avalonia/SunCode.Desktop.csproj --no-restore`: passed with zero warnings and zero errors.
- `jq empty contracts/vectors/runtime-sdk.json`: passed.
- `git diff --check`: passed.
- Native startup with isolated `SUNCODE_DATA_DIRECTORY` and `SUNCODE_DATABASE_PATH`: passed; the welcome window rendered correctly.

## Residual risks

- The populated trace tree and inspector could not be navigated automatically because macOS denied assistive access. Compile-time XAML validation passed, but a manual interaction pass with real trace data remains the residual UI risk.
