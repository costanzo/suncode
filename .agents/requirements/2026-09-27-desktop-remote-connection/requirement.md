# Requirement

## Background

Mobile reaches the Remote Server, but Desktop has no outbound connection. Add a Rust-owned connection so a paired phone can discover Desktop projects and Sessions and send supported chat actions through the existing agent SDK.

## Goals

- Configure a Remote Server URL and pairing code in Avalonia Settings.
- Persist configuration in SQLite and keep the pairing credential plaintext for this initial phase.
- Connect from Desktop to the Remote Server over SSE and send allowlisted agent events over HTTP.
- Correlate event uploads with the SSE request ID and identify the Desktop with `X-Host-Id`.
- Show connection state in Settings and the workspace footer; hide the footer indicator when no server is configured.
- Provide a QR pairing payload after Desktop registration.

## Non-goals

- Desktop user authentication or authorization.
- Remote Server implementation changes.
- Uploading `assistant.delta` or any other event outside the explicit event allowlist.
- Sending provider streaming deltas to the Remote Server.

## Requirements

- Rust owns persistence, transport, event selection, and command dispatch; Avalonia consumes typed SDK APIs.
- Desktop registration submits the configured pairing code and Host metadata, and receives a stable Host ID and Mobile pairing payload.
- The Desktop SSE connection reconnects with bounded backoff and consumes request envelopes containing a unique request ID.
- HTTP uploads include `X-Host-Id` and `X-Request-Id`.
- Allowlisted Rust events are serialized without event-payload rewriting. `assistant.delta` is excluded; final `message.assistant` is uploaded once as a complete payload.
- Remote operations use existing Rust SDK methods so local policy and state remain authoritative.
- The Remote worker is stopped during SDK shutdown and configuration replacement.

## Edge cases

- Invalid URL or pairing code leaves the previous saved connection intact.
- SSE disconnects update the visible state and retry without blocking the desktop UI.
- Duplicate/empty request IDs are rejected before local dispatch.
- A stale worker cannot overwrite connection state after settings are replaced.
- No configured server means the workspace footer has no Remote icon.

## Acceptance criteria

- Rust tests cover event allowlisting, delta exclusion, correlation headers, pairing configuration persistence, and reconnect state transitions.
- Avalonia Settings can configure, connect, disconnect, and show the returned Mobile QR payload.
- The workspace footer displays distinct connected and disconnected states only when configured.
- Focused Rust and Avalonia validation pass, and `git diff --check` is clean.

## Open questions

- Desktop-specific pairing, SSE, and event-ingest endpoints do not yet exist in the Remote Server. This delivery will document a versioned device contract for server implementation.
