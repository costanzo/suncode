# Architecture

## Current state

`apps/mobile` contains a shared Remote Control HTTP/WebSocket client and repository projection. The Fake repository remains available for previews and for launches without a configured Remote Server URL.

## Proposed design

Keep transport DTOs under `ai.suncode.mobile.remote.protocol`. The future HTTP client and WebSocket reducer will map these DTOs into the existing domain models. Event payloads are represented as `JsonObject` at this stage so additive event types and fields can pass through until the reducer is implemented.

## Boundaries and dependencies

Protocol DTOs depend only on `kotlinx.serialization` and common Kotlin code. The Ktor client depends on common Ktor APIs, with OkHttp on Android and Darwin on iOS. It does not depend on Compose, the Rust SDK, or the Remote Server implementation.

## Data and control flow

HTTP/WebSocket transport -> protocol DTOs -> repository mapping -> domain models/UI. Android and iOS bootstrap the same repository boundary when a Remote Server URL is configured. The repository refreshes expired access tokens once per request, hydrates Host Projects from the Host API, reconnects the event Flow with bounded backoff, resumes through a sync cursor, persists the Host and Session projection on each successful update, and reduces common Session events locally before falling back to detail refresh.

## Security and failure handling

Credentials are represented through `RemoteTokenStore`; Android has a Keystore-backed encrypted implementation, while the in-memory implementation remains for wiring/tests. Unknown event types remain data and are ignored by later reducers unless explicitly supported.

## Compatibility and migration

DTO field names follow the current v1 contracts. New fields remain optional where the contract permits them. Breaking changes require a new protocol version.

## Risks and rollback

The DTO layer is additive and can be removed without changing the current Fake repository or UI.

## Open questions

- Which KMP HTTP/WebSocket engine will be selected for the live client?
- Which platform secure storage implementation will hold opaque credentials?
