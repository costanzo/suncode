# Requirement

> Superseded record. The model-facing removal remains accepted, while the later `2026-08-23-remove-unused-tools` requirement also removes the retained compatibility execution paths.

## Background

SunCode advertised both `apply_patch` and structured `process` tools alongside the simpler file-edit and bash tools. The requested model surface should keep one editing path and one command-execution path.

## Goals

- Remove `apply_patch` and `process` from new model requests.
- Keep the remaining built-in tool contract explicit and tested.
- Preserve compatibility with historical persisted calls and internal operations.

## Non-goals

- Removing filesystem patch or structured process implementations from the audited operations package.
- Changing approval, recovery, checkpoint, or SQLite contracts.
- Changing the `bash`, `edit`, or `write` schemas.

## Requirements

1. New provider requests advertise exactly `read`, `glob`, `grep`, `write`, `edit`, and `bash`.
2. The host prompt does not instruct the model to call `process`.
3. Historical `apply_patch` and `process` calls retain their existing dispatch, policy, and argument translation.
4. Current feature and runtime specification documents distinguish advertised tools from internal compatibility operations.

## Edge cases

- Reloading a session containing an earlier `apply_patch` or `process` call must not make its result unreadable.
- Unknown tools must continue to fail closed.

## Acceptance criteria

- The built-in registry test asserts the exact six-tool set.
- No new model tool schema advertises `apply_patch` or `process`.
- Focused runtime tests and repository diff checks pass.

## Open questions

- None.
