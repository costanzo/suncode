# Test Plan

## Unit tests

- Verify the built-in question schema and exact tool registration.
- Verify prompt and answer validation, including custom answers and multiplicity.
- Verify pending recovery snapshots can be answered once and then become non-pending.

## Integration and conformance tests

- Run the Rust workspace tests.
- Build and test the Avalonia desktop client and projection suite.

## Regression checks

- Run Rust formatting, `git diff --check`, and inspect the final diff.

## Manual checks

- Open a session, trigger a question tool call, select options/custom text, submit, and verify the turn continues.
- Reload the session while a question is pending and verify the question card remains visible.
