# Requirement

## Background

SunCode's built-in model providers, WebFetch tool, and remote Streamable HTTP MCP connections create separate Rust HTTP clients. Users need one global proxy setting that applies consistently to every SunCode-owned HTTP request.

## Goals

- Add No proxy, System proxy, and Custom proxy modes to Network Settings.
- Apply the selected mode to built-in providers, WebFetch, and remote MCP HTTP connections.
- Support an HTTP or HTTPS proxy URL, Basic username/password authentication, and bypass rules.
- Persist proxy values in the existing global `configuration` table.
- Keep `proxy_password` out of setting read responses and expose only whether one is configured.
- Apply changes to subsequent provider and WebFetch requests and reconnect active remote MCP servers.

## Non-goals

- PAC or WPAD evaluation.
- SOCKS proxies.
- NTLM, Kerberos, or other integrated proxy authentication.
- Controlling HTTP requests made by local MCP child processes or trusted third-party provider implementations.
- Changing the current plaintext SQLite secret-storage policy.

## Requirements

1. `proxy_mode` is one of `no_proxy`, `system`, or `custom` and defaults to `system`.
2. Custom mode requires an absolute HTTP or HTTPS proxy URL with a host and without embedded credentials, query, or fragment.
3. Proxy credentials are stored separately from the URL. A missing password update preserves the stored password; an explicit clear removes it.
4. `list_settings` never returns `proxy_password`. It returns `proxy_password_configured` as a boolean instead.
5. Bypass rules support hostnames, domain suffixes, IP addresses, CIDR ranges, and `*`. Loopback names and ranges are always bypassed.
6. No proxy explicitly disables automatic system and environment proxies. System mode uses supported operating-system and standard environment proxy settings. Custom mode ignores system proxy selection.
7. Saving is atomic across every proxy configuration key.
8. In-flight requests are unchanged. Subsequent provider and WebFetch requests use the new configuration. Active remote MCP HTTP connections reconnect.
9. Proxy passwords, credential-bearing system proxy values, and authentication headers never enter SDK responses, events, diagnostics, or logs.

## Edge cases

- Switching away from custom mode preserves the saved custom values for later reuse.
- An empty stored password is treated as unconfigured.
- Invalid bypass entries prevent saving and identify the invalid rule.
- Remote MCP reconnection failure remains visible through its existing failed runtime state.

## Acceptance criteria

- Focused Rust tests verify validation, redaction, persistence, proxy selection, bypass behavior, and live configuration updates.
- C and C# bindings expose the atomic proxy update contract.
- The design-system and Avalonia Network pages cover all modes and password states in both themes and constrained widths.
- Provider, WebFetch, and remote MCP clients consume the same effective global proxy configuration.

## Open questions

- None.
