# Requirement

## Background

The agent core was named `runtime` throughout the Rust workspace, native ABI, desktop binding, contracts, and current product records. That name is ambiguous because async execution runtimes are also implementation details of the core.

## Goals

- Name the agent core and embedded SDK `harness` consistently.
- Keep lower-level executor terminology such as Tokio runtime when it describes the async library rather than the agent product boundary.
- Preserve existing local data where practical.

## Non-goals

- Changing agent behavior, persistence schema, provider behavior, or operation policy.
- Rewriting historical requirement prose or decision identifiers.

## Requirements

- Rename the Rust workspace directory and core package to `harness` and `suncode-agent`.
- Rename the native library and exported C ABI symbol family to `suncode_agent` and `suncode_agent_sdk_*`.
- Rename the Avalonia interop surface to `AgentSdk` under the `SunCode.Desktop.Agent` namespace.
- Rename current harness contracts, feature/specification paths, documentation, lock/log names, health field, and public harness error codes.
- Bump the C ABI version because the exported symbol family changed.
- Read an existing legacy `runtime.sqlite3` when the new `harness.sqlite3` path does not exist.

## Acceptance criteria

- No production build or current contract references the old core package, native library, ABI symbols, or desktop SDK type.
- Rust workspace tests pass.
- The Avalonia desktop project builds and links the renamed native library.
- Existing legacy database path fallback is covered by code inspection and does not alter schema contents.
