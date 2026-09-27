# Changes

## Source

- Add the standalone `tools/proxy` Next.js application.
- Add the `Proxy Tool` design-system project page and route.

## Contracts and generated artifacts

- None in SunCode production contracts.

## Configuration and persistence

- `PORT`, `PROXY_PORT`, `CAPTURE_LIMIT_BYTES`, and `MAX_CAPTURED_REQUESTS` are local environment variables.
- Captures are in memory only.

## Tests

- Add focused Node tests for redaction, bounded capture, and SSE parsing.

## Documentation

- Add `tools/proxy/README.md` with setup examples and limitations.
