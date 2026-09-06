# SunCode C# SDK

This project is the managed SDK consumed by the Avalonia desktop client. It owns the C ABI declarations, native handle lifetime, UTF-8 and JSON envelope marshalling, and the public C# API surface.

The native library is built from [`../c`](../c), which wraps the typed Rust facade in [`../rust`](../rust). Avalonia should reference this project rather than declaring P/Invoke functions or invoking Cargo directly.

The current public methods retain the ABI's JSON payload compatibility while the managed surface is being migrated to typed DTOs. New application code should use the typed models under `src/Models` where available.

`AgentSdk` keeps the low-level JSON-returning methods for compatibility and exposes typed request/response overloads in `TypedAgentSdk.cs`. The typed layer is the preferred surface for Avalonia and owns protocol field-name mappings explicitly; it does not access SQLite or Rust internals.
