# Requirement

## Background

Developers need a small standalone tool for inspecting API traffic while developing integrations. The tool must provide a local explicit HTTP proxy and a browser UI that makes request, response, header, body, timing, and Server-Sent Events data easy to inspect.

## Goals

- Add an independent Next.js development tool under `tools/proxy`.
- Run a monitoring UI on port 3000 and an explicit proxy listener on port 8080 by default.
- Record requests that pass through the proxy in memory and stream updates to the UI.
- Display request and response metadata, redacted headers, text or JSON payloads, and SSE events.
- Provide clear empty, loading, error, streaming, and large-payload states.
- Add a matching `Proxy Tool` project surface to the design system alongside Desktop and Mobile.

## Non-goals

- System-wide transparent interception.
- System-wide transparent interception or remote certificate distribution.
- Persistence, multi-user access, authentication, or hosted deployment.
- Connecting the tool to SunCode's production Rust agent or SDK.

## Requirements

1. `npm run dev` starts the Next.js UI and the proxy listener.
2. The UI defaults to `http://localhost:3000`; the proxy defaults to `http://localhost:8080`.
3. HTTP proxy requests use absolute-form URLs and are forwarded to their target.
4. HTTPS CONNECT requests are terminated by the local MITM proxy and forwarded upstream after the client trusts the generated local CA.
5. Request records include method, URL, target, start time, duration, status, content type, request headers, response headers, request body preview, response body preview, and transport state.
6. Sensitive header values are redacted before they reach the UI or logs.
7. `text/event-stream` responses are parsed incrementally into event records while raw chunks continue to stream to the client.
8. The UI can filter by method, status, content type, and text search, select a request, pause live updates, and clear the in-memory capture.
9. Payload previews are bounded and identify truncated or binary content.
10. The design system exposes the Proxy Tool project index and a representative monitoring workspace in light and dark themes.

## Edge cases

- Upstream DNS, connection, timeout, and protocol errors remain visible as failed request records.
- A disconnected SSE client leaves the upstream request record intact and marked with its final state.
- A request body or response body over the capture limit is forwarded but only partially retained for inspection.
- No captured traffic shows the proxy address and a copyable setup example.
- Invalid absolute URLs return a clear `400` response from the proxy.

## Acceptance criteria

- The proxy forwards HTTP requests and preserves request/response streaming behavior.
- The monitor UI updates without manual refresh when requests start, receive headers, receive SSE events, and finish.
- Design-system build passes and the new project is reachable under `/projects/proxy-tool`.
- Focused proxy tests cover redaction, bounded capture, SSE parsing, and failed upstream requests.

## Open questions

- Whether a later delivery should add certificate trust installation helpers for specific browsers and operating systems.
