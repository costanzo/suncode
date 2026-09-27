# Test Plan

## Scope

Validate Remote configuration persistence, event filtering, request correlation, reconnecting state, and Desktop UI bindings.

## Unit tests

- Test the explicit upload event allowlist and assert `assistant.delta` is excluded.
- Test serialization of final `message.assistant` as one complete event.
- Test request ID parsing and `X-Host-Id`/`X-Request-Id` header construction.
- Test paired and disconnected status projection.

## Integration and conformance tests

- Use a local HTTP/SSE test server to exercise pairing, command delivery, callback, and reconnect.

## Regression checks

- Rust SDK focused tests.
- Avalonia tests and build.
- Design-system build and relevant formatting checks.
- `git diff --check`.

## Manual checks

- Configure URL and pairing code, connect, display QR, disconnect, and inspect footer state.

## Commands and results

- Pending.

## Residual risks

- End-to-end tests remain unavailable until the Java Remote Server implements the Desktop contract.
