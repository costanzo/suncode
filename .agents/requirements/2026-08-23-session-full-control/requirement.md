# Requirement

## Background

Every approval-gated operation currently requires an allow-once or deny decision. Developers need an explicit session-level Full Control grant for sessions where repeated approvals impede the workflow.

## Goals

- Add an `Allow for session` approval decision.
- Persist Full Control as session-scoped configuration.
- Make the active elevated-authority state continuously visible and easy to turn off.

## Non-goals

- Bypassing argument validation, project boundaries, operation auditing, or unknown-tool denial.
- Adding a global or project-wide Full Control grant.
- Automatically enabling Full Control without an approval decision.

## Requirements

- `allow_session` atomically approves the pending operation and stores `full_control=true` for its session.
- Known approval-gated operations in that session are allowed without further approval while Full Control is enabled.
- `full_control=false` restores normal approval behavior for subsequent operations.
- The approval surface offers deny, allow once, and allow for session actions.
- The Agent Processes section shows a warning-styled Full Control indicator with a `Turn off` action only while Full Control is enabled.

## Edge cases

- A stale or already-resolved approval must not enable Full Control.
- Turning Full Control off does not cancel an operation already executing.
- Switching sessions reloads the selected session's effective configuration without leaking the previous session's state.
- Unknown tools and scope violations remain denied.

## Acceptance criteria

- Full Control survives session reload through the `configuration` table.
- A second approval-gated tool in the same session proceeds without another approval after `allow_session`.
- Turning Full Control off causes later approval-gated tools to request approval again.
- Focused Rust and Avalonia tests pass.
