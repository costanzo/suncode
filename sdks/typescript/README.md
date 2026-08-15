# TypeScript SDK

Planned package for TypeScript and JavaScript consumers of the SunCode runtime.

This directory intentionally has no package implementation yet. When implemented, it will use N-API to embed the Rust SDK in the Node.js host process. It must not connect to an HTTP runtime service, open SQLite independently, contact model providers directly, or reimplement runtime behavior. Node.js remains outside the Phase 1 Avalonia production dependency graph.
