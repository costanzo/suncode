# SunCode C# SDK

This project is the managed SDK consumed by the Avalonia desktop client. It owns the C ABI declarations, native handle lifetime, UTF-8 and JSON envelope marshalling, and the public C# API surface.

The native library is built from [`../c`](../c), which wraps the typed Rust facade in [`../rust`](../rust). Avalonia should reference this project rather than declaring P/Invoke functions or invoking Cargo directly.

The public managed surface is fully typed. Operations return DTOs from `src/Models`, requests use typed records, and session subscriptions deliver typed `AgentEvent` values. The native ABI still transports JSON internally, but that representation is private to the SDK implementation and is never exposed as a public `JsonObject` API.

Avalonia should use the typed methods on `AgentSdk` exclusively. `AgentSdk` owns the private native JSON envelope parsing and protocol field-name mappings; it does not expose SQLite, Rust internals, or compatibility methods that return dynamic JSON objects.
