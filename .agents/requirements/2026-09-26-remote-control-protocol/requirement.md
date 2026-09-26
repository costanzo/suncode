# Requirement

## Background

The CMP mobile client communicates with SunCode Desktop through a Java Remote Server. Mobile uses HTTP for request/response operations and WebSocket for live session, Host, approval, question, and connection events. The protocol must be documented before either client implementation is coupled to the relay.

## Goals

- Define a versioned HTTP API in OpenAPI format.
- Define the bidirectional WebSocket protocol in AsyncAPI format.
- Support one-time Desktop-generated QR pairing, multiple Hosts, multiple Projects per Host, primary Sessions, approvals, questions, cancellation, retry, and offline resynchronization.
- Make retries safe with explicit request and command idempotency.
- Keep credentials, provider data, and Desktop internals out of the mobile protocol.

## Non-goals

- Defining the Desktop-to-Rust `suncode-remote` internal protocol.
- Defining Remote Server persistence or deployment topology.
- Implementing camera scanning, HTTP clients, WebSocket clients, or server handlers.
- Allowing mobile to archive Sessions or revoke other devices.

## Requirements

1. Pairing consumes an opaque, one-time QR payload and returns a credential scoped to the current mobile device.
2. Authenticated requests use a bearer access token; refresh and logout affect only the current mobile device credential.
3. Session listing defaults to active primary Sessions and accepts Host and Project filters.
4. Mutating HTTP requests require `Idempotency-Key`; WebSocket commands carry the same logical key in `requestId`.
5. The WebSocket stream is live-only. A reconnect resumes from a cursor when possible and otherwise instructs the client to perform HTTP sync.
6. Event payloads contain normalized mobile DTOs, never provider wire payloads, credentials, raw tool secrets, or absolute Desktop filesystem paths.
7. Archived Sessions are not returned by the default list and are never archived by mobile. If a Session is archived elsewhere, the mobile projection removes it and reports the reason.
8. HTTP and WebSocket errors use stable machine codes and a retryable classification.

## Edge cases

- Expired or already-consumed QR payloads.
- Remote Server reachable while Desktop is offline.
- WebSocket cursor too old, invalid, or outside the current retention window.
- Duplicate message, approval, question, cancel, retry, or create-session requests.
- Host, Project, or Session disappearing while a mobile detail page is open.
- Access token expiry during a live WebSocket connection.

## Acceptance criteria

- `contracts/remote-control/http.openapi.yaml` is a self-contained OpenAPI 3.1 document.
- `contracts/remote-control/websocket.asyncapi.yaml` is a self-contained AsyncAPI 3.0 document.
- `contracts/remote-control/README.md` explains lifecycle, authority, cursors, idempotency, and offline behavior.
- The active contracts index links the new protocol documents.
- YAML parses successfully and the repository diff passes `git diff --check`.

## Open questions

- Production token format and key rotation remain Remote Server implementation decisions; the wire contract treats tokens as opaque.
- QR payload signing and the exact device public-key algorithm remain implementation decisions, subject to the fingerprint and one-time-use requirements.
