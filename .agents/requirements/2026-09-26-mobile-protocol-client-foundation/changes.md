# Changes

## Source

- Add shared remote-control protocol DTOs and event envelope types.
- Add a typed `RemoteControlClient` boundary for all current HTTP operations and the WebSocket event stream.
- Add a Ktor-based multiplatform implementation with Android OkHttp and iOS Darwin engines.
- Add access-token injection, token rotation storage hooks, HTTP error mapping, and WebSocket event decoding.
- Add focused common tests for representative contract payloads.

## Contracts and generated artifacts

- No contract files are changed.
- DTOs are hand-written; no code generation is used.

## Configuration and persistence

- Add kotlinx.serialization to the shared module.
- Add Ktor client dependencies to the shared module.
- Add Android Keystore and iOS Keychain credential storage plus Android SharedPreferences and iOS UserDefaults projection caches.

## Tests

- Decode pairing, session detail, sync, and WebSocket event envelopes.

## Documentation

- Keep this requirement package current while live transport, pairing, and device validation progress.
