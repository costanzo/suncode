# Test Plan

## Scope

Agent tool translation, platform shell selection, process argument validation, and spawn failure classification.

## Unit tests

- Verify structured process arguments remain argv-based.
- Verify Windows and Unix shell command construction under conditional compilation.
- Verify legacy `bash` calls remain accepted.

## Integration and conformance tests

- Run the operations and runtime crate test suites.

## Regression checks

- Ensure no new tool definition advertises the old `bash` name.
- Ensure process output normalization still works.

## Manual checks

- On Windows, run a shell script and verify it starts PowerShell rather than `/bin/sh`.

## Commands and results

- `cargo fmt --all -- --check`: passed.
- `cargo test -p suncode-tool`: passed, 21 tests, including the Windows no-window flag and actual PowerShell output capture.
- Focused `suncode-agent` agent/tool/policy tests: passed.
- `cargo test --workspace`: 63 passed; the unrelated existing `runtime_lock::tests::prevents_a_second_runtime_without_publishing_an_endpoint` failed because Windows returned `Uncategorized` instead of the asserted `AlreadyExists`.
- The normal desktop output was locked by the running SunCode process, so an equivalent `dotnet build` with a temporary workspace `OutDir` passed with zero warnings and errors; the temporary output was removed afterward.
- `git diff --check`: passed.

## Residual risks

- Shell syntax is intentionally platform-specific.
- Network/weather access is not part of this change.
- The runtime-lock Windows error-kind assertion remains outside this delivery.
