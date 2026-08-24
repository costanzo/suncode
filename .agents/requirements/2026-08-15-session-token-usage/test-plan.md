# Test Plan

## Scope

Verify durable usage projection, historical migration, SDK aggregation, Qt integration, compact formatting, and footer rendering.

## Unit tests

- Repeated usage events replace one turn's cumulative counters.
- Multiple turns sum to the session aggregate.
- Schema v11 data backfills into v12 token counters.
- The named SDK method returns the aggregate and rejects a missing session.

## Integration and conformance tests

- Exercise the C ABI session usage function.
- Build the Rust static library and Qt desktop application.

## Regression checks

- Run the Rust workspace tests and formatting checks.
- Run QML lint and `git diff --check`.

## Manual checks

- Inspect the footer at realistic desktop widths.
- Confirm zero and compact large-token formatting remain legible.

## Commands and results

- `cargo test --workspace`: passed, 37 tests.
- `cargo fmt --all -- --check`: passed.
- `cmake -S apps/desktop-qt -B apps/desktop-qt/build -DCMAKE_BUILD_TYPE=Debug`: passed.
- `cmake --build apps/desktop-qt/build --parallel`: passed with existing macOS SDK-version linker warnings.
- `cmake --build apps/desktop-qt/build --target all_qmllint`: passed with existing import and unqualified-access warnings.
- `jq empty contracts/vectors/runtime-sdk.json`: passed.
- Static-library inspection found `suncode_agent_sdk_session_usage` and no generic request symbol; Apple's `nm` also reported its existing LLVM reader-version warnings.
- `git diff --check`: passed.
- Project-window QML screenshots at 1440x900 and 900x620 kept the model and `Session 13.8k tokens` footer value legible without overlap.

## Residual risks

- Provider calls without usage metadata cannot contribute to the reported total.
