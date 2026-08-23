# Requirement

## Background

SunCode's model registry was narrowed to seven built-in tools, but historical aliases and unreachable operation entries remained in core, the operations package, desktop presentation, tests, and contracts. Those paths make the supported surface ambiguous and leave dead implementations available to future callers.

## Goals

- Make the seven registered tools the only model-call names accepted by the agent.
- Remove operation entries and implementations with no production caller.
- Remove the filesystem patch implementation completely.
- Preserve the internal process runner required by `bash`, including bounded output, timeout, cancellation, and artifacts.
- Keep SDK-owned Git and checkpoint restore operations.

## Non-goals

- Removing process execution from `bash`.
- Removing checkpoint capture, restore, operation journaling for writes, or managed artifact storage.
- Adding compatibility migration for persisted development calls.

## Requirements

1. Agent tool dispatch accepts exactly `read`, `glob`, `grep`, `write`, `edit`, `bash`, and `webfetch`.
2. Policy recognizes exactly those seven model tool names.
3. The operation dispatcher exposes only the seven canonical `tool/*` methods plus production SDK methods.
4. Patch, unused filesystem actions, unused capability operations, unused asynchronous process management, and historical operation aliases are removed from source and current contracts.
5. Stored historical tool rows remain displayable as raw records, but they cannot be newly executed.

## Edge cases

- A model-generated unknown or retired tool name must fail closed and return a recoverable tool error to the model.
- `bash` must continue to translate its command into the platform shell and use the synchronous audited process runner.
- Removing artifact read/sweep operation endpoints must not remove artifact creation used for bounded tool output.

## Acceptance criteria

- Source scans find no executable `apply_patch`, `fs.patch`, structured process tool, shell alias, or removed operation dispatcher entry.
- Focused core, operations, and desktop tests pass.
- The Rust workspace and Avalonia test suite pass.
- Formatting, clippy, and diff checks pass or unrelated failures are recorded.

## Open questions

- None.
