# Requirement

## Background

The project-window footer shows only the selected model. Provider usage is durably emitted per turn, and SQLite already reserves token columns on the `turns` projection, but those columns are not populated or exposed through the embedded SDK.

## Goals

- Show the selected session's cumulative provider-reported token consumption in the footer.
- Keep SQLite ownership and aggregation inside Rust.
- Keep the display compact and subordinate to the conversation surface.

## Non-goals

- Show current-turn usage, input/output breakdowns, context-window occupancy, cost estimates, budgets, charts, or warnings.
- Let Qt access SQLite directly.
- Estimate usage when a provider does not report it.

## Requirements

- Each `usage.updated` event updates the owning turn's cumulative input, output, and total token projection.
- Schema migration backfills existing turn projections from the latest durable usage event for each turn.
- The Rust SDK exposes a named `session_usage(session_id)` method and matching C ABI function.
- Session usage is the sum of the latest cumulative usage for every turn in the session.
- Qt refreshes the value on session selection and after live usage updates.
- The footer displays the model and a compact `Session <value> tokens` value.

## Edge cases

- A new or usage-free session displays `0 tokens`.
- Repeated cumulative usage events replace a turn projection rather than being summed together.
- Switching sessions cannot apply a late asynchronous result from the previous session.
- Missing provider usage contributes zero and is not estimated.

## Acceptance criteria

- Persistence tests cover cumulative replacement, cross-turn aggregation, and migration backfill.
- SDK and C ABI tests cover the named session usage operation.
- The Qt footer updates from durable session usage without direct database access.
- Rust tests, Qt build, QML lint, startup smoke test, and `git diff --check` pass.

## Open questions

- None.
