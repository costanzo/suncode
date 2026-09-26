# Test Plan

## Scope

Documentation-level validation of the Mobile ↔ Remote Server HTTP and WebSocket contracts.

## Unit tests

- None in this documentation-only delivery.

## Integration and conformance tests

- Parse both YAML documents with a YAML parser.
- Assert the OpenAPI document declares `/v1/pairings/exchange`, `/v1/sessions`, `/v1/sync`, bearer auth, and `Idempotency-Key`.
- Assert the AsyncAPI document declares a receive-only WebSocket operation and the full Rust `AgentEvent` event catalog.

## Regression checks

- `git diff --check`.
- Review that Mobile cannot archive Sessions or revoke other devices.
- Review that the WebSocket has no application-level command, subscribe, ping, snapshot, or error message surface.
- Review that tokens, provider payloads, and secrets are excluded from protocol DTOs.

## Manual checks

- Read the README lifecycle against the Desktop ↔ Remote Server ↔ Mobile topology.
- Confirm HTTP mutation names and WebSocket command names are semantically aligned.

## Commands and results

- `ruby -e 'require "yaml"; ...'` — expected to pass for both YAML documents.
- `git diff --check` — expected to pass.

## Residual risks

- Java Remote Server and Rust `suncode-remote` conformance tests do not exist until those implementations begin.
