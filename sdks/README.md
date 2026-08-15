# SunCode SDKs

Language SDKs live here. They embed and wrap the native Rust runtime boundary and do not own durable state, credentials, provider calls, SQLite, or project operations.

The Phase 1 product ships the Avalonia desktop app and the embedded Rust runtime SDK facade. TypeScript and Python SDKs are planned native packaging surfaces and are intentionally represented here only by placeholders until implementation starts. They will not connect to an HTTP runtime service.

Planned layout:

```text
sdks/
  typescript/   # N-API native binding and TypeScript API
  python/       # PyO3 native binding and Python API
```
