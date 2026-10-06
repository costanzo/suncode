# Remote Control

**Status:** Implemented and focused-tested per component; no committed end-to-end conformance suite

A paired phone can browse a Desktop's Projects and active primary Sessions, open one Session live, create Sessions, send messages, cancel or retry turns, and resolve approvals and questions. Traffic flows `Desktop ↔ Remote Server ↔ Mobile`; the Desktop only makes outbound connections. The normative wire contract is `contracts/remote-control/` (`README.md`, `mobile.openapi.yaml`, `desktop.openapi.yaml`); this note does not restate it.

## Components

- **Desktop connection (Rust agent transport, `agent/crates/remote`).** `suncode-remote` owns configuration, pairing, encryption, the reconnecting worker, SSE request/response transport, and event upload. `suncode-agent::Agent` owns remote snapshots and command dispatch; the Rust SDK facade supplies only the host configuration and event-stream bridge, then exposes the unchanged methods through the C ABI and typed C# SDK.
- **Desktop UI (Avalonia).** Settings has a Remote Server panel for server URL, Desktop pairing code, an end-to-end encryption toggle (on by default), connect/disconnect/clear, and the Mobile pairing QR code (rendered locally with QRCoder). The Workspace footer shows a connected/disconnected indicator only when a server is configured.
- **Remote Server (`apps/remote-server`, Java Spring Boot).** Relays Mobile HTTP requests to the Desktop SSE stream, serves Mobile Session SSE with snapshot, replay, and heartbeat, and issues Desktop and Mobile tokens. It is a relay, not an agent, provider, SQLite owner, or policy authority. Pairings, tokens, and relay state are held in memory, so a server restart requires re-pairing.
- **Mobile client.** See [`mobile-client/`](../mobile-client/README.md).

## Desktop behavior users can rely on

- Pairing posts the configured Desktop pairing code to `/v1/desktop/pairings`. The returned Host ID, Desktop tokens, and one-time Mobile pairing code are stored in the SQLite `configuration` table. Desktop then generates a local 256-bit AES key and builds the QR URL (endpoint, `hostId`, `code`, `k`, `e2e`).
- The worker opens `GET /v1/desktop/events` (SSE). It reconnects with exponential backoff from 1 s up to 30 s, posts a Desktop snapshot at connect time and every 30 s, and subscribes to existing Sessions.
- Each forwarded `mobile.*` request maps to a fixed operation allowlist: list projects or sessions, create or get a Session, send a message, cancel, retry, resolve an approval, or reply to a question. Unknown operations are rejected. Every operation goes through the normal `AsyncAgentSdk` methods, so local Rust policy, approvals, and state stay authoritative. Results are posted to `/v1/desktop/responses` and correlated by `X-Request-Id`.
- Only an explicit allowlist of Session events is uploaded: turn, tool, message, usage, compaction, approval, question, todo, and checkpoint events. Remote `assistant.delta` events are coalesced by turn and uploaded at paragraph boundaries or 160 Unicode characters, with a 120 ms maximum buffering interval. The final `message.assistant` remains the authoritative complete payload and replaces its streamed draft in Web.
- When end-to-end encryption is on, request bodies, results, snapshots, and events are AES-256-GCM `encPayload` values (`e2e-v1:` prefix). Path, query, and routing headers remain visible to the server.

## Security posture

- The connection is outbound only. The Desktop exposes no listening socket.
- The Desktop pairing code, Desktop access and refresh tokens, Mobile pairing code, and AES key are stored in **plaintext** in SQLite, consistent with the current local-secret policy. The settings panel can clear them.
- The Mobile pairing code is single-use and expires 5 minutes after Desktop pairing. The AES key never goes to the server; it reaches Mobile only through the QR code. To rotate it, clear the configuration and pair again.
- Both `http` and `https` endpoints are accepted, and TLS validation uses system defaults. Plain `http` exposes tokens and any plaintext bodies to the network.
- Mobile cannot archive or delete Sessions, revoke other devices, or widen Rust policy. The relay is not a sandbox: a paired phone has the same agent authority that local policy grants an interactive Desktop user.

## Not implemented

- Desktop does not call `/v1/desktop/auth/refresh`. It reuses the Desktop access token from pairing until re-pairing.
- Inline images in Mobile Session creation and message requests are accepted by the contract and the Mobile DTOs, but the Desktop dispatcher forwards only text.
- There is no committed cross-component conformance suite (Desktop, Server, and Mobile together), and no persistent server storage, background delivery, or push notifications.
