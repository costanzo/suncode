# Requirement

## Background

The CLI can create a session with one-shot `run` and list/archive sessions. Users need to add another one-shot turn to an existing session without introducing an interactive chat mode.

## Goals

- Resume an existing primary session by submitting exactly one new turn.
- Preserve prior session context through the Rust agent's durable history.
- Reuse typed event streaming, cancellation, output, and explicit shutdown behavior from `run`.
- Fail closed when the session is already suspended on an approval or structured question.

## Non-goals

- Interactive or multi-turn `chat`.
- Interactive approval/question resolution.
- History printing, TUI, line editing, PTY, or daemon attach.
- Session rename, pin, delete, or child-session submission.

## Requirements

- Add `suncode session resume SESSION_ID (--prompt TEXT | --stdin)`.
- Accept optional `--model` and `--reasoning-effort`, using `SUNCODE_MODEL` and `SUNCODE_REASONING_EFFORT` when flags are absent.
- Reopen an archived primary session before submission.
- Establish atomic `watch_session` before submitting the new turn.
- Reject a session with a durable pending approval or question using exit status 4.
- Consume typed events, re-watch after lag, drain tail events, and support first/second interrupt semantics.
- Text mode writes only final assistant text to stdout; JSONL ends with `session.resume.result`.
- Consume explicit SDK shutdown before reporting success.

## Edge cases

- Reject missing/conflicting/empty prompt sources.
- Reject missing sessions and child sessions through SDK validation.
- Reject an unexpected queued response.
- A resumed archived session remains active after the command.

## Acceptance criteria

- Focused tests cover grammar, archived reopening, durable context reuse, JSONL result type, stdin, and pending suspension rejection.
- CLI, data/agent, Rust SDK, C binding, Avalonia, Clippy, formatting, and diff checks pass.

## Open questions

- None.
