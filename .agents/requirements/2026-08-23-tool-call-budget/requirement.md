# Requirement

## Background

Turns currently use a hard-coded 32 tool-call limit. When a turn exceeds that limit, its projected terminal state can prevent the later failure write from retaining structured error details, and a multi-call provider response can be left partially processed.

## Goals

- Retain structured failure details after terminal state projection.
- Raise the default tool-call limit to 64.
- Allow each project to configure a limit from 1 through 256 in Settings.
- Apply one stable limit for the lifetime of a turn, including approval continuation.
- Reject an over-budget provider batch without partially authorizing or executing it.

## Non-goals

- Add global or session overrides for the tool-call limit.
- Change the iteration budget.
- Dynamically alter a turn already in progress when Settings changes.

## Requirements

- `tool_call_limit` is a project-only JSON integer in the inclusive range 1 through 256.
- Projects without the setting use 64.
- The resolved limit is serialized in the turn continuation; legacy continuations without it use 64.
- If the next provider batch would exceed the remaining budget, every call in that rejected batch is projected as requested and failed with `tool_budget_exceeded`; none is authorized or executed.
- `fail_turn` may enrich an already failed turn with `error_json` and `error_code`, but must not overwrite completed, cancelled, or interrupted turns.
- The desktop Settings window exposes a bounded numeric control and disables it when no project is open.

## Edge cases

- Persisted malformed or out-of-range project values fail typed resolution.
- A setting change affects only turns admitted after the save.
- Repeating `fail_turn` remains safe and retains a complete terminal error.

## Acceptance criteria

- Database and agent regression tests cover terminal error persistence and atomic budget overflow.
- SDK validation rejects invalid scope, type, and range.
- Avalonia can read and save the selected project's value.
- Focused tests, builds, formatting, and diff checks pass.
