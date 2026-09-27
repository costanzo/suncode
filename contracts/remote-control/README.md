# Mobile remote-control protocol

Status: Draft v1 contract for implementation.

This directory defines the public protocol between the CMP Mobile client and the Java Remote Server. The Remote Server relays bounded requests to an outbound Desktop connection; it is not an agent, provider, SQLite owner, or policy authority.

## Documents

- [`http.openapi.yaml`](http.openapi.yaml) — OpenAPI 3.1 HTTP request/response API.
- [`sse.asyncapi.yaml`](sse.asyncapi.yaml) — AsyncAPI 3.0 Mobile Session-scoped Server-Sent Events stream.

## Topology

```text
SunCode Desktop -- outbound authenticated WebSocket --> Remote Server
Mobile          -- HTTPS ---------------------------> Remote Server
Mobile          -- HTTPS / SSE (one Session) -------> Remote Server
```

The Remote Server correlates mobile requests to the correct Desktop connection and translates normalized protocol DTOs. It never forwards provider wire payloads, credentials, raw tool secrets, or absolute filesystem paths to Mobile.

## Versioning and envelopes

- HTTP paths are prefixed with `/v1`.
- The HTTP path prefix is `/v1`; SSE event names, event IDs, and payload fields are the compatibility boundary.
- New fields and event types are additive. Clients ignore unknown fields and non-required event types.
- Breaking HTTP changes require `/v2`; breaking Mobile SSE changes require a new SSE path or protocol version. The Desktop WebSocket is an independent private boundary.
- Every response and event carries a stable `requestId` or `eventId` where applicable.

## Pairing and authority

Desktop creates an opaque, short-lived, one-time QR payload. Mobile sends the payload to `POST /v1/pairings/exchange`; a successful exchange consumes it immediately and returns opaque access and refresh tokens bound to the current mobile device. Reusing the payload fails with `pairing_consumed`. There is no Host discovery endpoint; Mobile learns a Host only through pairing or the projection of an already paired device.

Mobile may inspect Hosts, Projects, active primary Sessions, cached content, approvals, and questions. It may create Sessions, send messages, cancel or retry turns, and resolve approvals/questions. It cannot archive Sessions, delete Sessions, revoke other mobile devices, or widen Rust policy.

## Authentication

- Authenticated HTTP calls use `Authorization: Bearer <access-token>`.
- SSE authentication uses the same bearer credential in the HTTP `Authorization` header. A reconnect repeats normal HTTP authentication.
- Refresh and logout operate on the current mobile device credential only.
- Tokens are opaque and must not be written to logs, events, analytics, or Session content.

## HTTP response envelope

The Java Spring Boot Remote Server uses `ApiBaseRet<T>` for JSON responses:

```json
{
  "code": 0,
  "message": "optional",
  "data": {}
}
```

`code` is always present. `message` and `data` are omitted when null because the server uses Jackson `NON_NULL` inclusion. HTTP error status codes still apply, but their JSON body uses the same envelope. `POST /v1/sessions` returns `201` with `sessionId` and the accepted timestamp. The client can immediately open the Session SSE stream; subsequent state arrives through that stream.

## Idempotency and concurrency

- Every mutating HTTP request requires `Idempotency-Key`.
- HTTP mutations use `Idempotency-Key`; SSE has no application-level command channel.
- Replaying the same HTTP key with the same semantic request returns the original result.
- Reusing a key with a different request body fails with `idempotency_key_reused`.
- Approval and question resolution additionally carry `expectedRevision`; a stale decision fails with `revision_conflict` and does not consume the pending interaction.

## Offline and reconnect behavior

The local Mobile cache is authoritative for what can be displayed while disconnected, but never for remote mutation success. When Mobile is in the foreground without an open Session, it polls `GET /v1/sync` for Host and Session projections. When a Session is opened, Mobile stops that list polling and opens `GET /v1/sessions/{sessionId}/events` with `Accept: text/event-stream`. When Mobile leaves the Session or enters the background, it closes the stream. On returning to the foreground, Mobile performs one `/v1/sync` reconciliation before restarting the selected stream or list polling.

The first connection without `Last-Event-ID` receives a `session.snapshot` event, followed by live events. Snapshot capture and live-subscription registration are atomic with event publication, so an event cannot fall between the snapshot boundary and delivery. The snapshot and every event carry a stable `event_id`, a per-Session monotonic `sequence`, and a non-decreasing `session_revision`; the snapshot ID is also emitted as the SSE `id` field. Mobile persists the last applied event ID for each Session and sends it as `Last-Event-ID` when reconnecting. The server replays events with a greater sequence before switching to live delivery; duplicate IDs are safe to ignore.

If the requested cursor is no longer retained, the server rejects the stream with HTTP `410` and `cursor_expired`. Mobile must fetch `GET /v1/sessions/{sessionId}`, discard the stale event cursor, and reconnect without `Last-Event-ID` to receive a fresh `session.snapshot`. A `401` requires token refresh before reconnecting. Other failures use bounded exponential backoff with jitter. The server sends an SSE `retry` hint where useful and emits a comment heartbeat at least every 20 seconds. Heartbeats have no event ID and do not advance the cursor.

## SSE event shape

The Mobile stream is server-to-Mobile only and carries one Session at a time. It uses standard UTF-8 SSE frames:

```text
id: session-id:42
event: agent.event
data: {"event_id":"session-id:42","sequence":42,...}

```

The first frame on a new stream is `event: session.snapshot`; its data contains the complete Session projection at the represented sequence. Each subsequent `agent.event` data object has `event_id`, `sequence`, `session_revision`, `session_id`, `occurred_at`, `event_type`, and the corresponding `payload`. The `event_type` discriminator is derived from `EventPayload::event_type()` and is added by the relay because the Rust enum stores the discriminator in its variant. The AsyncAPI document lists every current event type and payload shape, including the shared `QuestionAnsweredPayload` used by both `question.replied` and `question.rejected`.

## Connection states

`connected`, `connecting`, `degraded`, `offline`, `unauthorized`, and `incompatible` are presentation states. `degraded` means the relay is reachable but the selected Desktop is unavailable; it is not equivalent to a successful Desktop command.

## Error handling

HTTP errors use the `ApiBaseRet` envelope with a stable integer `code` and safe human-readable `message`. SSE connection failures use the same JSON envelope before a stream is established (`401`, `404`, `410`, `429`, or `503`). Once the stream is established, SSE carries no application-level error or command messages; provider errors and raw Desktop diagnostics stay behind the Remote Server boundary and appear only in the relevant Session event payloads.

## Desktop connection boundary

The Desktop-side `suncode-remote` protocol is separate and remains an outbound authenticated WebSocket. Its private command envelopes are outside this Mobile-facing contract. The relay uses SSE for Mobile event delivery and does not authorize the Java server to call Rust SDK methods directly or to bypass Desktop policy and approval state.
