# Progress

- Status: In progress
- Last updated: 2026-09-26

## Completed

- Requirement package initialized.
- Protocol DTO layer implemented.
- Typed Remote Control client boundary added.
- Ktor HTTP/WebSocket transport implemented for Android and iOS targets.
- Remote repository adapter added and session actions now call the repository boundary.
- Android SDK command-line tools, platform 37.0, build tools, and platform tools installed locally.
- Android Keystore token store added.
- iOS Keychain token store added.
- Session creation now reports HTTP acceptance without fabricating a Session; the real Session is inserted when its WebSocket event arrives.
- Android startup now uses the live repository when `-PremoteControlBaseUrl=...` is supplied, otherwise it keeps the prototype repository; Android internet access is declared.
- WebSocket collection now retries with bounded exponential backoff and runs `/v1/sync` before each connection; paged sync responses update the in-memory cursor and Session projection.
- Session projection and cursor are persisted on Android SharedPreferences and iOS UserDefaults; startup restores the cache before network recovery.
- Pairing payload exchange is wired through the repository and UI; returned credentials use the existing Android Keystore/iOS Keychain stores.
- Common WebSocket events now update the local Session projection directly for turn state, completion, messages, approval resolution, and question resolution; unsupported event shapes still fall back to Session refresh.
- iOS startup now reads `RemoteControlBaseURL` from the app configuration and wires Ktor, Keychain, and UserDefaults cache when configured; empty configuration keeps the Fake repository.
- Android pairing now launches a one-shot CameraX + ML Kit QR scanner with runtime camera permission and returns the decoded payload to the shared pairing dialog.
- iOS pairing now presents a native AVFoundation QR scanner with camera permission handling and returns the decoded payload to the shared pairing dialog.
- Host and Project projections now use `/v1/sync` Host records, hydrate project lists through `/v1/hosts/{hostId}/projects`, preserve connection states, and persist Hosts in the offline cache.
- Authenticated HTTP calls now retry once after a 401 by refreshing the stored refresh token; failed refresh clears the secure credential.
- Pairing failures are surfaced in the shared dialog with a retryable error state.
- Focused common tests added.

## In progress

- Native scanner UI still needs device-level visual verification once a full Xcode installation is available.

## Blocked

- No active blockers.

## Log

### 2026-09-26

- Added hand-written Kotlin serialization DTOs for the current remote-control v1 contract.
- Added Ktor transport and platform engine selection.
- Added DTO-to-domain projection and repository action wiring.
- Added Android Keystore-backed encrypted token storage.
