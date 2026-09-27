# Architecture

## Current state

The repository has no standalone developer proxy tool. The design system is a React/Vite review browser with project routes for Desktop and Mobile. SunCode production architecture does not include a client-facing web server.

## Proposed design

`tools/proxy` is an isolated Next.js application with a small custom Node host:

```text
Browser -> Next.js UI :3000
Browser -> /api/* on :3000 -> in-memory capture store
API client -> explicit HTTP proxy :8080 -> upstream API
```

The host owns the proxy listener, capture store, header/body redaction, bounded payload previews, and an SSE event publisher. Next.js owns only the UI route and static assets. The UI consumes the host's JSON snapshot and Server-Sent Events stream.

## Boundaries and dependencies

- The tool is independent of `agent/`, `apps/`, `sdks/`, and SunCode runtime state.
- `http-mitm-proxy` owns explicit HTTP forwarding, HTTPS CONNECT termination, per-host development certificates, and request/response stream hooks.
- Next.js, React, and React DOM are the only UI dependencies.
- The capture store is process-local and intentionally non-durable.
- The generated CA lives under the ignored `tools/proxy/.ca` directory and is never committed.

## Data and control flow

1. The proxy accepts an HTTP absolute-form request or a CONNECT request and terminates HTTPS with the local CA.
2. A capture record is created with a generated id and sanitized request headers.
3. Request bytes are forwarded while a bounded preview is collected.
4. Upstream response headers update the record and are forwarded immediately.
5. Response bytes are forwarded while a bounded preview is collected.
6. SSE lines are parsed incrementally and published as record updates.
7. The monitor UI receives snapshot and update events over `/api/events`.

## Security and failure handling

- Authorization, Cookie, Set-Cookie, Proxy-Authorization, and API-key headers are redacted.
- Capture size is bounded by `CAPTURE_LIMIT_BYTES` and defaults to 256 KiB per direction.
- HTTPS content is only inspectable after the client trusts the generated local CA.
- Error details are normalized to avoid leaking local stack traces into the UI.
- CORS is enabled on the explicit proxy for local development clients.

## Compatibility and migration

No production schema, SDK contract, or SunCode client is changed. `tools/proxy` has its own `package.json` and lockfile.

## Risks and rollback

The tool is process-local, so restarting it clears captures. The local CA changes trust and secret-handling behavior and is documented as development-only. Removing `tools/proxy` and its design-system route rolls the feature back without affecting production clients.

## Open questions

- Decide whether future certificate trust installation helpers should target specific browsers and operating systems.
