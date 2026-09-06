# SunCode SDKs

Language SDKs live here. They embed and wrap the native Rust runtime boundary and do not own durable state, credentials, provider calls, SQLite, or project operations.

The Phase 1 product ships the Avalonia desktop app, the typed Rust facade, and the embedded C ABI native binding. TypeScript and Python SDKs are planned native packaging surfaces and are intentionally represented here only by placeholders until implementation starts. They will not connect to an HTTP runtime service.

Planned layout:

```text
sdks/
  rust/         # typed Rust SDK facade over the agent harness
  c/            # stable C ABI and native library for Avalonia
  csharp/       # managed, strongly typed C# SDK and native integration for Avalonia
  typescript/   # N-API native binding and TypeScript API (placeholder)
  python/       # PyO3 native binding and Python API (placeholder)
```
