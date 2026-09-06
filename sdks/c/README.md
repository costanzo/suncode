# SunCode C SDK

This crate owns the stable C ABI consumed by the .NET Avalonia client. It wraps the typed Rust facade from `sdks/rust` and emits the platform native library (`libsuncode_agent.dylib`, `libsuncode_agent.so`, or `suncode_agent.dll`).

The ABI is hand-written and versioned. It does not open SQLite, contact providers, or implement agent behavior directly.
