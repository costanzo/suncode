# Changes

## Source

- Added shared HTTP proxy mode/configuration contracts.
- Applied no-proxy, system-proxy, and custom-proxy behavior to provider, WebFetch, and remote MCP clients.
- Reconciled active remote MCP connections after a proxy update.
- Added the Network Settings proxy controls to the design-system and Avalonia client.
- Kept No proxy and System proxy compact by omitting mode-specific explanatory notes beneath the selector.

## Contracts and generated artifacts

- Added the named `set_proxy_configuration` Rust, C, and C# SDK operation.
- Added typed request/result DTOs with write-only password patching and redacted configured state.

## Configuration and persistence

- Added global `proxy_mode`, `proxy_url`, `proxy_username`, `proxy_password`, and `proxy_bypass` configuration defaults.
- Added an atomic multi-key global configuration write.
- Removed `proxy_password` from settings reads and synthesized `proxy_password_configured`.

## Tests

- Added facade validation, atomic persistence, password preserve/clear, live state, and redaction tests.
- Added real custom-proxy routing tests for provider and WebFetch requests.
- Added MCP client construction coverage for every proxy mode and C# DTO conformance coverage.

## Documentation

- Updated architecture, agent specification, Phase 1 feature records, SDK contract, DESIGN.md, and the executable Settings specimen.
