# Mobile Client

**Status:** Implemented and focused-tested on Android; iOS compiles but camera and device behavior are unverified

`apps/mobile` is a Kotlin Multiplatform / Compose Multiplatform app for Android and iOS. It remote-controls a paired SunCode Desktop through the Remote Server (see [`remote-control/`](../remote-control/README.md)). It never accesses SQLite or model providers directly.

## What it does

- **Pairing.** A native QR scanner (Android CameraX with ML Kit, iOS AVFoundation) returns the payload to the shared pairing dialog. The app parses the endpoint, `hostId`, `code`, and the AES key (only when `e2e` is not `0`), then calls `/v1/mobile/pairings/exchange`. Pairing failures appear in the dialog and can be retried.
- **Credentials.** Access and refresh tokens and the per-Host AES key live in Android Keystore-backed encrypted storage or the iOS Keychain. An authenticated call that gets `401` refreshes once; if refresh fails, the stored credential is cleared.
- **Transport.** Ktor (OkHttp on Android, Darwin on iOS) uses hand-written `kotlinx.serialization` DTOs under `ai.suncode.mobile.remote.protocol`; nothing is generated. With an encryption key present, mutating request bodies are sent as AES-256-GCM `encPayload` and encrypted responses and events are decrypted.
- **Live updates.** Only one transport is active at a time:
  - In the foreground with no Session open, the app polls `/v1/mobile/hosts/{hostId}/sync` every 15 seconds.
  - With a Session open, polling stops and the app opens that Session's SSE stream. It applies the initial `session.snapshot`, then events in sequence. Duplicate or older events are ignored, and unknown event types still advance the cursor.
  - In the background, both transports are closed.
- **Recovery.** The last event ID per Session is persisted and sent as `Last-Event-ID`. A `410` clears the cursor, refreshes the Session, and reconnects for a fresh snapshot. Other failures use bounded exponential backoff with jitter.
- **Offline cache.** Hosts, Projects, Sessions, and event cursors are cached in SharedPreferences (Android) or UserDefaults (iOS) and restored at startup before network recovery. Streamed `assistant.delta` updates apply to memory immediately and are persisted with a debounce.
- **Actions.** Users can create a Session (the app opens it from the returned ID), send messages, cancel or retry turns, and resolve approvals and questions. Archiving Sessions and revoking devices are not available.
- **Localization.** Settings offers English (`en-US`) and 简体中文 (`zh-CN`); missing or unsupported values fall back to `en-US`. The choice applies immediately to every phone and tablet surface and open dialog, and persists in SharedPreferences or UserDefaults. Server data, model output, project names, paths, and protocol identifiers are not translated. There is no OS-language auto-detection.

The `FakeMobileRepository` exists only for Compose previews. Both platform entry points use the live `RemoteMobileRepository`, and the endpoint comes from the pairing QR code.

## Verification

Shared common tests cover DTO decoding, SSE snapshot and cursor fields, cache round-trips, E2E crypto, and localization. Run them with `./gradlew :shared:testAndroidHostTest`; Android packaging uses `./gradlew :androidApp:assembleDebug`. Linking and running iOS tests (`:shared:iosSimulatorArm64Test`) requires a full Xcode installation.

## Not implemented

- Sending images.
- Background event delivery and push notifications.
- Verifying the native scanners on real devices.
