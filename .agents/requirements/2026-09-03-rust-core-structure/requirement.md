# Requirement

Keep Rust core source files focused by responsibility and place the configuration crate inside the Rust workspace.

## Acceptance criteria

- No monolithic `core/src/agent.rs` remains.
- Agent submission, continuation, run loop, tool execution, lifecycle, support helpers, and tests are separate files under `core/src/agent/`.
- Configuration lives at `agent/crates/config/` and remains available as `suncode_config::Config`.
- Workspace builds and tests pass without behavior changes.
