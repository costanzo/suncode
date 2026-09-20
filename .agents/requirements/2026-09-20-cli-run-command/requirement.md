# Requirement

## Background

The native Rust CLI has an administrative foundation but needs the smallest complete coding workflow before interactive chat and session management. The SDK already owns project/session creation, turn submission, typed subscriptions, cancellation, policy outcomes, and shutdown.

## Goals

- Execute one coding turn with `suncode run`.
- Accept exactly one prompt source: explicit text or piped stdin.
- Render typed live events and a stable terminal result.
- Preserve SDK ownership of agent behavior, authority, persistence, and provider access.
- Handle cancellation, lag, approval/question suspension, and shutdown predictably.

## Non-goals

- Interactive multi-turn chat.
- Session list, resume, or archive.
- Interactive approval or structured-question prompts.
- Named non-interactive policy profiles.
- TUI, PTY, daemon/attach, Browser Use, or Computer Use.

## Requirements

- Add `run [PATH] (--prompt TEXT | --stdin)` with optional model and reasoning effort.
- All new environment variables use the `SUNCODE_` prefix.
- Open the project, create a primary session, and establish atomic watch before submission.
- Consume typed SDK events without depending on agent core, provider, SQLite, or operation crates.
- Recover from stream lag through a new atomic watch.
- Drain already-published events before writing the final result.
- First interrupt requests turn cancellation; second interrupt returns exit status 130.
- Approval or question suspension returns exit status 4 and does not read continuation input from stdin.
- Text stdout contains only final assistant text; progress uses stderr. JSONL emits typed event envelopes and `run.result`.
- Explicitly consume SDK shutdown on every normal command outcome after SDK open.

## Edge cases

- Reject missing, conflicting, interactive-terminal, or empty prompt input.
- Reject unsupported model/reasoning combinations through existing SDK validation.
- Treat an unexpected queued response as failure because each invocation creates a new session.
- Do not lose tail events when submission completion and event readiness occur together.

## Acceptance criteria

- Focused unit and integration tests cover grammar, prompt/stdin, typed deltas, final output separation, and environment-driven model/reasoning selection.
- CLI strict Clippy, formatting, locked/offline tests, agent/SDK/C regressions, desktop tests, and diff checks pass.

## Open questions

- None for this delivery.
