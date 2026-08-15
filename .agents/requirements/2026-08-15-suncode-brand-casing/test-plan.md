# Test Plan

## Scope

Verify source consistency, Qt/QML compilation, resource resolution, and application startup.

## Unit tests

No runtime behavior changed; existing Rust tests cover the renamed opaque handle types through the C ABI smoke test.

## Integration and conformance tests

- Build the desktop application, including the Rust static library.

## Regression checks

- Search source and tracked documentation with `rg 'Sun[c]ode'` for the legacy spelling.
- Run `git diff --check`.

## Manual checks

- Start the desktop application and confirm it reaches the Qt event loop without QML errors.

## Commands and results

- `cmake -S apps/desktop-qt -B apps/desktop-qt/build -DCMAKE_BUILD_TYPE=Debug`: passed.
- `cmake --build apps/desktop-qt/build --parallel`: passed with existing macOS SDK-version linker warnings.
- `cargo test --workspace`: passed, 35 tests.
- `cargo fmt --all -- --check`: passed.
- `cmake --build apps/desktop-qt/build --target all_qmllint`: passed with existing warnings.
- Offscreen desktop startup reached the Qt event loop with no QML runtime output.
- `rg 'Sun[c]ode'` across source and tracked documentation: no matches.
- `git diff --check`: passed.

## Residual risks

- Lowercase technical identifiers intentionally continue to contain `suncode` for compatibility.
