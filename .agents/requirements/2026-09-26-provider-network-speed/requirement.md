# Requirement

## Background

The Avalonia project Workspace footer has a compact status area, but the Rust LLM provider layer does not expose transport progress. Users cannot tell whether a model request is currently uploading prompt data or receiving a response.

## Goals

- Show aggregate LLM request upload and download rates in the bottom-right Workspace footer, refreshed every three seconds using a three-second average.
- Count provider HTTP request-body and response-body bytes in the Rust provider adapters.
- Deliver transient progress through the existing session event stream without persisting it.
- Keep the footer compact and expose provider/model details through its tooltip.

## Non-goals

- Measuring OS network-interface throughput or TLS, HTTP, and proxy framing overhead.
- Persisting transfer metrics in SQLite or Provider Trace.
- Changing the Anthropic response protocol to SSE in this delivery.

## Requirements

- Built-in OpenAI-compatible and Anthropic providers report request-body and response-body byte progress.
- Provider transfer progress events identify session, turn, exchange, provider, model, and cumulative uploaded/downloaded byte counts.
- Events are coalesced before publication to avoid flooding the bounded session event stream, with a final cumulative sample on completion.
- The Workspace footer shows current aggregate rates for active transfers.
- The Workspace footer remains present while a project is open and shows zero rates when no transfer is active.
- Rate values refresh every three seconds, average bytes transferred in that three-second interval, and use binary units consistently.
- Custom in-process providers remain source-compatible where practical; if a public trait change is required, update its contract and focused tests.

## Edge cases

- Failed, cancelled, and concurrent exchanges must stop contributing after completion.
- A Workspace that changes session must not display stale transfer state from the prior session.
- Very small transfers may show 0 B/s briefly while still exposing their cumulative byte counts in the tooltip.
- Anthropic error response bodies and successful response bodies both contribute downloaded bytes.

## Acceptance criteria

- Focused Rust tests prove both provider adapters report non-zero upload and download byte counts.
- Core maps progress into transient typed events and does not persist progress samples.
- C# deserializes the progress payload and the Workspace footer updates from live events.
- Design-system Workspace specimen documents footer placement, compact states, and tooltip content.
- Applicable Rust and .NET checks pass, and `git diff --check` is clean.

## Open questions

- None. The displayed rates represent HTTP body bytes observed by `reqwest`, not physical network-interface throughput.
