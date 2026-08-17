# Requirement

## Background

The Avalonia project window exposes cumulative session token usage and tool activity, but it cannot inspect the model-provider exchanges that produced a turn. Developers need a local, bounded, reviewable way to see what the Rust runtime sent to the configured provider and what normalized response, usage, and tool calls came back.

## Goals

- Add a docked Avalonia provider trace drawer modeled after the existing Git drawer.
- Show one row per provider exchange in the selected session.
- Show input tokens, output tokens, cache tokens when reported, total tokens, sent canonical messages, returned assistant content, finish reason, and tool calls.
- Keep provider tracing private to the current desktop data directory and Rust-owned.
- Preserve the Avalonia boundary: the client consumes SDK DTOs and subscriptions only.

## Non-goals

- A cost dashboard, budget system, chart-heavy analytics view, or warning system.
- Cross-session/global provider telemetry.
- Raw HTTP capture, headers, provider API keys, or unredacted credential values.
- Avalonia access to SQLite, provider clients, project files, or vendor-specific adapter internals.
- Estimating token counts when providers omit usage.

## Requirements

- Rust must persist bounded normalized provider-exchange records for each provider call in a turn.
- Provider exchange records must carry session ID, turn ID, exchange ID, provider, stable model ID, wire model, state, timestamps, usage, normalized input messages, normalized output message, normalized tool calls, finish reason, and redacted errors.
- Cache usage must be represented as nullable fields so providers that do not report it are distinguishable from zero.
- The SDK must expose named methods to list provider exchanges for a session and inspect one exchange.
- The C ABI and Avalonia P/Invoke wrapper must expose those methods without generic REST dispatch.
- Avalonia must render a bottom drawer with a request list and detail pane, plus compact filters and copy actions.
- The drawer must not cover the footer and must keep the conversation surface primary when closed.
- Live provider exchange events should update the drawer while a turn is running; durable reload must work after session switch or restart.

## Edge cases

- Provider call starts but is cancelled or fails before usage arrives.
- Provider returns tool calls but no text.
- Provider omits usage or cache token fields.
- Long prompts, long outputs, large tool arguments, and many provider exchanges in one session.
- Session switch while a trace load is in flight.
- Compact windows with both side panels expanded.
- Light and dark theme.

## Acceptance criteria

- Focused Rust tests cover provider exchange persistence and SDK listing/lookup.
- Avalonia builds with the new view and bindings.
- The drawer shows empty, loading, error, running, completed, and failed states.
- No provider secret values appear in SDK DTOs or UI.
- `git diff --check` passes.

## Open questions

- Whether a future opt-in developer mode should expose provider-specific raw wire JSON under a separate architecture decision.
