# Test Plan

## Scope

Validate schema registration, argument rules, turn execution, normalized persistence, live event projection, and desktop rendering bindings.

## Unit tests

- Rust accepts an empty or valid list and rejects invalid statuses, priorities, oversized content, and multiple active items.
- Avalonia maps status markers and opacity and restores the latest replacement list from a normalized snapshot.

## Integration and conformance tests

- Verify a successful call emits `todo.updated`, replaces the `session_turn_todo` rows, records a succeeded `todowrite` tool use, and feeds the JSON result to the next model request.

## Regression checks

- Exact built-in registry count and tool names.
- Existing question, approval, session snapshot, and tool timeline tests.

## Manual checks

- Open the desktop review sidebar, submit a multi-step turn, and confirm the current todo list updates as the agent progresses.

## Commands and results

- Recorded in the final task report after verification.

## Residual risks

- Todo state is intentionally not a cross-turn/project task store.
