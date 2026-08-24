# Test Plan

- Run the Rust workspace test suite and verify the agent uses only `agent.sqlite3`.
- Verify the ABI version assertion and agent lock/log tests.
- Build and test the Avalonia desktop project, which invokes Cargo and copies the `suncode_agent` native library.
- Run `git diff --check` and inspect residual production references.
