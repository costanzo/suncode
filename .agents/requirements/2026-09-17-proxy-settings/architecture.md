# Architecture

## Current state

Provider, WebFetch, and remote MCP HTTP clients share certificate settings but build their clients independently. Global settings live in the `configuration` table and are exposed through the SDK settings facade.

## Proposed design

Add a neutral `HttpProxyConfiguration` contract to `suncode-common`. The Rust SDK facade loads one live proxy snapshot and supplies it to the provider registry and audited operations dispatcher. MCP reads the same persisted global configuration when constructing a remote connection.

Persist `proxy_mode`, `proxy_url`, `proxy_username`, `proxy_password`, and `proxy_bypass` as global configuration keys. A named SDK update validates and writes all keys in one database transaction. The generic read projection removes `proxy_password` and adds `proxy_password_configured`.

## Boundaries and dependencies

- Rust remains the only persistence and HTTP-policy owner.
- Avalonia uses typed SDK DTOs and never reads SQLite.
- `suncode-llm`, `suncode-tool`, and `suncode-mcp` accept the neutral proxy contract without database dependencies.
- Existing reqwest versions remain unchanged; each HTTP-owning crate maps the common contract to its local builder.

## Data and control flow

1. Avalonia submits one proxy update request.
2. The SDK validates mode, URL, credentials, and bypass rules.
3. SQLite updates all proxy keys atomically in `configuration`.
4. The live provider and WebFetch proxy snapshot is replaced.
5. Active remote MCP servers reconcile and reconnect using the persisted snapshot.
6. The response contains only non-secret values plus `passwordConfigured`.

## Security and failure handling

The proxy password follows the current plaintext SQLite secret policy. It is write-only through public SDK reads. URLs containing credentials are rejected, errors are redacted, and no configuration type derives a secret-revealing debug representation.

Loopback destinations are always direct so local services cannot be accidentally sent to an external proxy. Invalid configuration is rejected before persistence. In-flight requests are not cancelled.

## Compatibility and migration

The current schema is unchanged. Seeded global configuration rows add non-secret proxy defaults and an empty password value idempotently. Existing databases receive these rows through the existing data manifest.

## Risks and rollback

System-proxy behavior depends on reqwest platform support and standard environment variables. The feature can be rolled back by selecting No proxy; removing the UI does not require a schema migration because the values remain ordinary configuration rows.

## Open questions

- None.
