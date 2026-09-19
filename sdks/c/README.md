# SunCode C SDK

This crate owns the stable C ABI consumed by the .NET Avalonia client. It wraps `suncode_sdk::blocking::AgentSdk`, the explicit runtime-owning adapter over the async-first Rust facade, and emits the platform native library (`libsuncode_agent.dylib`, `libsuncode_agent.so`, or `suncode_agent.dll`).

The ABI is hand-written and versioned. It does not open SQLite, contact providers, or implement agent behavior directly.

Session callbacks are an adapter over the Rust SDK's typed pull-based event stream. This crate owns the dormant watch handle, explicit callback activation, callback worker thread, legacy JSON envelope serialization, C-string lifetime, lag-to-`resync.required` translation, and subscription-handle shutdown. `watch_session` returns snapshot JSON plus an inactive handle; `subscription_start` begins delivery exactly once after the host applies the snapshot. Those FFI concerns do not enter `sdks/rust`.
