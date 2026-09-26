# Mobile remote-control protocol

Status: Draft v1 contract for implementation.

This directory defines the public protocol between the CMP Mobile client and the Java Remote Server. The Remote Server relays bounded requests to an outbound Desktop connection; it is not an agent, provider, SQLite owner, or policy authority.

## Documents

- [`http.openapi.yaml`](http.openapi.yaml) — OpenAPI 3.1 HTTP request/response API.
- [`websocket.asyncapi.yaml`](websocket.asyncapi.yaml) — AsyncAPI 3.0 server-to-Mobile Session event stream.

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

## HTTP response envelope

The Java Spring Boot Remote Server uses `ApiBaseRet<T>` for JSON responses:

```json
{
  "code": 0,
  "message": "optional",
  "data": {}
}
```

`code` is always present. `message` and `data` are omitted when null because the server uses Jackson `NON_NULL` inclusion. HTTP error status codes still apply, but their JSON body uses the same envelope. `POST /v1/sessions` is the exception specified by the API contract: a successful request returns `201` with no response body, and the created Session is announced through WebSocket events.

## Idempotency and concurrency

- Every mutating HTTP request requires `Idempotency-Key`.
- HTTP mutations use `Idempotency-Key`; WebSocket has no application-level command channel.
- Replaying the same HTTP key with the same semantic request returns the original result.
- Reusing a key with a different request body fails with `idempotency_key_reused`.
- Approval and question resolution additionally carry `expectedRevision`; a stale decision fails with `revision_conflict` and does not consume the pending interaction.

## Offline and reconnect behavior

The local Mobile cache is authoritative for what can be displayed while disconnected, but never for remote mutation success. The WebSocket carries only live Session events and has no application-level subscribe, ready, ping, or snapshot messages. After reconnect, Mobile calls `GET /v1/sync` with its last cursor and then resumes the event connection. Unsent messages remain local and visibly unsent until a command completion is received. `POST /v1/sessions` returns only HTTP `201`; the created Session and its subsequent state arrive through WebSocket events.

## WebSocket event shape

The WebSocket is server-to-Mobile only and carries only Session-scoped Rust `AgentEvent` values. Each message has `session_id`, `occurred_at`, a transport-level `event_type` discriminator derived from `EventPayload::event_type()`, and the corresponding `payload`. The `event_type` field is added by the relay because the Rust struct stores the discriminator in the enum variant rather than as a separate field. The AsyncAPI document lists every current `EventType` and payload shape, including the shared `QuestionAnsweredPayload` used by both `question.replied` and `question.rejected`.

## Connection states

`connected`, `connecting`, `degraded`, `offline`, `unauthorized`, and `incompatible` are presentation states. `degraded` means the relay is reachable but the selected Desktop is unavailable; it is not equivalent to a successful Desktop command.

## Error handling

HTTP errors use the `ApiBaseRet` envelope with a stable integer `code` and safe human-readable `message`. WebSocket carries no application-level error or command messages; provider errors and raw Desktop diagnostics stay behind the Remote Server boundary and appear only in the relevant Session event payloads.

## Desktop connection boundary

The Desktop-side `suncode-remote` protocol is separate. This contract describes only the Mobile-facing relay and does not authorize the Java server to call Rust SDK methods directly or to bypass Desktop policy and approval state.
