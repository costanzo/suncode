# Test Plan

## Scope

Validate root instruction injection, nested discovery, precedence, deduplication, scope enforcement, and regression behavior.

## Unit tests

- Root `AGENTS.md` produces a system message without an absolute path.
- Nested files are returned nearest-first, while root and direct-read instructions are omitted.
- Previously attached nested paths are not returned again within the turn.

## Integration and conformance tests

- Provider exchange input contains root repository instructions on every model call.
- A successful nested read persists `repository_instructions` in its normalized result.

## Regression checks

- Rust workspace and production-library clippy.
- Avalonia build and test suite.
- Formatting and diff validation.

## Manual checks

- Open a project containing root and nested `AGENTS.md` files, request work in the nested directory, and inspect provider/tool traces.

## Commands and results

- Recorded in the final task report.

## Residual risks

- Instruction files consume model context and are therefore bounded; oversized files are ignored rather than partially applied.
