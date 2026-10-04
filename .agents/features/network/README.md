# Network Settings

**Status:** Implemented and focused-tested

The Rust agent owns all network policy for HTTP requests it makes itself. The settings live as global rows in `configuration` and are changed through the SDK. Desktop Settings shows them on its Network page.

## HTTPS certificates

- `verify_https_certificates` (default `true`): `false` accepts invalid certificate chains and hostnames on later requests, equivalent to `curl -k`. URL validation, redirect limits, approval, auditing, and redaction still apply.
- `use_system_certificates` (default `true`) and an optional PEM or DER `certificate_path`: when verification is on, clients trust the system roots, add the custom root when one is configured, and use only the custom file when system roots are off. An unreadable or invalid certificate file fails the next request with a stable error.
- Built-in OpenAI-compatible and Anthropic providers, WebFetch, and remote Streamable HTTP MCP clients apply these settings without a restart. Browser Use Chromium keeps normal certificate verification and ignores the insecure toggle.

## Proxy settings

`set_proxy_configuration` validates the full proxy configuration and writes all keys atomically: `proxy_mode`, `proxy_url`, `proxy_username`, `proxy_password`, and `proxy_bypass`. Generic setting writes cannot change these keys.

- `no_proxy` turns off both system and environment proxies. `system` (the default) uses supported OS proxy settings and the standard environment variables. `custom` uses one HTTP or HTTPS proxy URL with no embedded credentials, query, or fragment, plus optional Basic authentication.
- Bypass rules accept hostnames, domain suffixes, IP addresses, CIDR ranges, and `*`. An invalid rule blocks the save and is named in the error. Localhost, IPv4 loopback, and IPv6 loopback always bypass the proxy.
- Saved custom values are kept when switching to another mode. Omitting the password keeps the stored one. Clearing it removes it. An empty stored password counts as unconfigured.
- Reads never return `proxy_password`; they return `proxy_password_configured` instead. Proxy credentials and auth headers stay out of DTOs, events, diagnostics, and logs. The password is stored as plaintext in SQLite, the same as provider keys.
- Requests already in flight are not changed. Later provider and WebFetch requests use the new live snapshot. Active remote MCP connections reconcile and reconnect, and a reconnect failure shows up as that server's `failed` state. A Browser Use worker reads the current mode, including custom credentials and bypass rules, the next time it starts.
- Child processes (bash, MCP stdio, language servers) receive only the six standard proxy environment variables already present on the host. HTTP made by local MCP processes and by trusted in-process provider implementations is not controlled. PAC/WPAD, SOCKS, and NTLM/Kerberos authentication are not supported.

## Developer proxy tool

`tools/proxy` is a standalone Next.js and Node development tool for inspecting API traffic. It has no link to `agent/`, `apps/`, or `sdks/` and is not a production dependency.

- `npm run dev` starts the monitor UI on port 3000 (`PORT`) and an explicit HTTP/HTTPS proxy on port 8080 (`PROXY_PORT`).
- HTTPS CONNECT is intercepted by `http-mitm-proxy` using a local CA generated under the ignored `tools/proxy/.ca`. The client must trust that CA, or use `curl -k`, before HTTPS bodies can be inspected.
- Captures are kept in memory only: up to `MAX_CAPTURED_REQUESTS` (500) records, with previews bounded by `CAPTURE_LIMIT_BYTES` (256 KiB per direction). Request and response bodies are still forwarded in full.
- Authorization, Cookie, Set-Cookie, Proxy-Authorization, and API-key headers are redacted. SSE responses are parsed into events as they stream.
- The UI filters by method, status, content type, and text. It can pause updates or clear captures, and it gets updates over `/api/events` plus a two-second summary poll.
- The design system has a matching `Proxy Tool` project at `/projects/proxy-tool`.
