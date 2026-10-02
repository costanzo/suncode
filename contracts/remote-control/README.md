# Mobile remote-control protocol

Status: Draft v1 contract for implementation.

This directory defines the public protocol between the CMP Mobile client and the Java Remote Server. The Remote Server relays bounded requests to an outbound Desktop connection; it is not an agent, provider, SQLite owner, or policy authority.

## Documents

- [`mobile.openapi.yaml`](mobile.openapi.yaml) — Normative Mobile to Server HTTP API and Server to Mobile SSE contract (OpenAPI 3.2).
- [`desktop.openapi.yaml`](desktop.openapi.yaml) — Desktop to Server HTTP API.

## Topology

```text
SunCode Desktop -- pairing/event HTTP + Mobile-request SSE --> Remote Server
Mobile          -- HTTPS ---------------------------> Remote Server
Mobile          -- HTTPS / SSE (one Session) -------> Remote Server
```

The Remote Server correlates mobile requests to the correct Desktop connection and translates normalized protocol DTOs. It never forwards provider wire payloads, credentials, raw tool secrets, or absolute filesystem paths to Mobile. Every application request or response body may use exactly one of two representations: the documented plaintext JSON object, or an object containing only `encPayload` with an opaque end-to-end encrypted value. The pairing exchange and token refresh endpoints are explicit plaintext exceptions because the Server must consume the pairing code and refresh token itself. Path variables, query parameters, and routing headers remain visible to the Server and are not encrypted. The Server forwards encrypted bodies without decrypting or interpreting them.

Session creation and message requests may include up to three inline images. Each image contains a MIME type and base64-encoded bytes, with an optional display filename. The Remote Server forwards image objects without interpreting or transforming their contents; encrypted requests continue to use the `encPayload` wrapper.

## Versioning and envelopes

- HTTP paths are prefixed with `/v1`; a deployment may add a servlet context path such as `/remote-server` before it.
- The two OpenAPI files deliberately split ownership: `mobile.openapi.yaml` defines Mobile HTTP plus Server-to-Mobile SSE, while `desktop.openapi.yaml` defines Desktop HTTP plus Server-to-Desktop SSE.
- SSE event names, event IDs, and payload fields are the compatibility boundary.
- New fields and event types are additive. Clients ignore unknown fields and non-required event types.
- Breaking HTTP changes require `/v2`; breaking Mobile SSE changes require a new SSE path or protocol version. The Desktop device contract is an independent private boundary.
- Every response and event carries a stable `requestId` or `eventId` where applicable.

## Pairing and authority

Desktop sends the configured pairing code to `POST /v1/desktop/pairings`; the Server creates a host ID and opaque Desktop bearer token and returns them with the one-time Mobile pairing code and Desktop endpoint paths. Desktop uses `X-Host-Id` and `Authorization: Bearer <desktop-token>` for every later HTTP request and for the command SSE connection. Mobile sends the pairing code to `POST /v1/mobile/pairings/exchange`; a successful exchange consumes it immediately and returns opaque access and refresh tokens bound to the current mobile device. Reusing the code fails with `pairing_consumed`. There is no Host discovery endpoint; Mobile learns a Host only through pairing or the projection of an already paired device.

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

`code` is always present. `message` and `data` are omitted when null because the server uses Jackson `NON_NULL` inclusion. HTTP error status codes still apply, but their JSON body uses the same envelope. `POST /v1/mobile/hosts/{hostId}/sessions` returns `201` with only `sessionId`. The client can immediately open the Session SSE stream; subsequent state arrives through that stream.

## Request identity and concurrency

- Every Mobile HTTP request requires a unique `X-Request-Id`; a retry of the same logical request reuses that ID.
- Mutations use `X-Request-Id` for correlation and deduplication; SSE has no application-level command channel.
- Replaying a mutation with the same request ID and semantic request returns the original result.
- Reusing a request ID with a different request body fails with `request_id_reused`.
- Approval and question resolution additionally carry `expectedRevision`; a stale decision fails with `revision_conflict` and does not consume the pending interaction.

## Offline and reconnect behavior

The local Mobile cache is authoritative for what can be displayed while disconnected, but never for remote mutation success. When Mobile is in the foreground without an open Session, it polls `GET /v1/mobile/hosts/{hostId}/sync` for the selected Host's projection. When a Session is opened, Mobile stops that list polling and opens `GET /v1/mobile/hosts/{hostId}/sessions/{sessionId}/events` with `Accept: text/event-stream`. When Mobile leaves the Session or enters the background, it closes the stream. On returning to the foreground, Mobile performs one `/v1/mobile/hosts/{hostId}/sync` reconciliation before restarting the selected stream or list polling. The sync response contains one `host` object because the path already selects the Host, plus the selected Desktop's authoritative Session pages. After all pages are consumed, Mobile removes locally cached Sessions that are absent from that complete list; each Session entry is Host-free because its Host is already known from the path and `host` field. The Server does not return `removedSessionIds` because it does not know Mobile's local cache. If `resetRequired` is true, Mobile discards its previous sync cursor and cached Host/Session projection and replaces it with the complete synchronization result.

