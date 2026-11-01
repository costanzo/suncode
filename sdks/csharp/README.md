# SunCode C# SDK

This project is the managed SDK consumed by the Avalonia desktop client. It owns the C ABI declarations, native handle lifetime, UTF-8 and JSON envelope marshalling, and the public C# API surface.

The native library is built from [`../c`](../c), which wraps the typed Rust facade in [`../rust`](../rust). Avalonia should reference this project rather than declaring P/Invoke functions or invoking Cargo directly.

The public managed surface is fully typed. Operations return DTOs from `src/Models`, requests use typed records, and session subscriptions deliver typed `AgentEvent` values. Primary-session loading uses `SessionWatch`: native watch creation returns an atomic typed snapshot with a dormant subscription, and the host explicitly calls `Start` only after applying the matching presentation state. Disposing before start releases the stream without creating a callback thread. The native ABI still transports JSON internally, but that representation is private to the SDK implementation and is never exposed as a public `JsonObject` API.

Avalonia should use the typed methods on `AgentSdk` exclusively. `AgentSdk` owns the private native JSON envelope parsing and protocol field-name mappings; it does not expose SQLite, Rust internals, or compatibility methods that return dynamic JSON objects.

Hosts may call `AgentSdk.ConfigureNativeLibrary(absolutePath)` before the first native call to select a verified runtime. The assembly resolver loads that exact path once and caches its handle; changing the selected path after native resolution and in-process runtime swapping are rejected. Without an override, development builds retain normal bundled-library resolution. `RequiredNativeAbiVersion` (currently 27) and `GetNativeAbiVersion()` expose compatibility identity; `GetVersionAsync()` supplies the separate product version. These queries do not open an Agent or its database. Desktop owns update verification and restart coordination, documented in [`Infrastructure/Updates`](../../apps/desktop-avalonia/Infrastructure/Updates/README.md); the managed binding adds no Rust update API.
