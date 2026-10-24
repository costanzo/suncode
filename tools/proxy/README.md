# Proxy Tool

Standalone local API proxy and request inspector for development work.

## Start

```sh
cd tools/proxy
npm install
npm run dev
```

- Monitor UI: <http://localhost:3000>
- Explicit HTTP/HTTPS proxy: <http://localhost:8080>

Configure an API client with `HTTP_PROXY` / `HTTPS_PROXY` or its equivalent. Plain HTTP and HTTPS requests are captured and forwarded. HTTPS inspection uses a local development CA generated under `tools/proxy/.ca`.

```sh
HTTP_PROXY=http://127.0.0.1:8080 HTTPS_PROXY=http://127.0.0.1:8080 curl -k https://example.com
```

For trusted local inspection, import `tools/proxy/.ca/certs/ca.pem` into the client or operating-system trust store. `curl -k` is useful for a one-off local check, but it disables certificate verification for that command.

## Configuration

| Variable | Default | Purpose |
| --- | ---: | --- |
| `PORT` | `3000` | Monitor UI and local control API |
| `PROXY_PORT` | `8080` | Explicit proxy listener |
| `CAPTURE_LIMIT_BYTES` | `262144` | Retained preview bytes per direction |
| `MAX_CAPTURED_REQUESTS` | `500` | Maximum in-memory request records |

Captures are process-local and disappear when the tool stops. Sensitive headers are redacted before they reach the UI. The MITM CA is for local development only and must not be shared.

## Provider inspector

The monitor recognizes the terminal paths `/responses`, `/chat/completions`, and `/messages` on any configured host or base path. A request row shows the detected protocol, model, actual route, response format, HTTP status, duration, and capture state.

The selected call has Request and Response directions. Each direction supports Readable, Raw, and Headers; SSE responses also expose Events. Readable requests decode provider-specific wire shapes: Responses `input` items and standalone function calls, Chat Completions messages with nested `tool_calls` and `tool_call_id`, and Anthropic Messages system/tool content blocks. Tool arguments and results stay linked by call ID. Readable responses reconstruct text and tool-call deltas only from captured events, show provider-reported usage and response identity, and keep incomplete, malformed, truncated, and failed captures explicit. Raw views preserve retained bytes separately from formatted previews.

The UI pauses display updates without stopping proxy forwarding. Clearing the in-memory capture store requires inline confirmation. The monitor APIs supply the active proxy listener address and capture limit to the footer and empty state. Search matches endpoint, URL, model, local capture ID, and provider request ID. Generic HTTP requests remain inspectable.