The first connection without `Last-Event-ID` receives a `session.snapshot` event, followed by live events. Snapshot capture and live-subscription registration are atomic with event publication, so an event cannot fall between the snapshot boundary and delivery. In plaintext mode, the snapshot and every event carry a stable `event_id`, a per-Session monotonic `sequence`, and a non-decreasing `session_revision`; the snapshot ID is also emitted as the SSE `id` field. In encrypted mode, the SSE `data` JSON contains only `encPayload`; the event `id` remains visible for replay, ordering, and deduplication, while the encrypted payload contains the corresponding event envelope. Mobile persists the last applied event ID for each Session and sends it as `Last-Event-ID` when reconnecting. The server replays events with a greater sequence before switching to live delivery; duplicate IDs are safe to ignore.

If the requested cursor is no longer retained, the server rejects the stream with HTTP `410` and `cursor_expired`. Mobile must fetch `GET /v1/mobile/hosts/{hostId}/sessions/{sessionId}`, discard the stale event cursor, and reconnect without `Last-Event-ID` to receive a fresh `session.snapshot`. A `401` requires token refresh before reconnecting. Other failures use bounded exponential backoff with jitter. The server sends an SSE `retry` hint where useful and emits a comment heartbeat at least every 20 seconds. Heartbeats have no event ID and do not advance the cursor.

## SSE event shape

The Mobile stream is server-to-Mobile only and carries one Session at a time. It uses standard UTF-8 SSE frames:

```text
id: session-id:42
event: agent.event
data: {"event_id":"session-id:42","sequence":42,...}

```

The first frame on a new stream is `event: session.snapshot`; its data contains the complete Session projection at the represented sequence. Each subsequent `agent.event` data object has `event_id`, `sequence`, `session_revision`, `session_id`, `occurred_at`, `event_type`, and the corresponding `payload`. The `event_type` discriminator is derived from `EventPayload::event_type()` and is added by the relay because the Rust enum stores the discriminator in its variant. The `mobile.openapi.yaml` document lists every current event type and payload shape, including the shared `QuestionAnsweredPayload` used by both `question.replied` and `question.rejected`.

## Connection states

`connected`, `connecting`, `degraded`, `offline`, `unauthorized`, and `incompatible` are presentation states. `degraded` means the relay is reachable but the selected Desktop is unavailable; it is not equivalent to a successful Desktop command.

## Error handling

HTTP errors use the `ApiBaseRet` envelope with a stable integer `code` and safe human-readable `message`. SSE connection failures use the same JSON envelope before a stream is established (`401`, `404`, `410`, `429`, or `503`). Once the stream is established, SSE carries no application-level error or command messages; provider errors and raw Desktop diagnostics stay behind the Remote Server boundary and appear only in the relevant Session event payloads.

## Desktop connection boundary

The Desktop-side `suncode-remote` protocol is defined by `desktop.openapi.yaml`. Desktop pairs through `POST /v1/desktop/pairings`, then opens `GET /v1/desktop/events` with `X-Host-Id`, `X-Request-Id`, and the issued bearer token. The stream sends `desktop.connected`, then one route-specific `mobile.*` custom event for each Mobile HTTP request forwarded to Desktop. Each event's SSE `id` equals the originating Mobile request's `X-Request-Id`; its `data` is a JSON string containing `pathParam`, `queryParam`, and `requestBody`, or only `encPayload` for end-to-end encrypted requests. Successful Mobile pairing and logout also emit sanitized lifecycle events; pairing codes and authentication credentials are excluded. Other Server-owned APIs are not forwarded. Results, snapshots, and allowlisted Rust events are posted over the Desktop HTTP contract with `X-Host-Id`, `X-Request-Id`, and the issued Desktop bearer token.

Desktop periodically posts a snapshot to `/v1/desktop/snapshot` and posts allowlisted Rust session events to `eventsUrl`. Every upload includes `X-Host-Id`, `X-Request-Id`, and the issued Desktop bearer token. `assistant.delta` and provider byte progress are excluded; the complete `message.assistant` event is uploaded once. The desktop worker reconnects with bounded backoff and does not bypass local Rust policy or approval state.
