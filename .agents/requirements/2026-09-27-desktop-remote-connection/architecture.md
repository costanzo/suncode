# Architecture

## Current state

The Rust SDK owns SQLite and in-process event subscriptions. Mobile has HTTP and SSE clients. The Java Remote Server currently has no Desktop pairing or SSE request endpoints.

## Proposed design

Add a Rust SDK Remote controller. It stores the Desktop URL and pairing code through a dedicated Rust API, performs pairing with HTTP, reconnects to a Desktop request SSE stream, dispatches supported commands through an SDK view over the same agent state, and posts allowlisted agent events to the Remote Server. The C ABI and typed C# SDK expose configuration, connection control, and status. Avalonia provides the Settings panel, QR presentation, and footer indicator.

## Boundaries and dependencies

- Rust SDK owns the transport worker and SQLite settings.
- C ABI owns only typed serialization and lifecycle bridging.
- C# SDK and Avalonia own presentation and QR rendering.
- Remote requests cannot bypass the existing SDK facade or local agent policy.

## Data and control flow

1. User enters server URL and pairing code.
2. Rust stores the configuration and pairs with `POST /v1/desktop/pairings`.
3. Server returns `host_id`, `mobile_pairing_payload`, and Desktop SSE/event HTTP paths.
4. Rust opens the returned SSE path. Each `desktop.request` event has `request_id` and a bounded operation envelope.
5. Rust dispatches the request through typed SDK methods and posts the result to the returned result path with `X-Host-Id` and `X-Request-Id`.
6. Rust session subscriptions upload only the named event allowlist to the returned events path with the same headers.

## Security and failure handling

Desktop authentication is deferred. The pairing code is stored in plaintext SQLite and sent only to the configured Remote Server. TLS certificate validation follows system defaults. No credentials or event bodies are written to logs. Remote command dispatch accepts only named methods and known fields.

## Compatibility and migration

Settings use the existing SQLite `configuration` table and require no schema migration. Desktop transport endpoints are a new `/v1/desktop` contract and must be implemented by the Remote Server before end-to-end operation.

## Risks and rollback

The Remote Server implementation may choose different endpoint names. Keep endpoint paths in one Rust protocol module and update this contract with the server implementation. Removing the saved configuration disables the worker and hides the footer status.

## Open questions

- The endpoint contract is proposed for coordination; current server source does not yet implement it.
