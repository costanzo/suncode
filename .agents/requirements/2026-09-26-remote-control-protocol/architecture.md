# Architecture

## Current state

The mobile client is a CMP application with a UI-first repository placeholder. SunCode Desktop embeds the Rust agent and does not expose a client-facing socket. The approved future topology is `Desktop ↔ Remote Server ↔ Mobile`.

## Proposed design

The Java Remote Server owns the public HTTP and WebSocket surface. A paired Desktop maintains an outbound authenticated WebSocket to the Remote Server. Mobile sends HTTP commands to the relay; the WebSocket from Remote Server to Mobile carries only normalized Session-scoped Rust `AgentEvent` values. Desktop-originated state changes are projected to Mobile as those WebSocket events.

## Boundaries and dependencies

- Mobile owns local navigation, cache, token storage, camera permission, and presentation.
- Remote Server owns mobile authentication, pairing consumption, connection routing, request correlation, rate limits, and protocol adaptation.
- Desktop `suncode-remote` owns the Desktop-side WebSocket client, translation to Rust SDK calls, and redaction of Rust errors/events into protocol DTOs.
- Rust agent remains the authority for Session lifecycle, approvals, questions, policy, and machine operations.

## Data and control flow

1. Desktop creates a one-time pairing payload through the Remote Server and renders it as a QR code.
2. Mobile exchanges the payload for an opaque access/refresh credential pair.
3. Mobile lists Hosts, Projects, and active primary Sessions over HTTP.
4. Mobile opens WebSocket `/v1/ws` with the bearer credential and receives only live Session events.
5. Mutations use HTTP and are accepted by the relay, correlated to the correct Desktop connection, and reflected later through Session events.
6. After reconnect, Mobile performs `/v1/sync` and then resumes the event connection; there is no application-level WebSocket subscribe or snapshot message.

## Security and failure handling

- All production HTTP and WebSocket traffic uses TLS.
- Pairing payloads are opaque, short-lived, single-use, and bound to the Desktop/Remote Server pairing transaction.
- Access and refresh tokens are opaque to the clients and never appear in events or logs.
- Mobile cannot archive Sessions or revoke other devices.
- Remote Server does not claim that Desktop filesystem changes are undoable through mobile; the protocol exposes only Rust-owned approval and Session state.
- Every mutation is idempotent or explicitly rejected when a safe replay cannot be guaranteed.

## Compatibility and migration

The HTTP path prefix `/v1` is the stable compatibility boundary. WebSocket message `type` values and payload fields are versioned by the WebSocket path/protocol contract; additive fields and event types are allowed. Unknown fields and unknown event types must be ignored by clients unless the envelope marks the event as required. Breaking HTTP changes require `/v2`.

## Risks and rollback

The protocol is documentation-only in this delivery. If implementation reveals an incompatible routing or authentication assumption, update both schemas and the requirement package before shipping code; do not silently fork HTTP and WebSocket semantics.

## Open questions

- Remote Server deployment authentication between Desktop and relay is intentionally outside the mobile-facing contract.
- Mobile token storage uses platform secure storage but the exact library is a CMP implementation choice.
