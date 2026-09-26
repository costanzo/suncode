# Mobile remote-control protocol

Status: Draft v1 contract for implementation.

This directory defines the public protocol between the CMP Mobile client and the Java Remote Server. The Remote Server relays bounded requests to an outbound Desktop connection; it is not an agent, provider, SQLite owner, or policy authority.

## Documents

- [`http.openapi.yaml`](http.openapi.yaml) — OpenAPI 3.1 HTTP request/response API.
- [`websocket.asyncapi.yaml`](websocket.asyncapi.yaml) — AsyncAPI 3.0 bidirectional WebSocket protocol.

## Topology

```text
SunCode Desktop -- outbound authenticated WebSocket --> Remote Server
Mobile          -- HTTPS / WSS ----------------------> Remote Server
```

The Remote Server correlates mobile requests to the correct Desktop connection and translates normalized protocol DTOs. It never forwards provider wire payloads, credentials, raw tool secrets, or absolute filesystem paths to Mobile.

## Versioning and envelopes

- HTTP paths are prefixed with `/v1`.
- The HTTP path prefix is `/v1`; WebSocket message `type` values and payload fields are the compatibility boundary.
- New fields and event types are additive. Clients ignore unknown fields and non-required event types.
- Breaking HTTP changes require `/v2`; breaking WebSocket changes require a new WebSocket path or protocol version.
- Every response and event carries a stable `requestId` or `eventId` where applicable.

## Pairing and authority

Desktop creates an opaque, short-lived, one-time QR payload. Mobile sends the payload to `POST /v1/pairings/exchange`; a successful exchange consumes it immediately and returns opaque access and refresh tokens bound to the current mobile device. Reusing the payload fails with `pairing_consumed`. There is no Host discovery endpoint; Mobile learns a Host only through pairing or the projection of an already paired device.

Mobile may inspect Hosts, Projects, active primary Sessions, cached content, approvals, and questions. It may create Sessions, send messages, cancel or retry turns, and resolve approvals/questions. It cannot archive Sessions, delete Sessions, revoke other mobile devices, or widen Rust policy.

## Authentication

- Authenticated HTTP calls use `Authorization: Bearer <access-token>`.
- WebSocket authentication uses the same bearer credential during the handshake.
- Refresh and logout operate on the current mobile device credential only.
- Tokens are opaque and must not be written to logs, events, analytics, or Session content.

## Idempotency and concurrency

- Every mutating HTTP request requires `Idempotency-Key`.
- Every mutating WebSocket command carries `requestId`, which is also the relay correlation key.
- Replaying the same key with the same semantic request returns the original result.
- Reusing a key with a different request body fails with `idempotency_key_reused`.
- Approval and question resolution additionally carry `expectedRevision`; a stale decision fails with `revision_conflict` and does not consume the pending interaction.

## Offline and reconnect behavior

The local Mobile cache is authoritative for what can be displayed while disconnected, but never for remote mutation success. The WebSocket handshake accepts `resumeCursor`. If the cursor is resumable, the server sends missed events followed by `ready`; otherwise it sends `snapshot.required`, and Mobile calls `GET /v1/sync` with its last cursor before resubscribing. Unsent messages remain local and visibly unsent until a command completion is received. `POST /v1/sessions` returns only HTTP `201`; the created Session and its subsequent state arrive through WebSocket events.

## Connection states

`connected`, `connecting`, `degraded`, `offline`, `unauthorized`, and `incompatible` are presentation states. `degraded` means the relay is reachable but the selected Desktop is unavailable; it is not equivalent to a successful Desktop command.

## Error handling

HTTP errors use the `ErrorResponse` schema. WebSocket errors use the `error` event. Both include a stable `code`, safe human-readable `message`, `retryable`, and `requestId` when a request exists. Provider errors and raw Desktop diagnostics stay behind the Remote Server boundary.

## Desktop connection boundary

The Desktop-side `suncode-remote` protocol is separate. This contract describes only the Mobile-facing relay and does not authorize the Java server to call Rust SDK methods directly or to bypass Desktop policy and approval state.
