# Requirement

**Status:** Implemented

## Background

The Rust agent needs a first-class way to pause when the user's intent is ambiguous. OpenCode and Claude Code expose this as a question tool that presents structured choices, accepts optional custom text, and resumes the same turn after the user responds.

## Goals

- Advertise a model-facing `question` tool with one or more structured prompts.
- Suspend the active turn until the desktop client submits answers or skips the request.
- Preserve the continuation in the current turn recovery snapshot and restore it after a process restart.
- Expose live and snapshot DTOs through the Rust SDK and C ABI.
- Present single-select, multi-select, and custom-answer controls in Avalonia.

## Non-goals

- Remote question delivery or multi-user sessions.
- Compatibility with retired runtime or native API names.
- Persisting a separate question history table.

## Requirements

1. Each prompt contains `question`, `header`, `options`, `multiple`, and optional `custom` fields.
2. Invalid prompt shapes and invalid answer labels fail closed before resuming the turn.
3. `question.asked`, `question.replied`, and `question.rejected` events correlate request, turn, and tool-call IDs.
4. A question pauses later sibling calls in the same provider response and resumes them after the answer.
5. Skipping continues with an explicit rejected/unanswered tool result.

## Acceptance criteria

- Rust registry, validation, recovery, and continuation tests pass.
- Avalonia displays pending questions from both snapshots and live events.
- The C ABI exposes answer and reject operations.
- Rust workspace tests, Avalonia tests, formatting, and diff validation pass.

## Open questions

- None.
