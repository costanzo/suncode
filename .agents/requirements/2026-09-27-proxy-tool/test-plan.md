# Test Plan

## Scope

Verify proxy forwarding, capture redaction, bounded body previews, SSE parsing, UI build, and design-system routing.

## Unit tests

- Header redaction never returns credential values.
- Capture buffers truncate at the configured limit.
- SSE parser emits complete events and handles an unfinished final event.

## Integration and conformance tests

- Local upstream HTTP server receives a proxied request and returns a JSON response.
- Local upstream SSE server remains streaming while the capture record updates.
- Upstream failure creates a visible failed record.

## Regression checks

- Existing design-system routes remain buildable.
- No production Rust, Avalonia, or mobile targets change.

## Manual checks

- Open `/projects/proxy-tool` in both themes.
- Start the proxy, send a request through `localhost:8080`, and inspect it in the UI.
- Resize the monitor to a narrow viewport and verify the detail panel yields cleanly.

## Commands and results

- `npm test` in `tools/proxy` — passed.
- `npm run build` in `tools/proxy` — passed.
- `npm run build` in `design-system` — passed.
- Local upstream JSON and SSE forwarding through port 8080 — passed.
- `git diff --check` — passed.

## Residual risks

- HTTPS inspection requires the client to trust the generated local CA; untrusted clients fail closed with a certificate error.
